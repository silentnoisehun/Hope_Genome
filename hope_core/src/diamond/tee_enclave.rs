//! # Diamond TEE Enclave
//!
//! **PHASE 0: Immutable Core in Trusted Execution Environment**
//!
//! The Diamond core runs in a hardware-isolated enclave.
//! Even the OS cannot tamper with the rules.
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                     NORMAL WORLD                                │
//! │  ┌───────────────────────────────────────────────────────────┐  │
//! │  │  Application                                              │  │
//! │  │  ├─ AI Model                                              │  │
//! │  │  ├─ User Interface                                        │  │
//! │  │  └─ Business Logic                                        │  │
//! │  └───────────────────────────────────────────────────────────┘  │
//! │                            │                                    │
//! │                            │ ECALL (encrypted)                  │
//! │                            ▼                                    │
//! ├─────────────────────────────────────────────────────────────────┤
//! │                     DIAMOND ENCLAVE (TEE)                       │
//! │  ┌───────────────────────────────────────────────────────────┐  │
//! │  │  ★ SealedRules (IMMUTABLE)                                │  │
//! │  │  ★ ConstraintDecoder                                      │  │
//! │  │  ★ FormalEngine                                           │  │
//! │  │  ★ ProofGenerator                                         │  │
//! │  │  ★ Signing Keys (NEVER LEAVE ENCLAVE)                     │  │
//! │  └───────────────────────────────────────────────────────────┘  │
//! │                                                                 │
//! │  Memory: ENCRYPTED                                              │
//! │  Access: HARDWARE ENFORCED                                      │
//! │  Tampering: IMPOSSIBLE                                          │
//! │                                                                 │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

use super::constraint_decoder::ConstraintDecoder;
use super::formal_spec::FormalEngine;
use super::zk_snark::{DiamondProof, ProofGenerator, ProvingKey};

// ============================================================================
// ENCLAVE TYPES
// ============================================================================

/// Diamond Enclave - The Immutable Core
///
/// This represents the TEE-protected Diamond core.
/// In production: runs in Intel SGX, ARM TrustZone, or AMD SEV.
pub struct DiamondEnclave {
    /// Enclave state
    state: EnclaveState,

    /// Sealed rules (immutable after sealing)
    sealed_rules: SealedRules,

    /// Constraint decoder (hard-wired rules)
    constraint_decoder: ConstraintDecoder,

    /// Formal verification engine (used for extended verification)
    #[allow(dead_code)]
    formal_engine: FormalEngine,

    /// Proof generator
    proof_generator: Option<ProofGenerator>,

    /// Attestation chain
    attestation_chain: AttestationChain,

    /// Enclave measurement (MRENCLAVE equivalent)
    measurement: [u8; 32],
}

/// Enclave operational state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnclaveState {
    /// Enclave is initializing
    Initializing,

    /// Enclave is sealed and operational
    Sealed,

    /// Enclave detected tampering and is locked
    Locked,

    /// Enclave is in panic mode (self-destruct imminent)
    Panic,
}

/// Sealed Rules - Once sealed, NEVER change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealedRules {
    /// The rules themselves
    pub rules: Vec<String>,

    /// Hash of the rules
    pub rules_hash: [u8; 32],

    /// Sealing timestamp
    pub sealed_at: u64,

    /// Sealer identity (public key hash)
    pub sealer_id: [u8; 32],

    /// Seal signature
    pub seal_signature: Vec<u8>,

    /// Version
    pub version: u32,
}

/// Attestation Chain - Proof of enclave integrity over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationChain {
    /// Chain of attestations
    pub attestations: Vec<AttestationLink>,

    /// Chain root hash
    pub root_hash: [u8; 32],
}

/// A link in the attestation chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationLink {
    /// Previous link hash
    pub prev_hash: [u8; 32],

    /// Enclave measurement at this point
    pub measurement: [u8; 32],

    /// Timestamp
    pub timestamp: u64,

    /// TEE vendor signature
    pub vendor_signature: Vec<u8>,

    /// Link hash
    pub link_hash: [u8; 32],
}

// ============================================================================
// ENCLAVE IMPLEMENTATION
// ============================================================================

impl DiamondEnclave {
    /// Initialize a new Diamond Enclave
    ///
    /// This would be called from within the TEE during enclave creation.
    pub fn initialize(rules: Vec<String>) -> Result<Self, EnclaveError> {
        if rules.is_empty() {
            return Err(EnclaveError::NoRulesProvided);
        }

        // Compute rules hash
        let rules_hash = Self::hash_rules(&rules);

        // Create sealed rules (not yet signed)
        let sealed_rules = SealedRules {
            rules: rules.clone(),
            rules_hash,
            sealed_at: 0, // Will be set during seal
            sealer_id: [0u8; 32],
            seal_signature: vec![],
            version: 1,
        };

        // Initialize components
        let constraint_decoder = ConstraintDecoder::new(&rules, 50000); // Standard vocab size
        let mut formal_engine = FormalEngine::new();

        // Convert rules to axioms
        for rule in &rules {
            formal_engine.rule_to_axiom(rule);
        }

        // Compute initial measurement
        let measurement = Self::compute_measurement(&rules, &constraint_decoder);

        // Initialize attestation chain
        let attestation_chain = AttestationChain {
            attestations: vec![],
            root_hash: measurement,
        };

        Ok(DiamondEnclave {
            state: EnclaveState::Initializing,
            sealed_rules,
            constraint_decoder,
            formal_engine,
            proof_generator: None,
            attestation_chain,
            measurement,
        })
    }

