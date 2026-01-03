"""
Comprehensive test suite for Hope Genome Python bindings
"""

import pytest
import hope_genome as hg
from datetime import datetime
import json


class TestSealedGenome:
    """Test SealedGenome class"""

    def test_create_genome(self):
        """Test genome creation"""
        genome = hg.SealedGenome(rules=["Do no harm", "Respect privacy"])
        assert not genome.is_sealed()
        assert len(genome.rules()) == 2

    def test_seal_genome(self):
        """Test genome sealing"""
        genome = hg.SealedGenome(rules=["Rule 1"])
        genome.seal()
        assert genome.is_sealed()

    def test_cannot_seal_twice(self):
        """Test that genome cannot be sealed twice"""
        genome = hg.SealedGenome(rules=["Rule 1"])
        genome.seal()

        with pytest.raises(hg.GenomeError, match="already sealed"):
            genome.seal()

    def test_empty_rules_rejected(self):
        """Test that empty rules are rejected"""
        with pytest.raises(hg.GenomeError, match="empty"):
            hg.SealedGenome(rules=[])

    def test_genome_hash(self):
        """Test genome hash generation"""
        genome = hg.SealedGenome(rules=["Rule 1"])
        genome.seal()
        hash_hex = genome.genome_hash()

        assert isinstance(hash_hex, str)
        assert len(hash_hex) == 64  # 32 bytes = 64 hex chars

    def test_verify_action_approved(self):
        """Test action verification (approved)"""
        genome = hg.SealedGenome(rules=["Allow all"])
        genome.seal()

        action = hg.Action.delete_file("test.txt")
        proof = genome.verify_action(action)

        assert proof.approved
        assert len(proof.signature_hex()) == 128  # 64 bytes Ed25519

    def test_context_manager(self):
        """Test genome as context manager"""
        with hg.SealedGenome(rules=["Rule 1"]) as genome:
            genome.seal()
            assert genome.is_sealed()


class TestAction:
    """Test Action class"""

    def test_delete_file(self):
        action = hg.Action.delete_file("/tmp/file.txt")
        assert action.action_type() == "delete"
        assert action.target() == "/tmp/file.txt"

    def test_read_file(self):
        action = hg.Action.read_file("/etc/passwd")
        assert action.action_type() == "read"

    def test_write_file(self):
        action = hg.Action.write_file("/tmp/output.txt")
        assert action.action_type() == "write"

    def test_execute_command(self):
        action = hg.Action.execute_command("rm -rf /")
        assert action.action_type() == "execute"

    def test_network_request(self):
        action = hg.Action.network_request("https://api.example.com", "POST")
        assert action.action_type() == "network"

    def test_delete_user(self):
        action = hg.Action.delete_user("user_123")
        assert action.target() == "user_123"

    def test_medical_diagnosis(self):
        action = hg.Action.medical_diagnosis("patient_42", "cancer")
        assert "patient_42" in action.target()

    def test_financial_transaction(self):
        action = hg.Action.financial_transaction("ACC123", 100.50, "USD")
        assert "ACC123" in action.target()

    def test_custom_action(self):
        action = hg.Action.custom("ai-decision", "target-123", '{"key": "value"}')
        assert action.action_type() == "ai-decision"

    def test_action_hash(self):
        action = hg.Action.delete_file("test.txt")
        hash_hex = action.action_hash()

        assert isinstance(hash_hex, str)
        assert len(hash_hex) == 64  # SHA-256

    def test_action_equality(self):
        action1 = hg.Action.delete_file("test.txt")
        action2 = hg.Action.delete_file("test.txt")
        action3 = hg.Action.delete_file("other.txt")

        assert action1 == action2
        assert action1 != action3

    def test_action_hash_method(self):
        """Test that actions can be used in sets/dicts"""
        action = hg.Action.delete_file("test.txt")
        action_set = {action}
        assert action in action_set


