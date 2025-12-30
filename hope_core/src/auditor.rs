//! # Hope Genome v1.4.0 - Proof Auditor
//!
//! **Hardened Security Edition - Pluggable Architecture**
//!
//! ## Major Changes in v1.4.0
//!
//! ### 1. Persistent Nonce Store Integration
//! - **Replay Attack Protection**: Survives process restarts
//! - **Pluggable Backends**: Memory, RocksDB, Redis
//! - **Production Ready**: No more lost nonces on crash!
//!
//! ### 2. KeyStore Trait Integration
//! - **Flexible Cryptography**: Software or HSM
//! - **Ed25519 Support**: Fast, secure signatures
//! - **Future-Proof**: Easy to add new key backends
//!
//! ## Example (New API)
//!
//! ```rust
//! use hope_core::auditor::ProofAuditor;
//! use hope_core::crypto::SoftwareKeyStore;
//! use hope_core::nonce_store::MemoryNonceStore;
//!
//! // Create with pluggable backends
//! let key_store = SoftwareKeyStore::generate().unwrap();
//! let nonce_store = MemoryNonceStore::new();
//!
//! let auditor = ProofAuditor::new(
//!     Box::new(key_store),
//!     Box::new(nonce_store),
//! );
//! ```
//!
//! ---
//!
//! **Date**: 2025-12-30
//! **Version**: 1.4.0 (Hardened Security Edition)
//! **Author**: Máté Róbert <stratosoiteam@gmail.com>

use crate::crypto::{CryptoError, KeyStore};
use crate::nonce_store::{NonceStore, NonceStoreError};
use crate::proof::IntegrityProof;
use thiserror::Error;

// ============================================================================
// ERROR TYPES
// ============================================================================

#[derive(Debug, Error)]
pub enum AuditorError {
    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Proof expired: issued at {issued}, now {now}, TTL {ttl}s")]
    ProofExpired { issued: u64, now: u64, ttl: u64 },

    #[error("Nonce already used (replay attack detected): {0}")]
    NonceReused(String),

    #[error("Nonce store error: {0}")]
    NonceStoreError(#[from] NonceStoreError),

    #[error("Crypto error: {0}")]
    CryptoError(String),
}

impl From<CryptoError> for AuditorError {
    fn from(err: CryptoError) -> Self {
        match err {
            CryptoError::InvalidSignature => AuditorError::InvalidSignature,
            other => AuditorError::CryptoError(other.to_string()),
        }
    }
}

pub type Result<T> = std::result::Result<T, AuditorError>;

// ============================================================================
// PROOF AUDITOR (v1.4.0 - Trait-Based Architecture)
// ============================================================================

/// Proof verification engine with pluggable backends
///
/// The auditor performs cryptographic verification of `IntegrityProof` objects
/// with multi-layer security:
///
/// 1. **Signature Verification**: Ed25519 cryptographic signatures (KeyStore)
/// 2. **TTL Enforcement**: Time-based proof expiry
/// 3. **Replay Attack Prevention**: Persistent nonce tracking (NonceStore)
///
/// ## Architecture (v1.4.0)
///
/// ```text
/// ┌─────────────────┐
/// │  ProofAuditor   │
/// └────────┬────────┘
///          │
///          ├─── KeyStore ───────> SoftwareKeyStore (Ed25519)
///          │                      HsmKeyStore (PKCS#11)
///          │
///          └─── NonceStore ─────> MemoryNonceStore (testing)
///                                 RocksDbNonceStore (production)
///                                 RedisNonceStore (distributed)
/// ```
///
/// ## Example (Production Setup)
///
/// ```no_run
/// use hope_core::auditor::ProofAuditor;
/// use hope_core::crypto::SoftwareKeyStore;
/// # #[cfg(feature = "rocksdb-nonce-store")]
/// use hope_core::nonce_store::RocksDbNonceStore;
///
/// # #[cfg(feature = "rocksdb-nonce-store")]
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Production setup: persistent nonce store
/// let key_store = SoftwareKeyStore::generate()?;
/// let nonce_store = RocksDbNonceStore::new("./hope_nonces.db")?;
///
/// let mut auditor = ProofAuditor::new(
///     Box::new(key_store),
///     Box::new(nonce_store),
/// );
///
/// // Verify proofs - nonces persist across restarts!
/// // auditor.verify_proof(&proof)?;
/// # Ok(())
/// # }
/// # #[cfg(not(feature = "rocksdb-nonce-store"))]
/// # fn main() {}
/// ```
pub struct ProofAuditor {
    /// Cryptographic key store (pluggable: software or HSM)
    key_store: Box<dyn KeyStore>,

