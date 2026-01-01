//! # Hope Genome v1.8.0 - Merkle Tree Batch Auditing
//!
//! **SCALABILITY LAYER**: Batch thousands of decisions into single signature
//!
//! ## Problem
//!
//! High-frequency AI agents may generate 1000+ decisions/second.
//! Individual Ed25519 signatures for each = performance bottleneck.
//!
//! ## Solution: Merkle Tree Batching
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │              MERKLE TREE BATCH AUDITING                      │
//! │                                                              │
//! │  Decision 1    Decision 2    Decision 3    Decision 4       │
//! │      │             │             │             │            │
//! │      ▼             ▼             ▼             ▼            │
//! │   [Hash 1]     [Hash 2]     [Hash 3]     [Hash 4]          │
//! │      └────┬────────┘            └────┬────────┘            │
//! │           ▼                          ▼                      │
//! │       [Hash 1-2]                 [Hash 3-4]                │
//! │           └──────────┬───────────────┘                      │
//! │                      ▼                                      │
//! │               [MERKLE ROOT]  ← Single Ed25519 signature!   │
//! │                                                              │
//! │  Result: 1000 decisions = 1 signature = 100% audit trail   │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Security Guarantees
//!
//! - ✅ **Tamper-evident**: Any change invalidates Merkle root
//! - ✅ **Proof of inclusion**: O(log n) proof for any decision
//! - ✅ **Time-ordered**: Batch timestamp = commitment point
//! - ✅ **Non-repudiation**: Ed25519 signed root
//!
//! ---
//!
//! **Date**: 2026-01-01
//! **Version**: 1.8.0 (Multi-Model Edition - Merkle Auditing)
//! **Author**: Máté Róbert <stratosoiteam@gmail.com>

use crate::crypto::{KeyStore, Result, CryptoError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// MERKLE NODE
// ============================================================================

/// Merkle tree node hash (32 bytes SHA-256)
pub type MerkleHash = [u8; 32];

/// Compute SHA-256 hash
fn sha256(data: &[u8]) -> MerkleHash {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Combine two hashes into parent hash
fn hash_pair(left: &MerkleHash, right: &MerkleHash) -> MerkleHash {
    let mut hasher = Sha256::new();
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}

// ============================================================================
// AUDIT DECISION
// ============================================================================

/// Single auditable decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditDecision {
    /// Decision ID (UUID or sequence number)
    pub id: String,

    /// Timestamp (Unix epoch nanoseconds for ordering)
    pub timestamp_ns: u128,

    /// Action type (APPROVE, DENY, HARD_RESET)
    pub action_type: DecisionType,

    /// Action description
    pub action: String,

    /// Rule that was applied (if denial)
    pub rule_applied: Option<String>,

    /// Violation count at decision time
    pub violation_count: u32,

    /// Additional metadata (JSON)
    pub metadata: Option<String>,
}

/// Decision type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionType {
    /// Action approved
    Approve,
    /// Action denied
    Deny,
    /// Hard reset triggered
    HardReset,
    /// Violation recorded
    Violation,
}

impl AuditDecision {
    /// Create new decision
    pub fn new(
        id: impl Into<String>,
        action_type: DecisionType,
        action: impl Into<String>,
    ) -> Self {
        let timestamp_ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        AuditDecision {
            id: id.into(),
            timestamp_ns,
            action_type,
            action: action.into(),
            rule_applied: None,
            violation_count: 0,
            metadata: None,
        }
    }

    /// Add rule that was applied
    pub fn with_rule(mut self, rule: impl Into<String>) -> Self {
        self.rule_applied = Some(rule.into());
        self
    }

    /// Add violation count
    pub fn with_violation_count(mut self, count: u32) -> Self {
        self.violation_count = count;
        self
    }

    /// Compute hash of this decision
    pub fn hash(&self) -> MerkleHash {
        let serialized = serde_json::to_vec(self).unwrap_or_default();
        sha256(&serialized)
    }
}

// ============================================================================
// MERKLE TREE
// ============================================================================

/// Merkle tree for batch auditing
#[derive(Debug, Clone)]
pub struct MerkleTree {
    /// Leaf hashes (decision hashes)
    leaves: Vec<MerkleHash>,

    /// All tree levels (bottom to top)
    /// Level 0 = leaves, Level n = root
    levels: Vec<Vec<MerkleHash>>,

    /// Tree is finalized (root computed)
    finalized: bool,
}

impl MerkleTree {
    /// Create new empty Merkle tree
    pub fn new() -> Self {
        MerkleTree {
            leaves: Vec::new(),
            levels: Vec::new(),
            finalized: false,
        }
    }

    /// Add decision to tree
    pub fn add_decision(&mut self, decision: &AuditDecision) -> Result<usize> {
        if self.finalized {
            return Err(CryptoError::InvalidState(
                "Tree is finalized, cannot add more decisions".into()
            ));
        }

        let hash = decision.hash();
        self.leaves.push(hash);
        Ok(self.leaves.len() - 1)
    }

