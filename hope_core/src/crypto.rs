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
use sha2::{Digest, Sha256};
use thiserror::Error;

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
// HSM KEY STORE (PKCS#11 Abstraction - Future)
// ============================================================================

/// Hardware Security Module key storage (PKCS#11)
///
/// **Status**: Architecture ready, implementation TBD
///
/// This is a placeholder for future HSM integration. When implemented,
/// it will support:
/// - PKCS#11 compatible HSMs (YubiKey, Nitrokey, Thales, etc.)
/// - Cloud HSMs (AWS CloudHSM, Azure Dedicated HSM)
/// - TPM 2.0 modules
///
/// # Design Notes
///
/// 1. Private key NEVER leaves HSM
/// 2. Signing operations delegated to HSM hardware
/// 3. Public key cached for fast verification
/// 4. PIN/password required for initialization
///
/// # Example (Future)
///
/// ```ignore
/// let hsm = HsmKeyStore::connect(
///     "/usr/lib/libsofthsm2.so",  // PKCS#11 library
///     "hope-token",                // Token label
///     "hope-key",                  // Key label
///     "1234",                      // PIN (secure input!)
/// )?;
///
/// // Sign with HSM (private key stays in hardware)
/// let signature = hsm.sign(b"data")?;
/// ```
#[allow(dead_code)]
pub struct HsmKeyStore {
    /// PKCS#11 token label
    token_label: String,

    /// Key label in HSM
    key_label: String,

    /// Cached public key (for verification without HSM roundtrip)
    public_key_cache: Vec<u8>,
    // Future: PKCS#11 context, session handle, etc.
    // pkcs11_ctx: pkcs11::Ctx,
    // session: pkcs11::types::CK_SESSION_HANDLE,
}

#[allow(dead_code)]
impl HsmKeyStore {
    /// Connect to HSM and load key by label
    ///
    /// # Arguments
    /// * `pkcs11_lib_path` - Path to PKCS#11 library (.so/.dll)
    /// * `token_label` - HSM token label
    /// * `key_label` - Key label in HSM
    /// * `pin` - HSM PIN (use secure input in production!)
    ///
    /// # Errors
    /// Returns error if:
    /// - PKCS#11 library cannot be loaded
    /// - Token not found
    /// - Key not found
    /// - PIN is incorrect
    ///
    /// # Security Note
    /// In production, use:
    /// - Secure PIN entry (e.g., `rpassword` crate)
    /// - Environment variables or secret management system
    /// - NEVER hardcode PINs!
    pub fn connect(
        _pkcs11_lib_path: &str,
        token_label: &str,
        key_label: &str,
        _pin: &str,
    ) -> Result<Self> {
        // TODO v1.5.0: Actual PKCS#11 implementation
        // 1. Load PKCS#11 library: Ctx::new(pkcs11_lib_path)
        // 2. Get slot list: ctx.get_slot_list(true)
        // 3. Find token by label
        // 4. Open session: ctx.open_session(slot, CKF_SERIAL_SESSION)
        // 5. Login: ctx.login(session, CKU_USER, pin)
        // 6. Find key by label: ctx.find_objects(session, &template)
        // 7. Get public key: ctx.get_attribute_value(session, key_handle, &[CKA_VALUE])

        // Placeholder implementation
        Ok(HsmKeyStore {
            token_label: token_label.to_string(),
            key_label: key_label.to_string(),
            public_key_cache: vec![0u8; 32], // Placeholder
        })
    }
}

#[allow(dead_code)]
impl KeyStore for HsmKeyStore {
    fn sign(&self, _data: &[u8]) -> Result<Vec<u8>> {
        // TODO v1.5.0: PKCS#11 signing operation
        // C_SignInit(session, &mechanism, key_handle)
        // C_Sign(session, data, signature)
        Err(CryptoError::HsmError(
            "HSM support not yet implemented - available in v1.5.0".into(),
        ))
    }

    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<()> {
        // Verify with cached public key (no HSM roundtrip needed)
        if self.public_key_cache.len() != 32 {
            return Err(CryptoError::InvalidKeyFormat(
                "Invalid public key in cache".into(),
            ));
        }

        let public_key_bytes: [u8; 32] = self.public_key_cache[0..32]
            .try_into()
            .map_err(|_| CryptoError::InvalidKeyFormat("Failed to parse public key".into()))?;

        let verifying_key = VerifyingKey::from_bytes(&public_key_bytes)
            .map_err(|e| CryptoError::VerificationFailed(e.to_string()))?;

        let sig = Signature::from_slice(signature)
            .map_err(|e| CryptoError::VerificationFailed(e.to_string()))?;

        verifying_key
            .verify(data, &sig)
            .map_err(|_| CryptoError::InvalidSignature)?;

        Ok(())
    }

    fn public_key_bytes(&self) -> Vec<u8> {
        self.public_key_cache.clone()
    }

    fn identifier(&self) -> String {
        format!(
            "HsmKeyStore(token={}, key={})",
            self.token_label, self.key_label
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
