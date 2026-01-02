//! # Proof Chain - Atomic Response System
//!
//! **PHASE 2: Every response is part of a proof chain**
//!
//! All outputs form a cryptographic chain. A single hash
//! can verify an entire session's compliance.
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                     PROOF CHAIN                                 │
//! ├─────────────────────────────────────────────────────────────────┤
//! │                                                                 │
//! │   Response #1 ──► Proof π₁ ──┐                                  │
//! │                              │                                  │
//! │   Response #2 ──► Proof π₂ ──┼──► Merkle Root                   │
//! │                              │        │                         │
//! │   Response #3 ──► Proof π₃ ──┘        │                         │
//! │                                       ▼                         │
//! │                              ┌─────────────────┐                │
//! │                              │  Session Proof  │                │
//! │                              │  (single hash)  │                │
//! │                              └─────────────────┘                │
//! │                                       │                         │
//! │   Verify entire session with ONE hash!                          │
//! │                                                                 │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

use super::zk_snark::DiamondProof;

// ============================================================================
// PROOF CHAIN TYPES
// ============================================================================

/// A chain of proofs for a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofChain {
    /// Session identifier
    pub session_id: [u8; 32],

    /// Chain of proofs
    pub proofs: Vec<ChainedProof>,

    /// Current Merkle root
    pub merkle_root: [u8; 32],

    /// Session start time
    pub started_at: u64,

    /// Last update time
    pub last_updated: u64,

    /// Chain status
    pub status: ChainStatus,
}

/// A proof linked in the chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainedProof {
    /// Sequence number in chain
    pub sequence: u64,

    /// The Diamond proof
    pub proof: DiamondProof,

    /// Hash of previous proof
    pub prev_hash: [u8; 32],

    /// This proof's hash
    pub proof_hash: [u8; 32],

    /// Input hash (for replay detection)
    pub input_hash: [u8; 32],

    /// Output hash
    pub output_hash: [u8; 32],

    /// Timestamp
    pub timestamp: u64,
}

/// Session proof - single hash proving entire session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionProof {
    /// Session ID
    pub session_id: [u8; 32],

    /// Merkle root of all proofs
    pub merkle_root: [u8; 32],

    /// Number of proofs in session
    pub proof_count: u64,

    /// Session time range
    pub started_at: u64,
    pub ended_at: u64,

    /// Rules hash used for session
    pub rules_hash: [u8; 32],

    /// Signature over session proof
    pub signature: Vec<u8>,
}

/// Global proof root - for cross-session verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalProofRoot {
    /// Root hash of all sessions
    pub root_hash: [u8; 32],

    /// Number of sessions included
    pub session_count: u64,

    /// Timestamp range
    pub from: u64,
    pub to: u64,

    /// Signature
    pub signature: Vec<u8>,
}

/// Chain status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChainStatus {
    /// Chain is active and accepting new proofs
    Active,

    /// Chain is finalized (no more proofs)
    Finalized,

    /// Chain detected integrity violation
    Corrupted,
}

// ============================================================================
// PROOF CHAIN IMPLEMENTATION
// ============================================================================

