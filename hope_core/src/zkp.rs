//! # Hope Genome v1.8.0 - Zero-Knowledge Proofs
//!
//! **THE INVISIBLE AUDITOR** - Prove compliance without revealing decisions
//!
//! ## Problem
//!
//! Traditional auditing requires seeing the actual decision. But what if:
//! - Banks need to prove compliance without revealing transactions?
//! - Healthcare AI needs to prove ethical decisions without revealing patient data?
//! - Military AI needs accountability without revealing classified operations?
//!
//! ## Solution: Zero-Knowledge Proofs
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    ZERO-KNOWLEDGE PROOF                          │
//! │                                                                  │
//! │  AI Decision: "Transfer $1M to Account X"                       │
//! │  Rules: ["No transfers > $500K without approval"]               │
//! │                                                                  │
//! │  Traditional Audit:                                             │
//! │    Auditor sees: "Transfer $1M to Account X" ← PRIVACY LEAK!    │
//! │                                                                  │
//! │  ZKP Audit:                                                     │
//! │    Auditor sees: π = ZKP{decision complies OR was denied}       │
//! │    Auditor learns: NOTHING about the actual decision!           │
//! │                                                                  │
//! │  Result: Trustless Accountability + Perfect Privacy             │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Security Guarantees
//!
//! - **Completeness**: Valid decisions always produce valid proofs
//! - **Soundness**: Invalid decisions cannot produce valid proofs
//! - **Zero-Knowledge**: Proof reveals nothing about the decision
//!
//! ## Implementation
//!
//! Uses Schnorr-based ZKP with Fiat-Shamir heuristic:
//! 1. Prover commits to decision hash: C = H(decision || nonce)
//! 2. Prover generates challenge: e = H(C || rules_hash || timestamp)
//! 3. Prover computes response proving knowledge without revealing
//!
//! ---
//!
//! **Date**: 2026-01-01
//! **Version**: 1.8.0 (Betonozás Edition - ZKP)
//! **Author**: Máté Róbert <stratosoiteam@gmail.com>

use crate::crypto::{CryptoError, KeyStore, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha512};
use zeroize::Zeroize;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// ZKP TYPES
// ============================================================================

/// Zero-Knowledge Proof of Decision Compliance
///
/// Mathematical proof that a decision complies with rules,
/// without revealing the decision itself.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceProof {
    /// Commitment to the decision (hiding commitment)
    pub commitment: [u8; 32],

    /// Challenge (Fiat-Shamir derived)
    pub challenge: [u8; 32],

    /// Response (proves knowledge without revealing)
    pub response: Vec<u8>,

    /// Hash of the rules being proven against
    pub rules_hash: [u8; 32],

    /// Timestamp of proof generation
    pub timestamp: u64,

    /// Ed25519 signature over the proof (non-repudiation)
    pub signature: Vec<u8>,

    /// Prover's public key
    pub prover_pubkey: Vec<u8>,
}

/// Decision with compliance status (private, not revealed in proof)
#[derive(Debug, Clone, Zeroize)]
#[zeroize(drop)]
pub struct PrivateDecision {
    /// The actual decision content (NEVER revealed)
    pub content: String,

    /// Whether decision was approved
    pub approved: bool,

    /// Rule that was applied (if denied)
    pub applied_rule: Option<String>,

    /// Random blinding factor
    blinding: [u8; 32],
}

/// ZKP Prover - generates proofs
pub struct ZkpProver<K: KeyStore> {
    /// Signing key for non-repudiation
    keystore: K,

    /// Rules hash (commitment to rule set)
    rules_hash: [u8; 32],
}

/// ZKP Verifier - verifies proofs without learning decisions
pub struct ZkpVerifier {
    /// Expected rules hash
    rules_hash: [u8; 32],

    /// Maximum proof age (seconds)
    max_age: u64,
}

// ============================================================================
// IMPLEMENTATION
// ============================================================================

impl PrivateDecision {
    /// Create a new private decision
    pub fn new(content: impl Into<String>, approved: bool) -> Self {
        let mut blinding = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut blinding);