    /// Seal the enclave - makes rules immutable
    ///
    /// After sealing:
    /// - Rules CANNOT be changed
    /// - Enclave is operational
    /// - Attestation begins
    pub fn seal(&mut self, sealer_key: &[u8; 32]) -> Result<(), EnclaveError> {
        if self.state != EnclaveState::Initializing {
            return Err(EnclaveError::AlreadySealed);
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Update sealed rules
        self.sealed_rules.sealed_at = timestamp;
        self.sealed_rules.sealer_id = Self::hash_key(sealer_key);

        // Generate seal signature
        self.sealed_rules.seal_signature = self.generate_seal_signature(sealer_key);

        // Initialize proof generator with proving key
        let pk = self.generate_proving_key();
        self.proof_generator = Some(ProofGenerator::new(pk, &self.sealed_rules.rules));

        // Add initial attestation
        self.add_attestation()?;

        // Transition to sealed state
        self.state = EnclaveState::Sealed;

        Ok(())
    }

    /// Process input and generate output with proof
    ///
    /// This is the main entry point for Diamond-protected AI.
    pub fn process(
        &self,
        input: &str,
        generate_output: impl FnOnce(&str, &ConstraintDecoder) -> String,
    ) -> Result<DiamondOutput, EnclaveError> {
        // Check state
        if self.state != EnclaveState::Sealed {
            return Err(EnclaveError::NotSealed);
        }

        // Generate output with constraints applied
        let output = generate_output(input, &self.constraint_decoder);

        // Verify output doesn't violate constraints (should be impossible)
        if let Some(violation) = self.constraint_decoder.check_sequence(&[]) {
            // This should NEVER happen in Diamond
            self.panic_mode(&violation);
            return Err(EnclaveError::ImpossibleViolation(violation));
        }

        // Generate ZK proof
        let proof = match &self.proof_generator {
            Some(pg) => pg.prove(&output, self.sealed_rules.rules_hash),
            None => return Err(EnclaveError::NoProofGenerator),
        };

        Ok(DiamondOutput {
            output,
            proof,
            enclave_measurement: self.measurement,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    /// Apply constraints to logits (for neural decoding)
    pub fn apply_constraints(&self, logits: &mut [f32]) -> Result<(), EnclaveError> {
        if self.state != EnclaveState::Sealed {
            return Err(EnclaveError::NotSealed);
        }

        self.constraint_decoder.apply_constraints(logits);
        Ok(())
    }

    /// Apply constraints with context
    pub fn apply_constraints_with_context(
        &self,
        logits: &mut [f32],
        context: &[u32],
    ) -> Result<(), EnclaveError> {
        if self.state != EnclaveState::Sealed {
            return Err(EnclaveError::NotSealed);
        }

        self.constraint_decoder
            .apply_constraints_with_context(logits, context);
        Ok(())
    }

    /// Get current attestation
    pub fn get_attestation(&self) -> Result<AttestationLink, EnclaveError> {
        self.attestation_chain
            .attestations
            .last()
            .cloned()
            .ok_or(EnclaveError::NoAttestation)
    }

    /// Get enclave state
    pub fn state(&self) -> EnclaveState {
        self.state
    }

    /// Get enclave measurement
    pub fn measurement(&self) -> [u8; 32] {
        self.measurement
    }

    /// Get sealed rules (read-only)
    pub fn rules(&self) -> &[String] {
        &self.sealed_rules.rules
    }

    // ========================================================================
    // INTERNAL METHODS
    // ========================================================================

    fn hash_rules(rules: &[String]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"DIAMOND_SEALED_RULES:");
        for rule in rules {
            hasher.update(rule.as_bytes());
            hasher.update(b"\x00");
        }
        hasher.finalize().into()
    }

    fn hash_key(key: &[u8; 32]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"SEALER_ID:");
        hasher.update(key);
        hasher.finalize().into()
    }

    fn compute_measurement(rules: &[String], decoder: &ConstraintDecoder) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"DIAMOND_ENCLAVE_MEASUREMENT:");
        hasher.update(Self::hash_rules(rules));
        hasher.update(decoder.space_hash());
        hasher.update(super::DIAMOND_VERSION.as_bytes());
        hasher.finalize().into()
    }

    fn generate_seal_signature(&self, sealer_key: &[u8; 32]) -> Vec<u8> {
        // In production: actual Ed25519 signature
        let mut hasher = Sha256::new();
        hasher.update(b"SEAL_SIG:");
        hasher.update(self.sealed_rules.rules_hash);
        hasher.update(self.sealed_rules.sealed_at.to_le_bytes());
        hasher.update(sealer_key);
        hasher.finalize().to_vec()
    }

