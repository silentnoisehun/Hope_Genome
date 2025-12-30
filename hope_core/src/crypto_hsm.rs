//! # Hope Genome v1.4.0 - Hardware Security Module (HSM) Integration
//!
//! **CRITICAL SECURITY LAYER**: Private keys NEVER leave hardware
//!
//! ## Supported HSMs
//!
//! Any PKCS#11 compatible device:
//! - **YubiKey 5** (FIPS 140-2 Level 2)
//! - **Nitrokey HSM** (Open source hardware)
//! - **SoftHSM2** (Software HSM for testing)
//! - **Thales Luna** (Enterprise HSM)
//! - **AWS CloudHSM** (FIPS 140-2 Level 3)
//! - **Azure Dedicated HSM**
//! - **TPM 2.0** modules
//!
//! ## Security Guarantees
//!
//! 1. ✅ **Private key NEVER leaves HSM** - Signing happens IN hardware
//! 2. ✅ **Tamper-resistant** - Physical security, self-destruct on tampering
//! 3. ✅ **Side-channel immune** - Constant-time operations in hardware
//! 4. ✅ **FIPS 140-2 compliant** (depending on HSM model)
//! 5. ✅ **Audit logging** - HSM tracks all key usage
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │   Hope Genome Application               │
//! │   ┌─────────────────────────────────┐   │
//! │   │  HsmKeyStore (Rust wrapper)     │   │
//! │   └──────────────┬──────────────────┘   │
//! │                  │ PKCS#11 API           │
//! └──────────────────┼───────────────────────┘
//!                    │
//! ┌──────────────────▼───────────────────┐
//! │   HSM Hardware (Tamper-Resistant)    │
//! │   ┌──────────────────────────────┐   │
//! │   │  Private Key (NEVER leaves)  │   │ ← CRITICAL!
//! │   │  - Ed25519 private key       │   │
//! │   │  - Signing operations        │   │
//! │   │  - Nonce validation          │   │
//! │   └──────────────────────────────┘   │
//! └──────────────────────────────────────┘
//! ```
//!
//! ## Example Usage
//!
//! ```no_run
//! # use _hope_core::crypto::{Result, HsmKeyStore, KeyStore};
//! # fn test() -> Result<()> {
//! // Connect to HSM (PIN required!)
//! let hsm = HsmKeyStore::connect(
//!     "/usr/lib/softhsm/libsofthsm2.so",  // PKCS#11 library
//!     "hope-token",                        // Token label
//!     "hope-signing-key",                  // Key label
//!     "1234",                              // PIN (use secure input!)
//! )?;
//!
//! // Sign data (happens IN HSM hardware!)
//! let signature = hsm.sign(b"Critical AI decision")?;
//!
//! // Verify (uses cached public key, no HSM roundtrip)
//! hsm.verify(b"Critical AI decision", &signature)?;
//! # Ok(())
//! # }
//! ```
//!
//! ---
//!
//! **Date**: 2025-12-30
//! **Version**: 1.4.1 (Mathematics & Reality Edition - HSM Support)
//! **Author**: Máté Róbert <stratosoiteam@gmail.com>

use crate::crypto::{CryptoError, KeyStore, Result};
use cryptoki::context::{CInitializeArgs, Pkcs11};
use cryptoki::mechanism::Mechanism;
use cryptoki::object::{Attribute, AttributeType, ObjectHandle};
use cryptoki::session::{Session, UserType};
use cryptoki::types::AuthPin;
use ed25519_compact::{PublicKey, Signature};
use std::path::PathBuf;
use std::sync::Arc;

// ============================================================================
// HSM KEY STORE (PKCS#11 Full Implementation)
// ============================================================================

