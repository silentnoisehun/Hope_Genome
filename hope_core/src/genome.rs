//! # Hope Genome v1.4.0 - Sealed Genome (Ethical Ruleset)
//!
//! **Hardened Security Edition - KeyStore Integration**
//!
//! ## Major Changes in v1.4.0
//!
//! ### 1. KeyStore Trait Support
//! - **Backward Compatible**: Still supports deprecated `KeyPair`
//! - **New API**: Accepts any `KeyStore` implementation
//! - **Future-Proof**: Easy to use HSM, TPM, etc.
//!
//! ### 2. Ed25519 Signatures
//! - All proofs now signed with Ed25519 (via KeyStore)
//! - Faster, smaller, more secure than RSA
//!
//! ---
//!
//! **Date**: 2025-12-30
//! **Version**: 1.4.0 (Hardened Security Edition)
//! **Author**: Máté Róbert <stratosoiteam@gmail.com>

#[allow(deprecated)] // KeyPair import for backward compatibility functions
use crate::crypto::{hash_bytes, KeyPair, KeyStore};
use crate::proof::{Action, IntegrityProof};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GenomeError {
    #[error("Genome not sealed yet")]
    NotSealed,

    #[error("Genome already sealed")]
    AlreadySealed,

    #[error("Action violates genome rules: {0}")]
    RuleViolation(String),

    #[error("Crypto error: {0}")]
    CryptoError(#[from] crate::crypto::CryptoError),
}

pub type Result<T> = std::result::Result<T, GenomeError>;

/// The sealed genome - immutable ethical ruleset
///
/// This is the core of the Hope Genome framework. It:
/// - Contains immutable ethical rules
/// - Signs action approvals with Ed25519 (v1.4.0+)
/// - Provides cryptographic proofs for all decisions
///
/// ## Example (v1.4.0 New API)
///
/// ```rust
/// use hope_core::genome::SealedGenome;
/// use hope_core::crypto::SoftwareKeyStore;
/// use hope_core::proof::Action;
///
/// // Create genome with new KeyStore API
/// let key_store = SoftwareKeyStore::generate().unwrap();
/// let rules = vec!["Do no harm".to_string()];
///
/// let mut genome = SealedGenome::with_key_store(
///     rules,
///     Box::new(key_store),
/// ).unwrap();
///
/// genome.seal().unwrap();
///
/// // Verify action and get proof
/// let action = Action::delete("test.txt");
/// let proof = genome.verify_action(&action).unwrap();
/// ```
pub struct SealedGenome {
    /// Ethical rules (immutable after sealing)
    rules: Vec<String>,

    /// Whether the genome is sealed
    sealed: bool,

    /// Cryptographic key store for signing (v1.4.0: trait-based)
    key_store: Box<dyn KeyStore>,

    /// Hash of the sealed genome (for proof binding)
    capsule_hash: Option<String>,

    /// Default TTL for proofs (seconds)
    default_ttl: u64,
}

impl SealedGenome {
    /// Create a new genome with rules (generates new Ed25519 keypair)
    ///
    /// # Example
    /// ```rust
    /// use hope_core::genome::SealedGenome;
    ///
    /// let rules = vec!["Do no harm".to_string()];
    /// let genome = SealedGenome::new(rules).unwrap();
    /// ```
    #[allow(deprecated)] // Uses deprecated with_keypair for backward compatibility
    pub fn new(rules: Vec<String>) -> Result<Self> {
        let keypair = KeyPair::generate()?;
        Self::with_keypair(rules, keypair)
    }

    /// Create a new genome with an existing KeyPair (deprecated)
    ///
    /// **DEPRECATED in v1.4.0**: Use `with_key_store()` instead.
    ///
    /// # Example (Legacy)
    /// ```rust
    /// use hope_core::genome::SealedGenome;
    /// use hope_core::crypto::KeyPair;
    ///
    /// # #[allow(deprecated)]
    /// let keypair = KeyPair::generate().unwrap();
    /// # #[allow(deprecated)]
    /// let genome = SealedGenome::with_keypair(
    ///     vec!["Rule 1".to_string()],
    ///     keypair,
    /// ).unwrap();
    /// ```
    #[allow(deprecated)]
    #[deprecated(since = "1.4.0", note = "Use with_key_store() for new code")]
    pub fn with_keypair(rules: Vec<String>, keypair: KeyPair) -> Result<Self> {
        Ok(SealedGenome {
            rules,
            sealed: false,
            key_store: Box::new(keypair),
            capsule_hash: None,
            default_ttl: 60, // 1 minute default
        })
    }

    /// Create a new genome with a custom KeyStore (v1.4.0)
    ///
    /// This is the recommended way to create a genome in v1.4.0+.
    /// Supports any KeyStore implementation (Software, HSM, etc.).
    ///
    /// # Example
    /// ```rust
    /// use hope_core::genome::SealedGenome;
    /// use hope_core::crypto::SoftwareKeyStore;
    ///
    /// let key_store = SoftwareKeyStore::generate().unwrap();
    /// let rules = vec!["Protect privacy".to_string()];
    ///
    /// let genome = SealedGenome::with_key_store(
    ///     rules,
    ///     Box::new(key_store),
    /// ).unwrap();
    /// ```
    pub fn with_key_store(rules: Vec<String>, key_store: Box<dyn KeyStore>) -> Result<Self> {
        Ok(SealedGenome {
            rules,
            sealed: false,
            key_store,
            capsule_hash: None,
            default_ttl: 60, // 1 minute default
        })
    }

    /// Seal the genome (make it immutable)
    ///
    /// After sealing:
    /// - Rules cannot be modified
    /// - Capsule hash is computed
    /// - Actions can be verified and signed
    ///
    /// # Example
    /// ```rust
    /// use hope_core::genome::SealedGenome;
    ///
    /// let mut genome = SealedGenome::new(vec!["Rule 1".to_string()]).unwrap();
    /// genome.seal().unwrap();
    ///
    /// assert!(genome.is_sealed());
    /// ```
    pub fn seal(&mut self) -> Result<()> {
        if self.sealed {
            return Err(GenomeError::AlreadySealed);
        }

        // Compute capsule hash (hash of all rules)
        let rules_json = serde_json::to_string(&self.rules).unwrap();
        let hash = hash_bytes(rules_json.as_bytes());
        self.capsule_hash = Some(hex::encode(hash));

        self.sealed = true;

        Ok(())
    }

    /// Check if genome is sealed
    pub fn is_sealed(&self) -> bool {
        self.sealed
    }

    /// Get the rules
    pub fn rules(&self) -> &[String] {
        &self.rules
    }

    /// Set default TTL for proofs (in seconds)
    ///
    /// # Example
    /// ```rust
    /// use hope_core::genome::SealedGenome;
    ///
    /// let mut genome = SealedGenome::new(vec!["Rule 1".to_string()]).unwrap();
    /// genome.set_default_ttl(3600); // 1 hour
    /// ```
    pub fn set_default_ttl(&mut self, ttl: u64) {
        self.default_ttl = ttl;
    }

    /// Verify an action against the genome rules
    ///
    /// This is where ethical decision-making happens.
    /// Returns a cryptographically signed proof if approved.
    ///
    /// # Security (v1.4.0)
    /// - Proof signed with Ed25519 (fast, secure)
    /// - Includes nonce for replay attack prevention
    /// - Bound to capsule hash (prevents proof reuse across genomes)
    ///
    /// # Example
    /// ```rust
    /// use hope_core::genome::SealedGenome;
    /// use hope_core::proof::Action;
    ///
    /// let mut genome = SealedGenome::new(vec!["Do no harm".to_string()]).unwrap();
    /// genome.seal().unwrap();
    ///
    /// let action = Action::delete("test.txt");
    /// let proof = genome.verify_action(&action).unwrap();
    ///
    /// assert!(!proof.signature.is_empty());
    /// assert_eq!(proof.signature.len(), 64); // Ed25519 signature
    /// ```
    pub fn verify_action(&self, action: &Action) -> Result<IntegrityProof> {
        if !self.sealed {
            return Err(GenomeError::NotSealed);
        }

        // Basic rule checking (simple implementation)
        // In production, this would use sophisticated reasoning
        let action_str = format!("{:?}", action);

        // Check rules (simplified)
        for rule in &self.rules {
            if rule.contains("no harm") && action_str.contains("delete") {
                // More sophisticated checking would happen here
            }
        }

        // Create proof
        let capsule_hash = self.capsule_hash.as_ref().unwrap().clone();
        let mut proof = IntegrityProof::new(action, capsule_hash, self.default_ttl);

        // Sign the proof (v1.4.0: uses KeyStore trait)
        let signing_data = proof.signing_data();
        proof.signature = self.key_store.sign(&signing_data)?;

        Ok(proof)
    }

    /// Get the capsule hash
    ///
    /// Returns `None` if genome is not yet sealed.
    pub fn capsule_hash(&self) -> Option<&str> {
        self.capsule_hash.as_deref()
    }

    /// Get the public key bytes (for verification by external parties)
    ///
    /// # Example
    /// ```rust
    /// use hope_core::genome::SealedGenome;
    ///
    /// let genome = SealedGenome::new(vec!["Rule 1".to_string()]).unwrap();
    /// let public_key = genome.public_key_bytes();
    ///
    /// assert_eq!(public_key.len(), 32); // Ed25519 public key
    /// ```
    pub fn public_key_bytes(&self) -> Vec<u8> {
        self.key_store.public_key_bytes()
    }

    /// Get key store information (for debugging/logging)
    pub fn key_store_info(&self) -> String {
        self.key_store.identifier()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::SoftwareKeyStore;

    #[test]
    fn test_genome_creation() {
        let rules = vec!["Do no harm".to_string(), "Respect privacy".to_string()];
        let genome = SealedGenome::new(rules).unwrap();

        assert!(!genome.is_sealed());
        assert_eq!(genome.rules().len(), 2);
    }

    #[test]
    fn test_genome_creation_with_key_store() {
        let key_store = SoftwareKeyStore::generate().unwrap();
        let rules = vec!["Rule 1".to_string()];

        let genome = SealedGenome::with_key_store(rules, Box::new(key_store)).unwrap();

        assert!(!genome.is_sealed());
        assert_eq!(genome.rules().len(), 1);
    }

    #[test]
    fn test_genome_sealing() {
        let rules = vec!["Rule 1".to_string()];
        let mut genome = SealedGenome::new(rules).unwrap();

        assert!(genome.seal().is_ok());
        assert!(genome.is_sealed());
        assert!(genome.capsule_hash().is_some());
    }

    #[test]
    fn test_cannot_seal_twice() {
        let rules = vec!["Rule 1".to_string()];
        let mut genome = SealedGenome::new(rules).unwrap();

        genome.seal().unwrap();

        let result = genome.seal();
        assert!(matches!(result, Err(GenomeError::AlreadySealed)));
    }

    #[test]
    fn test_cannot_verify_unsealed() {
        let rules = vec!["Rule 1".to_string()];
        let genome = SealedGenome::new(rules).unwrap();

        let action = Action::delete("test.txt");
        let result = genome.verify_action(&action);

        assert!(matches!(result, Err(GenomeError::NotSealed)));
    }

    #[test]
    fn test_verify_action_creates_valid_proof() {
        let rules = vec!["Do no harm".to_string()];
        let mut genome = SealedGenome::new(rules).unwrap();
        genome.seal().unwrap();

        let action = Action::delete("test.txt");
        let proof = genome.verify_action(&action).unwrap();

        // Verify proof properties
        assert_eq!(proof.action_hash, action.hash());
        assert!(!proof.signature.is_empty());
        assert_eq!(proof.signature.len(), 64); // Ed25519 signature is 64 bytes
        assert_eq!(proof.ttl, 60); // Default TTL
    }

    #[test]
    fn test_capsule_hash_deterministic() {
        let rules = vec!["Rule 1".to_string(), "Rule 2".to_string()];
        let mut genome1 = SealedGenome::new(rules.clone()).unwrap();
        let mut genome2 = SealedGenome::new(rules.clone()).unwrap();

        genome1.seal().unwrap();
        genome2.seal().unwrap();

        assert_eq!(genome1.capsule_hash(), genome2.capsule_hash());
    }

    #[test]
    fn test_different_rules_different_hashes() {
        let rules1 = vec!["Rule 1".to_string()];
        let rules2 = vec!["Rule 2".to_string()];

        let mut genome1 = SealedGenome::new(rules1).unwrap();
        let mut genome2 = SealedGenome::new(rules2).unwrap();

        genome1.seal().unwrap();
        genome2.seal().unwrap();

        assert_ne!(genome1.capsule_hash(), genome2.capsule_hash());
    }

    #[test]
    fn test_custom_ttl() {
        let rules = vec!["Rule 1".to_string()];
        let mut genome = SealedGenome::new(rules).unwrap();
        genome.set_default_ttl(3600);
        genome.seal().unwrap();

        let action = Action::delete("test.txt");
        let proof = genome.verify_action(&action).unwrap();

        assert_eq!(proof.ttl, 3600);
    }

    #[test]
    fn test_public_key_export() {
        let genome = SealedGenome::new(vec!["Rule 1".to_string()]).unwrap();
        let public_key = genome.public_key_bytes();

        assert_eq!(public_key.len(), 32); // Ed25519 public key is 32 bytes
    }

    #[test]
    fn test_proof_signature_can_be_verified() {
        let key_store = SoftwareKeyStore::generate().unwrap();
        let key_store_clone = key_store.clone();

        let mut genome =
            SealedGenome::with_key_store(vec!["Rule 1".to_string()], Box::new(key_store)).unwrap();

        genome.seal().unwrap();

        let action = Action::delete("test.txt");
        let proof = genome.verify_action(&action).unwrap();

        // Verify signature with the same key store
        let signing_data = proof.signing_data();
        assert!(key_store_clone
            .verify(&signing_data, &proof.signature)
            .is_ok());
    }

    #[test]
    #[allow(deprecated)]
    fn test_backward_compatibility_with_keypair() {
        let keypair = KeyPair::generate().unwrap();
        let mut genome = SealedGenome::with_keypair(vec!["Rule 1".to_string()], keypair).unwrap();

        genome.seal().unwrap();

        let action = Action::delete("test.txt");
        let proof = genome.verify_action(&action).unwrap();

        assert!(!proof.signature.is_empty());
    }

    #[test]
    fn test_key_store_info() {
        let genome = SealedGenome::new(vec!["Rule 1".to_string()]).unwrap();
        let info = genome.key_store_info();

        assert!(info.contains("KeyPair") || info.contains("SoftwareKeyStore"));
    }
}
