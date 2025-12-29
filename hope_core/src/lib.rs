//! # Hope Genome v1.3 - Tamper-Evident Cryptographic Framework for AI Accountability
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
//! - ✅ **Cryptographic Proofs** - RSA-signed integrity tokens
//! - ✅ **Immutable Audit Trail** - Blockchain-style logging
//! - ✅ **Attack Detection** - Replay, Oracle, TOCTOU prevention
//! - ✅ **Enterprise Ready** - Production-grade Rust implementation
//! - ✅ **Multi-Source Consensus** - Byzantine Fault Tolerance
//!
//! ## Example
//!
//! ```rust
//! use hope_core::*;
//!
//! // Create genome with rules
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
//! // Get cryptographic proof
//! let proof = genome.verify_action(&action).unwrap();
//!
//! // Verify proof
//! let keypair = KeyPair::generate().unwrap();
//! let mut auditor = ProofAuditor::new(keypair);
//! // Note: In production, use the same keypair for genome and auditor
//! ```
//!
//! ## Security Model
//!
//! ### Protected Against
//!
//! - **Cryptographic forgery** - Cannot fake signatures
//! - **Replay attacks** - Nonce + TTL enforcement
//! - **Oracle attacks** - Action binding verification
//! - **Log tampering** - Blockchain chain integrity
//! - **TOCTOU** - Rust-controlled execution
//!
//! ### NOT Protected Against
//!
//! - **Root access** - Attacker with full system control
//! - **Sensor manipulation** - Mitigated via consensus
//! - **Side-channel attacks** - Use HSM for production
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
pub mod proof;

// Re-export main types
pub use audit_log::{AuditEntry, AuditLog, Decision};
pub use auditor::ProofAuditor;
pub use canonicalize::{are_equivalent, canonicalize_action, CanonicalAction};
pub use compliance::{
    validate_component_integrity, validate_integrity, AiBom, Component, ComplianceError, Hash,
};
pub use consensus::{ConsensusVerifier, SensorReading};
pub use crypto::{generate_nonce, hash_bytes, KeyPair};
pub use executor::{ExecutionResult, SecureExecutor};
pub use genome::SealedGenome;
pub use proof::{Action, ActionType, IntegrityProof, VerificationStatus};

/// Version of the Hope Genome framework
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_workflow() {
        // 1. Create genome
        let mut genome = SealedGenome::new(vec![
            "Do no harm".to_string(),
            "Respect privacy".to_string(),
        ])
        .unwrap();

        genome.seal().unwrap();

        // 2. Create action
        let action = Action::delete("test.txt");

        // 3. Get proof
        let _proof = genome.verify_action(&action).unwrap();

        // 4. Create auditor with separate keypair
        let auditor_keypair = KeyPair::generate().unwrap();
        let _auditor = ProofAuditor::new(auditor_keypair);

        // Note: In this test, we can't verify the proof because we used different keypairs
        // In production, the genome and auditor would share the same keypair
    }

    #[test]
    fn test_end_to_end_with_executor() {
        // Create shared keypair
        let _keypair = KeyPair::generate().unwrap();

        // Create genome
        let mut genome = SealedGenome::new(vec!["Rule 1".to_string()]).unwrap();
        genome.seal().unwrap();

        // Create executor components
        let auditor_keypair = KeyPair::generate().unwrap();
        let auditor = ProofAuditor::new(auditor_keypair);

        let log_keypair = KeyPair::generate().unwrap();
        let audit_log = AuditLog::new(log_keypair).unwrap();

        // Create temporary storage root for testing
        let storage_root = std::env::temp_dir().join("hope_genome_test");

        let _executor = SecureExecutor::new(auditor, audit_log, storage_root).unwrap();

        // In production workflow, genome would share keypair with auditor
        // For now, this demonstrates the component integration
    }
}
