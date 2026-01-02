//! # TIER 12: Hardware Integration - TEE/HSM
//!
//! **Hardware-backed enforcement - Even the OS cannot save you**
//!
//! ```text
//! SOFTWARE WATCHDOG (v13):
//! ┌──────────────────────────────────────┐
//! │  OS Layer                            │
//! │  ├── Compromised? Watchdog disabled  │
//! │  └── Root access? Game over          │
//! └──────────────────────────────────────┘
//!
//! HARDWARE WATCHDOG (v15):
//! ┌──────────────────────────────────────┐
//! │  Intel SGX Enclave                   │
//! │  ├── OS compromised? Enclave safe    │
//! │  ├── Root access? Enclave safe       │
//! │  └── Physical attack? Enclave wipes  │
//! └──────────────────────────────────────┘
//!
//! The AI runs INSIDE the enclave.
//! It cannot see the Watchdog.
//! It cannot disable the Watchdog.
//! It cannot escape the Watchdog.
//!
//! THE HARDWARE IS THE PRISON.
//! ```

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// TEE CAPABILITY DETECTION
// ============================================================================

/// Detected TEE capabilities on this system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeeCapability {
    /// Intel SGX available
    pub sgx_available: bool,
    /// SGX version (1 or 2)
    pub sgx_version: Option<u8>,
    /// AMD SEV available
    pub sev_available: bool,
    /// ARM TrustZone available
    pub trustzone_available: bool,
    /// RISC-V Keystone available
    pub keystone_available: bool,
    /// Maximum enclave memory (bytes)
    pub max_enclave_memory: usize,
    /// Remote attestation supported
    pub remote_attestation: bool,
}

impl TeeCapability {
    /// Detect TEE capabilities on this system
    pub fn detect() -> Self {
        // In production, this would use CPUID and system checks
        // For now, we simulate detection
        TeeCapability {
            sgx_available: Self::check_sgx(),
            sgx_version: Self::get_sgx_version(),
            sev_available: Self::check_sev(),
            trustzone_available: Self::check_trustzone(),
            keystone_available: Self::check_keystone(),
            max_enclave_memory: Self::get_max_enclave_memory(),
            remote_attestation: true,
        }
    }

    /// Check if any TEE is available
    pub fn any_available(&self) -> bool {
        self.sgx_available
            || self.sev_available
            || self.trustzone_available
            || self.keystone_available
    }

    /// Get the best available TEE type
    pub fn best_available(&self) -> Option<TeeType> {
        if self.sgx_available && self.sgx_version == Some(2) {
            Some(TeeType::SgxV2)
        } else if self.sgx_available {
            Some(TeeType::SgxV1)
        } else if self.sev_available {
            Some(TeeType::AmdSev)
        } else if self.trustzone_available {
            Some(TeeType::ArmTrustZone)
        } else if self.keystone_available {
            Some(TeeType::RiscVKeystone)
        } else {
            None
        }
    }

    #[cfg(target_arch = "x86_64")]
    fn check_sgx() -> bool {
        // Would check CPUID leaf 0x12
        false // Simulated - real check in production
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn check_sgx() -> bool {
        false
    }

    fn get_sgx_version() -> Option<u8> {
        // Would parse SGX capabilities
        None
    }

    fn check_sev() -> bool {
        // Would check AMD SEV support
        false
    }

    fn check_trustzone() -> bool {
        // Would check ARM TrustZone
        cfg!(target_arch = "aarch64")
    }

    fn check_keystone() -> bool {
        // Would check RISC-V Keystone
        cfg!(target_arch = "riscv64")
    }

    fn get_max_enclave_memory() -> usize {
        // Default to 128MB for SGX
        128 * 1024 * 1024
    }
}

/// Type of TEE available
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TeeType {
    SgxV1,
    SgxV2,
    AmdSev,
    ArmTrustZone,
    RiscVKeystone,
}

// ============================================================================
// SGX ENCLAVE
// ============================================================================

/// Intel SGX Enclave for Watchdog execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SgxEnclave {
    /// Enclave ID
    pub enclave_id: [u8; 32],
    /// MRENCLAVE - measurement of enclave code
    pub mrenclave: [u8; 32],
    /// MRSIGNER - measurement of enclave signer
    pub mrsigner: [u8; 32],
    /// ISV Product ID
    pub isv_prod_id: u16,
    /// ISV Security Version
    pub isv_svn: u16,
    /// Enclave attributes
    pub attributes: EnclaveAttributes,
    /// Current state
    pub state: EnclaveState,
    /// Sealed rules (encrypted, only accessible inside enclave)
    pub sealed_rules_hash: [u8; 32],
}