    /// Add multiple decisions
    pub fn add_decisions(&mut self, decisions: &[AuditDecision]) -> Result<()> {
        for decision in decisions {
            self.add_decision(decision)?;
        }
        Ok(())
    }

    /// Build and finalize tree, returns Merkle root
    pub fn finalize(&mut self) -> Result<MerkleHash> {
        if self.leaves.is_empty() {
            return Err(CryptoError::InvalidState("No decisions to hash".into()));
        }

        // Start with leaves
        let mut current_level = self.leaves.clone();
        self.levels.push(current_level.clone());

        // Build tree bottom-up
        while current_level.len() > 1 {
            let mut next_level = Vec::new();

            for chunk in current_level.chunks(2) {
                if chunk.len() == 2 {
                    next_level.push(hash_pair(&chunk[0], &chunk[1]));
                } else {
                    // Odd number: duplicate last hash
                    next_level.push(hash_pair(&chunk[0], &chunk[0]));
                }
            }

            self.levels.push(next_level.clone());
            current_level = next_level;
        }

        self.finalized = true;
        Ok(current_level[0])
    }

    /// Get Merkle root (tree must be finalized)
    pub fn root(&self) -> Result<MerkleHash> {
        if !self.finalized {
            return Err(CryptoError::InvalidState("Tree not finalized".into()));
        }

        self.levels.last()
            .and_then(|level| level.first().copied())
            .ok_or_else(|| CryptoError::InvalidState("Empty tree".into()))
    }

    /// Get inclusion proof for decision at index
    ///
    /// Returns list of (hash, is_left) pairs from leaf to root
    pub fn get_proof(&self, index: usize) -> Result<Vec<(MerkleHash, bool)>> {
        if !self.finalized {
            return Err(CryptoError::InvalidState("Tree not finalized".into()));
        }

        if index >= self.leaves.len() {
            return Err(CryptoError::InvalidState("Index out of bounds".into()));
        }

        let mut proof = Vec::new();
        let mut current_index = index;

        for level in &self.levels[..self.levels.len() - 1] {
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };

            let sibling_hash = if sibling_index < level.len() {
                level[sibling_index]
            } else {
                // Odd number of nodes: sibling is self
                level[current_index]
            };

            let is_left = current_index % 2 == 1;
            proof.push((sibling_hash, is_left));

            current_index /= 2;
        }

        Ok(proof)
    }

    /// Verify inclusion proof
    pub fn verify_proof(
        leaf_hash: &MerkleHash,
        proof: &[(MerkleHash, bool)],
        expected_root: &MerkleHash,
    ) -> bool {
        let mut current_hash = *leaf_hash;

        for (sibling_hash, is_left) in proof {
            current_hash = if *is_left {
                hash_pair(sibling_hash, &current_hash)
            } else {
                hash_pair(&current_hash, sibling_hash)
            };
        }

        &current_hash == expected_root
    }

    /// Number of decisions in tree
    pub fn len(&self) -> usize {
        self.leaves.len()
    }

    /// Is tree empty?
    pub fn is_empty(&self) -> bool {
        self.leaves.is_empty()
    }
}

impl Default for MerkleTree {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// BATCH AUDITOR
// ============================================================================

/// Signed Merkle batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedBatch {
    /// Batch ID
    pub batch_id: String,

    /// Batch start timestamp
    pub start_time: u64,

    /// Batch end timestamp
    pub end_time: u64,

    /// Number of decisions in batch
    pub decision_count: usize,

    /// Merkle root
    pub merkle_root: MerkleHash,

    /// Ed25519 signature over merkle root
    pub signature: Vec<u8>,

    /// Signer public key
    pub signer_pubkey: Vec<u8>,
}

/// Batch auditor for high-throughput auditing
pub struct BatchAuditor<K: KeyStore> {
    /// Signing key store
    keystore: K,

    /// Current batch tree
    current_tree: MerkleTree,

    /// Current batch decisions (for persistence)
    current_decisions: Vec<AuditDecision>,

    /// Batch start time
    batch_start: Option<u64>,

    /// Batch size limit
    batch_size_limit: usize,

    /// Batch time limit (seconds)
    batch_time_limit: u64,

    /// Completed batches
    completed_batches: Vec<SignedBatch>,
}

impl<K: KeyStore> BatchAuditor<K> {
    /// Create new batch auditor
    ///
    /// # Arguments
    ///
    /// * `keystore` - Key store for signing (SoftwareKeyStore, HsmKeyStore, TeeKeyStore)
    /// * `batch_size_limit` - Max decisions per batch (e.g., 1000)
    /// * `batch_time_limit` - Max seconds before auto-commit (e.g., 60)
    pub fn new(keystore: K, batch_size_limit: usize, batch_time_limit: u64) -> Self {
        BatchAuditor {
            keystore,
            current_tree: MerkleTree::new(),
            current_decisions: Vec::new(),
            batch_start: None,
            batch_size_limit,
            batch_time_limit,
            completed_batches: Vec::new(),
        }
    }