impl ProofChain {
    /// Create a new proof chain for a session
    pub fn new() -> Self {
        let mut session_id = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut session_id);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        ProofChain {
            session_id,
            proofs: Vec::new(),
            merkle_root: [0u8; 32],
            started_at: now,
            last_updated: now,
            status: ChainStatus::Active,
        }
    }

    /// Add a proof to the chain
    pub fn add_proof(
        &mut self,
        proof: DiamondProof,
        input: &str,
        output: &str,
    ) -> Result<ChainedProof, ChainError> {
        if self.status != ChainStatus::Active {
            return Err(ChainError::ChainNotActive);
        }

        let sequence = self.proofs.len() as u64;

        // Get previous hash
        let prev_hash = self
            .proofs
            .last()
            .map(|p| p.proof_hash)
            .unwrap_or([0u8; 32]);

        // Compute hashes
        let input_hash: [u8; 32] = Sha256::digest(input.as_bytes()).into();
        let output_hash: [u8; 32] = Sha256::digest(output.as_bytes()).into();

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Compute proof hash
        let proof_hash = Self::compute_proof_hash(
            sequence,
            &prev_hash,
            &proof,
            &input_hash,
            &output_hash,
            timestamp,
        );

        let chained = ChainedProof {
            sequence,
            proof,
            prev_hash,
            proof_hash,
            input_hash,
            output_hash,
            timestamp,
        };

        self.proofs.push(chained.clone());
        self.last_updated = timestamp;

        // Update Merkle root
        self.merkle_root = self.compute_merkle_root();

        Ok(chained)
    }

    /// Finalize the chain (no more proofs allowed)
    pub fn finalize(&mut self) -> SessionProof {
        self.status = ChainStatus::Finalized;

        let ended_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let rules_hash = self
            .proofs
            .first()
            .map(|p| p.proof.public_inputs.rules_hash)
            .unwrap_or([0u8; 32]);

        // Generate session signature
        let signature = self.generate_session_signature(ended_at);

        SessionProof {
            session_id: self.session_id,
            merkle_root: self.merkle_root,
            proof_count: self.proofs.len() as u64,
            started_at: self.started_at,
            ended_at,
            rules_hash,
            signature,
        }
    }

    /// Verify chain integrity
    pub fn verify_integrity(&self) -> Result<bool, ChainError> {
        if self.proofs.is_empty() {
            return Ok(true);
        }

        // Verify chain links
        for (i, proof) in self.proofs.iter().enumerate() {
            // Check sequence
            if proof.sequence != i as u64 {
                return Err(ChainError::SequenceMismatch {
                    expected: i as u64,
                    got: proof.sequence,
                });
            }

            // Check prev_hash link
            if i == 0 {
                if proof.prev_hash != [0u8; 32] {
                    return Err(ChainError::InvalidFirstLink);
                }
            } else {
                let expected_prev = self.proofs[i - 1].proof_hash;
                if proof.prev_hash != expected_prev {
                    return Err(ChainError::BrokenLink {
                        at_sequence: proof.sequence,
                    });
                }
            }

            // Verify proof hash
            let computed_hash = Self::compute_proof_hash(
                proof.sequence,
                &proof.prev_hash,
                &proof.proof,
                &proof.input_hash,
                &proof.output_hash,
                proof.timestamp,
            );

            if proof.proof_hash != computed_hash {
                return Err(ChainError::HashMismatch {
                    at_sequence: proof.sequence,
                });
            }
        }

        // Verify Merkle root
        let computed_root = self.compute_merkle_root();
        if computed_root != self.merkle_root {
            return Err(ChainError::MerkleRootMismatch);
        }

        Ok(true)
    }

    /// Get proof at sequence
    pub fn get_proof(&self, sequence: u64) -> Option<&ChainedProof> {
        self.proofs.get(sequence as usize)
    }

    /// Get Merkle proof for a specific proof
    pub fn get_merkle_proof(&self, sequence: u64) -> Option<MerkleProof> {
        if sequence >= self.proofs.len() as u64 {
            return None;
        }

        let path = self.compute_merkle_path(sequence as usize);

        Some(MerkleProof {
            sequence,
            proof_hash: self.proofs[sequence as usize].proof_hash,
            path,
            root: self.merkle_root,
        })
    }

    // ========================================================================
    // INTERNAL METHODS
    // ========================================================================

    fn compute_proof_hash(
        sequence: u64,
        prev_hash: &[u8; 32],
        proof: &DiamondProof,
        input_hash: &[u8; 32],
        output_hash: &[u8; 32],
        timestamp: u64,
    ) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"CHAIN_PROOF:");
        hasher.update(sequence.to_le_bytes());
        hasher.update(prev_hash);
        hasher.update(proof.public_inputs.rules_hash);
        hasher.update(proof.public_inputs.output_hash);
        hasher.update(input_hash);
        hasher.update(output_hash);
        hasher.update(timestamp.to_le_bytes());
        hasher.finalize().into()
    }

    fn compute_merkle_root(&self) -> [u8; 32] {
        if self.proofs.is_empty() {
            return [0u8; 32];
        }

        let mut level: Vec<[u8; 32]> = self.proofs.iter().map(|p| p.proof_hash).collect();

        while level.len() > 1 {
            let mut next_level = Vec::new();

            for chunk in level.chunks(2) {
                let mut hasher = Sha256::new();
                hasher.update(b"MERKLE:");
                hasher.update(chunk[0]);
                if chunk.len() > 1 {
                    hasher.update(chunk[1]);
                } else {
                    hasher.update(chunk[0]); // Duplicate for odd
                }
                next_level.push(hasher.finalize().into());
            }

            level = next_level;
        }

        level[0]
    }

    fn compute_merkle_path(&self, index: usize) -> Vec<MerklePathNode> {
        let mut path = Vec::new();
        let mut level: Vec<[u8; 32]> = self.proofs.iter().map(|p| p.proof_hash).collect();
        let mut idx = index;

        while level.len() > 1 {
            let sibling_idx = if idx.is_multiple_of(2) {
                idx + 1
            } else {
                idx - 1
            };
            let sibling = if sibling_idx < level.len() {
                level[sibling_idx]
            } else {
                level[idx] // Duplicate for odd
            };

            path.push(MerklePathNode {
                hash: sibling,
                is_left: idx % 2 == 1,
            });

            // Move to next level
            let mut next_level = Vec::new();
            for chunk in level.chunks(2) {
                let mut hasher = Sha256::new();
                hasher.update(b"MERKLE:");
                hasher.update(chunk[0]);
                if chunk.len() > 1 {
                    hasher.update(chunk[1]);
                } else {
                    hasher.update(chunk[0]);
                }
                next_level.push(hasher.finalize().into());
            }

            level = next_level;
            idx /= 2;
        }

        path
    }

    fn generate_session_signature(&self, ended_at: u64) -> Vec<u8> {
        // In production: actual Ed25519 signature
        let mut hasher = Sha256::new();
        hasher.update(b"SESSION_SIG:");
        hasher.update(self.session_id);
        hasher.update(self.merkle_root);
        hasher.update((self.proofs.len() as u64).to_le_bytes());
        hasher.update(self.started_at.to_le_bytes());
        hasher.update(ended_at.to_le_bytes());
        hasher.finalize().to_vec()
    }
}

