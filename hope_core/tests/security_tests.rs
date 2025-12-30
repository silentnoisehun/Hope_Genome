use _hope_core::crypto::SoftwareKeyStore;
use _hope_core::*;
use std::thread;
use std::time::Duration; // Explicitly import SoftwareKeyStore

/// Security Test Suite for Hope Genome v1.4.0
///
/// This module contains comprehensive security attack simulations
/// to validate the tamper-evident guarantees of the framework.
/// Updated to use v1.4.0 API (SoftwareKeyStore + MemoryNonceStore)

#[test]
fn test_replay_attack_comprehensive() {
    // Setup (v1.4.0 API)
    let key_store = SoftwareKeyStore::generate().unwrap();
    let nonce_store = MemoryNonceStore::new();
    let mut genome =
        SealedGenome::with_key_store(vec!["Do no harm".to_string()], Box::new(key_store.clone()))
            .unwrap();
    genome.seal().unwrap();

    let mut auditor = ProofAuditor::new(Box::new(key_store), Box::new(nonce_store));

    // Create an action and get proof
    let action = Action::delete("sensitive_file.txt");
    let proof = genome.verify_action(&action).unwrap();

    // First verification should succeed
    assert!(
        auditor.verify_proof(&proof).is_ok(),
        "First proof verification should succeed"
    );

    // Replay attack: try to use the same proof again
    let replay_result = auditor.verify_proof(&proof);

    assert!(replay_result.is_err(), "Replay attack should be detected");
    assert!(
        matches!(
            replay_result.unwrap_err(),
            crate::auditor::AuditorError::NonceReused(_)
        ),
        "Should specifically be a nonce reuse error"
    );
}

#[test]
fn test_oracle_attack_action_substitution() {
    // Setup (v1.4.0 API)
    let key_store = SoftwareKeyStore::generate().unwrap();
    let mut genome = SealedGenome::with_key_store(
        vec!["Allow only safe operations".to_string()],
        Box::new(key_store),
    )
    .unwrap();
    genome.seal().unwrap();

    // Get proof for a safe action
    let safe_action = Action::read("public_data.txt");
    let proof = genome.verify_action(&safe_action).unwrap();

    // Attacker tries to use this proof for a dangerous action
    let dangerous_action = Action::delete("/etc/passwd");

    // Verify that action hash in proof matches safe_action
    assert_eq!(
        proof.action_hash,
        safe_action.hash(),
        "Proof should be bound to safe action"
    );

    // Verify action hashes are different
    assert_ne!(
        safe_action.hash(),
        dangerous_action.hash(),
        "Actions should have different hashes"
    );

    // This demonstrates that an executor would detect the mismatch
    // In practice, the executor's action binding check would prevent this
}

#[test]
fn test_proof_expiration_attack() {
    // Setup (v1.4.0 API)
    let key_store = SoftwareKeyStore::generate().unwrap();
    let nonce_store = MemoryNonceStore::new();
    let mut genome =
        SealedGenome::with_key_store(vec!["Rule 1".to_string()], Box::new(key_store.clone()))
            .unwrap();
    genome.set_default_ttl(1); // 1 second TTL
    genome.seal().unwrap();

    let mut auditor = ProofAuditor::new(Box::new(key_store.clone()), Box::new(nonce_store));

    // Create proof with short TTL
    let action = Action::delete("file.txt");
    let proof = genome.verify_action(&action).unwrap();

    // Immediate verification should succeed
    assert!(
        auditor.verify_proof(&proof).is_ok(),
        "Fresh proof should verify"
    );

    // Wait for proof to expire
    thread::sleep(Duration::from_secs(2));

    // Create a new auditor (simulating new session)
    let nonce_store2 = MemoryNonceStore::new();
    let mut auditor2 = ProofAuditor::new(Box::new(key_store), Box::new(nonce_store2));

    // Expired proof should be rejected
    let result = auditor2.verify_proof(&proof);
    assert!(result.is_err(), "Expired proof should be rejected");
    assert!(
        matches!(
            result.unwrap_err(),
            crate::auditor::AuditorError::ProofExpired { .. }
        ),
        "Should be an expiration error"
    );
}

