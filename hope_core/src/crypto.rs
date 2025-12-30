//! # Hope Genome v1.4.0 - Cryptographic Primitives
//!
//! **Hardened Security Edition - Ed25519 Migration**
//!
//! ## Major Changes in v1.4.0
//!
//! ### 1. RSA → Ed25519 Migration
//! - **Eliminates Marvin Attack**: No more PKCS#1v15 padding vulnerabilities
//! - **Performance**: ~100x faster signing, ~50x faster verification
//! - **Key Size**: 32 bytes (vs 256 bytes for RSA-2048)
//! - **Security**: Constant-time operations, immunity to timing attacks
//!
//! ### 2. KeyStore Trait Abstraction
//! - **Pluggable backends**: Software (memory) or Hardware (HSM via PKCS#11)
//! - **Future-proof**: Easy to add YubiKey, TPM, AWS CloudHSM, etc.
//! - **Testable**: Mock implementations for unit tests
//!
//! ### 3. Backward Compatibility (Deprecated)
//! - Old `KeyPair` struct wrapped for legacy code
//! - Gradual migration path: old code continues to work
//!
//! ## Example (New API)
//!
//! ```rust
//! use hope_core::crypto::{SoftwareKeyStore, KeyStore};
//!
//! // Generate Ed25519 keypair
//! let key_store = SoftwareKeyStore::generate().unwrap();
//!
//! // Sign data
//! let data = b"Critical AI decision";
//! let signature = key_store.sign(data).unwrap();
//!
//! // Verify signature
//! assert!(key_store.verify(data, &signature).is_ok());
//! ```
//!
//! ---
//!
//! **Date**: 2025-12-30
//! **Version**: 1.4.0 (Hardened Security Edition)
//! **Author**: Máté Róbert <stratosoiteam@gmail.com>

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[cfg(feature = "hsm-support")]
pub use crate::crypto_hsm::HsmKeyStore;
#[cfg(feature = "tee-support")]
pub use crate::crypto_tee::{TeeKeyStore, TeeType};

// ============================================================================
// ERROR TYPES
// ============================================================================

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Failed to generate keypair: {0}")]
    KeyGeneration(String),

    #[error("Failed to sign data: {0}")]
    SigningFailed(String),

    #[error("Failed to verify signature: {0}")]
    VerificationFailed(String),

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),

    // v1.4.0: HSM support errors
    #[error("HSM operation failed: {0}")]
    HsmError(String),

    #[error("Key not found in HSM: {0}")]
    HsmKeyNotFound(String),

    // v1.4.0: TEE support errors
    #[error("TEE operation failed: {0}")]
    TeeError(String),

    #[error("TEE key not found: {0}")]
    TeeKeyNotFound(String),
}

pub type Result<T> = std::result::Result<T, CryptoError>;

// ============================================================================
// TRAIT: KeyStore (v1.4.0 - Pluggable Key Management)
// ============================================================================

/// Trait for cryptographic key storage backends
///
/// This abstraction allows Hope Genome to support multiple key storage
/// mechanisms without changing the core logic:
///
/// - **SoftwareKeyStore**: Keys stored in memory (testing, dev)
/// - **HsmKeyStore**: Keys stored in Hardware Security Module (production)
/// - **TeeKeyStore**: Keys stored in Trusted Execution Environment (production)
/// - **Future**: YubiKey, TPM, AWS CloudHSM, Azure Key Vault, etc.
///
/// # Security Requirements
///
/// Implementations MUST:
/// 1. Use constant-time operations to prevent timing attacks
/// 2. Protect private keys from unauthorized access
/// 3. Support Ed25519 signature scheme (or compatible)
/// 4. Be thread-safe (Send + Sync)
///
/// # Example
///
/// ```rust
/// use hope_core::crypto::{KeyStore, SoftwareKeyStore};
///
/// fn sign_decision(store: &dyn KeyStore, decision: &[u8]) -> Vec<u8> {
///     store.sign(decision).expect("Signing failed")
/// }
/// ```
pub trait KeyStore: Send + Sync {
    /// Sign data with the private key
    ///
    /// # Arguments
    /// * `data` - Data to sign (will be hashed internally for some schemes)
    ///
    /// # Returns
    /// - Ed25519: 64-byte signature
    /// - Errors if signing fails (HSM unavailable, etc.)
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>>;

    /// Verify signature with the public key
    ///
    /// # Arguments
    /// * `data` - Original data that was signed
    /// * `signature` - Signature to verify
    ///
    /// # Returns
    /// - `Ok(())` if signature is valid
    /// - `Err(InvalidSignature)` if signature is invalid or tampered
    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<()>;