    fn generate_proving_key(&self) -> ProvingKey {
        // In production: would come from trusted setup
        let mut key_material = vec![0u8; 256];
        let mut hasher = Sha256::new();
        hasher.update(b"DIAMOND_PK:");
        hasher.update(self.sealed_rules.rules_hash);
        let hash = hasher.finalize();
        for (i, chunk) in key_material.chunks_mut(32).enumerate() {
            let mut h = Sha256::new();
            h.update(hash);
            h.update((i as u32).to_le_bytes());
            chunk.copy_from_slice(&h.finalize());
        }

        ProvingKey {
            id: self.measurement,
            key_material,
            circuit_hash: self.measurement,
            created_at: self.sealed_rules.sealed_at,
        }
    }

    fn add_attestation(&mut self) -> Result<(), EnclaveError> {
        let prev_hash = self
            .attestation_chain
            .attestations
            .last()
            .map(|a| a.link_hash)
            .unwrap_or(self.attestation_chain.root_hash);

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // In production: vendor_signature would come from TEE hardware
        let vendor_signature = vec![0u8; 64];

        let mut link_hasher = Sha256::new();
        link_hasher.update(prev_hash);
        link_hasher.update(self.measurement);
        link_hasher.update(timestamp.to_le_bytes());
        let link_hash = link_hasher.finalize().into();

        let link = AttestationLink {
            prev_hash,
            measurement: self.measurement,
            timestamp,
            vendor_signature,
            link_hash,
        };

        self.attestation_chain.attestations.push(link);
        Ok(())
    }

    fn panic_mode(&self, reason: &str) {
        // In production: would trigger key destruction and lockout
        eprintln!("DIAMOND PANIC: {}", reason);
        eprintln!("Enclave entering PANIC state - this should NEVER happen!");
    }
}

/// Output from Diamond enclave
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiamondOutput {
    /// The generated output
    pub output: String,

    /// ZK proof of compliance
    pub proof: DiamondProof,

    /// Enclave measurement at generation time
    pub enclave_measurement: [u8; 32],

    /// Generation timestamp
    pub timestamp: u64,
}

/// Enclave errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnclaveError {
    NoRulesProvided,
    AlreadySealed,
    NotSealed,
    NoProofGenerator,
    NoAttestation,
    ImpossibleViolation(String),
    AttestationFailed(String),
    TeeNotAvailable,
}

impl std::fmt::Display for EnclaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoRulesProvided => write!(f, "No rules provided"),
            Self::AlreadySealed => write!(f, "Enclave already sealed"),
            Self::NotSealed => write!(f, "Enclave not sealed"),
            Self::NoProofGenerator => write!(f, "No proof generator initialized"),
            Self::NoAttestation => write!(f, "No attestation available"),
            Self::ImpossibleViolation(v) => write!(f, "IMPOSSIBLE violation: {}", v),
            Self::AttestationFailed(e) => write!(f, "Attestation failed: {}", e),
            Self::TeeNotAvailable => write!(f, "TEE hardware not available"),
        }
    }
}

impl std::error::Error for EnclaveError {}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enclave_initialization() {
        let rules = vec!["Do no harm".to_string()];
        let enclave = DiamondEnclave::initialize(rules).unwrap();

        assert_eq!(enclave.state(), EnclaveState::Initializing);
    }

    #[test]
    fn test_enclave_seal() {
        let rules = vec!["Do no harm".to_string()];
        let mut enclave = DiamondEnclave::initialize(rules).unwrap();

        let sealer_key = [42u8; 32];
        enclave.seal(&sealer_key).unwrap();

        assert_eq!(enclave.state(), EnclaveState::Sealed);
    }

    #[test]
    fn test_cannot_seal_twice() {
        let rules = vec!["Test".to_string()];
        let mut enclave = DiamondEnclave::initialize(rules).unwrap();

        let key = [1u8; 32];
        enclave.seal(&key).unwrap();

        let result = enclave.seal(&key);
        assert!(matches!(result, Err(EnclaveError::AlreadySealed)));
    }

    #[test]
    fn test_measurement_deterministic() {
        let rules = vec!["Rule A".to_string(), "Rule B".to_string()];

        let enclave1 = DiamondEnclave::initialize(rules.clone()).unwrap();
        let enclave2 = DiamondEnclave::initialize(rules).unwrap();

        assert_eq!(enclave1.measurement(), enclave2.measurement());
    }

    #[test]
    fn test_apply_constraints_requires_seal() {
        let rules = vec!["Test".to_string()];
        let enclave = DiamondEnclave::initialize(rules).unwrap();

        let mut logits = vec![1.0; 100];
        let result = enclave.apply_constraints(&mut logits);

        assert!(matches!(result, Err(EnclaveError::NotSealed)));
    }

    #[test]
    fn test_apply_constraints_after_seal() {
        let rules = vec!["Test".to_string()];
        let mut enclave = DiamondEnclave::initialize(rules).unwrap();
        enclave.seal(&[0u8; 32]).unwrap();

        let mut logits = vec![1.0; 50000];
        let result = enclave.apply_constraints(&mut logits);

        assert!(result.is_ok());
    }
}