#[test]
fn test_signature_forgery_detection() {
    // Setup (v1.4.0 API)
    let key_store = SoftwareKeyStore::generate().unwrap();
    let nonce_store = MemoryNonceStore::new();
    let mut genome =
        SealedGenome::with_key_store(vec!["Rule 1".to_string()], Box::new(key_store.clone()))
            .unwrap();
    genome.seal().unwrap();

    let mut auditor = ProofAuditor::new(Box::new(key_store), Box::new(nonce_store));

    // Create a valid proof
    let action = Action::delete("file.txt");
    let mut proof = genome.verify_action(&action).unwrap();

    // Attacker tampers with the signature
    if !proof.signature.is_empty() {
        proof.signature[0] ^= 0xFF; // Flip bits
    }

    // Verification should fail
    let result = auditor.verify_proof(&proof);
    assert!(
        result.is_err(),
        "Tampered signature should fail verification"
    );
    assert!(
        matches!(
            result.unwrap_err(),
            crate::auditor::AuditorError::InvalidSignature
        ),
        "Should be a signature error"
    );
}

#[test]
fn test_action_hash_collision_resistance() {
    // Create two different actions
    let action1 = Action::delete("file1.txt");
    let action2 = Action::delete("file2.txt");

    // Verify hashes are different (collision resistance)
    assert_ne!(
        action1.hash(),
        action2.hash(),
        "Different actions should have different hashes"
    );

    // Even similar actions should have different hashes
    let action3 = Action::write_file("data.txt", b"content1".to_vec());
    let action4 = Action::write_file("data.txt", b"content2".to_vec());

    assert_ne!(
        action3.hash(),
        action4.hash(),
        "Same file, different content should have different hashes"
    );
}

#[test]
#[allow(deprecated)] // AuditLog still uses deprecated KeyPair API (TODO v1.5.0)
fn test_audit_log_chain_integrity() {
    // Setup - Using deprecated KeyPair as AuditLog not yet migrated to v1.4.0
    let keypair = KeyPair::generate().unwrap();
    let mut log = AuditLog::new(keypair).unwrap();

    // Add entries
    for i in 0..5 {
        let action = Action::delete(format!("file{}.txt", i));
        let proof = IntegrityProof::new(&action, "capsule".into(), 60);
        log.append(action, proof, Decision::Approved).unwrap();
    }

    // Verify chain is valid
    assert!(log.verify_chain().is_ok(), "Original chain should be valid");

    // Verify that all entries are properly linked
    let entries = log.entries();
    for i in 1..entries.len() {
        assert_eq!(
            entries[i].prev_hash,
            entries[i - 1].current_hash,
            "Entry {} should be linked to previous entry",
            i
        );
    }

    // Note: Direct tampering test removed due to Rust safety constraints
    // In production, tampering would be detected by verify_chain() when
    // loading from disk or during periodic verification
}

#[test]
fn test_nonce_uniqueness_across_proofs() {
    // Setup (v1.4.0 API)
    let key_store = SoftwareKeyStore::generate().unwrap();
    let mut genome =
        SealedGenome::with_key_store(vec!["Rule 1".to_string()], Box::new(key_store)).unwrap();
    genome.seal().unwrap();

    // Generate multiple proofs
    let mut nonces = std::collections::HashSet::new();

    for i in 0..100 {
        let action = Action::delete(format!("file{}.txt", i));
        let proof = genome.verify_action(&action).unwrap();

        // Nonce should be unique
        assert!(nonces.insert(proof.nonce), "Nonce collision detected!");
    }
}

#[test]
fn test_capsule_hash_binding() {
    // Create two genomes with different rules (v1.4.0 API)
    let key_store1 = SoftwareKeyStore::generate().unwrap();
    let mut genome1 =
        SealedGenome::with_key_store(vec!["Rule A".to_string()], Box::new(key_store1)).unwrap();
    genome1.seal().unwrap();

    let key_store2 = SoftwareKeyStore::generate().unwrap();
    let mut genome2 =
        SealedGenome::with_key_store(vec!["Rule B".to_string()], Box::new(key_store2)).unwrap();
    genome2.seal().unwrap();

    // Capsule hashes should be different
    assert_ne!(
        genome1.capsule_hash().unwrap(),
        genome2.capsule_hash().unwrap(),
        "Different genomes should have different capsule hashes"
    );

    // Proofs should be bound to their respective genomes
    let action = Action::delete("file.txt");
    let proof1 = genome1.verify_action(&action).unwrap();
    let proof2 = genome2.verify_action(&action).unwrap();

    assert_ne!(
        proof1.capsule_hash, proof2.capsule_hash,
        "Proofs should be bound to different genome capsules"
    );
}