    /// Nonce store for replay attack prevention (pluggable: memory, RocksDB, Redis)
    nonce_store: Box<dyn NonceStore>,
}

impl ProofAuditor {
    /// Create a new proof auditor with custom backends
    ///
    /// # Arguments
    /// * `key_store` - Cryptographic key storage (SoftwareKeyStore, HsmKeyStore, etc.)
    /// * `nonce_store` - Nonce tracking storage (MemoryNonceStore, RocksDbNonceStore, etc.)
    ///
    /// # Example
    /// ```rust
    /// use hope_core::auditor::ProofAuditor;
    /// use hope_core::crypto::SoftwareKeyStore;
    /// use hope_core::nonce_store::MemoryNonceStore;
    ///
    /// let key_store = SoftwareKeyStore::generate().unwrap();
    /// let nonce_store = MemoryNonceStore::new();
    ///
    /// let auditor = ProofAuditor::new(
    ///     Box::new(key_store),
    ///     Box::new(nonce_store),
    /// );
    /// ```
    pub fn new(key_store: Box<dyn KeyStore>, nonce_store: Box<dyn NonceStore>) -> Self {
        ProofAuditor {
            key_store,
            nonce_store,
        }
    }

    /// Verify a cryptographic proof
    ///
    /// This performs comprehensive multi-layer verification:
    ///
    /// 1. **Signature Verification**: Validates Ed25519 signature
    /// 2. **TTL Check**: Ensures proof hasn't expired
    /// 3. **Nonce Check**: Prevents replay attacks (atomic check-and-insert)
    ///
    /// # Arguments
    /// * `proof` - The `IntegrityProof` to verify
    ///
    /// # Returns
    /// - `Ok(())` if proof is valid and not replayed
    /// - `Err(InvalidSignature)` if signature verification fails
    /// - `Err(ProofExpired)` if proof TTL has expired
    /// - `Err(NonceReused)` if nonce was already used (replay attack)
    ///
    /// # Security Guarantees
    ///
    /// - **Constant-time**: Ed25519 verification prevents timing attacks
    /// - **Atomic nonce check**: Race-condition free (even in distributed setup)
    /// - **Persistent protection**: With RocksDB/Redis, survives restarts
    ///
    /// # Example
    /// ```rust
    /// use hope_core::auditor::ProofAuditor;
    /// use hope_core::crypto::{KeyStore, SoftwareKeyStore}; // KeyStore trait needed for .sign()
    /// use hope_core::nonce_store::MemoryNonceStore;
    /// use hope_core::proof::{Action, IntegrityProof};
    ///
    /// let key_store = SoftwareKeyStore::generate().unwrap();
    /// let nonce_store = MemoryNonceStore::new();
    /// let mut auditor = ProofAuditor::new(
    ///     Box::new(key_store.clone()),
    ///     Box::new(nonce_store),
    /// );
    ///
    /// // Create and sign a proof
    /// let action = Action::delete("test.txt");
    /// let mut proof = IntegrityProof::new(&action, "capsule123".into(), 3600);
    /// proof.signature = key_store.sign(&proof.signing_data()).unwrap();
    ///
    /// // Verify proof
    /// assert!(auditor.verify_proof(&proof).is_ok());
    ///
    /// // Replay attack: blocked!
    /// assert!(auditor.verify_proof(&proof).is_err());
    /// ```
    pub fn verify_proof(&mut self, proof: &IntegrityProof) -> Result<()> {
        // Step 1: Verify signature (most critical - fail fast)
        self.verify_signature(proof)?;

        // Step 2: Check TTL (time-to-live)
        let now = chrono::Utc::now().timestamp() as u64;
        if now - proof.timestamp > proof.ttl {
            return Err(AuditorError::ProofExpired {
                issued: proof.timestamp,
                now,
                ttl: proof.ttl,
            });
        }

        // Step 3: Check and insert nonce (atomic operation)
        // This prevents replay attacks even across restarts (with persistent store)
        self.nonce_store
            .check_and_insert(proof.nonce, proof.ttl)
            .map_err(|e| match e {
                NonceStoreError::NonceReused(hex) => AuditorError::NonceReused(hex),
                other => AuditorError::NonceStoreError(other),
            })?;

        Ok(())
    }