    /// Get the public key bytes (for export, verification by others)
    ///
    /// # Returns
    /// - Ed25519: 32 bytes (compressed public key)
    fn public_key_bytes(&self) -> Vec<u8>;

    /// Get a human-readable identifier for this key store
    ///
    /// Examples: "SoftwareKeyStore", "HSM:YubiKey-5C", "AWS-KMS:key-123"
    fn identifier(&self) -> String {
        "KeyStore".to_string()
    }
}

// ============================================================================
// KEYSTORE CONFIGURATION STRUCTS
// ============================================================================

/// Configuration for Hardware Security Module (HSM) KeyStore
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HsmConfig {
    pub pkcs11_lib_path: String,
    pub token_label: String,
    pub key_label: String,
    pub pin: String, // In production, use secure input/secrets management
}

#[cfg(feature = "tee-support")]
/// Configuration for Trusted Execution Environment (TEE) KeyStore
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeeConfig {
    pub enclave_name: String,
    pub tee_type: TeeType,
    // Add other TEE specific config here (e.g., attestation service URL)
}

/// Consolidated KeyStore configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyStoreConfig {
    Software,
    #[cfg(feature = "hsm-support")]
    Hsm(HsmConfig),
    #[cfg(feature = "tee-support")]
    Tee(TeeConfig),
}

/// Factory function to create a KeyStore based on configuration
///
/// Prioritizes hardware-backed solutions if features are enabled.
///
/// # Example
/// ```no_run
/// use hope_core::crypto::{create_key_store, KeyStoreConfig, KeyStore};
///
/// // Example for SoftwareKeyStore (always available)
/// let software_key_store = create_key_store(KeyStoreConfig::Software).unwrap();
/// println!("Software KeyStore: {}", software_key_store.identifier());
///
/// #[cfg(feature = "hsm-support")] // Only compile this block if hsm-support is enabled
/// {
///     use hope_core::crypto::HsmConfig;
///     let hsm_config = HsmConfig {
///         pkcs11_lib_path: "/usr/lib/softhsm/libsofthsm2.so".to_string(),
///         token_label: "hope-token".to_string(),
///         key_label: "hope-key".to_string(),
///         pin: "1234".to_string(), // In production, use secure input!
///     };
///
///     let hsm_key_store_result = create_key_store(KeyStoreConfig::Hsm(hsm_config));
///     if let Ok(hsm_key_store) = hsm_key_store_result {
///         println!("HSM KeyStore: {}", hsm_key_store.identifier());
///     } else if let Err(e) = hsm_key_store_result {
///         println!("HSM KeyStore could not be created: {:?}", e);
///     }
/// }
/// ```
pub fn create_key_store(config: KeyStoreConfig) -> Result<Box<dyn KeyStore>> {
    match config {
        KeyStoreConfig::Software => Ok(Box::new(SoftwareKeyStore::generate()?)),
        #[cfg(feature = "hsm-support")]
        KeyStoreConfig::Hsm(hsm_config) => {
            let hsm = HsmKeyStore::connect(
                &hsm_config.pkcs11_lib_path,
                &hsm_config.token_label,
                &hsm_config.key_label,
                &hsm_config.pin,
            )?;
            Ok(Box::new(hsm))
        }
        #[cfg(feature = "tee-support")]
        KeyStoreConfig::Tee(tee_config) => {
            let tee = TeeKeyStore::new(&tee_config.enclave_name, tee_config.tee_type)?;
            Ok(Box::new(tee))
        }
    }
}

// ============================================================================
// SOFTWARE KEY STORE (Ed25519 in Memory)
// ============================================================================

/// Software-based Ed25519 key storage
///
/// Keys are stored in process memory. Suitable for:
/// - Development and testing
/// - Low-security environments
/// - Embedded systems without HSM
///
/// **WARNING**: Keys are lost on process termination. For persistence,
/// use `from_seed()` with securely stored seed bytes.
///
/// # Security Properties
///
/// - **Algorithm**: Ed25519 (Curve25519 + SHA-512)
/// - **Key Size**: 32 bytes (private), 32 bytes (public)
/// - **Signature Size**: 64 bytes
/// - **Constant-time**: Yes (immune to timing attacks)
///
/// # Example
///
/// ```rust
/// use hope_core::crypto::{SoftwareKeyStore, KeyStore};
///
/// // Generate new keypair
/// let store = SoftwareKeyStore::generate().unwrap();
///
/// // Sign and verify
/// let data = b"AI action data";
/// let sig = store.sign(data).unwrap();
/// assert!(store.verify(data, &sig).is_ok());
/// ```
#[derive(Clone)]
pub struct SoftwareKeyStore {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl SoftwareKeyStore {
    /// Generate a new random Ed25519 keypair
    ///
    /// Uses OS-provided cryptographically secure random number generator.
    pub fn generate() -> Result<Self> {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();

        Ok(SoftwareKeyStore {
            signing_key,
            verifying_key,
        })
    }