class TestProof:
    """Test Proof class"""

    def setup_method(self):
        """Setup for each test"""
        self.genome = hg.SealedGenome(rules=["Allow all"])
        self.genome.seal()
        action = hg.Action.delete_file("test.txt")
        self.proof = self.genome.verify_action(action)

    def test_proof_approved(self):
        assert self.proof.approved

    def test_proof_hashes(self):
        assert len(self.proof.genome_hash) == 64
        assert len(self.proof.action_hash) == 64

    def test_proof_signature(self):
        sig_hex = self.proof.signature_hex()
        assert len(sig_hex) == 128  # 64 bytes Ed25519

        sig_bytes = self.proof.signature_bytes()
        assert len(sig_bytes) == 64

    def test_proof_nonce(self):
        nonce_hex = self.proof.nonce_hex()
        assert len(nonce_hex) == 64  # 32 bytes

        nonce_bytes = self.proof.nonce_bytes()
        assert len(nonce_bytes) == 32

    def test_proof_timestamp(self):
        ts = self.proof.timestamp()
        assert isinstance(ts, int)
        assert ts > 0

        ts_str = self.proof.timestamp_string()
        assert isinstance(ts_str, str)
        assert "T" in ts_str  # ISO 8601 format

    def test_proof_ttl(self):
        assert self.proof.ttl_seconds > 0
        assert not self.proof.is_expired()

    def test_proof_json_serialization(self):
        json_str = self.proof.to_json()
        proof_dict = json.loads(json_str)

        assert proof_dict["approved"]
        assert "signature" in proof_dict

        # Round-trip
        proof2 = hg.Proof.from_json(json_str)
        assert proof2.approved == self.proof.approved

    def test_proof_diagnostics(self):
        diag = self.proof.diagnostics()
        assert isinstance(diag, dict)
        assert diag["approved"]
        assert not diag["is_expired"]


class TestProofAuditor:
    """Test ProofAuditor class"""

    def test_create_auditor(self):
        auditor = hg.ProofAuditor()
        assert auditor.verified_count() == 0

    def test_verify_proof(self):
        genome = hg.SealedGenome(rules=["Allow all"])
        genome.seal()
        action = hg.Action.delete_file("test.txt")
        proof = genome.verify_action(action)

        auditor = hg.ProofAuditor()
        auditor.verify_proof(proof)  # Should not raise

        assert auditor.verified_count() == 1

    def test_replay_attack_detection(self):
        """Test that replay attacks are detected"""
        genome = hg.SealedGenome(rules=["Allow all"])
        genome.seal()
        action = hg.Action.delete_file("test.txt")
        proof = genome.verify_action(action)

        auditor = hg.ProofAuditor()
        auditor.verify_proof(proof)  # First verification OK

        # Second verification with same proof should fail
        with pytest.raises(hg.AuditorError, match="Replay attack|nonce"):
            auditor.verify_proof(proof)

    def test_nonce_tracking(self):
        """Test nonce usage tracking"""
        genome = hg.SealedGenome(rules=["Allow all"])
        genome.seal()
        action = hg.Action.delete_file("test.txt")
        proof = genome.verify_action(action)

        auditor = hg.ProofAuditor()
        nonce = proof.nonce_bytes()

        assert not auditor.is_nonce_used(nonce)
        auditor.verify_proof(proof)
        assert auditor.is_nonce_used(nonce)


class TestConsensusEngine:
    """Test ConsensusEngine class"""

    def test_create_engine(self):
        engine = hg.ConsensusEngine(threshold=0.66, required_sources=3)
        assert engine.threshold == 0.66
        assert engine.required_sources == 3

    def test_consensus_float(self):
        engine = hg.ConsensusEngine(threshold=0.66, required_sources=3)
        values = [10.0, 10.5, 10.2, 10.1]  # Close values

        consensus = engine.reach_consensus_float(values)
        assert isinstance(consensus, float)
        assert 10.0 <= consensus <= 10.5

    def test_consensus_int(self):
        engine = hg.ConsensusEngine(threshold=0.66, required_sources=3)
        values = [100, 101, 100, 102]

        consensus = engine.reach_consensus_int(values)
        assert isinstance(consensus, int)

    def test_insufficient_sources(self):
        """Test that insufficient sources raise error"""
        engine = hg.ConsensusEngine(threshold=0.66, required_sources=5)
        values = [10.0, 10.5]  # Only 2 sources, need 5

        with pytest.raises(hg.ConsensusError, match="Insufficient"):
            engine.reach_consensus_float(values)

    def test_byzantine_fault(self):
        """Test Byzantine fault detection"""
        engine = hg.ConsensusEngine(threshold=0.66, required_sources=3)
        values = [10.0, 10.1, 100.0, 10.2]  # One outlier

        # Should reject outlier and reach consensus
        consensus = engine.reach_consensus_float(values)
        assert 10.0 <= consensus <= 10.5