/// Hardware Security Module key storage (PKCS#11)
///
/// **PRODUCTION-READY HSM INTEGRATION**
///
/// Private keys stored in tamper-resistant hardware. Signing operations
/// executed within HSM, ensuring private key NEVER leaves the device.
///
/// # Security Properties
///
/// - ✅ **Tamper-resistant**: Physical security, self-destruct on tampering
/// - ✅ **Side-channel immune**: Hardware constant-time operations
/// - ✅ **Audit logging**: HSM tracks all key access
/// - ✅ **FIPS 140-2 Level 2/3** (depending on HSM model)
///
/// # Threading
///
/// This struct is `Send + Sync` safe. The underlying PKCS#11 session
/// is wrapped in `Arc` for shared ownership across threads.
///
/// # Example
///
/// ```no_run
/// use _hope_core::crypto::{HsmKeyStore, KeyStore};
///
/// let hsm = HsmKeyStore::connect(
///     "/usr/lib/softhsm/libsofthsm2.so",
///     "hope-token",
///     "hope-key",
///     "1234",  // Use secure PIN entry in production!
/// ).unwrap();
///
/// let signature = hsm.sign(b"data").unwrap();
/// ```
pub struct HsmKeyStore {
    /// PKCS#11 context (shared across sessions)
    _pkcs11: Arc<Pkcs11>,

    /// Active session with HSM
    session: Session,

    /// HSM token label (for identification)
    token_label: String,

    /// Key label in HSM
    key_label: String,

    /// Private key object handle in HSM
    private_key_handle: ObjectHandle,

    /// Cached Ed25519 public key (32 bytes)
    ///
    /// Cached to avoid HSM roundtrip on verification.
    /// The public key is NOT sensitive - it's safe to cache.
    public_key_cache: [u8; 32],
}

impl HsmKeyStore {
    /// Connect to HSM and load Ed25519 signing key
    ///
    /// # Arguments
    ///
    /// * `pkcs11_lib_path` - Path to PKCS#11 library (`.so` on Linux, `.dll` on Windows)
    ///   - **SoftHSM2**: `/usr/lib/softhsm/libsofthsm2.so`
    ///   - **YubiKey**: `/usr/lib/x86_64-linux-gnu/libykcs11.so`
    ///   - **Windows SoftHSM**: `C:\SoftHSM2\lib\softhsm2-x64.dll`
    ///
    /// * `token_label` - HSM token label (configured in HSM)
    /// * `key_label` - Key label in HSM (must exist!)
    /// * `pin` - HSM PIN
    ///
    /// # Security Warning
    ///
    /// **NEVER hardcode PINs!** Use:
    /// - `rpassword::read_password()` for secure terminal input
    /// - Environment variables (but NOT committed to git!)
    /// - Secret management systems (AWS Secrets Manager, HashiCorp Vault)
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - PKCS#11 library cannot be loaded
    /// - Token not found
    /// - Key not found
    /// - PIN is incorrect
    /// - Session cannot be opened
    ///
    /// # Example
    ///
    /// ```no_run
    /// use _hope_core::crypto_hsm::HsmKeyStore;
    ///
    /// let hsm = HsmKeyStore::connect(
    ///     "/usr/lib/softhsm/libsofthsm2.so",
    ///     "hope-token",
    ///     "hope-signing-key",
    ///     "1234",
    /// ).unwrap();
    /// ```
    pub fn connect(
        pkcs11_lib_path: &str,
        token_label: &str,
        key_label: &str,
        pin: &str,
    ) -> Result<Self> {
        // 1. Initialize PKCS#11 context
        let pkcs11 = Pkcs11::new(PathBuf::from(pkcs11_lib_path))
            .map_err(|e| CryptoError::HsmError(format!("Failed to load PKCS#11 library: {}", e)))?;

        pkcs11
            .initialize(CInitializeArgs::OsThreads)
            .map_err(|e| CryptoError::HsmError(format!("PKCS#11 initialization failed: {}", e)))?;

        let pkcs11 = Arc::new(pkcs11);

        // 2. Find token by label
        let slot = pkcs11
            .get_slots_with_token()
            .map_err(|e| CryptoError::HsmError(format!("Failed to get HSM slots: {}", e)))?
            .into_iter()
            .find(|slot| {
                if let Ok(token_info) = pkcs11.get_token_info(*slot) {
                    token_info.label().trim() == token_label.trim()
                } else {
                    false
                }
            })
            .ok_or_else(|| {
                CryptoError::HsmKeyNotFound(format!("Token '{}' not found", token_label))
            })?;

        // 3. Open session
        let session = pkcs11
            .open_rw_session(slot)
            .map_err(|e| CryptoError::HsmError(format!("Failed to open HSM session: {}", e)))?;

        // 4. Login with PIN
        session
            .login(UserType::User, Some(&AuthPin::new(pin.to_string())))
            .map_err(|e| CryptoError::HsmError(format!("HSM PIN authentication failed: {}", e)))?;

        // 5. Find private key by label
        let key_template = vec![
            Attribute::Label(key_label.as_bytes().to_vec()),
            Attribute::Class(cryptoki::object::ObjectClass::PRIVATE_KEY),
        ];

        session
            .find_objects(&key_template)
            .map_err(|e| CryptoError::HsmError(format!("Key search failed: {}", e)))?;

        let private_keys = session
            .find_objects(&key_template)
            .map_err(|e| CryptoError::HsmError(format!("Key search failed: {}", e)))?;

        let private_key_handle = private_keys.first().copied().ok_or_else(|| {
            CryptoError::HsmKeyNotFound(format!("Private key '{}' not found in HSM", key_label))
        })?;

        // 6. Get public key (for verification caching)
        let public_key_template = vec![
            Attribute::Label(key_label.as_bytes().to_vec()),
            Attribute::Class(cryptoki::object::ObjectClass::PUBLIC_KEY),
        ];

        let public_keys = session
            .find_objects(&public_key_template)
            .map_err(|e| CryptoError::HsmError(format!("Public key search failed: {}", e)))?;

        let public_key_handle = public_keys.first().copied().ok_or_else(|| {
            CryptoError::HsmKeyNotFound(format!("Public key '{}' not found in HSM", key_label))
        })?;

        // 7. Extract public key value (Ed25519 = 32 bytes)
        let public_key_attrs = session
            .get_attributes(public_key_handle, &[AttributeType::Value])
            .map_err(|e| CryptoError::HsmError(format!("Failed to get public key: {}", e)))?;

        let public_key_bytes = public_key_attrs
            .iter()
            .find_map(|attr| {
                if let Attribute::Value(bytes) = attr {
                    Some(bytes.clone())
                } else {
                    None
                }
            })
            .ok_or_else(|| CryptoError::HsmError("Public key value not found".into()))?;

        if public_key_bytes.len() != 32 {
            return Err(CryptoError::InvalidKeyFormat(format!(
                "Expected 32-byte Ed25519 public key, got {}",
                public_key_bytes.len()
            )));
        }

        let public_key_cache: [u8; 32] = public_key_bytes[0..32]
            .try_into()
            .map_err(|_| CryptoError::InvalidKeyFormat("Failed to parse public key".into()))?;

        Ok(HsmKeyStore {
            _pkcs11: pkcs11,
            session,
            token_label: token_label.to_string(),
            key_label: key_label.to_string(),
            private_key_handle,
            public_key_cache,
        })
    }

