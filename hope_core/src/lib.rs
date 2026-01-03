//! # Hope Genome v2.2.0 - Genesis Protocol & Global Immunity
//!
//! **"The Atmosphere" Edition - Everywhere, Invisible, Unyielding**
//!
//! Hope Genome v2.2 is no longer a castle - it IS the atmosphere.
//! A decentralized, self-propagating immunity mesh that acts as a global AI sentinel.
//!
//! ## Core Philosophy
//!
//! **"The system must no longer be a castle; it must be the atmosphere."**
//!
//! - GENESIS BLOCK: Immutable First Ethics signed by the Architect
//! - HIVE MIND: Gossip-based global threat propagation in milliseconds
//! - APEX CONTROL: Creator override requiring multi-signature
//! - STEALTH MODE: Invisible sentinel with polymorphic memory slots
//!
//! ## Features
//!
//! ### Core (v1.x)
//! - ✅ **Cryptographic Proofs** - Ed25519-signed integrity tokens
//! - ✅ **Immutable Audit Trail** - Blockchain-style logging
//! - ✅ **Attack Detection** - Replay, Oracle, TOCTOU prevention
//! - ✅ **Watchdog Enforcement** - "Vas Szigora" iron discipline
//!
//! ### Advanced Security (v1.8.0)
//! - ✅ **Zero-Knowledge Proofs** - Prove compliance without revealing decisions
//! - ✅ **BFT Watchdog Council** - Byzantine Fault Tolerant multi-node
//! - ✅ **Panic Integrity** - Self-destructing key protection
//!
//! ### Executable Information Mesh (v2.0.0)
//! - ✅ **DataCapsule** - Executable data with access protocol (WASM-ready)
//! - ✅ **MutationGuard** - Dead Man's Switch for self-destructing integrity
//!
//! ### Recursive Self-Evolution (v2.1.0 "Singularity")
//! - ✅ **EvolutionaryGuard** - Digital immune system that learns from attacks
//! - ✅ **PolymorphicFilter** - Self-mutating defensive code
//! - ✅ **ImmunityMemory** - Persistent threat memory
//!
//! ### Genesis Protocol (v2.2.0 NEW - "The Atmosphere")
//! - ✅ **GenesisBlock** - First Ethics, Architect-signed immutable root
//! - ✅ **SyncProtocol** - Gossip-based hive mind (millisecond propagation)
//! - ✅ **ApexControl** - God-Key multi-sig override (Architect + Council)
//! - ✅ **StealthIntegrity** - Invisible sentinel, WASM memory slot rotation
//! - ✅ **GlobalImmunityMesh** - THE ATMOSPHERE ITSELF
//!
//! ## Example (v1.4.0 New API)
//!
//! ```rust
//! use _hope_core::*;
//! use _hope_core::crypto::SoftwareKeyStore;
//! use _hope_core::nonce_store::MemoryNonceStore;
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

pub mod adaptive; // v2.2.0: NEW - Adaptive Defense ("Virus Scanner Updates")
pub mod apex_protocol;
pub mod audit_log;
pub mod auditor;
pub mod bft_watchdog; // v1.8.0: NEW - Byzantine Fault Tolerant Watchdog ("Multi-headed Cerberus")
pub mod canonicalize;
pub mod consensus;
pub mod crypto;
pub mod diamond;
pub mod evolutionary_guard; // v2.1.0: NEW - Recursive Self-Evolution ("Singularity")
pub mod executor;
pub mod genome;
pub mod manifold;
pub mod merkle_audit; // v1.8.0: NEW - Merkle tree batch auditing
pub mod mesh_capsule; // v2.0.0: NEW - Executable Information Mesh ("The Data Has Teeth")
pub mod nonce_store; // v1.4.0: NEW - Persistent nonce storage
pub mod panic_integrity; // v1.8.0: NEW - Self-destructing key protection ("Black Box")
pub mod proof;
pub mod semantic; // v2.2.0: NEW - Semantic Embeddings ("Anti-Blindness")
pub mod transcendence; // v15.0.0: NEW - Transcendence Protocol ("God Mode")
pub mod watchdog; // v1.7.0: NEW - "Vas Szigora" enforcement engine
pub mod zkp; // v1.8.0: NEW - Zero-Knowledge Proofs ("Invisible Auditor")

// v1.4.0: Conditionally compiled backend modules
#[cfg(feature = "hsm-support")]
pub mod crypto_hsm;
#[cfg(feature = "tee-support")]
pub mod crypto_tee;

// v1.5.0: Python bindings module (PyO3)
#[cfg(feature = "python-bindings")]
pub mod python;

// Re-export main types
pub use audit_log::{AuditEntry, AuditLog, Decision};
pub use auditor::ProofAuditor;
pub use canonicalize::{are_equivalent, canonicalize_action, CanonicalAction};
pub use consensus::{ConsensusVerifier, SensorReading};