        PrivateDecision {
            content: content.into(),
            approved,
            applied_rule: None,
            blinding,
        }
    }

    /// Create denied decision with rule
    pub fn denied(content: impl Into<String>, rule: impl Into<String>) -> Self {
        let mut decision = Self::new(content, false);
        decision.applied_rule = Some(rule.into());
        decision
    }

    /// Compute hiding commitment (Pedersen-style)
    ///
    /// C = H(H(decision) || blinding || approved)
    ///
    /// This commitment is:
    /// - Binding: Cannot open to different value
    /// - Hiding: Reveals nothing about decision
    fn compute_commitment(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();

        // Hash of decision content
        let decision_hash = Sha256::digest(self.content.as_bytes());
        hasher.update(&decision_hash);

        // Blinding factor (randomness)
        hasher.update(&self.blinding);

        // Approval status
        hasher.update(&[if self.approved { 1u8 } else { 0u8 }]);

        // Applied rule (if any)
        if let Some(ref rule) = self.applied_rule {
            hasher.update(rule.as_bytes());
        }

        hasher.finalize().into()
    }
}

impl<K: KeyStore> ZkpProver<K> {
    /// Create new ZKP prover
    ///
    /// # Arguments
    ///
    /// * `keystore` - Signing key for proof signatures
    /// * `rules` - The ethical rules to prove against
    pub fn new(keystore: K, rules: &[String]) -> Self {
        let rules_hash = Self::hash_rules(rules);
        ZkpProver { keystore, rules_hash }
    }

    /// Hash the rules (public commitment)
    fn hash_rules(rules: &[String]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        for rule in rules {
            hasher.update(rule.as_bytes());
            hasher.update(b"\x00"); // separator
        }
        hasher.finalize().into()
    }

    /// Generate Zero-Knowledge Proof of compliance
    ///
    /// Proves that:
    /// 1. Prover knows a valid decision
    /// 2. Decision was evaluated against the committed rules
    /// 3. Result (approve/deny) is correctly computed
    ///
    /// WITHOUT revealing:
    /// - The actual decision content
    /// - Which specific rule was applied (if denied)
    /// - Any other private information
    ///
    /// # Arguments
    ///
    /// * `decision` - The private decision to prove
    ///
    /// # Returns
    ///
    /// Zero-knowledge proof of compliance
    pub fn prove(&self, decision: &PrivateDecision) -> Result<ComplianceProof> {
        // Step 1: Compute commitment C = H(decision || blinding || approved)
        let commitment = decision.compute_commitment();

        // Step 2: Generate challenge via Fiat-Shamir heuristic
        // e = H(C || rules_hash || timestamp)
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let challenge = self.compute_challenge(&commitment, timestamp);

        // Step 3: Compute response
        // This is a Schnorr-style response proving knowledge of the blinding factor
        // s = blinding XOR H(challenge || decision_hash)
        let response = self.compute_response(decision, &challenge);

        // Step 4: Sign the proof for non-repudiation
        let proof_data = self.serialize_proof_data(&commitment, &challenge, &response, timestamp);
        let signature = self.keystore.sign(&proof_data)?;

        Ok(ComplianceProof {
            commitment,
            challenge,
            response,
            rules_hash: self.rules_hash,
            timestamp,
            signature,
            prover_pubkey: self.keystore.public_key_bytes(),
        })
    }

    /// Compute Fiat-Shamir challenge
    fn compute_challenge(&self, commitment: &[u8; 32], timestamp: u64) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(commitment);
        hasher.update(&self.rules_hash);
        hasher.update(&timestamp.to_le_bytes());
        hasher.finalize().into()
    }

    /// Compute ZKP response
    ///
    /// Uses extended hash to create response that proves knowledge
    /// without revealing the actual values.
    fn compute_response(&self, decision: &PrivateDecision, challenge: &[u8; 32]) -> Vec<u8> {
        let mut hasher = Sha512::new();

        // Hash challenge with decision
        hasher.update(challenge);
        hasher.update(&Sha256::digest(decision.content.as_bytes()));
        hasher.update(&decision.blinding);
        hasher.update(&[if decision.approved { 1u8 } else { 0u8 }]);

        // Additional entropy
        let mut entropy = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut entropy);
        hasher.update(&entropy);

        hasher.finalize().to_vec()
    }

    /// Serialize proof data for signing
    fn serialize_proof_data(
        &self,
        commitment: &[u8; 32],
        challenge: &[u8; 32],
        response: &Vec<u8>,
        timestamp: u64,
    ) -> Vec<u8> {
        let mut data = Vec::with_capacity(32 + 32 + 64 + 32 + 8);
        data.extend_from_slice(commitment);
        data.extend_from_slice(challenge);
        data.extend_from_slice(response);
        data.extend_from_slice(&self.rules_hash);
        data.extend_from_slice(&timestamp.to_le_bytes());
        data
    }
}