    /// Verify signature only (without nonce/TTL checks)
    ///
    /// Use this for read-only verification without state changes.
    /// Does NOT mark nonce as used.
    ///
    /// # Example
    /// ```rust
    /// use hope_core::auditor::ProofAuditor;
    /// use hope_core::crypto::{KeyStore, SoftwareKeyStore}; // KeyStore trait needed for .sign()
    /// use hope_core::nonce_store::MemoryNonceStore;
    /// use hope_core::proof::{Action, IntegrityProof};
    ///
    /// let key_store = SoftwareKeyStore::generate().unwrap();
    /// let nonce_store = MemoryNonceStore::new();
    /// let auditor = ProofAuditor::new(
    ///     Box::new(key_store.clone()),
    ///     Box::new(nonce_store),
    /// );
    ///
    /// let action = Action::delete("test.txt");
    /// let mut proof = IntegrityProof::new(&action, "capsule123".into(), 3600);
    /// proof.signature = key_store.sign(&proof.signing_data()).unwrap();
    ///
    /// // Verify signature multiple times (no nonce consumption)
    /// assert!(auditor.verify_signature(&proof).is_ok());
    /// assert!(auditor.verify_signature(&proof).is_ok());
    /// ```
    pub fn verify_signature(&self, proof: &IntegrityProof) -> Result<()> {
        let message = proof.signing_data();
        self.key_store.verify(&message, &proof.signature)?;
        Ok(())
    }

    /// Check if a nonce has been used
    ///
    /// Read-only operation - does not modify state.
    pub fn is_nonce_used(&self, nonce: &[u8; 32]) -> bool {
        self.nonce_store.contains(nonce)
    }

    /// Get count of used nonces
    ///
    /// Useful for monitoring and debugging.
    pub fn used_nonce_count(&self) -> usize {
        self.nonce_store.count()
    }

    /// Clear all nonces (DANGEROUS - use only for testing!)
    ///
    /// # Security Warning
    /// This allows replay attacks! Only use for:
    /// - Unit tests
    /// - Development reset
    /// - Maintenance with full system shutdown
    pub fn clear_nonces(&mut self) -> Result<()> {
        self.nonce_store.clear()?;
        Ok(())
    }

    /// Cleanup expired nonces (optional maintenance)
    ///
    /// Most backends handle this automatically (e.g., Redis TTL),
    /// but RocksDB may benefit from periodic cleanup.
    ///
    /// # Returns
    /// Number of nonces removed
    pub fn cleanup_expired_nonces(&mut self) -> Result<usize> {
        Ok(self.nonce_store.cleanup_expired()?)
    }

