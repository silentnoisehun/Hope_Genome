"""
Basic tests for Hope Genome Python bindings
"""
import pytest


def test_import():
    """Test that the module can be imported"""
    import hope_genome
    assert hope_genome.__version__ == "1.6.0"
    assert "Máté Róbert" in hope_genome.__author__


def test_create_genome():
    """Test creating a genome with rules"""
    from hope_genome import HopeGenome

    genome = HopeGenome(rules=[
        "Do no harm",
        "Respect privacy",
        "Ensure fairness"
    ])

    assert genome.rules() == ["Do no harm", "Respect privacy", "Ensure fairness"]
    assert not genome.is_sealed()


def test_seal_genome():
    """Test sealing a genome"""
    from hope_genome import HopeGenome

    genome = HopeGenome(rules=["Rule 1"])
    genome.seal()

    assert genome.is_sealed()
    assert genome.capsule_hash() is not None


def test_create_actions():
    """Test creating different action types"""
    from hope_genome import Action

    # Delete action
    delete_action = Action.delete("test.txt")
    assert delete_action.action_type() == "Delete"
    assert delete_action.target() == "test.txt"

    # Write action
    write_action = Action.write_file("output.txt", b"test content")
    assert write_action.action_type() == "Write"
    assert write_action.target() == "output.txt"

    # Read action
    read_action = Action.read("input.txt")
    assert read_action.action_type() == "Read"

    # Execute action
    exec_action = Action.execute("ls -la")
    assert exec_action.action_type() == "Execute"


def test_verify_action():
    """Test getting a proof for an action"""
    from hope_genome import HopeGenome, Action

    genome = HopeGenome(rules=["Rule 1"])
    genome.seal()

    action = Action.delete("test.txt")
    proof = genome.verify_action(action)

    assert proof is not None
    assert proof.timestamp > 0
    assert proof.ttl > 0
    assert len(proof.nonce) == 32
    assert len(proof.action_hash) == 32
    assert len(proof.signature) > 0
    assert proof.status == "OK"


def test_proof_not_expired():
    """Test that a fresh proof is not expired"""
    from hope_genome import HopeGenome, Action

    genome = HopeGenome(rules=["Rule 1"])
    genome.seal()

    action = Action.read("file.txt")
    proof = genome.verify_action(action)

    assert not proof.is_expired()


def test_proof_timestamp_string():
    """Test proof timestamp formatting"""
    from hope_genome import HopeGenome, Action

    genome = HopeGenome(rules=["Rule 1"])
    genome.seal()

    action = Action.read("file.txt")
    proof = genome.verify_action(action)

    timestamp_str = proof.timestamp_string()
    assert isinstance(timestamp_str, str)
    assert len(timestamp_str) > 0


def test_create_auditor():
    """Test creating an auditor"""
    from hope_genome import Auditor

    auditor = Auditor()
    assert auditor.used_nonce_count() == 0


def test_create_audit_log():
    """Test creating an audit log"""
    from hope_genome import AuditLog

    log = AuditLog()
    assert log.is_empty()
    assert len(log) == 0


def test_audit_log_append():
    """Test appending to audit log"""
    from hope_genome import HopeGenome, Action, AuditLog

    genome = HopeGenome(rules=["Rule 1"])
    genome.seal()

    action = Action.delete("test.txt")
    proof = genome.verify_action(action)

    log = AuditLog()
    log.append(action, proof, approved=True)

    assert len(log) == 1
    assert not log.is_empty()


def test_audit_log_verify_chain():
    """Test verifying audit log chain integrity"""
    from hope_genome import HopeGenome, Action, AuditLog

    genome = HopeGenome(rules=["Rule 1"])
    genome.seal()

    log = AuditLog()

    # Add multiple entries
    for i in range(5):
        action = Action.delete(f"file{i}.txt")
        proof = genome.verify_action(action)
        log.append(action, proof, approved=True)

    # Verify chain integrity
    log.verify_chain()  # Should not raise

    assert len(log) == 5


def test_consensus_verifier():
    """Test creating a consensus verifier"""
    from hope_genome import ConsensusVerifier

    verifier = ConsensusVerifier(required_sources=3, tolerance=0.1)
    assert verifier is not None


def test_full_workflow():
    """Test complete workflow from genome to audit"""
    from hope_genome import HopeGenome, Action, AuditLog

    # Create and seal genome
    genome = HopeGenome(rules=[
        "Do no harm",
        "Respect privacy",
        "Ensure fairness"
    ])
    genome.seal()

    # Create audit log
    log = AuditLog()

    # Perform several actions
    actions = [
        Action.read("user_profile.json"),
        Action.write_file("recommendation.txt", b"User recommendation"),
        Action.delete("temp_cache.dat")
    ]

    for action in actions:
        proof = genome.verify_action(action)
        log.append(action, proof, approved=True)

    # Verify everything
    assert len(log) == 3
    log.verify_chain()  # Should not raise

    print(f"✅ Successfully processed {len(log)} actions")


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