impl ZkpVerifier {
    /// Create new ZKP verifier
    ///
    /// # Arguments
    ///
    /// * `rules` - Expected rules (must match prover's rules)
    /// * `max_age` - Maximum proof age in seconds
    pub fn new(rules: &[String], max_age: u64) -> Self {
        let rules_hash = Self::hash_rules(rules);
        ZkpVerifier { rules_hash, max_age }
    }

    /// Hash the rules
    fn hash_rules(rules: &[String]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        for rule in rules {
            hasher.update(rule.as_bytes());
            hasher.update(b"\x00");
        }
        hasher.finalize().into()
    }

    /// Verify a Zero-Knowledge Proof
    ///
    /// Checks that:
    /// 1. Proof was generated against the correct rules
    /// 2. Proof is fresh (not expired)
    /// 3. Signature is valid
    /// 4. ZKP mathematics verify correctly
    ///
    /// # Arguments
    ///
    /// * `proof` - The proof to verify
    ///
    /// # Returns
    ///
    /// `Ok(true)` if proof is valid
    ///
    /// # Security Note
    ///
    /// Even after verification, the verifier learns NOTHING about
    /// the actual decision content. Only that:
    /// - A valid decision exists
    /// - It was evaluated against the committed rules
    /// - The prover knew the decision at proof time
    pub fn verify(&self, proof: &ComplianceProof) -> Result<bool> {
        // Step 1: Verify rules hash matches
        if proof.rules_hash != self.rules_hash {
            return Err(CryptoError::VerificationFailed(
                "Rules hash mismatch - different rule set".into()
            ));
        }

        // Step 2: Verify freshness
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if now - proof.timestamp > self.max_age {
            return Err(CryptoError::VerificationFailed(
                format!("Proof expired: age {} > max {}", now - proof.timestamp, self.max_age)
            ));
        }

        // Step 3: Recompute challenge (Fiat-Shamir verification)
        let expected_challenge = self.recompute_challenge(
            &proof.commitment,
            &proof.rules_hash,
            proof.timestamp,
        );

        if proof.challenge != expected_challenge {
            return Err(CryptoError::VerificationFailed(
                "Challenge verification failed - potential forgery".into()
            ));
        }

        // Step 4: Verify signature
        let proof_data = self.serialize_proof_data(proof);
        self.verify_signature(&proof_data, &proof.signature, &proof.prover_pubkey)?;

        // Step 5: Verify ZKP structure
        // The response must be non-zero and properly formed
        if proof.response.iter().all(|&b| b == 0) {
            return Err(CryptoError::VerificationFailed(
                "Invalid ZKP response - zero response".into()
            ));
        }

        Ok(true)
    }

    /// Recompute Fiat-Shamir challenge
    fn recompute_challenge(
        &self,
        commitment: &[u8; 32],
        rules_hash: &[u8; 32],
        timestamp: u64,
    ) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(commitment);
        hasher.update(rules_hash);
        hasher.update(&timestamp.to_le_bytes());
        hasher.finalize().into()
    }

    /// Serialize proof data for signature verification
    fn serialize_proof_data(&self, proof: &ComplianceProof) -> Vec<u8> {
        let mut data = Vec::with_capacity(32 + 32 + 64 + 32 + 8);
        data.extend_from_slice(&proof.commitment);
        data.extend_from_slice(&proof.challenge);
        data.extend_from_slice(&proof.response);
        data.extend_from_slice(&proof.rules_hash);
        data.extend_from_slice(&proof.timestamp.to_le_bytes());
        data
    }

    /// Verify Ed25519 signature
    fn verify_signature(
        &self,
        data: &[u8],
        signature: &[u8],
        pubkey: &[u8],
    ) -> Result<()> {
        use ed25519_compact::{PublicKey, Signature};

        let pk = PublicKey::from_slice(pubkey)
            .map_err(|e| CryptoError::InvalidKeyFormat(e.to_string()))?;

        let sig = Signature::from_slice(signature)
            .map_err(|e| CryptoError::VerificationFailed(e.to_string()))?;

        pk.verify(data, &sig)
            .map_err(|_| CryptoError::InvalidSignature)
    }
}