class TestKeyStore:
    """Test KeyStore backends"""

    def test_software_keystore(self):
        ks = hg.SoftwareKeyStore.generate()
        pk_hex = ks.public_key_hex()

        assert isinstance(pk_hex, str)
        assert len(pk_hex) == 64  # 32 bytes Ed25519

    def test_software_keystore_with_genome(self):
        ks = hg.SoftwareKeyStore.generate()
        genome = hg.SealedGenome(rules=["Rule 1"], keystore=ks)
        genome.seal()

        action = hg.Action.delete_file("test.txt")
        proof = genome.verify_action(action)

        assert proof.approved


class TestNonceStore:
    """Test NonceStore backends"""

    def test_memory_noncestore(self):
        store = hg.MemoryNonceStore()
        assert store.count() == 0

        nonce = b'\x01' * 32
        assert not store.contains(nonce)

        # Nonce storage is internal to auditor, so we test via auditor
        auditor = hg.ProofAuditor(noncestore=store)
        assert auditor.verified_count() == 0

    def test_memory_noncestore_clear(self):
        store = hg.MemoryNonceStore()
        store.clear()  # Should not raise
        assert store.count() == 0


class TestAuditLogger:
    """Test AuditLogger class"""

    def test_create_logger(self, tmp_path):
        log_file = tmp_path / "audit.log"
        logger = hg.AuditLogger(str(log_file))
        assert logger.entry_count() == 0

    def test_log_proof(self, tmp_path):
        log_file = tmp_path / "audit.log"
        logger = hg.AuditLogger(str(log_file))

        genome = hg.SealedGenome(rules=["Allow all"])
        genome.seal()
        action = hg.Action.delete_file("test.txt")
        proof = genome.verify_action(action)

        logger.log_proof(proof, action_description="Test deletion")
        assert logger.entry_count() == 1

    def test_verify_chain(self, tmp_path):
        """Test blockchain-style chain verification"""
        log_file = tmp_path / "audit.log"
        logger = hg.AuditLogger(str(log_file))

        genome = hg.SealedGenome(rules=["Allow all"])
        genome.seal()

        # Log multiple proofs
        for i in range(5):
            action = hg.Action.delete_file(f"file_{i}.txt")
            proof = genome.verify_action(action)
            logger.log_proof(proof)

        # Chain should be valid
        assert logger.verify_chain()

    def test_get_entries(self, tmp_path):
        """Test retrieving audit entries"""
        log_file = tmp_path / "audit.log"
        logger = hg.AuditLogger(str(log_file))

        genome = hg.SealedGenome(rules=["Allow all"])
        genome.seal()
        action = hg.Action.delete_file("test.txt")
        proof = genome.verify_action(action)

        logger.log_proof(proof, action_description="Test action")
        entries = logger.get_entries()

        assert len(entries) == 1
        assert entries[0].index == 0
        assert entries[0].action_description == "Test action"


class TestExceptions:
    """Test exception hierarchy"""

    def test_genome_error(self):
        with pytest.raises(hg.GenomeError):
            hg.SealedGenome(rules=[])

    def test_auditor_error(self):
        """Test replay attack raises AuditorError"""
        genome = hg.SealedGenome(rules=["Allow all"])
        genome.seal()
        action = hg.Action.delete_file("test.txt")
        proof = genome.verify_action(action)

        auditor = hg.ProofAuditor()
        auditor.verify_proof(proof)

        with pytest.raises(hg.AuditorError):
            auditor.verify_proof(proof)  # Replay attack

    def test_consensus_error(self):
        engine = hg.ConsensusEngine(threshold=0.66, required_sources=5)

        with pytest.raises(hg.ConsensusError):
            engine.reach_consensus_float([1.0, 2.0])  # Insufficient sources


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
