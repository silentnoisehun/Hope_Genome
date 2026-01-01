//! Python bindings for Merkle Batch Auditing
//!
//! v1.8.0 - Provides Python access to scalable batch auditing.
//!
//! Features:
//! - 1000 decisions → 1 signature
//! - O(log n) inclusion proofs
//! - Tamper-evident audit trail

use pyo3::prelude::*;
use pyo3::types::PyBytes;

use crate::crypto::SoftwareKeyStore;
use crate::merkle_audit::{AuditDecision, BatchAuditor, DecisionType, MerkleTree, SignedBatch};

use super::errors::{to_py_result, PyCryptoError};

// ============================================================================
// PyDecisionType
// ============================================================================

/// Decision type enum
#[pyclass(name = "DecisionType")]
#[derive(Clone)]
pub struct PyDecisionType {
    inner: DecisionType,
}

#[pymethods]
impl PyDecisionType {
    /// Create APPROVE decision type
    #[staticmethod]
    fn approve() -> Self {
        PyDecisionType {
            inner: DecisionType::Approve,
        }
    }

    /// Create DENY decision type
    #[staticmethod]
    fn deny() -> Self {
        PyDecisionType {
            inner: DecisionType::Deny,
        }
    }

    /// Create HARD_RESET decision type
    #[staticmethod]
    fn hard_reset() -> Self {
        PyDecisionType {
            inner: DecisionType::HardReset,
        }
    }

    /// Create VIOLATION decision type
    #[staticmethod]
    fn violation() -> Self {
        PyDecisionType {
            inner: DecisionType::Violation,
        }
    }

    fn __repr__(&self) -> String {
        match self.inner {
            DecisionType::Approve => "DecisionType.APPROVE".to_string(),
            DecisionType::Deny => "DecisionType.DENY".to_string(),
            DecisionType::HardReset => "DecisionType.HARD_RESET".to_string(),
            DecisionType::Violation => "DecisionType.VIOLATION".to_string(),
        }
    }
}

// ============================================================================
// PyAuditDecision
// ============================================================================

/// Single auditable decision
///
/// # Example
/// ```python
/// decision = AuditDecision(
///     id="decision-1",
///     decision_type=DecisionType.approve(),
///     action="User query: Hello"
/// )
/// decision.with_rule("No harm")
/// decision.with_violation_count(0)
/// ```
#[pyclass(name = "AuditDecision")]
#[derive(Clone)]
pub struct PyAuditDecision {
    inner: AuditDecision,
}

#[pymethods]
impl PyAuditDecision {
    #[new]
    fn new(id: String, decision_type: &PyDecisionType, action: String) -> Self {
        PyAuditDecision {
            inner: AuditDecision::new(id, decision_type.inner, action),
        }
    }

    /// Add rule that was applied
    fn with_rule(&mut self, rule: String) {
        self.inner.rule_applied = Some(rule);
    }

    /// Add violation count
    fn with_violation_count(&mut self, count: u32) {
        self.inner.violation_count = count;
    }

    /// Get decision ID
    #[getter]
    fn id(&self) -> &str {
        &self.inner.id
    }

    /// Get action description
    #[getter]
    fn action(&self) -> &str {
        &self.inner.action
    }

    /// Get applied rule (if any)
    #[getter]
    fn rule_applied(&self) -> Option<&str> {
        self.inner.rule_applied.as_deref()
    }

    /// Get violation count
    #[getter]
    fn violation_count(&self) -> u32 {
        self.inner.violation_count
    }

    /// Get timestamp (nanoseconds)
    #[getter]
    fn timestamp_ns(&self) -> u128 {
        self.inner.timestamp_ns
    }

    /// Compute SHA-256 hash of this decision
    fn hash<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        let hash = self.inner.hash();
        PyBytes::new(py, &hash)
    }

    fn __repr__(&self) -> String {
        format!(
            "AuditDecision(id='{}', action='{}', rule={:?})",
            self.inner.id, self.inner.action, self.inner.rule_applied
        )
    }
}