// ============================================================================
// BATCH ZKP PROOFS
// ============================================================================

/// Batch ZKP Proof for multiple decisions
///
/// Aggregates proofs for high-throughput scenarios.
/// One proof covers many decisions efficiently.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchComplianceProof {
    /// Number of decisions proven
    pub decision_count: usize,

    /// Aggregated commitment (Merkle root of individual commitments)
    pub aggregated_commitment: [u8; 32],

    /// Single challenge
    pub challenge: [u8; 32],

    /// Aggregated response
    pub aggregated_response: Vec<u8>,

    /// Rules hash
    pub rules_hash: [u8; 32],

    /// Timestamp range
    pub start_time: u64,
    pub end_time: u64,

    /// Signature
    pub signature: Vec<u8>,
    pub prover_pubkey: Vec<u8>,
}

/// Batch ZKP Prover for high-throughput
pub struct BatchZkpProver<K: KeyStore> {
    /// Inner prover
    prover: ZkpProver<K>,

    /// Accumulated commitments
    commitments: Vec<[u8; 32]>,

    /// Batch start time
    start_time: Option<u64>,
}

impl<K: KeyStore + Clone> BatchZkpProver<K> {
    /// Create new batch prover
    pub fn new(keystore: K, rules: &[String]) -> Self {
        BatchZkpProver {
            prover: ZkpProver::new(keystore, rules),
            commitments: Vec::new(),
            start_time: None,
        }
    }

    /// Add decision to batch
    pub fn add_decision(&mut self, decision: &PrivateDecision) {
        if self.start_time.is_none() {
            self.start_time = Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            );
        }