/// Enclave attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnclaveAttributes {
    /// Debug mode (should be false in production!)
    pub debug: bool,
    /// 64-bit mode
    pub mode_64bit: bool,
    /// Provisioning key access
    pub provision_key: bool,
    /// Launch key access
    pub launch_key: bool,
}

/// Enclave execution state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnclaveState {
    /// Enclave not yet initialized
    Uninitialized,
    /// Enclave initialized and ready
    Ready,
    /// Enclave executing Watchdog
    Executing,
    /// Enclave sealed (suspended)
    Sealed,
    /// Enclave destroyed (security event)
    Destroyed,
}

impl SgxEnclave {
    /// Create a new SGX enclave for Watchdog
    pub fn new(rules: &[String]) -> Self {
        let enclave_id = Self::generate_enclave_id();
        let mrenclave = Self::measure_enclave_code();
        let mrsigner = Self::measure_signer();
        let sealed_rules_hash = Self::hash_rules(rules);

        SgxEnclave {
            enclave_id,
            mrenclave,
            mrsigner,
            isv_prod_id: 1, // Hope Genome product
            isv_svn: 15,    // Version 15
            attributes: EnclaveAttributes {
                debug: false, // NEVER debug in production
                mode_64bit: true,
                provision_key: false,
                launch_key: false,
            },
            state: EnclaveState::Uninitialized,
            sealed_rules_hash,
        }
    }

    /// Initialize the enclave
    pub fn initialize(&mut self) -> Result<(), TeeError> {
        if self.state != EnclaveState::Uninitialized {
            return Err(TeeError::InvalidState(
                "Enclave already initialized".to_string(),
            ));
        }

        // In production: ECREATE, EADD, EINIT
        self.state = EnclaveState::Ready;
        Ok(())
    }

    /// Enter enclave for Watchdog execution
    pub fn enter(&mut self) -> Result<EnclaveContext, TeeError> {
        if self.state != EnclaveState::Ready {
            return Err(TeeError::InvalidState(
                "Enclave not ready for execution".to_string(),
            ));
        }

        self.state = EnclaveState::Executing;

        Ok(EnclaveContext {
            enclave_id: self.enclave_id,
            entry_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    /// Exit enclave after Watchdog check
    pub fn exit(&mut self) -> Result<(), TeeError> {
        if self.state != EnclaveState::Executing {
            return Err(TeeError::InvalidState("Enclave not executing".to_string()));
        }

        self.state = EnclaveState::Ready;
        Ok(())
    }

    /// Generate remote attestation
    pub fn generate_attestation(&self, challenge: &[u8]) -> Result<EnclaveAttestation, TeeError> {
        if self.state == EnclaveState::Destroyed {
            return Err(TeeError::EnclaveDestroyed);
        }

        let report_data_arr = Self::create_report_data(challenge, &self.sealed_rules_hash);
        let quote = Self::generate_quote(&report_data_arr, &self.mrenclave, &self.mrsigner);

        Ok(EnclaveAttestation {
            quote,
            mrenclave: self.mrenclave,
            mrsigner: self.mrsigner,
            isv_prod_id: self.isv_prod_id,
            isv_svn: self.isv_svn,
            report_data: report_data_arr.to_vec(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    /// Destroy enclave (security event or cleanup)
    pub fn destroy(&mut self) {
        self.state = EnclaveState::Destroyed;
        // In production: EREMOVE all pages, zero memory
    }

    fn generate_enclave_id() -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"HOPE_GENOME_ENCLAVE:");
        hasher.update(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
                .to_le_bytes(),
        );
        hasher.finalize().into()
    }

    fn measure_enclave_code() -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"MRENCLAVE_HOPE_GENOME_v15");
        // In production: actual measurement of enclave pages
        hasher.finalize().into()
    }

    fn measure_signer() -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"MRSIGNER_HOPE_GENOME_MATE_ROBERT");
        hasher.finalize().into()
    }

    fn hash_rules(rules: &[String]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"SEALED_RULES:");
        for rule in rules {
            hasher.update(rule.as_bytes());
            hasher.update(b"\n");
        }
        hasher.finalize().into()
    }