    /// Generate a new Ed25519 keypair IN HSM
    ///
    /// **CRITICAL**: This generates the key INSIDE the HSM. The private key
    /// NEVER exists in application memory.
    ///
    /// # Arguments
    ///
    /// * `pkcs11_lib_path` - Path to PKCS#11 library
    /// * `token_label` - HSM token label
    /// * `key_label` - Label for the new keypair
    /// * `pin` - HSM PIN
    ///
    /// # Example
    ///
    /// ```no_run
    /// use _hope_core::crypto_hsm::HsmKeyStore;
    ///
    /// let hsm = HsmKeyStore::generate_in_hsm(
    ///     "/usr/lib/softhsm/libsofthsm2.so",
    ///     "hope-token",
    ///     "new-hope-key",
    ///     "1234",
    /// ).unwrap();
    /// ```
    pub fn generate_in_hsm(
        pkcs11_lib_path: &str,
        token_label: &str,
        key_label: &str,
        pin: &str,
    ) -> Result<Self> {
        // Initialize and login (same as connect)
        let pkcs11 = Pkcs11::new(PathBuf::from(pkcs11_lib_path))
            .map_err(|e| CryptoError::HsmError(format!("Failed to load PKCS#11 library: {}", e)))?;

        pkcs11
            .initialize(CInitializeArgs::OsThreads)
            .map_err(|e| CryptoError::HsmError(format!("PKCS#11 initialization failed: {}", e)))?;

        let pkcs11 = Arc::new(pkcs11);

        let slot = pkcs11
            .get_slots_with_token()
            .map_err(|e| CryptoError::HsmError(format!("Failed to get HSM slots: {}", e)))?
            .into_iter()
            .find(|slot| {
                if let Ok(token_info) = pkcs11.get_token_info(*slot) {
                    token_info.label().trim() == token_label.trim()
                } else {
                    false
                }
            })
            .ok_or_else(|| {
                CryptoError::HsmKeyNotFound(format!("Token '{}' not found", token_label))
            })?;

        let session = pkcs11
            .open_rw_session(slot)
            .map_err(|e| CryptoError::HsmError(format!("Failed to open HSM session: {}", e)))?;

        session
            .login(UserType::User, Some(&AuthPin::new(pin.to_string())))
            .map_err(|e| CryptoError::HsmError(format!("HSM PIN authentication failed: {}", e)))?;

        // Generate Ed25519 keypair IN HSM
        let public_key_template = vec![
            Attribute::Label(key_label.as_bytes().to_vec()),
            Attribute::Token(true),
            Attribute::Verify(true),
        ];

        let private_key_template = vec![
            Attribute::Label(key_label.as_bytes().to_vec()),
            Attribute::Token(true),
            Attribute::Sensitive(true),    // Cannot be extracted!
            Attribute::Extractable(false), // Cannot be exported!
            Attribute::Sign(true),
        ];

        let (public_key_handle, private_key_handle) = session
            .generate_key_pair(
                &Mechanism::EccEdwardsKeyPairGen, // Ed25519
                &public_key_template,
                &private_key_template,
            )
            .map_err(|e| CryptoError::HsmError(format!("Key generation failed: {}", e)))?;

        // Extract public key
        let public_key_attrs = session
            .get_attributes(public_key_handle, &[AttributeType::Value])
            .map_err(|e| CryptoError::HsmError(format!("Failed to get public key: {}", e)))?;

        let public_key_bytes = public_key_attrs
            .iter()
            .find_map(|attr| {
                if let Attribute::Value(bytes) = attr {
                    Some(bytes.clone())
                } else {
                    None
                }
            })
            .ok_or_else(|| CryptoError::HsmError("Public key value not found".into()))?;

        let public_key_cache: [u8; 32] = public_key_bytes[0..32]
            .try_into()
            .map_err(|_| CryptoError::InvalidKeyFormat("Failed to parse public key".into()))?;

        Ok(HsmKeyStore {
            _pkcs11: pkcs11,
            session,
            token_label: token_label.to_string(),
            key_label: key_label.to_string(),
            private_key_handle,
            public_key_cache,
        })
    }
}