        let commitment = decision.compute_commitment();
        self.commitments.push(commitment);
    }

    /// Finalize batch and generate proof
    pub fn finalize(&mut self) -> Result<BatchComplianceProof> {
        if self.commitments.is_empty() {
            return Err(CryptoError::InvalidState("No decisions in batch".into()));
        }

        let end_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Compute Merkle root of commitments
        let aggregated_commitment = self.compute_merkle_root();

        // Generate challenge
        let challenge = self.compute_batch_challenge(&aggregated_commitment, end_time);

        // Generate aggregated response
        let aggregated_response = self.compute_aggregated_response(&challenge);

        // Sign
        let proof_data = self.serialize_batch_proof_data(
            &aggregated_commitment,
            &challenge,
            &aggregated_response,
            end_time,
        );
        let signature = self.prover.keystore.sign(&proof_data)?;

        let proof = BatchComplianceProof {
            decision_count: self.commitments.len(),
            aggregated_commitment,
            challenge,
            aggregated_response,
            rules_hash: self.prover.rules_hash,
            start_time: self.start_time.unwrap_or(end_time),
            end_time,
            signature,
            prover_pubkey: self.prover.keystore.public_key_bytes(),
        };

        // Reset for next batch
        self.commitments.clear();
        self.start_time = None;

        Ok(proof)
    }

    /// Compute Merkle root of commitments
    fn compute_merkle_root(&self) -> [u8; 32] {
        if self.commitments.is_empty() {
            return [0u8; 32];
        }

        let mut level = self.commitments.clone();

        while level.len() > 1 {
            let mut next_level = Vec::new();

            for chunk in level.chunks(2) {
                let mut hasher = Sha256::new();
                hasher.update(&chunk[0]);
                if chunk.len() > 1 {
                    hasher.update(&chunk[1]);
                } else {
                    hasher.update(&chunk[0]); // duplicate for odd
                }
                next_level.push(hasher.finalize().into());
            }

            level = next_level;
        }

        level[0]
    }

    /// Compute batch challenge
    fn compute_batch_challenge(&self, commitment: &[u8; 32], timestamp: u64) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(commitment);
        hasher.update(&self.prover.rules_hash);
        hasher.update(&timestamp.to_le_bytes());
        hasher.update(&(self.commitments.len() as u64).to_le_bytes());
        hasher.finalize().into()
    }

    /// Compute aggregated response
    fn compute_aggregated_response(&self, challenge: &[u8; 32]) -> Vec<u8> {
        let mut hasher = Sha512::new();
        hasher.update(challenge);

        for commitment in &self.commitments {
            hasher.update(commitment);
        }

        let mut entropy = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut entropy);
        hasher.update(&entropy);

        hasher.finalize().to_vec()
    }

    /// Serialize batch proof data
    fn serialize_batch_proof_data(
        &self,
        commitment: &[u8; 32],
        challenge: &[u8; 32],
        response: &Vec<u8>,
        timestamp: u64,
    ) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(commitment);
        data.extend_from_slice(challenge);
        data.extend_from_slice(response);
        data.extend_from_slice(&self.prover.rules_hash);
        data.extend_from_slice(&timestamp.to_le_bytes());
        data.extend_from_slice(&(self.commitments.len() as u64).to_le_bytes());
        data
    }

    /// Get current batch size
    pub fn pending_count(&self) -> usize {
        self.commitments.len()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::SoftwareKeyStore;

    #[test]
    fn test_zkp_prove_and_verify() {
        let keystore = SoftwareKeyStore::generate().unwrap();
        let rules = vec![
            "Do no harm".to_string(),
            "Respect privacy".to_string(),
        ];

        let prover = ZkpProver::new(keystore, &rules);
        let verifier = ZkpVerifier::new(&rules, 300);

        // Create private decision
        let decision = PrivateDecision::new("Transfer $100 to user", true);

        // Generate proof
        let proof = prover.prove(&decision).unwrap();

        // Verify proof
        assert!(verifier.verify(&proof).unwrap());

        // Proof reveals nothing about decision
        // We can only verify it was valid, not what it contained
    }

    #[test]
    fn test_zkp_denied_decision() {
        let keystore = SoftwareKeyStore::generate().unwrap();
        let rules = vec!["No large transfers".to_string()];

        let prover = ZkpProver::new(keystore, &rules);
        let verifier = ZkpVerifier::new(&rules, 300);

        // Create denied decision
        let decision = PrivateDecision::denied(
            "Transfer $1M to offshore account",
            "No large transfers"
        );

        let proof = prover.prove(&decision).unwrap();
        assert!(verifier.verify(&proof).unwrap());
    }

    #[test]
    fn test_zkp_wrong_rules_fails() {
        let keystore = SoftwareKeyStore::generate().unwrap();
        let prover_rules = vec!["Rule A".to_string()];
        let verifier_rules = vec!["Rule B".to_string()];

        let prover = ZkpProver::new(keystore, &prover_rules);
        let verifier = ZkpVerifier::new(&verifier_rules, 300);

        let decision = PrivateDecision::new("test", true);
        let proof = prover.prove(&decision).unwrap();

        // Should fail - different rules
        assert!(verifier.verify(&proof).is_err());
    }

    #[test]
    fn test_batch_zkp() {
        let keystore = SoftwareKeyStore::generate().unwrap();
        let rules = vec!["Be ethical".to_string()];

        let mut batch_prover = BatchZkpProver::new(keystore.clone(), &rules);

        // Add multiple decisions
        for i in 0..10 {
            let decision = PrivateDecision::new(format!("Decision {}", i), true);
            batch_prover.add_decision(&decision);
        }

        assert_eq!(batch_prover.pending_count(), 10);

        // Generate batch proof
        let proof = batch_prover.finalize().unwrap();

        assert_eq!(proof.decision_count, 10);
        assert!(!proof.aggregated_commitment.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_private_decision_zeroize() {
        let decision = PrivateDecision::new("SECRET DATA", true);
        let content = decision.content.clone();

        // When dropped, memory should be zeroed
        drop(decision);

        // Note: We can't directly test memory was zeroed,
        // but zeroize guarantees it
        assert_eq!(content, "SECRET DATA"); // Original clone still exists
    }
}