    /// Get key store identifier (for logging/debugging)
    pub fn key_store_info(&self) -> String {
        self.key_store.identifier()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::SoftwareKeyStore;
    use crate::nonce_store::MemoryNonceStore;
    use crate::proof::Action;

    fn create_test_auditor() -> (ProofAuditor, SoftwareKeyStore) {
        let key_store = SoftwareKeyStore::generate().unwrap();
        let key_store_clone = key_store.clone();
        let nonce_store = MemoryNonceStore::new();

        let auditor = ProofAuditor::new(Box::new(key_store), Box::new(nonce_store));

        (auditor, key_store_clone)
    }

    fn create_test_proof(key_store: &SoftwareKeyStore) -> IntegrityProof {
        let action = Action::delete("test.txt");
        let mut proof = IntegrityProof::new(&action, "test_capsule".into(), 3600);

        // Sign the proof
        let signing_data = proof.signing_data();
        proof.signature = key_store.sign(&signing_data).unwrap();

        proof
    }

    #[test]
    fn test_verify_valid_proof() {
        let (mut auditor, key_store) = create_test_auditor();
        let proof = create_test_proof(&key_store);

        // First verification: should succeed
        assert!(auditor.verify_proof(&proof).is_ok());
    }

    #[test]
    fn test_replay_attack_prevention() {
        let (mut auditor, key_store) = create_test_auditor();
        let proof = create_test_proof(&key_store);

        // First use: should succeed
        assert!(auditor.verify_proof(&proof).is_ok());
        assert_eq!(auditor.used_nonce_count(), 1);

        // Second use: should FAIL (nonce reused)
        let result = auditor.verify_proof(&proof);
        assert!(result.is_err());
        assert!(matches!(result, Err(AuditorError::NonceReused(_))));
    }

    #[test]
    fn test_ttl_expiration() {
        let (mut auditor, key_store) = create_test_auditor();

        // Create proof with very short TTL
        let action = Action::delete("test.txt");
        let mut proof = IntegrityProof::new(&action, "test_capsule".into(), 0);

        // Set timestamp to 10 seconds ago
        proof.timestamp = chrono::Utc::now().timestamp() as u64 - 10;

        // Sign it
        let signing_data = proof.signing_data();
        proof.signature = key_store.sign(&signing_data).unwrap();

        // Should fail (expired)
        let result = auditor.verify_proof(&proof);
        assert!(result.is_err());
        assert!(matches!(result, Err(AuditorError::ProofExpired { .. })));
    }

    #[test]
    fn test_invalid_signature() {
        let (mut auditor, key_store) = create_test_auditor();
        let mut proof = create_test_proof(&key_store);

        // Tamper with signature
        proof.signature[0] ^= 0xFF;

        // Should fail (invalid signature)
        let result = auditor.verify_proof(&proof);
        assert!(result.is_err());
        assert!(matches!(result, Err(AuditorError::InvalidSignature)));
    }

    #[test]
    fn test_verify_signature_readonly() {
        let (auditor, key_store) = create_test_auditor();
        let proof = create_test_proof(&key_store);

        // Verify signature multiple times (should not consume nonce)
        assert!(auditor.verify_signature(&proof).is_ok());
        assert!(auditor.verify_signature(&proof).is_ok());
        assert!(auditor.verify_signature(&proof).is_ok());

        assert_eq!(auditor.used_nonce_count(), 0); // No nonce consumed
    }

    #[test]
    fn test_nonce_tracking() {
        let (mut auditor, key_store) = create_test_auditor();

        // Create two different proofs
        let proof1 = create_test_proof(&key_store);
        let proof2 = create_test_proof(&key_store);

        assert_eq!(auditor.used_nonce_count(), 0);

        auditor.verify_proof(&proof1).unwrap();
        assert_eq!(auditor.used_nonce_count(), 1);

        auditor.verify_proof(&proof2).unwrap();
        assert_eq!(auditor.used_nonce_count(), 2);
    }

    #[test]
    fn test_clear_nonces() {
        let (mut auditor, key_store) = create_test_auditor();

        // Add some nonces
        for _ in 0..5 {
            let proof = create_test_proof(&key_store);
            auditor.verify_proof(&proof).unwrap();
        }

        assert_eq!(auditor.used_nonce_count(), 5);

        // Clear all nonces
        auditor.clear_nonces().unwrap();
        assert_eq!(auditor.used_nonce_count(), 0);
    }

    #[test]
    fn test_different_actions_different_proofs() {
        let (mut auditor, key_store) = create_test_auditor();

        let action1 = Action::delete("file1.txt");
        let mut proof1 = IntegrityProof::new(&action1, "capsule1".into(), 3600);
        proof1.signature = key_store.sign(&proof1.signing_data()).unwrap();

        let action2 = Action::delete("file2.txt");
        let mut proof2 = IntegrityProof::new(&action2, "capsule2".into(), 3600);
        proof2.signature = key_store.sign(&proof2.signing_data()).unwrap();

        // Both should verify successfully (different nonces)
        assert!(auditor.verify_proof(&proof1).is_ok());
        assert!(auditor.verify_proof(&proof2).is_ok());
    }

    #[test]
    fn test_is_nonce_used() {
        let (mut auditor, key_store) = create_test_auditor();
        let proof = create_test_proof(&key_store);

        assert!(!auditor.is_nonce_used(&proof.nonce));

        auditor.verify_proof(&proof).unwrap();

        assert!(auditor.is_nonce_used(&proof.nonce));
    }

    #[test]
    fn test_key_store_info() {
        let (auditor, _) = create_test_auditor();
        let info = auditor.key_store_info();

        assert!(info.contains("SoftwareKeyStore"));
        assert!(info.contains("Ed25519"));
    }
}
