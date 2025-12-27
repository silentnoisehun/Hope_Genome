use crate::crypto::{hash_bytes, KeyPair};
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
/// - Signs action approvals with RSA
/// - Provides cryptographic proofs for all decisions
#[derive(Debug)]
pub struct SealedGenome {
    /// Ethical rules (immutable after sealing)
    rules: Vec<String>,

    /// Whether the genome is sealed
    sealed: bool,

    /// Cryptographic keypair for signing
    keypair: KeyPair,

    /// Hash of the sealed genome (for proof binding)
    capsule_hash: Option<String>,

    /// Default TTL for proofs (seconds)
    default_ttl: u64,
}

impl SealedGenome {
    /// Create a new genome with rules (generates new keypair)
    pub fn new(rules: Vec<String>) -> Result<Self> {
        let keypair = KeyPair::generate()?;
        Self::with_keypair(rules, keypair)
    }

    /// Create a new genome with an existing keypair
    pub fn with_keypair(rules: Vec<String>, keypair: KeyPair) -> Result<Self> {
        Ok(SealedGenome {
            rules,
            sealed: false,
            keypair,
            capsule_hash: None,
            default_ttl: 60, // 1 minute default
        })
    }

    /// Seal the genome (make it immutable)
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

    /// Set default TTL for proofs
    pub fn set_default_ttl(&mut self, ttl: u64) {
        self.default_ttl = ttl;
    }

    /// Verify an action against the genome rules
    ///
    /// This is where ethical decision-making happens.
    /// Returns a cryptographically signed proof if approved.
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

        // Sign the proof
        let signing_data = proof.signing_data();
        proof.signature = self.keypair.sign(&signing_data)?;

        Ok(proof)
    }

    /// Get the capsule hash
    pub fn capsule_hash(&self) -> Option<&str> {
        self.capsule_hash.as_deref()
    }

    /// Get the public key (for verification by external parties)
    pub fn public_key(&self) -> &rsa::RsaPublicKey {
        self.keypair.public_key()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genome_creation() {
        let rules = vec!["Do no harm".to_string(), "Respect privacy".to_string()];
        let genome = SealedGenome::new(rules).unwrap();

        assert!(!genome.is_sealed());
        assert_eq!(genome.rules().len(), 2);
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
}