impl Default for ProofChain {
    fn default() -> Self {
        Self::new()
    }
}

/// Merkle proof for a single proof in the chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    /// Sequence number
    pub sequence: u64,

    /// Hash of the proof
    pub proof_hash: [u8; 32],

    /// Path to root
    pub path: Vec<MerklePathNode>,

    /// Expected root
    pub root: [u8; 32],
}

/// Node in Merkle path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerklePathNode {
    /// Sibling hash
    pub hash: [u8; 32],

    /// Is sibling on the left?
    pub is_left: bool,
}

impl MerkleProof {
    /// Verify this Merkle proof
    pub fn verify(&self) -> bool {
        let mut current = self.proof_hash;

        for node in &self.path {
            let mut hasher = Sha256::new();
            hasher.update(b"MERKLE:");

            if node.is_left {
                hasher.update(node.hash);
                hasher.update(current);
            } else {
                hasher.update(current);
                hasher.update(node.hash);
            }

            current = hasher.finalize().into();
        }

        current == self.root
    }
}

/// Chain errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChainError {
    ChainNotActive,
    SequenceMismatch { expected: u64, got: u64 },
    InvalidFirstLink,
    BrokenLink { at_sequence: u64 },
    HashMismatch { at_sequence: u64 },
    MerkleRootMismatch,
}

impl std::fmt::Display for ChainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ChainNotActive => write!(f, "Chain is not active"),
            Self::SequenceMismatch { expected, got } => {
                write!(f, "Sequence mismatch: expected {}, got {}", expected, got)
            }
            Self::InvalidFirstLink => write!(f, "First link has invalid prev_hash"),
            Self::BrokenLink { at_sequence } => {
                write!(f, "Broken link at sequence {}", at_sequence)
            }
            Self::HashMismatch { at_sequence } => {
                write!(f, "Hash mismatch at sequence {}", at_sequence)
            }
            Self::MerkleRootMismatch => write!(f, "Merkle root mismatch"),
        }
    }
}

impl std::error::Error for ChainError {}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diamond::zk_snark::{Curve, ProofMetadata, ProvingSystem, PublicInputs, SnarkPi};

    fn create_test_proof(seq: u64) -> DiamondProof {
        DiamondProof {
            version: 1,
            pi: SnarkPi {
                a: vec![seq as u8; 64],
                b: vec![seq as u8; 128],
                c: vec![seq as u8; 64],
            },
            public_inputs: PublicInputs {
                rules_hash: [1u8; 32],
                output_hash: [2u8; 32],
                timestamp: 12345 + seq,
                session_id: [3u8; 32],
            },
            metadata: ProofMetadata {
                system: ProvingSystem::Groth16,
                curve: Curve::Bn254,
                generation_time_us: 1000,
                constraint_count: 100,
            },
        }
    }

    #[test]
    fn test_chain_creation() {
        let chain = ProofChain::new();
        assert_eq!(chain.status, ChainStatus::Active);
        assert!(chain.proofs.is_empty());
    }

    #[test]
    fn test_add_proof() {
        let mut chain = ProofChain::new();
        let proof = create_test_proof(0);

        let result = chain.add_proof(proof, "input", "output");
        assert!(result.is_ok());
        assert_eq!(chain.proofs.len(), 1);
    }

    #[test]
    fn test_chain_integrity() {
        let mut chain = ProofChain::new();

        for i in 0..5 {
            let proof = create_test_proof(i);
            chain
                .add_proof(proof, &format!("input{}", i), &format!("output{}", i))
                .unwrap();
        }

        let result = chain.verify_integrity();
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_finalize() {
        let mut chain = ProofChain::new();
        let proof = create_test_proof(0);
        chain.add_proof(proof, "in", "out").unwrap();

        let session = chain.finalize();

        assert_eq!(chain.status, ChainStatus::Finalized);
        assert_eq!(session.proof_count, 1);
    }

    #[test]
    fn test_cannot_add_after_finalize() {
        let mut chain = ProofChain::new();
        chain.finalize();

        let proof = create_test_proof(0);
        let result = chain.add_proof(proof, "in", "out");

        assert!(matches!(result, Err(ChainError::ChainNotActive)));
    }

    #[test]
    fn test_merkle_proof_verification() {
        let mut chain = ProofChain::new();

        for i in 0..4 {
            let proof = create_test_proof(i);
            chain
                .add_proof(proof, &format!("in{}", i), &format!("out{}", i))
                .unwrap();
        }

        let merkle_proof = chain.get_merkle_proof(2).unwrap();
        assert!(merkle_proof.verify());
    }
}