// ============================================================================
// PyMerkleTree
// ============================================================================

/// Merkle tree for batch auditing
///
/// Build a tree of decision hashes, then finalize to get the Merkle root.
///
/// # Example
/// ```python
/// tree = MerkleTree()
///
/// d1 = AuditDecision("1", DecisionType.approve(), "action1")
/// d2 = AuditDecision("2", DecisionType.deny(), "action2")
///
/// tree.add_decision(d1)
/// tree.add_decision(d2)
///
/// root = tree.finalize()  # bytes
/// proof = tree.get_proof(0)  # Proof for decision 1
/// ```
#[pyclass(name = "MerkleTree")]
pub struct PyMerkleTree {
    inner: MerkleTree,
}

#[pymethods]
impl PyMerkleTree {
    #[new]
    fn new() -> Self {
        PyMerkleTree {
            inner: MerkleTree::new(),
        }
    }

    /// Add a decision to the tree
    fn add_decision(&mut self, decision: &PyAuditDecision) -> PyResult<usize> {
        to_py_result(self.inner.add_decision(&decision.inner))
    }

    /// Finalize tree and get Merkle root
    fn finalize<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        let root = to_py_result(self.inner.finalize())?;
        Ok(PyBytes::new(py, &root))
    }

    /// Get Merkle root (tree must be finalized)
    fn root<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        let root = to_py_result(self.inner.root())?;
        Ok(PyBytes::new(py, &root))
    }

    /// Get inclusion proof for decision at index
    ///
    /// Returns list of (hash, is_left) tuples
    fn get_proof<'py>(
        &self,
        py: Python<'py>,
        index: usize,
    ) -> PyResult<Vec<(Bound<'py, PyBytes>, bool)>> {
        let proof = to_py_result(self.inner.get_proof(index))?;
        let result: Vec<(Bound<'py, PyBytes>, bool)> = proof
            .into_iter()
            .map(|(hash, is_left)| (PyBytes::new(py, &hash), is_left))
            .collect();
        Ok(result)
    }

    /// Verify an inclusion proof
    ///
    /// Returns True if the proof is valid
    #[staticmethod]
    fn verify_proof(
        leaf_hash: &[u8],
        proof: Vec<(Vec<u8>, bool)>,
        expected_root: &[u8],
    ) -> PyResult<bool> {
        if leaf_hash.len() != 32 || expected_root.len() != 32 {
            return Err(PyCryptoError::new_err("Hash must be 32 bytes"));
        }

        let mut leaf: [u8; 32] = [0; 32];
        leaf.copy_from_slice(leaf_hash);

        let mut root: [u8; 32] = [0; 32];
        root.copy_from_slice(expected_root);

        let proof_array: Vec<([u8; 32], bool)> = proof
            .into_iter()
            .map(|(hash, is_left)| {
                let mut arr: [u8; 32] = [0; 32];
                arr.copy_from_slice(&hash);
                (arr, is_left)
            })
            .collect();

        Ok(MerkleTree::verify_proof(&leaf, &proof_array, &root))
    }

    /// Number of decisions in tree
    fn __len__(&self) -> usize {
        self.inner.len()
    }

    /// Is tree empty?
    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn __repr__(&self) -> String {
        format!("MerkleTree(len={})", self.inner.len())
    }
}

// ============================================================================
// PySignedBatch
// ============================================================================

/// Signed Merkle batch
///
/// Contains the Merkle root and Ed25519 signature for a batch of decisions.
#[pyclass(name = "SignedBatch")]
#[derive(Clone)]
pub struct PySignedBatch {
    inner: SignedBatch,
}

#[pymethods]
impl PySignedBatch {
    /// Batch ID
    #[getter]
    fn batch_id(&self) -> &str {
        &self.inner.batch_id
    }

    /// Batch start timestamp (Unix seconds)
    #[getter]
    fn start_time(&self) -> u64 {
        self.inner.start_time
    }