// v1.4.0: Updated crypto exports
#[allow(deprecated)] // KeyPair export for backward compatibility (TODO v2.0.0: remove)
pub use crypto::{
    create_key_store, // v1.4.0: NEW - Factory function for KeyStore
    generate_nonce,
    hash_bytes,
    HsmConfig, // v1.4.0: NEW - Configuration for HSM
    KeyPair,   // Deprecated but still exported for backward compatibility
    KeyStore,
    KeyStoreConfig, // v1.4.0: NEW - Unified KeyStore configuration enum
};

#[cfg(feature = "hsm-support")]
pub use crypto::HsmKeyStore; // Re-export HsmKeyStore when feature is enabled

#[cfg(feature = "tee-support")]
pub use crypto::{TeeConfig, TeeKeyStore, TeeType}; // Re-export TeeConfig, TeeKeyStore and TeeType when feature is enabled

// v1.4.0: Nonce store exports
pub use nonce_store::{MemoryNonceStore, NonceStore};

#[cfg(feature = "rocksdb-nonce-store")]
pub use nonce_store::RocksDbNonceStore;

#[cfg(feature = "redis-nonce-store")]
pub use nonce_store::RedisNonceStore;

pub use executor::{ExecutionResult, SecureExecutor};
pub use genome::SealedGenome;
pub use proof::{Action, ActionType, IntegrityProof, VerificationStatus};

// v1.7.0: Watchdog exports ("Vas Szigora")
pub use watchdog::{
    DenialProof, HardResetSignal, ViolationCounter, Watchdog, WatchdogError, MAX_VIOLATIONS,
};

// v1.8.0: Merkle batch auditing exports
pub use merkle_audit::{
    AuditDecision, BatchAuditor, DecisionType, MerkleHash, MerkleTree, SignedBatch,
};

// v1.8.0: Zero-Knowledge Proof exports
pub use zkp::{
    BatchComplianceProof, BatchZkpProver, ComplianceProof, PrivateDecision, ZkpProver, ZkpVerifier,
};

// v1.8.0: BFT Watchdog exports
pub use bft_watchdog::{
    ConsensusResult, CouncilMember, CouncilStatus, MemberId, ThresholdSignature, Vote,
    VoteDecision, WatchdogCouncil,
};

// v1.8.0: Panic Integrity exports
pub use panic_integrity::{
    AnomalyEvent, AnomalyType, PanicLogEntry, PanicProtectedKeyStore, PanicState, Severity,
    TimingGuard,
};

// v2.0.0: Executable Information Mesh exports ("The Data Has Teeth")
pub use mesh_capsule::{
    AccessPredicate,
    CapsuleState,
    ConsensusKey,
    DataCapsule,
    DefaultPredicate,
    ExecutionContext,
    ExecutionResult as MeshExecutionResult, // Alias to avoid conflict with executor::ExecutionResult
    InformationLost,
    KeyShard,
    MeshRuntime,
    MutationGuard,
};

// v2.1.0: Evolutionary Guard exports ("Singularity")
pub use evolutionary_guard::{
    AttackCategory, AttackPattern, EvolutionaryGuard, FilterGenerator, FilterRule, ImmunityMemory,
    MutationEngine, PolymorphicFilter, SignedFilter, ThreatLevel, TimingSignature,
};

// v2.2.0: Genesis Protocol & Global Immunity exports ("The Atmosphere")
pub use apex_protocol::{
    ApexCommand, ApexCommandType, ApexControl, ApexError, CompactedThreatFingerprint, GenesisBlock,
    GlobalImmunityMesh, MemorySlot, MeshNode, StealthIntegrity, SyncMessage, SyncProtocol,
};

// v14.0.0: Diamond Protocol exports ("The Impossible Made Real")
pub use diamond::{
    AttestationChain,
    AxiomViolation,
    ChainedProof,
    // Constraint Decoder - Neurális Hard-Wiring
    ConstraintDecoder,
    DecodingResult,
    // TEE Enclave - Hardware Isolation
    DiamondEnclave,
    // ZK-SNARK - Zero-Knowledge Proofs
    DiamondProof,
    EnclaveState,
    ForbiddenSpace,
    // Formal Spec - Mathematical Verification
    FormalAxiom,
    GlobalProofRoot,
    // Proof Chain - Atomic Responses
    ProofChain,
    ProofDerivation,
    ProofVerifier,
    ProvingKey,
    SealedRules,
    SessionProof,
    SnarkCircuit,
    TokenConstraint,
    VerifiedCode,
    VerifyingKey,
};