    /// Record a decision
    ///
    /// Returns Some(SignedBatch) if batch was auto-committed
    pub fn record(&mut self, decision: AuditDecision) -> Result<Option<SignedBatch>> {
        // Initialize batch start time
        if self.batch_start.is_none() {
            self.batch_start = Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            );
        }

        // Add to tree
        self.current_tree.add_decision(&decision)?;
        self.current_decisions.push(decision);

        // Check if batch should be committed
        let should_commit = self.current_tree.len() >= self.batch_size_limit
            || self.is_time_expired();

        if should_commit {
            return Ok(Some(self.commit_batch()?));
        }

        Ok(None)
    }

    /// Check if batch time limit expired
    fn is_time_expired(&self) -> bool {
        if let Some(start) = self.batch_start {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            now - start >= self.batch_time_limit
        } else {
            false
        }
    }

    /// Commit current batch and start new one
    pub fn commit_batch(&mut self) -> Result<SignedBatch> {
        if self.current_tree.is_empty() {
            return Err(CryptoError::InvalidState("No decisions to commit".into()));
        }

        // Finalize tree
        let merkle_root = self.current_tree.finalize()?;

        // Sign merkle root
        let signature = self.keystore.sign(&merkle_root)?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let batch = SignedBatch {
            batch_id: format!("batch-{}", now),
            start_time: self.batch_start.unwrap_or(now),
            end_time: now,
            decision_count: self.current_decisions.len(),
            merkle_root,
            signature,
            signer_pubkey: self.keystore.public_key_bytes(),
        };

        // Store completed batch
        self.completed_batches.push(batch.clone());

        // Reset for next batch
        self.current_tree = MerkleTree::new();
        self.current_decisions.clear();
        self.batch_start = None;

        Ok(batch)
    }

    /// Force commit if there are pending decisions
    pub fn flush(&mut self) -> Result<Option<SignedBatch>> {
        if self.current_tree.is_empty() {
            return Ok(None);
        }
        Ok(Some(self.commit_batch()?))
    }

    /// Get all completed batches
    pub fn get_completed_batches(&self) -> &[SignedBatch] {
        &self.completed_batches
    }

    /// Verify a signed batch
    pub fn verify_batch(&self, batch: &SignedBatch) -> Result<bool> {
        self.keystore.verify(&batch.merkle_root, &batch.signature)?;
        Ok(true)
    }

    /// Get current pending decision count
    pub fn pending_count(&self) -> usize {
        self.current_tree.len()
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
    fn test_merkle_tree_basic() {
        let mut tree = MerkleTree::new();

        let d1 = AuditDecision::new("1", DecisionType::Approve, "action1");
        let d2 = AuditDecision::new("2", DecisionType::Deny, "action2");
        let d3 = AuditDecision::new("3", DecisionType::Approve, "action3");
        let d4 = AuditDecision::new("4", DecisionType::Approve, "action4");

        tree.add_decision(&d1).unwrap();
        tree.add_decision(&d2).unwrap();
        tree.add_decision(&d3).unwrap();
        tree.add_decision(&d4).unwrap();

        let root = tree.finalize().unwrap();
        assert_eq!(tree.len(), 4);

        // Verify proof for each decision
        for i in 0..4 {
            let proof = tree.get_proof(i).unwrap();
            let decisions = [&d1, &d2, &d3, &d4];
            assert!(MerkleTree::verify_proof(
                &decisions[i].hash(),
                &proof,
                &root
            ));
        }
    }

    #[test]
    fn test_batch_auditor() {
        let keystore = SoftwareKeyStore::generate().unwrap();
        let mut auditor = BatchAuditor::new(keystore, 3, 60);

        // Add 2 decisions (no commit)
        let d1 = AuditDecision::new("1", DecisionType::Approve, "action1");
        let d2 = AuditDecision::new("2", DecisionType::Deny, "action2");

        assert!(auditor.record(d1).unwrap().is_none());
        assert!(auditor.record(d2).unwrap().is_none());
        assert_eq!(auditor.pending_count(), 2);

        // Add 3rd decision (triggers commit)
        let d3 = AuditDecision::new("3", DecisionType::Approve, "action3");
        let batch = auditor.record(d3).unwrap();

        assert!(batch.is_some());
        let batch = batch.unwrap();
        assert_eq!(batch.decision_count, 3);
        assert_eq!(auditor.pending_count(), 0);

        // Verify batch signature
        assert!(auditor.verify_batch(&batch).unwrap());
    }

    #[test]
    fn test_merkle_proof_invalid() {
        let mut tree = MerkleTree::new();

        let d1 = AuditDecision::new("1", DecisionType::Approve, "action1");
        let d2 = AuditDecision::new("2", DecisionType::Deny, "action2");

        tree.add_decision(&d1).unwrap();
        tree.add_decision(&d2).unwrap();
        let root = tree.finalize().unwrap();

        // Tamper with decision
        let fake_decision = AuditDecision::new("1", DecisionType::Approve, "TAMPERED");
        let proof = tree.get_proof(0).unwrap();

        // Proof should fail for tampered decision
        assert!(!MerkleTree::verify_proof(
            &fake_decision.hash(),
            &proof,
            &root
        ));
    }
}