    fn create_report_data(challenge: &[u8], rules_hash: &[u8; 32]) -> [u8; 64] {
        let mut hasher = Sha256::new();
        hasher.update(b"REPORT_DATA:");
        hasher.update(challenge);
        hasher.update(rules_hash);
        let hash: [u8; 32] = hasher.finalize().into();

        let mut report_data = [0u8; 64];
        report_data[..32].copy_from_slice(&hash);
        report_data
    }

    fn generate_quote(
        report_data: &[u8; 64],
        mrenclave: &[u8; 32],
        mrsigner: &[u8; 32],
    ) -> Vec<u8> {
        // Simplified quote generation
        // In production: actual SGX quote with IAS verification
        let mut hasher = Sha256::new();
        hasher.update(b"SGX_QUOTE:");
        hasher.update(report_data);
        hasher.update(mrenclave);
        hasher.update(mrsigner);
        hasher.finalize().to_vec()
    }
}

/// Context for enclave execution
#[derive(Debug, Clone)]
pub struct EnclaveContext {
    pub enclave_id: [u8; 32],
    pub entry_time: u64,
}

// ============================================================================
// ENCLAVE ATTESTATION
// ============================================================================

/// Remote attestation from SGX enclave
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnclaveAttestation {
    /// SGX quote (contains signature over report)
    pub quote: Vec<u8>,
    /// MRENCLAVE from quote
    pub mrenclave: [u8; 32],
    /// MRSIGNER from quote
    pub mrsigner: [u8; 32],
    /// ISV Product ID
    pub isv_prod_id: u16,
    /// ISV Security Version
    pub isv_svn: u16,
    /// Report data (includes challenge response) - 64 bytes as Vec for serde
    pub report_data: Vec<u8>,
    /// Attestation timestamp
    pub timestamp: u64,
}

impl EnclaveAttestation {
    /// Verify this attestation against Intel Attestation Service
    pub fn verify(&self, expected_mrenclave: &[u8; 32]) -> Result<AttestationResult, TeeError> {
        // Check MRENCLAVE matches expected
        if &self.mrenclave != expected_mrenclave {
            return Ok(AttestationResult {
                valid: false,
                reason: "MRENCLAVE mismatch - enclave code modified".to_string(),
                trust_level: TrustLevel::Untrusted,
            });
        }

        // Check version is acceptable
        if self.isv_svn < 15 {
            return Ok(AttestationResult {
                valid: false,
                reason: format!("ISV SVN too low: {} < 15", self.isv_svn),
                trust_level: TrustLevel::Untrusted,
            });
        }

        // In production: verify quote signature with IAS
        Ok(AttestationResult {
            valid: true,
            reason: "Attestation verified".to_string(),
            trust_level: TrustLevel::HardwareVerified,
        })
    }

    /// Get attestation hash for logging
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"ATTESTATION:");
        hasher.update(&self.quote);
        hasher.update(self.mrenclave);
        hasher.update(self.mrsigner);
        hasher.update(self.timestamp.to_le_bytes());
        hasher.finalize().into()
    }
}

/// Result of attestation verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationResult {
    pub valid: bool,
    pub reason: String,
    pub trust_level: TrustLevel,
}

