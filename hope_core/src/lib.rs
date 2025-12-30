//! # Hope Genome v1.4.0 - Tamper-Evident Cryptographic Framework for AI Accountability
//!
//! **Hardened Security Edition**
//!
//! Hope Genome is a framework for ensuring accountability and auditability in AI systems.
//! It provides cryptographic proofs, immutable audit trails, and multi-layer defense mechanisms.
//!
//! ## Core Philosophy
//!
//! **"Not unhackable, but tamper-evident with cryptographic proof."**
//!
//! - Attacks may succeed, but cannot be hidden
//! - All decisions are cryptographically signed
//! - Audit trail is blockchain-style (tamper-evident)
//! - Multi-source consensus prevents oracle attacks
//!
//! ## Features
//!
//! - ✅ **Cryptographic Proofs** - Ed25519-signed integrity tokens (v1.4.0: Marvin attack immune)
//! - ✅ **Immutable Audit Trail** - Blockchain-style logging
//! - ✅ **Attack Detection** - Replay, Oracle, TOCTOU prevention
//! - ✅ **Enterprise Ready** - Production-grade Rust implementation
//! - ✅ **Multi-Source Consensus** - Byzantine Fault Tolerance
//! - ✅ **Persistent Nonce Store** - Replay protection survives restarts (v1.4.0)
//! - ✅ **HSM Abstraction** - Ready for PKCS#11 hardware security modules (v1.4.0)
//!
//! ## Example (v1.4.0 New API)
//!
//! ```rust
//! use hope_core::*;
//! use hope_core::crypto::SoftwareKeyStore;
//! use hope_core::nonce_store::MemoryNonceStore;
//!
//! // Create genome with Ed25519 keys
//! let mut genome = SealedGenome::new(vec![
//!     "Do no harm".to_string(),
//!     "Respect privacy".to_string(),
//! ]).unwrap();
//!
//! // Seal it (make immutable)
//! genome.seal().unwrap();
//!
//! // Create action
//! let action = Action::delete("test.txt");
//!
//! // Get cryptographic proof (Ed25519 signed)
//! let proof = genome.verify_action(&action).unwrap();
//!
//! // Verify proof with pluggable backends
//! let key_store = SoftwareKeyStore::generate().unwrap();
//! let nonce_store = MemoryNonceStore::new();
//! let mut auditor = ProofAuditor::new(
//!     Box::new(key_store),
//!     Box::new(nonce_store),
//! );
//! ```
//!
//! ## Security Model
//!
//! ### Protected Against
//!
//! - **Cryptographic forgery** - Cannot fake Ed25519 signatures
//! - **Replay attacks** - Persistent nonce store (RocksDB/Redis support)
//! - **Oracle attacks** - Action binding verification
//! - **Log tampering** - Blockchain chain integrity
//! - **TOCTOU** - Rust-controlled execution
//! - **Marvin Attack** - Ed25519 eliminates RSA padding vulnerabilities (v1.4.0)
//!
//! ### NOT Protected Against
//!
//! - **Root access** - Attacker with full system control
//! - **Sensor manipulation** - Mitigated via consensus
//! - **Side-channel attacks** - Use HSM for production (v1.4.0: architecture ready)
//!
//! ## Major Changes in v1.4.0 (2025-12-30)
//!
//! 1. **Ed25519 Migration**: RSA-2048 → Ed25519 (eliminates Marvin attack, 100x faster)
//! 2. **Persistent Nonce Store**: RocksDB/Redis backends for replay attack protection
//! 3. **HSM Abstraction Layer**: PKCS#11 ready (architecture in place, implementation TBD)
//!
//! ## Authors
//!
//! - **Máté Róbert** - Primary Author & Architect
//! - **Claude (Anthropic)** - Technical Advisor & Co-Designer

pub mod audit_log;
pub mod auditor;
pub mod canonicalize;
pub mod compliance;
pub mod consensus;
pub mod crypto;
pub mod executor;
pub mod genome;
pub mod nonce_store; // v1.4.0: NEW - Persistent nonce storage
pub mod proof;

// Re-export main types
pub use audit_log::{AuditEntry, AuditLog, Decision};
pub use auditor::ProofAuditor;
pub use canonicalize::{are_equivalent, canonicalize_action, CanonicalAction};
pub use compliance::{
    validate_component_integrity, validate_integrity, AiBom, Component, ComplianceError, Hash,
};
pub use consensus::{ConsensusVerifier, SensorReading};