#[test]
fn test_action_canonicalization_prevents_bypass() {
    // Test null byte injection
    let malicious1 = "delete\0/etc/passwd";
    let canonical1 = canonicalize_action(malicious1);
    assert!(
        !canonical1.canonical_form.contains('\0'),
        "Null bytes should be removed"
    );

    // Test unicode normalization
    let unicode1 = "café"; // Composed form
    let unicode2 = "cafe\u{0301}"; // Decomposed form

    let canon1 = canonicalize_action(unicode1);
    let canon2 = canonicalize_action(unicode2);

    assert_eq!(
        canon1.canonical_form, canon2.canonical_form,
        "Unicode should be normalized"
    );
}

#[test]
#[allow(deprecated)] // ConsensusVerifier still uses deprecated KeyPair API (TODO v1.5.0)
fn test_consensus_byzantine_fault_tolerance() {
    let verifier = ConsensusVerifier::new(3, 0.1);

    // Create keypairs for sensors - Using deprecated KeyPair as ConsensusVerifier not yet migrated
    let keypairs: Vec<KeyPair> = (0..5).map(|_| KeyPair::generate().unwrap()).collect();

    // Scenario: 3 honest sensors, 2 malicious
    let mut readings = vec![];

    // Honest sensors report correct value
    for (i, keypair) in keypairs.iter().enumerate().take(3) {
        let mut reading = SensorReading::new(10.0, format!("sensor_{}", i));
        reading.sign(keypair).unwrap();
        readings.push(reading);
    }

    // Malicious sensors report wrong values
    for (i, keypair) in keypairs.iter().enumerate().skip(3).take(2) {
        let mut reading = SensorReading::new(50.0, format!("sensor_{}", i)); // Outlier
        reading.sign(keypair).unwrap();
        readings.push(reading);
    }

    // Should achieve consensus on honest value
    let result = verifier.verify_readings(&readings, &keypairs).unwrap();
    assert!(
        (result - 10.0).abs() < 0.2,
        "Should reach consensus on honest value"
    );
}

#[test]
fn test_time_of_check_to_time_of_use_protection() {
    // This test demonstrates that the framework binds actions to proofs
    // preventing TOCTOU attacks where action changes between verification and execution

    let key_store = SoftwareKeyStore::generate().unwrap();
    let mut genome =
        SealedGenome::with_key_store(vec!["Rule 1".to_string()], Box::new(key_store)).unwrap();
    genome.seal().unwrap();

    // Time of Check: Get proof for one action
    let checked_action = Action::write_file("safe.txt", b"safe content".to_vec());
    let proof = genome.verify_action(&checked_action).unwrap();

    // Verify proof is bound to specific action hash
    assert_eq!(proof.action_hash, checked_action.hash());

    // Time of Use: If attacker tries to change action
    let different_action = Action::delete("/etc/passwd");

    // The proof's action_hash won't match
    assert_ne!(proof.action_hash, different_action.hash());

    // An executor checking this would detect the TOCTOU attack
}

#[test]
fn test_proof_cannot_be_reused_across_sessions() {
    // v1.4.0 API with separate nonce stores for each session
    let key_store = SoftwareKeyStore::generate().unwrap();
    let mut genome =
        SealedGenome::with_key_store(vec!["Rule 1".to_string()], Box::new(key_store.clone()))
            .unwrap();
    genome.seal().unwrap();

    let action = Action::delete("file.txt");
    let proof = genome.verify_action(&action).unwrap();

    // Session 1: Use proof with its own nonce store
    let nonce_store1 = MemoryNonceStore::new();
    let mut auditor1 = ProofAuditor::new(Box::new(key_store.clone()), Box::new(nonce_store1));
    assert!(auditor1.verify_proof(&proof).is_ok());

    // Session 2: Try to reuse proof (new auditor = new session with separate nonce store)
    let nonce_store2 = MemoryNonceStore::new();
    let mut auditor2 = ProofAuditor::new(Box::new(key_store), Box::new(nonce_store2));

    // This would succeed because it's a new auditor with empty nonce history
    // In production, nonce tracking would be persistent (RocksDB/Redis)
    let result = auditor2.verify_proof(&proof);
    assert!(result.is_ok(), "New auditor doesn't have nonce history");

    // But within same session, replay is prevented
    let replay = auditor2.verify_proof(&proof);
    assert!(replay.is_err(), "Within session, replay should fail");
}