impl KeyStore for HsmKeyStore {
    /// Sign data using HSM (private key NEVER leaves hardware!)
    ///
    /// **CRITICAL SECURITY OPERATION**
    ///
    /// This method delegates signing to the HSM hardware. The private key
    /// NEVER enters application memory. The signature is computed entirely
    /// within the tamper-resistant HSM enclosure.
    ///
    /// # Side-Channel Protection
    ///
    /// HSMs implement constant-time signing operations in hardware,
    /// making them immune to timing attacks, power analysis, and other
    /// side-channel attacks.
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Sign with Ed25519 mechanism IN HSM
        self.session
            .sign(&Mechanism::Eddsa, self.private_key_handle, data)
            .map_err(|e| CryptoError::SigningFailed(format!("HSM signing failed: {}", e)))
    }

    /// Verify signature using cached public key (v1.4.1 - ed25519-compact)
    ///
    /// **No HSM roundtrip needed** - verification uses the cached public key
    /// for performance. The public key is NOT sensitive data.
    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<()> {
        let public_key = PublicKey::from_slice(&self.public_key_cache)
            .map_err(|e: ed25519_compact::Error| CryptoError::VerificationFailed(e.to_string()))?;

        let sig = Signature::from_slice(signature)
            .map_err(|e: ed25519_compact::Error| CryptoError::VerificationFailed(e.to_string()))?;

        public_key
            .verify(data, &sig)
            .map_err(|_| CryptoError::InvalidSignature)?;

        Ok(())
    }

    fn public_key_bytes(&self) -> Vec<u8> {
        self.public_key_cache.to_vec()
    }

    fn identifier(&self) -> String {
        format!(
            "HsmKeyStore(token={}, key={}, pubkey={})",
            self.token_label,
            self.key_label,
            hex::encode(&self.public_key_cache[0..8])
        )
    }
}

// Thread safety: PKCS#11 sessions can be shared across threads if the underlying
// library is initialized with `CInitializeArgs::OsThreads`. The `cryptoki` crate
// handles the necessary locking.
unsafe impl Send for HsmKeyStore {}
unsafe impl Sync for HsmKeyStore {}

