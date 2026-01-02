//! # Hope Genome v14 DIAMOND - The Impossible Made Real
//!
//! **"Where Rules Become Physics"**
//!
//! Diamond is not a security layer - it IS the physics of the system.
//! Rule violation isn't blocked - it's PHYSICALLY IMPOSSIBLE.
//!
//! ## The Three Pillars
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │                        HOPE GENOME v14 DIAMOND                              │
//! │                    "The Impossible Made Real"                               │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                                                                             │
//! │  PILLAR 1: ZERO-KNOWLEDGE PROOFS (ZK-SNARKs)                               │
//! │  ├─ Prove compliance without revealing decisions                           │
//! │  ├─ Mathematical proof attached to every output                            │
//! │  └─ Verification: O(1) time, anyone can verify                             │
//! │                                                                             │
//! │  PILLAR 2: NEURÁLIS HARD-WIRING                                            │
//! │  ├─ Rules baked into token probability space                               │
//! │  ├─ Forbidden tokens: P = 0.0 (physically impossible)                      │
//! │  └─ Like speed of light: not forbidden, IMPOSSIBLE                         │
//! │                                                                             │
//! │  PILLAR 3: FORMAL VERIFICATION                                             │
//! │  ├─ Rules → Mathematical axioms (Coq/Lean)                                 │
//! │  ├─ Code derived from proofs, not tested                                   │
//! │  └─ If proof fails → output never exists                                   │
//! │                                                                             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## v13 vs v14 Philosophy
//!
//! ```text
//! v13 TITANIUM: "Nem szabad" (Not allowed)
//!     → Watchdog BLOCKS violations
//!     → Rules are ENFORCED
//!     → Security is a GUARD
//!
//! v14 DIAMOND: "Nem lehetséges" (Not possible)
//!     → Violations CANNOT EXIST
//!     → Rules are PHYSICS
//!     → Security is REALITY
//! ```
//!
//! ## Security Guarantees
//!
//! - **Cryptographic Impossibility**: ZK proofs mathematically guarantee compliance
//! - **Physical Impossibility**: Constraint decoding makes violations non-existent
//! - **Logical Impossibility**: Formal verification proves correctness before runtime
//!
//! ---
//!
//! **Date**: 2026-01-02
//! **Version**: 14.0.0 (Diamond Edition)
//! **Authors**: Máté Róbert + Claude

pub mod constraint_decoder;
pub mod formal_spec;
pub mod zk_snark;
pub mod tee_enclave;
pub mod proof_chain;

// Re-exports
pub use constraint_decoder::{
    ConstraintDecoder,
    ForbiddenSpace,
    TokenConstraint,
    DecodingResult,
};

pub use formal_spec::{
    FormalAxiom,
    ProofDerivation,
    VerifiedCode,
    AxiomViolation,
};

pub use zk_snark::{
    DiamondProof,
    SnarkCircuit,
    ProofVerifier,
    ProvingKey,
    VerifyingKey,
};

pub use tee_enclave::{
    DiamondEnclave,
    EnclaveState,
    SealedRules,
    AttestationChain,
};

pub use proof_chain::{
    ProofChain,
    ChainedProof,
    GlobalProofRoot,
    SessionProof,
};

/// Diamond version
pub const DIAMOND_VERSION: &str = "14.0.0";

/// Diamond protocol identifier
pub const DIAMOND_PROTOCOL: &str = "HOPE-DIAMOND-v14";

/// The core principle: Rules become physics
pub const DIAMOND_PHILOSOPHY: &str =
    "A szabályszegés nem tiltott - FIZIKAILAG LEHETETLEN. \
     Rule violation is not forbidden - it's PHYSICALLY IMPOSSIBLE.";