    /// Batch end timestamp (Unix seconds)
    #[getter]
    fn end_time(&self) -> u64 {
        self.inner.end_time
    }

    /// Number of decisions in batch
    #[getter]
    fn decision_count(&self) -> usize {
        self.inner.decision_count
    }

    /// Merkle root (32 bytes)
    fn merkle_root<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.merkle_root)
    }

    /// Ed25519 signature (64 bytes)
    fn signature<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.signature)
    }

    /// Signer public key (32 bytes)
    fn signer_pubkey<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.signer_pubkey)
    }

    fn __repr__(&self) -> String {
        format!(
            "SignedBatch(id='{}', decisions={}, time={}-{})",
            self.inner.batch_id,
            self.inner.decision_count,
            self.inner.start_time,
            self.inner.end_time
        )
    }
}

// ============================================================================
// PyBatchAuditor
// ============================================================================

/// Batch auditor for high-throughput auditing
///
/// Collects decisions and automatically commits when batch size or time limit
/// is reached.
///
/// # Example
/// ```python
/// auditor = BatchAuditor(
///     batch_size_limit=1000,  # Commit every 1000 decisions
///     batch_time_limit=60     # Or every 60 seconds
/// )
///
/// # Record decisions
/// for i in range(1500):
///     decision = AuditDecision(f"d-{i}", DecisionType.approve(), f"action-{i}")
///     batch = auditor.record(decision)
///     if batch:
///         print(f"Batch committed: {batch.batch_id}")
///
/// # Force commit remaining
/// final_batch = auditor.flush()
/// ```
#[pyclass(name = "BatchAuditor")]
pub struct PyBatchAuditor {
    inner: BatchAuditor<SoftwareKeyStore>,
}

#[pymethods]
impl PyBatchAuditor {
    /// Create new batch auditor
    ///
    /// Args:
    ///     batch_size_limit: Max decisions per batch (e.g., 1000)
    ///     batch_time_limit: Max seconds before auto-commit (e.g., 60)
    #[new]
    fn new(batch_size_limit: usize, batch_time_limit: u64) -> PyResult<Self> {
        let keystore = to_py_result(SoftwareKeyStore::generate())?;
        Ok(PyBatchAuditor {
            inner: BatchAuditor::new(keystore, batch_size_limit, batch_time_limit),
        })
    }

    /// Record a decision
    ///
    /// Returns SignedBatch if batch was auto-committed, None otherwise.
    fn record(&mut self, decision: &PyAuditDecision) -> PyResult<Option<PySignedBatch>> {
        let result = to_py_result(self.inner.record(decision.inner.clone()))?;
        Ok(result.map(|batch| PySignedBatch { inner: batch }))
    }

    /// Commit current batch and start new one
    fn commit_batch(&mut self) -> PyResult<PySignedBatch> {
        let batch = to_py_result(self.inner.commit_batch())?;
        Ok(PySignedBatch { inner: batch })
    }

    /// Force commit if there are pending decisions
    fn flush(&mut self) -> PyResult<Option<PySignedBatch>> {
        let result = to_py_result(self.inner.flush())?;
        Ok(result.map(|batch| PySignedBatch { inner: batch }))
    }

    /// Get all completed batches
    fn get_completed_batches(&self) -> Vec<PySignedBatch> {
        self.inner
            .get_completed_batches()
            .iter()
            .cloned()
            .map(|batch| PySignedBatch { inner: batch })
            .collect()
    }

    /// Verify a signed batch
    fn verify_batch(&self, batch: &PySignedBatch) -> PyResult<bool> {
        to_py_result(self.inner.verify_batch(&batch.inner))
    }

    /// Get current pending decision count
    fn pending_count(&self) -> usize {
        self.inner.pending_count()
    }

    fn __repr__(&self) -> String {
        format!(
            "BatchAuditor(pending={}, batches={})",
            self.inner.pending_count(),
            self.inner.get_completed_batches().len()
        )
    }
}