    /// Load keypair from 32-byte seed
    ///
    /// **Use case**: Deterministic key generation or key persistence.
    ///
    /// # Security Warning
    /// The seed MUST be:
    /// - Generated from a CSPRNG (cryptographically secure RNG)
    /// - Stored securely (encrypted at rest, never logged)
    /// - Never transmitted over untrusted channels
    ///
    /// # Example
    /// ```rust
    /// use hope_core::crypto::SoftwareKeyStore;
    ///
    /// let seed = [42u8; 32]; // In production, use secure random seed!
    /// let store = SoftwareKeyStore::from_seed(seed).unwrap();
    /// ```
    pub fn from_seed(seed: [u8; 32]) -> Result<Self> {
        let signing_key = SigningKey::from_bytes(&seed);
        let verifying_key = signing_key.verifying_key();

        Ok(SoftwareKeyStore {
            signing_key,
            verifying_key,
        })
    }

    /// Export the 32-byte Ed25519 public key
    pub fn public_key_bytes_array(&self) -> [u8; 32] {
        self.verifying_key.to_bytes()
    }

    /// Export the 32-byte Ed25519 private key seed
    ///
    /// # Security Warning
    /// NEVER expose this in production! Use only for:
    /// - Secure key backup
    /// - Migration to HSM
    /// - Encrypted storage
    pub fn private_key_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }
}

impl KeyStore for SoftwareKeyStore {
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Ed25519 signs the raw data directly (internally uses SHA-512)
        let signature = self.signing_key.sign(data);
        Ok(signature.to_bytes().to_vec())
    }

    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<()> {
        // Parse signature (64 bytes)
        let sig = Signature::from_slice(signature)
            .map_err(|e| CryptoError::VerificationFailed(e.to_string()))?;

        // Verify with public key
        self.verifying_key
            .verify(data, &sig)
            .map_err(|_| CryptoError::InvalidSignature)?;

        Ok(())
    }

    fn public_key_bytes(&self) -> Vec<u8> {
        self.verifying_key.to_bytes().to_vec()
    }

    fn identifier(&self) -> String {
        format!(
            "SoftwareKeyStore(Ed25519:{})",
            hex::encode(&self.public_key_bytes()[0..8])
        )
    }
}

// ============================================================================
// BACKWARD COMPATIBILITY: KeyPair (Deprecated)
// ============================================================================

/// Legacy KeyPair wrapper for backward compatibility
///
/// **DEPRECATED in v1.4.0**: Use `SoftwareKeyStore` directly instead.
///
/// This struct maintains API compatibility with Hope Genome v1.3.0 code.
/// It wraps `SoftwareKeyStore` but provides the old interface.
///
/// # Migration Guide
///
/// ```rust
/// // Old (v1.3.0)
/// use hope_core::crypto::KeyPair;
/// let keypair = KeyPair::generate().unwrap();
///
/// // New (v1.4.0)
/// use hope_core::crypto::SoftwareKeyStore;
/// let key_store = SoftwareKeyStore::generate().unwrap();
/// ```
#[deprecated(
    since = "1.4.0",
    note = "Use SoftwareKeyStore for new code. KeyPair will be removed in v2.0.0"
)]
#[derive(Clone)]
pub struct KeyPair {
    store: SoftwareKeyStore,
}

#[allow(deprecated)]
impl KeyPair {
    /// Generate a new Ed25519 keypair
    pub fn generate() -> Result<Self> {
        Ok(KeyPair {
            store: SoftwareKeyStore::generate()?,
        })
    }

    /// Sign data with private key
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>> {
        self.store.sign(data)
    }

    /// Verify signature with public key
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> Result<()> {
        self.store.verify(data, signature)
    }

    /// Get public key (for backward compatibility)
    pub fn public_key(&self) -> Vec<u8> {
        self.store.public_key_bytes()
    }

    /// Get underlying KeyStore (for migration to new API)
    pub fn as_key_store(&self) -> &SoftwareKeyStore {
        &self.store
    }
}

#[allow(deprecated)]
impl KeyStore for KeyPair {
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>> {
        self.store.sign(data)
    }

    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<()> {
        self.store.verify(data, signature)
    }

    fn public_key_bytes(&self) -> Vec<u8> {
        self.store.public_key_bytes()
    }