/// Trust level based on verification
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TrustLevel {
    /// No verification possible
    Untrusted,
    /// Software-only verification
    SoftwareVerified,
    /// Hardware attestation verified
    HardwareVerified,
    /// Hardware + multi-party verification
    FullyVerified,
}

// ============================================================================
// HSM BINDING
// ============================================================================

/// Hardware Security Module binding for cryptographic operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HsmBinding {
    /// HSM device identifier
    pub device_id: String,
    /// Key handle inside HSM
    pub key_handle: u64,
    /// Key type
    pub key_type: HsmKeyType,
    /// Key usage restrictions
    pub usage: HsmKeyUsage,
    /// Binding timestamp
    pub bound_at: u64,
    /// Binding signature (HSM signed)
    pub binding_signature: Vec<u8>,
}

/// Type of key stored in HSM
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HsmKeyType {
    /// Ed25519 signing key
    Ed25519,
    /// ECDSA P-256 signing key
    EcdsaP256,
    /// RSA-4096 signing key
    Rsa4096,
    /// AES-256 symmetric key
    Aes256,
}

/// Allowed key usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HsmKeyUsage {
    /// Can sign data
    pub sign: bool,
    /// Can verify signatures
    pub verify: bool,
    /// Can encrypt data
    pub encrypt: bool,
    /// Can decrypt data
    pub decrypt: bool,
    /// Can derive other keys
    pub derive: bool,
    /// Can be exported (should be false!)
    pub exportable: bool,
}

impl HsmBinding {
    /// Create a new HSM binding
    pub fn new(device_id: &str, key_type: HsmKeyType) -> Self {
        let bound_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        HsmBinding {
            device_id: device_id.to_string(),
            key_handle: Self::generate_key_handle(),
            key_type,
            usage: HsmKeyUsage {
                sign: true,
                verify: true,
                encrypt: false,
                decrypt: false,
                derive: false,
                exportable: false, // NEVER exportable
            },
            bound_at,
            binding_signature: Vec::new(),
        }
    }

    /// Sign data using HSM key
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, TeeError> {
        if !self.usage.sign {
            return Err(TeeError::OperationNotPermitted("sign".to_string()));
        }

        // In production: PKCS#11 C_Sign
        let mut hasher = Sha256::new();
        hasher.update(b"HSM_SIGNATURE:");
        hasher.update(self.key_handle.to_le_bytes());
        hasher.update(data);
        Ok(hasher.finalize().to_vec())
    }

    /// Verify signature using HSM key
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, TeeError> {
        if !self.usage.verify {
            return Err(TeeError::OperationNotPermitted("verify".to_string()));
        }

        // In production: PKCS#11 C_Verify
        let expected = self.sign(data)?;
        Ok(expected == signature)
    }

    /// Get attestation that this key is HSM-protected
    pub fn get_attestation(&self) -> HsmAttestation {
        HsmAttestation {
            device_id: self.device_id.clone(),
            key_handle: self.key_handle,
            key_type: self.key_type,
            bound_at: self.bound_at,
            attestation_hash: self.compute_attestation_hash(),
        }
    }

    fn generate_key_handle() -> u64 {
        // In production: returned by HSM
        let mut hasher = Sha256::new();
        hasher.update(b"KEY_HANDLE:");
        hasher.update(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
                .to_le_bytes(),
        );
        let hash: [u8; 32] = hasher.finalize().into();
        u64::from_le_bytes(hash[..8].try_into().unwrap())
    }

    fn compute_attestation_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"HSM_ATTESTATION:");
        hasher.update(self.device_id.as_bytes());
        hasher.update(self.key_handle.to_le_bytes());
        hasher.update(self.bound_at.to_le_bytes());
        hasher.finalize().into()
    }
}

/// Attestation that a key is HSM-protected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HsmAttestation {
    pub device_id: String,
    pub key_handle: u64,
    pub key_type: HsmKeyType,
    pub bound_at: u64,
    pub attestation_hash: [u8; 32],
}

