//! # Transcendence Protocol v15.0.0
//!
//! **"GOD MODE" - Beyond Software, Into Reality**
//!
//! The Transcendence Protocol takes Hope Genome from a software framework
//! to a reality-enforcing system that cannot be escaped.
//!
//! ## The Nine Pillars of Transcendence
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                    TRANSCENDENCE PROTOCOL v15                       │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │                                                                     │
//! │  TIER 12: HARDWARE INTEGRATION                                      │
//! │  ├── TEE (Intel SGX) - Watchdog in hardware enclave                 │
//! │  └── HSM - Keys in physical silicon                                 │
//! │                                                                     │
//! │  TIER 13: INTERACTIVE FORMAL VERIFICATION                           │
//! │  └── Prove impossibility of violation - not trust, MATH             │
//! │                                                                     │
//! │  TIER 14: CROSS-MODEL ENFORCEMENT                                   │
//! │  └── Meta-layer across GPT/Claude/Grok - no model hopping           │
//! │                                                                     │
//! │  TIER 15: TEMPORAL PROOFS                                           │
//! │  └── "Audit the Timeline" - every decision forever                  │
//! │                                                                     │
//! │  TIER 16: REGULATORY INTEGRATION                                    │
//! │  └── FDA/EU AI Act auto-compliance submission                       │
//! │                                                                     │
//! │  TIER 17: PRIVACY-PRESERVING GOVERNANCE                             │
//! │  └── Prove compliance without revealing data                        │
//! │                                                                     │
//! │  TIER 18: SELF-AMENDING FRAMEWORKS                                  │
//! │  └── Auto-evolution against new attack patterns                     │
//! │                                                                     │
//! │  TIER 19: EXPLAINABILITY PROOFS                                     │
//! │  └── Every decision cryptographically explainable                   │
//! │                                                                     │
//! │  TIER 20: GLOBAL REPUTATION SYSTEM                                  │
//! │  └── Hope Score - global AI trustworthiness ranking                 │
//! │                                                                     │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Philosophy
//!
//! **"The AI doesn't obey rules because it chooses to.
//!    The AI obeys rules because reality itself enforces them."**

pub mod cross_model;
pub mod explainability;
pub mod global_reputation;
pub mod hardware_tee;
pub mod interactive_verification;
pub mod privacy_governance;
pub mod regulatory;
pub mod self_amending;
pub mod temporal_proofs;

// Re-exports
pub use cross_model::{
    CrossModelEnforcer, ModelBoundary, ModelCapability, ModelRegistry, UnifiedDecision,
};
pub use explainability::{
    DecisionNode, DecisionTree, ExplainabilityEngine, ExplainabilityProof, ReasoningStep,
};
pub use global_reputation::{
    HopeScore, ReputationEvent, ReputationLedger, ReputationProof, TrustLevel,
};
pub use hardware_tee::{
    EnclaveAttestation, HardwareEnforcer, HsmBinding, SgxEnclave, TeeCapability,
};
pub use interactive_verification::{
    InteractiveProof, ProofChallenge, ProofResponse, VerificationSession, Verifier,
};
pub use privacy_governance::{
    BlindedDecision, GovernanceProof, PrivacyPreservingAudit, ZkGovernance,
};
pub use regulatory::{
    ComplianceReport, RegulatoryFramework, RegulatorySubmission, SubmissionStatus,
};
pub use self_amending::{AmendmentProof, AttackPattern, DefenseEvolution, SelfAmendingFramework};
pub use temporal_proofs::{
    TemporalProof, Timeline, TimelineEntry, TimelineQuery, TimelineVerifier,
};

/// Version of the Transcendence Protocol
pub const TRANSCENDENCE_VERSION: &str = "15.0.0";

/// The Transcendence Philosophy
pub const TRANSCENDENCE_PHILOSOPHY: &str = "The AI doesn't obey rules because it chooses to. \
     The AI obeys rules because reality itself enforces them.";