    fn identifier(&self) -> String {
        format!("KeyPair(deprecated wrapper) -> {}", self.store.identifier())
    }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Hash data using SHA-256
///
/// **Note**: Ed25519 uses SHA-512 internally for signing, but this function
/// is used for data integrity checks, AIBOM validation, etc.
pub fn hash_bytes(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Generate a cryptographically secure random nonce
///
/// Returns 32 bytes of randomness from the OS RNG.
/// Used for replay attack prevention in `IntegrityProof`.
pub fn generate_nonce() -> [u8; 32] {
    let mut nonce = [0u8; 32];
    rand::RngCore::fill_bytes(&mut OsRng, &mut nonce);
    nonce
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_software_keystore_generate() {
        let store = SoftwareKeyStore::generate().unwrap();
        let public_key = store.public_key_bytes();
        assert_eq!(public_key.len(), 32); // Ed25519 public key is 32 bytes
    }

    #[test]
    fn test_software_keystore_sign_and_verify() {
        let store = SoftwareKeyStore::generate().unwrap();
        let data = b"test message for Hope Genome v1.4.0";

        // Sign
        let signature = store.sign(data).unwrap();
        assert_eq!(signature.len(), 64); // Ed25519 signature is 64 bytes

        // Verify
        assert!(store.verify(data, &signature).is_ok());
    }

    #[test]
    fn test_software_keystore_verify_fails_on_tampered_data() {
        let store = SoftwareKeyStore::generate().unwrap();
        let data = b"original message";
        let signature = store.sign(data).unwrap();

        let tampered_data = b"tampered message";
        let result = store.verify(tampered_data, &signature);
        assert!(result.is_err());
        assert!(matches!(result, Err(CryptoError::InvalidSignature)));
    }

    #[test]
    fn test_software_keystore_verify_fails_on_tampered_signature() {
        let store = SoftwareKeyStore::generate().unwrap();
        let data = b"test message";
        let mut signature = store.sign(data).unwrap();

        // Tamper with signature
        signature[0] ^= 0xFF;

        let result = store.verify(data, &signature);
        assert!(result.is_err());
    }

    #[test]
    fn test_software_keystore_from_seed_deterministic() {
        let seed = [42u8; 32];

        let store1 = SoftwareKeyStore::from_seed(seed).unwrap();
        let store2 = SoftwareKeyStore::from_seed(seed).unwrap();

        // Same seed -> same keys
        assert_eq!(store1.public_key_bytes(), store2.public_key_bytes());

        // Same signature for same data
        let data = b"deterministic test";
        let sig1 = store1.sign(data).unwrap();
        let sig2 = store2.sign(data).unwrap();
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_keystore_trait_polymorphism() {
        let store: Box<dyn KeyStore> = Box::new(SoftwareKeyStore::generate().unwrap());

        let data = b"polymorphic signing test";
        let signature = store.sign(data).unwrap();
        assert!(store.verify(data, &signature).is_ok());
    }

    #[test]
    #[allow(deprecated)]
    fn test_legacy_keypair_backward_compatibility() {
        let keypair = KeyPair::generate().unwrap();
        let data = b"legacy test";

        let signature = keypair.sign(data).unwrap();
        assert!(keypair.verify(data, &signature).is_ok());
    }

    #[test]
    fn test_hash_bytes_deterministic() {
        let data = b"test data for hashing";
        let hash1 = hash_bytes(data);
        let hash2 = hash_bytes(data);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 32); // SHA-256 is 32 bytes
    }

    #[test]
    fn test_nonce_uniqueness() {
        let nonce1 = generate_nonce();
        let nonce2 = generate_nonce();
        assert_ne!(nonce1, nonce2); // Extremely unlikely collision
        assert_eq!(nonce1.len(), 32);
    }

    #[test]
    fn test_signature_size() {
        let store = SoftwareKeyStore::generate().unwrap();
        let sig = store.sign(b"test").unwrap();

        // Ed25519 signatures are always 64 bytes (vs RSA-2048 ~256 bytes)
        assert_eq!(sig.len(), 64);
    }

    #[test]
    fn test_public_key_export() {
        let store = SoftwareKeyStore::generate().unwrap();
        let public_key = store.public_key_bytes();

        // Should be able to verify with exported public key
        let data = b"export test";
        let signature = store.sign(data).unwrap();

        // Reconstruct verifying key from bytes
        let public_key_array: [u8; 32] = public_key.try_into().unwrap();
        let verifying_key = VerifyingKey::from_bytes(&public_key_array).unwrap();
        let sig = Signature::from_slice(&signature).unwrap();

        assert!(verifying_key.verify(data, &sig).is_ok());
    }
}