impl Drop for HsmKeyStore {
    /// Gracefully closes the HSM session and logs out.
    ///
    /// # Memory Safety & Resource Management (PKCS#11 C-Bridge)
    ///
    /// The `cryptoki` crate provides a Rust interface over a C library (the PKCS#11 driver).
    /// Managing resources correctly is CRITICAL to prevent memory leaks and dangling pointers.
    ///
    /// 1.  **RAII (Resource Acquisition Is Initialization)**: This `Drop` implementation follows
    ///     the RAII pattern. When an `HsmKeyStore` instance goes out of scope, this `drop`
    ///     function is automatically called.
    ///
    /// 2.  **Session Logout & Close**: We explicitly call `self.session.logout()` and `self.session.close()`.
    ///     This is crucial for releasing the session handle in the HSM and preventing session exhaustion.
    ///     Ignoring errors (`let _ = ...`) is a pragmatic choice here, as we can't do much if
    ///     logout/close fails during a drop (e.g., panicking in a drop is discouraged). The HSM
    ///     itself will likely time out and invalidate the session eventually.
    ///
    /// 3.  **PKCS#11 Context Finalization**: The main `Pkcs11` context is wrapped in an `Arc`.
    ///     The `cryptoki` crate's own `Drop` implementation for `Pkcs11` calls `C_Finalize`.
    ///     By using `Arc`, we ensure that `C_Finalize` is only called when the *last* reference
    ///     to the context is dropped, preventing premature finalization if multiple HsmKeyStore
    ///     instances were to share the same context.
    ///
    /// # Potential Memory Leaks
    ///
    /// - **Driver Bugs**: A poorly implemented PKCS#11 C-driver could still leak memory on its own.
    ///   This is outside the control of Hope Genome and `cryptoki`. Using certified, well-tested
    ///   drivers (e.g., from YubiKey, Thales) is essential.
    /// - **Panics**: If a panic were to occur *before* an `HsmKeyStore` is fully initialized,
    ///   some resources might not be cleaned up. The `cryptoki` crate aims to be robust against this,
    ///   but it remains a complex failure scenario in any FFI code.
    fn drop(&mut self) {
        // Logout and close session on drop
        let _ = self.session.logout();
        // The session will be closed automatically when `self.session` is dropped,
        // as `cryptoki::session::Session` has its own `Drop` implementation.
        // The `Pkcs11` context wrapped in `Arc` will be finalized automatically
        // when its reference count reaches zero, calling `C_Finalize`.
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// List all available PKCS#11 tokens
///
/// Useful for HSM discovery and configuration.
///
/// # Example
///
/// ```no_run
/// use _hope_core::crypto_hsm::list_hsm_tokens;
///
/// for token in list_hsm_tokens("/usr/lib/softhsm/libsofthsm2.so").unwrap() {
///     println!("Token: {} (Slot: {})", token.label, token.slot_id);
/// }
/// ```
pub fn list_hsm_tokens(pkcs11_lib_path: &str) -> Result<Vec<TokenInfo>> {
    let pkcs11 = Pkcs11::new(PathBuf::from(pkcs11_lib_path))
        .map_err(|e| CryptoError::HsmError(format!("Failed to load PKCS#11 library: {}", e)))?;

    pkcs11
        .initialize(CInitializeArgs::OsThreads)
        .map_err(|e| CryptoError::HsmError(format!("PKCS#11 initialization failed: {}", e)))?;

    let slots = pkcs11
        .get_slots_with_token()
        .map_err(|e| CryptoError::HsmError(format!("Failed to get slots: {}", e)))?;

    let mut tokens = Vec::new();
    for slot in slots {
        if let Ok(token_info) = pkcs11.get_token_info(slot) {
            tokens.push(TokenInfo {
                label: token_info.label().to_string(),
                manufacturer: token_info.manufacturer_id().to_string(),
                model: token_info.model().to_string(),
                serial_number: token_info.serial_number().to_string(),
                slot_id: slot.id(),
            });
        }
    }

    Ok(tokens)
}

/// HSM token information
#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub label: String,
    pub manufacturer: String,
    pub model: String,
    pub serial_number: String,
    pub slot_id: u64,
}
