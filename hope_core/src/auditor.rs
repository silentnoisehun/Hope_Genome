use crate::crypto::KeyPair;
use crate::proof::IntegrityProof;
use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuditorError {
    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Proof expired: issued at {issued}, now {now}, TTL {ttl}s")]
    ProofExpired { issued: u64, now: u64, ttl: u64 },

    #[error("Nonce already used (replay attack detected)")]
    NonceReused([u8; 32]),

    #[error("Crypto error: {0}")]
    CryptoError(#[from] crate::crypto::CryptoError),
}

pub type Result<T> = std::result::Result<T, AuditorError>;

/// Proof verification engine with anti-replay protection
///
/// The auditor maintains state to prevent replay attacks:
/// - Tracks used nonces
/// - Verifies TTL (time-to-live)
/// - Validates RSA signatures
pub struct ProofAuditor {
    /// Set of nonces that have been used (anti-replay)
    used_nonces: HashSet<[u8; 32]>,

    /// RSA keypair for signature verification
    keypair: KeyPair,
}

impl ProofAuditor {
    /// Create a new proof auditor
    pub fn new(keypair: KeyPair) -> Self {
        ProofAuditor {
            used_nonces: HashSet::new(),
            keypair,
        }
    }

    /// Verify a cryptographic proof
    ///
    /// This performs comprehensive verification:
    /// 1. Signature verification (prevents forgery)
    /// 2. TTL check (prevents stale proofs)
    /// 3. Nonce check (prevents replay attacks)
    pub fn verify_proof(&mut self, proof: &IntegrityProof) -> Result<()> {
        // 1. Verify signature first (most critical)
        self.verify_signature(proof)?;

        // 2. Check TTL (time-to-live)
        let now = chrono::Utc::now().timestamp() as u64;
        if now - proof.timestamp > proof.ttl {
            return Err(AuditorError::ProofExpired {
                issued: proof.timestamp,
                now,
                ttl: proof.ttl,
            });
        }

        // 3. Check nonce (anti-replay)
        if self.used_nonces.contains(&proof.nonce) {
            return Err(AuditorError::NonceReused(proof.nonce));
        }

        // 4. Mark nonce as used
        self.used_nonces.insert(proof.nonce);

        Ok(())
    }

    /// Verify just the signature (without state changes)
    pub fn verify_signature(&self, proof: &IntegrityProof) -> Result<()> {
        let message = proof.signing_data();
        self.keypair
            .verify(&message, &proof.signature)
            .map_err(|_| AuditorError::InvalidSignature)?;

        Ok(())
    }

    /// Check if a nonce has been used
    pub fn is_nonce_used(&self, nonce: &[u8; 32]) -> bool {
        self.used_nonces.contains(nonce)
    }

    /// Get count of used nonces
    pub fn used_nonce_count(&self) -> usize {
        self.used_nonces.len()
    }

    /// Clear old nonces (for memory management in long-running systems)
    /// In production, use a time-based LRU cache
    pub fn clear_nonces(&mut self) {
        self.used_nonces.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proof::Action;

    fn create_test_proof(keypair: &KeyPair) -> IntegrityProof {
        let action = Action::delete("test.txt");
        let mut proof = IntegrityProof::new(&action, "test_capsule".into(), 60);

        // Sign it
        let signing_data = proof.signing_data();
        proof.signature = keypair.sign(&signing_data).unwrap();

        proof
    }

    #[test]
    fn test_verify_valid_proof() {
        let keypair = KeyPair::generate().unwrap();
        let mut auditor = ProofAuditor::new(keypair);

        let proof = create_test_proof(&auditor.keypair);

        assert!(auditor.verify_proof(&proof).is_ok());
    }

    #[test]
    fn test_replay_attack_prevention() {
        let keypair = KeyPair::generate().unwrap();
        let mut auditor = ProofAuditor::new(keypair);

        let proof = create_test_proof(&auditor.keypair);

        // First use: should succeed
        assert!(auditor.verify_proof(&proof).is_ok());

        // Second use: should FAIL (nonce reused)
        let result = auditor.verify_proof(&proof);
        assert!(matches!(result, Err(AuditorError::NonceReused(_))));
    }

    #[test]
    fn test_ttl_expiration() {
        let keypair = KeyPair::generate().unwrap();
        let mut auditor = ProofAuditor::new(keypair);

        // Create proof with very short TTL
        let action = Action::delete("test.txt");
        let mut proof = IntegrityProof::new(&action, "test_capsule".into(), 0);

        // Set timestamp to 10 seconds ago
        proof.timestamp = chrono::Utc::now().timestamp() as u64 - 10;

        // Sign it
        let signing_data = proof.signing_data();
        proof.signature = auditor.keypair.sign(&signing_data).unwrap();

        // Should fail (expired)
        let result = auditor.verify_proof(&proof);
        assert!(matches!(result, Err(AuditorError::ProofExpired { .. })));
    }

    #[test]
    fn test_invalid_signature() {
        let keypair = KeyPair::generate().unwrap();
        let mut auditor = ProofAuditor::new(keypair);

        let mut proof = create_test_proof(&auditor.keypair);

        // Tamper with signature
        proof.signature[0] ^= 0xFF;

        // Should fail (invalid signature)
        let result = auditor.verify_proof(&proof);
        assert!(matches!(result, Err(AuditorError::InvalidSignature)));
    }

    #[test]
    fn test_nonce_tracking() {
        let keypair = KeyPair::generate().unwrap();
        let mut auditor = ProofAuditor::new(keypair);

        let proof1 = create_test_proof(&auditor.keypair);
        let proof2 = create_test_proof(&auditor.keypair);

        assert_eq!(auditor.used_nonce_count(), 0);

        auditor.verify_proof(&proof1).unwrap();
        assert_eq!(auditor.used_nonce_count(), 1);

        auditor.verify_proof(&proof2).unwrap();
        assert_eq!(auditor.used_nonce_count(), 2);
    }
}