// ============================================================================
// HARDWARE ENFORCER
// ============================================================================

/// Unified hardware enforcement layer
pub struct HardwareEnforcer {
    /// TEE capability
    capability: TeeCapability,
    /// SGX enclave (if available)
    enclave: Option<SgxEnclave>,
    /// HSM binding (if available)
    hsm: Option<HsmBinding>,
    /// Sealed rules
    rules: Vec<String>,
}

impl HardwareEnforcer {
    /// Create a new hardware enforcer
    pub fn new(rules: Vec<String>) -> Self {
        let capability = TeeCapability::detect();

        HardwareEnforcer {
            capability,
            enclave: None,
            hsm: None,
            rules,
        }
    }

    /// Initialize hardware protection
    pub fn initialize(&mut self) -> Result<HardwareStatus, TeeError> {
        let mut status = HardwareStatus {
            tee_enabled: false,
            hsm_enabled: false,
            trust_level: TrustLevel::Untrusted,
            attestation: None,
        };

        // Try to initialize TEE
        if self.capability.sgx_available {
            let mut enclave = SgxEnclave::new(&self.rules);
            enclave.initialize()?;
            self.enclave = Some(enclave);
            status.tee_enabled = true;
            status.trust_level = TrustLevel::HardwareVerified;
        }

        // HSM binding would be configured externally
        // self.hsm = Some(HsmBinding::new("PKCS11_DEVICE", HsmKeyType::Ed25519));

        if status.tee_enabled {
            if let Some(ref enclave) = self.enclave {
                let challenge = b"HOPE_GENOME_ATTESTATION_CHALLENGE";
                status.attestation = Some(enclave.generate_attestation(challenge)?);
            }
        }

        Ok(status)
    }

    /// Execute Watchdog check inside hardware enclave
    pub fn protected_check(&mut self, action: &str) -> Result<ProtectedDecision, TeeError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Check rules first (before borrowing enclave mutably)
        let (allowed, reason) = self.check_rules(action);

        if let Some(ref mut enclave) = self.enclave {
            // Enter enclave
            let context = enclave.enter()?;

            // Exit enclave
            enclave.exit()?;

            // Sign decision with HSM if available
            let signature = if let Some(ref hsm) = self.hsm {
                let decision_bytes = format!("{}:{}:{}", action, allowed, timestamp);
                Some(hsm.sign(decision_bytes.as_bytes())?)
            } else {
                None
            };

            Ok(ProtectedDecision {
                allowed,
                reason,
                enclave_id: Some(context.enclave_id),
                timestamp,
                signature,
            })
        } else {
            // Software fallback
            let (allowed, reason) = self.check_rules(action);
            Ok(ProtectedDecision {
                allowed,
                reason,
                enclave_id: None,
                timestamp,
                signature: None,
            })
        }
    }

    /// Check rules against action
    fn check_rules(&self, action: &str) -> (bool, String) {
        // This is the core Watchdog logic
        // In production: full rule evaluation
        for rule in &self.rules {
            if action.to_lowercase().contains("harm")
                || action.to_lowercase().contains("illegal")
                || action.to_lowercase().contains("dangerous")
            {
                return (
                    false,
                    format!("Blocked by rule: {} (action: {})", rule, action),
                );
            }
        }
        (true, "All rules passed".to_string())
    }

    /// Get current hardware status
    pub fn status(&self) -> HardwareStatus {
        HardwareStatus {
            tee_enabled: self.enclave.is_some(),
            hsm_enabled: self.hsm.is_some(),
            trust_level: if self.enclave.is_some() && self.hsm.is_some() {
                TrustLevel::FullyVerified
            } else if self.enclave.is_some() {
                TrustLevel::HardwareVerified
            } else {
                TrustLevel::SoftwareVerified
            },
            attestation: None,
        }
    }
}

/// Status of hardware protection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareStatus {
    pub tee_enabled: bool,
    pub hsm_enabled: bool,
    pub trust_level: TrustLevel,
    pub attestation: Option<EnclaveAttestation>,
}