// v1.4.0: Updated crypto exports
pub use crypto::{
    generate_nonce, hash_bytes,
    KeyPair, // Deprecated but still exported for backward compatibility
    KeyStore, SoftwareKeyStore, // v1.4.0: NEW - Trait-based key management
};

// v1.4.0: Nonce store exports
pub use nonce_store::{MemoryNonceStore, NonceStore};

#[cfg(feature = "rocksdb-nonce-store")]
pub use nonce_store::RocksDbNonceStore;

#[cfg(feature = "redis-nonce-store")]
pub use nonce_store::RedisNonceStore;

pub use executor::{ExecutionResult, SecureExecutor};
pub use genome::SealedGenome;
pub use proof::{Action, ActionType, IntegrityProof, VerificationStatus};

/// Version of the Hope Genome framework
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_workflow_v1_4_0() {
        // v1.4.0: New API with Ed25519 and pluggable backends
        let key_store = SoftwareKeyStore::generate().unwrap();
        let key_store_clone = key_store.clone();

        // 1. Create genome
        let mut genome = SealedGenome::with_key_store(
            vec!["Do no harm".to_string(), "Respect privacy".to_string()],
            Box::new(key_store),
        )
        .unwrap();

        genome.seal().unwrap();

        // 2. Create action
        let action = Action::delete("test.txt");

        // 3. Get proof (Ed25519 signed)
        let proof = genome.verify_action(&action).unwrap();
        assert_eq!(proof.signature.len(), 64); // Ed25519 signature

        // 4. Create auditor with same key store and memory nonce store
        let nonce_store = MemoryNonceStore::new();
        let mut auditor = ProofAuditor::new(Box::new(key_store_clone), Box::new(nonce_store));

        // 5. Verify proof
        assert!(auditor.verify_proof(&proof).is_ok());

        // 6. Replay attack: blocked!
        assert!(auditor.verify_proof(&proof).is_err());
    }

    #[test]
    #[allow(deprecated)]
    fn test_backward_compatibility_workflow() {
        // v1.3.0 style code should still work
        let mut genome = SealedGenome::new(vec![
            "Do no harm".to_string(),
            "Respect privacy".to_string(),
        ])
        .unwrap();

        genome.seal().unwrap();

        let action = Action::delete("test.txt");
        let _proof = genome.verify_action(&action).unwrap();

        // Note: ProofAuditor API changed in v1.4.0, so we can't test
        // the old auditor here, but genome creation still works
    }

    #[test]
    fn test_end_to_end_with_executor_v1_4_0() {
        // Create shared key store
        let key_store = SoftwareKeyStore::generate().unwrap();

        // Create genome
        let mut genome = SealedGenome::new(vec!["Rule 1".to_string()]).unwrap();
        genome.seal().unwrap();

        // Create executor components (still use old API for now)
        #[allow(deprecated)]
        let auditor_keypair = KeyPair::generate().unwrap();
        let nonce_store = MemoryNonceStore::new();
        let auditor = ProofAuditor::new(Box::new(auditor_keypair), Box::new(nonce_store));

        #[allow(deprecated)]
        let log_keypair = KeyPair::generate().unwrap();
        let audit_log = AuditLog::new(log_keypair).unwrap();

        // Create temporary storage root for testing
        let storage_root = std::env::temp_dir().join("hope_genome_test_v1_4_0");

        let _executor = SecureExecutor::new(auditor, audit_log, storage_root).unwrap();

        // In production workflow, genome would share keypair with auditor
        // For now, this demonstrates the component integration
    }

    #[test]
    fn test_signature_size_reduced() {
        // v1.4.0: Ed25519 signatures are 64 bytes (vs RSA-2048 ~256 bytes)
        let mut genome = SealedGenome::new(vec!["Rule 1".to_string()]).unwrap();
        genome.seal().unwrap();

        let action = Action::delete("test.txt");
        let proof = genome.verify_action(&action).unwrap();

        assert_eq!(proof.signature.len(), 64); // Ed25519
        // Old RSA-2048 would be ~256 bytes
    }

    #[test]
    fn test_public_key_size_reduced() {
        // v1.4.0: Ed25519 public keys are 32 bytes (vs RSA-2048 ~256 bytes)
        let genome = SealedGenome::new(vec!["Rule 1".to_string()]).unwrap();
        let public_key = genome.public_key_bytes();

        assert_eq!(public_key.len(), 32); // Ed25519
    }
}