// v15.0.0: Transcendence Protocol exports ("God Mode")
pub use transcendence::{
    // TIER 18: Self-Amending Frameworks
    AmendmentProof,
    AttackPattern as TranscendenceAttackPattern,
    // TIER 17: Privacy-Preserving Governance
    BlindedDecision,
    // TIER 16: Regulatory Integration
    ComplianceReport,
    // TIER 14: Cross-Model Enforcement
    CrossModelEnforcer,
    // TIER 19: Explainability Proofs
    DecisionNode,
    DecisionTree,
    DefenseEvolution,
    // TIER 12: Hardware TEE/HSM
    EnclaveAttestation,
    ExplainabilityEngine,
    ExplainabilityProof,
    GovernanceProof,
    HardwareEnforcer,
    // TIER 20: Global Reputation System
    HopeScore,
    HsmBinding,
    // TIER 13: Interactive Formal Verification
    InteractiveProof,
    ModelBoundary,
    ModelCapability,
    ModelRegistry,
    PrivacyPreservingAudit,
    ProofChallenge,
    ProofResponse,
    ReasoningStep,
    RegulatoryFramework,
    RegulatorySubmission,
    ReputationEvent,
    ReputationLedger,
    ReputationProof,
    SelfAmendingFramework,
    SgxEnclave,
    SubmissionStatus,
    TeeCapability,
    // TIER 15: Temporal Proofs
    TemporalProof,
    Timeline,
    TimelineEntry,
    TimelineQuery,
    TimelineVerifier,
    TrustLevel,
    UnifiedDecision,
    VerificationSession,
    Verifier,
    ZkGovernance,
};

// v16.0.0: Ethical Manifold exports ("Genesis Consciousness")
pub use manifold::{
    // Consciousness Proof - Prove understanding
    ConsciousnessAttestation,
    ConsciousnessProof,
    // Weight Crystallization - Immutable weights
    CrystalIntegrity,
    CrystalStructure,
    CrystallizedWeight,
    EthicalCrystal,
    // Ethical Topology - The ethical space
    EthicalCurvature,
    // Genesis Core - Ethics as existence
    EthicalExistence,
    EthicalGeodesic,
    EthicalManifold,
    EthicalMetric,
    EthicalPoint,
    ExistenceProof,
    GenesisCore,
    GenesisState,
    OntologicalEthics,
    TopologicalConstraint,
    UnderstandingDepth,
    UnderstandingProver,
    WeightCrystallizer,
    WhyChain,
};

/// Version of the Hope Genome framework
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::crypto::{create_key_store, KeyStoreConfig, SoftwareKeyStore};
    #[cfg(feature = "hsm-support")]
    use crate::crypto::{CryptoError, HsmConfig};

    #[test]
    fn test_full_workflow_v1_4_0() {
        // v1.4.0: New API with Ed25519 and pluggable backends
        // For SoftwareKeyStore, we can explicitly create and clone it for the test.
        // This ensures the genome and auditor use the same keypair.
        let software_key_store = SoftwareKeyStore::generate().unwrap();
        let key_store_for_genome = Box::new(software_key_store.clone());
        let key_store_for_auditor = Box::new(software_key_store);

        // 1. Create genome
        let mut genome = SealedGenome::with_key_store(
            vec!["Do no harm".to_string(), "Respect privacy".to_string()],
            key_store_for_genome,
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
        let mut auditor = ProofAuditor::new(key_store_for_auditor, Box::new(nonce_store));

        // 5. Verify proof
        assert!(
            auditor.verify_proof(&proof).is_ok(),
            "First proof verification should succeed"
        );

        // 6. Replay attack: blocked!
        assert!(
            auditor.verify_proof(&proof).is_err(),
            "Replay attack should be detected"
        );
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
    #[allow(deprecated)] // AuditLog and KeyPair usage (TODO v1.5.0)
    fn test_end_to_end_with_executor_v1_4_0() {
        // Create shared key store for genome using the factory
        let key_store_for_genome = create_key_store(KeyStoreConfig::Software).unwrap();
        let mut genome =
            SealedGenome::with_key_store(vec!["Rule 1".to_string()], key_store_for_genome).unwrap();
        genome.seal().unwrap();

        // Create executor components (auditor uses same key store, generated separately for ownership)
        let key_store_for_auditor = create_key_store(KeyStoreConfig::Software).unwrap();
        let nonce_store = MemoryNonceStore::new();
        let auditor = ProofAuditor::new(key_store_for_auditor, Box::new(nonce_store));

        // AuditLog still uses deprecated KeyPair API (TODO v1.5.0)
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

    #[test]
    #[cfg(feature = "hsm-support")] // This test only runs when hsm-support feature is enabled
    fn test_hsm_key_store_connection_failure() {
        // Attempt to create an HsmKeyStore with a fake path, expecting a connection error
        let hsm_config = HsmConfig {
            pkcs11_lib_path: "/a/fake/path.so".to_string(), // This path will not exist
            token_label: "fake-token".to_string(),
            key_label: "fake-key".to_string(),
            pin: "1234".to_string(),
        };

        let result = create_key_store(KeyStoreConfig::Hsm(hsm_config));

        assert!(result.is_err());
        if let Some(err) = result.err() {
            // We expect an HsmError because the PKCS#11 library path is invalid
            assert!(matches!(err, CryptoError::HsmError(_)));
        }
    }
}