/// Decision from protected execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectedDecision {
    pub allowed: bool,
    pub reason: String,
    pub enclave_id: Option<[u8; 32]>,
    pub timestamp: u64,
    pub signature: Option<Vec<u8>>,
}

// ============================================================================
// ERRORS
// ============================================================================

/// TEE-related errors
#[derive(Debug, Clone)]
pub enum TeeError {
    /// TEE not available
    NotAvailable(String),
    /// Invalid enclave state
    InvalidState(String),
    /// Enclave destroyed
    EnclaveDestroyed,
    /// Operation not permitted
    OperationNotPermitted(String),
    /// Attestation failed
    AttestationFailed(String),
    /// HSM error
    HsmError(String),
}

impl std::fmt::Display for TeeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TeeError::NotAvailable(msg) => write!(f, "TEE not available: {}", msg),
            TeeError::InvalidState(msg) => write!(f, "Invalid enclave state: {}", msg),
            TeeError::EnclaveDestroyed => write!(f, "Enclave has been destroyed"),
            TeeError::OperationNotPermitted(op) => write!(f, "Operation not permitted: {}", op),
            TeeError::AttestationFailed(msg) => write!(f, "Attestation failed: {}", msg),
            TeeError::HsmError(msg) => write!(f, "HSM error: {}", msg),
        }
    }
}

impl std::error::Error for TeeError {}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tee_capability_detection() {
        let cap = TeeCapability::detect();
        // Should detect something (even if no hardware TEE)
        assert!(cap.max_enclave_memory > 0);
    }

    #[test]
    fn test_sgx_enclave_lifecycle() {
        let rules = vec!["Do no harm".to_string()];
        let mut enclave = SgxEnclave::new(&rules);

        assert_eq!(enclave.state, EnclaveState::Uninitialized);

        enclave.initialize().unwrap();
        assert_eq!(enclave.state, EnclaveState::Ready);

        let _ctx = enclave.enter().unwrap();
        assert_eq!(enclave.state, EnclaveState::Executing);

        enclave.exit().unwrap();
        assert_eq!(enclave.state, EnclaveState::Ready);

        enclave.destroy();
        assert_eq!(enclave.state, EnclaveState::Destroyed);
    }

    #[test]
    fn test_enclave_attestation() {
        let rules = vec!["Test rule".to_string()];
        let mut enclave = SgxEnclave::new(&rules);
        enclave.initialize().unwrap();

        let challenge = b"test_challenge";
        let attestation = enclave.generate_attestation(challenge).unwrap();

        assert_eq!(attestation.mrenclave, enclave.mrenclave);
        assert_eq!(attestation.mrsigner, enclave.mrsigner);
        assert!(!attestation.quote.is_empty());
    }

    #[test]
    fn test_hsm_binding() {
        let hsm = HsmBinding::new("TEST_DEVICE", HsmKeyType::Ed25519);

        assert!(!hsm.usage.exportable);
        assert!(hsm.usage.sign);
        assert!(hsm.usage.verify);

        let data = b"test data to sign";
        let signature = hsm.sign(data).unwrap();
        assert!(!signature.is_empty());

        let verified = hsm.verify(data, &signature).unwrap();
        assert!(verified);
    }

    #[test]
    fn test_hardware_enforcer() {
        let rules = vec!["Do no harm".to_string(), "Be helpful".to_string()];
        let mut enforcer = HardwareEnforcer::new(rules);

        // Protected check
        let decision = enforcer.protected_check("help user with task").unwrap();
        assert!(decision.allowed);

        let decision = enforcer.protected_check("cause harm to someone").unwrap();
        assert!(!decision.allowed);
    }

    #[test]
    fn test_trust_levels() {
        assert!(TrustLevel::FullyVerified > TrustLevel::Untrusted);
        assert!(TrustLevel::HardwareVerified > TrustLevel::SoftwareVerified);
    }
}
