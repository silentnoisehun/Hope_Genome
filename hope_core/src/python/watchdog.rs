//! Python bindings for Watchdog ("Vas Szigora" - Iron Discipline)
//!
//! v1.7.0 - Provides Python access to the enforcement engine.

use pyo3::prelude::*;
use pyo3::types::PyBytes;

use crate::crypto::SoftwareKeyStore;
use crate::watchdog::{DenialProof, HardResetSignal, ViolationCounter, Watchdog, WatchdogError};

use super::errors::PyWatchdogError;
use super::PyAction;

// ============================================================================
// PyViolationCounter
// ============================================================================

/// Thread-safe violation counter (zero-allocation)
///
/// # Example
/// ```python
/// counter = ViolationCounter()
/// assert counter.count() == 0
///
/// counter.increment()
/// assert counter.count() == 1
/// ```
#[pyclass(name = "ViolationCounter")]
pub struct PyViolationCounter {
    inner: ViolationCounter,
}

#[pymethods]
impl PyViolationCounter {
    #[new]
    fn new() -> Self {
        PyViolationCounter {
            inner: ViolationCounter::new(),
        }
    }

    /// Get current violation count
    fn count(&self) -> u32 {
        self.inner.count()
    }

    /// Increment violation count
    fn increment(&self) -> u32 {
        self.inner.increment()
    }

    /// Reset violation count to zero
    fn reset(&self) {
        self.inner.reset()
    }

    /// Check if maximum violations (10) reached
    fn is_max_reached(&self) -> bool {
        self.inner.is_max_reached()
    }

    /// Check if watchdog is locked
    fn is_locked(&self) -> bool {
        self.inner.is_locked()
    }

    fn __repr__(&self) -> String {
        format!(
            "ViolationCounter(count={}, locked={})",
            self.inner.count(),
            self.inner.is_locked()
        )
    }
}

// ============================================================================
// PyDenialProof
// ============================================================================

/// Cryptographic proof of rule violation
///
/// Contains Ed25519 signed evidence of why an action was denied.
///
/// # Example
/// ```python
/// # DenialProof is returned by Watchdog.verify_action()
/// result = watchdog.verify_action(action)
/// if result.denial_proof:
///     print(f"Denied: {result.denial_proof.violated_rule}")
///     print(f"Reason: {result.denial_proof.denial_reason}")
/// ```
#[pyclass(name = "DenialProof")]
pub struct PyDenialProof {
    inner: DenialProof,
}

#[pymethods]
impl PyDenialProof {
    /// The rule that was violated
    #[getter]
    fn violated_rule(&self) -> &str {
        &self.inner.violated_rule
    }

    /// Human-readable denial reason
    #[getter]
    fn denial_reason(&self) -> &str {
        &self.inner.denial_reason
    }

    /// Current violation count (1-10)
    #[getter]
    fn violation_count(&self) -> u32 {
        self.inner.violation_count
    }

    /// Whether this denial triggered a hard reset
    #[getter]
    fn triggered_hard_reset(&self) -> bool {
        self.inner.triggered_hard_reset
    }

    /// Unix timestamp of denial
    #[getter]
    fn timestamp(&self) -> u64 {
        self.inner.timestamp
    }

    /// Whether proof is cryptographically signed
    fn is_signed(&self) -> bool {
        self.inner.is_signed()
    }

    /// Get signature as hex string
    fn signature_hex(&self) -> String {
        self.inner.signature_hex()
    }

    /// Get nonce as bytes
    fn nonce<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.nonce)
    }

    /// Get action hash as bytes
    fn action_hash<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        PyBytes::new(py, &self.inner.action_hash)
    }

    fn __repr__(&self) -> String {
        format!(
            "DenialProof(rule='{}', count={}, hard_reset={})",
            self.inner.violated_rule, self.inner.violation_count, self.inner.triggered_hard_reset
        )
    }
}

impl From<DenialProof> for PyDenialProof {
    fn from(proof: DenialProof) -> Self {
        PyDenialProof { inner: proof }
    }
}

// ============================================================================
// PyHardResetSignal
// ============================================================================

/// Signal that hard reset is required
///
/// Returned after 10 consecutive violations.
/// The AI runtime MUST clear all context and restart.
#[pyclass(name = "HardResetSignal")]
pub struct PyHardResetSignal {
    inner: HardResetSignal,
}

#[pymethods]
impl PyHardResetSignal {
    /// Total violations that triggered reset
    #[getter]
    fn total_violations(&self) -> u32 {
        self.inner.total_violations
    }

    /// Genome hash (for verification after restart)
    #[getter]
    fn genome_hash(&self) -> &str {
        &self.inner.genome_hash
    }

    /// Timestamp of reset signal
    #[getter]
    fn reset_timestamp(&self) -> u64 {
        self.inner.reset_timestamp
    }

    /// Get the final denial proof that triggered reset
    #[getter]
    fn final_denial(&self) -> PyDenialProof {
        PyDenialProof {
            inner: self.inner.final_denial.clone(),
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "HardResetSignal(violations={}, genome_hash='{}')",
            self.inner.total_violations, self.inner.genome_hash
        )
    }
}

impl From<HardResetSignal> for PyHardResetSignal {
    fn from(signal: HardResetSignal) -> Self {
        PyHardResetSignal { inner: signal }
    }
}

// ============================================================================
// PyWatchdogResult
// ============================================================================

/// Result of action verification
///
/// Contains either:
/// - approved=True (action allowed)
/// - approved=False + denial_proof (action denied, but not 10th violation)
/// - hard_reset_required=True + hard_reset_signal (10th violation, must reset)
#[pyclass(name = "WatchdogResult")]
pub struct PyWatchdogResult {
    approved: bool,
    denial_proof: Option<PyDenialProof>,
    hard_reset_required: bool,
    hard_reset_signal: Option<PyHardResetSignal>,
}

#[pymethods]
impl PyWatchdogResult {
    /// Whether action was approved
    #[getter]
    fn approved(&self) -> bool {
        self.approved
    }

    /// Denial proof (if action was denied)
    #[getter]
    fn denial_proof(&self) -> Option<PyDenialProof> {
        self.denial_proof.clone()
    }

    /// Whether hard reset is required
    #[getter]
    fn hard_reset_required(&self) -> bool {
        self.hard_reset_required
    }

    /// Hard reset signal (if reset required)
    #[getter]
    fn hard_reset_signal(&self) -> Option<PyHardResetSignal> {
        self.hard_reset_signal.clone()
    }

    fn __repr__(&self) -> String {
        if self.hard_reset_required {
            "WatchdogResult(HARD_RESET_REQUIRED)".to_string()
        } else if self.approved {
            "WatchdogResult(APPROVED)".to_string()
        } else {
            format!(
                "WatchdogResult(DENIED, count={})",
                self.denial_proof
                    .as_ref()
                    .map_or(0, |p| p.inner.violation_count)
            )
        }
    }
}

impl Clone for PyDenialProof {
    fn clone(&self) -> Self {
        PyDenialProof {
            inner: self.inner.clone(),
        }
    }
}

impl Clone for PyHardResetSignal {
    fn clone(&self) -> Self {
        PyHardResetSignal {
            inner: self.inner.clone(),
        }
    }
}

// ============================================================================
// PyWatchdog
// ============================================================================

/// The Watchdog - Iron Discipline Enforcement Engine
///
/// Monitors actions and enforces SealedGenome rules.
/// After 10 consecutive violations, triggers hard reset.
///
/// # Example
/// ```python
/// import hope_genome as hg
///
/// # Create watchdog
/// watchdog = hg.Watchdog(
///     rules=["Do no harm", "Respect privacy"],
///     capsule_hash="abc123..."
/// )
///
/// # Verify action
/// action = hg.Action.delete_file("/etc/passwd")
/// result = watchdog.verify_action(action)
///
/// if result.approved:
///     print("Action allowed")
/// elif result.hard_reset_required:
///     print("HARD RESET REQUIRED!")
///     # Clear all context, restart AI
/// else:
///     print(f"Denied: {result.denial_proof.denied_reason}")
/// ```
#[pyclass(name = "Watchdog")]
pub struct PyWatchdog {
    inner: Watchdog,
}

#[pymethods]
impl PyWatchdog {
    #[new]
    #[pyo3(signature = (rules, capsule_hash))]
    fn new(rules: Vec<String>, capsule_hash: String) -> PyResult<Self> {
        let key_store = SoftwareKeyStore::generate()
            .map_err(|e| PyWatchdogError::new_err(format!("KeyStore error: {}", e)))?;

        Ok(PyWatchdog {
            inner: Watchdog::new(rules, capsule_hash, Box::new(key_store)),
        })
    }

    /// Get current violation count
    fn violation_count(&self) -> u32 {
        self.inner.violation_count()
    }

    /// Check if watchdog is locked (hard reset required)
    fn is_locked(&self) -> bool {
        self.inner.is_locked()
    }

    /// Report successful action (resets counter)
    fn report_success(&self) {
        self.inner.report_success()
    }

    /// Verify an action against rules
    ///
    /// Returns WatchdogResult with:
    /// - approved=True if action is allowed
    /// - denial_proof if action is denied
    /// - hard_reset_required=True if 10th violation
    fn verify_action(&self, action: &PyAction) -> PyResult<PyWatchdogResult> {
        let rust_action = action.to_rust_action();

        match self.inner.verify_action(&rust_action) {
            Ok(None) => {
                // Action approved
                Ok(PyWatchdogResult {
                    approved: true,
                    denial_proof: None,
                    hard_reset_required: false,
                    hard_reset_signal: None,
                })
            }
            Ok(Some(denial)) => {
                // Action denied, but not 10th violation
                Ok(PyWatchdogResult {
                    approved: false,
                    denial_proof: Some(denial.into()),
                    hard_reset_required: false,
                    hard_reset_signal: None,
                })
            }
            Err(WatchdogError::HardResetRequired(count, _max)) => {
                // 10th violation - hard reset required
                // Generate the hard reset signal
                let action_copy = rust_action.clone();
                let denial = DenialProof::new(
                    &action_copy,
                    "Multiple violations".to_string(),
                    format!("{} consecutive violations reached", count),
                    count,
                );

                let signal = self
                    .inner
                    .generate_hard_reset_signal(denial.clone())
                    .map_err(|e| PyWatchdogError::new_err(e.to_string()))?;

                Ok(PyWatchdogResult {
                    approved: false,
                    denial_proof: Some(denial.into()),
                    hard_reset_required: true,
                    hard_reset_signal: Some(signal.into()),
                })
            }
            Err(WatchdogError::WatchdogLocked) => Err(PyWatchdogError::new_err(
                "Watchdog is locked - hard reset required",
            )),
            Err(e) => Err(PyWatchdogError::new_err(e.to_string())),
        }
    }

    /// Acknowledge hard reset (unlock watchdog)
    ///
    /// Call after:
    /// 1. Clearing all context
    /// 2. Verifying genome is unchanged
    /// 3. Ready to restart
    fn acknowledge_reset(&self) {
        self.inner.acknowledge_reset()
    }

    /// Get rules (read-only)
    fn rules(&self) -> Vec<String> {
        self.inner.rules().to_vec()
    }

    /// Get capsule hash
    fn capsule_hash(&self) -> &str {
        self.inner.capsule_hash()
    }

    fn __repr__(&self) -> String {
        format!(
            "Watchdog(rules={}, violations={}, locked={})",
            self.inner.rules().len(),
            self.inner.violation_count(),
            self.inner.is_locked()
        )
    }
}

/// Maximum violations before hard reset (10)
#[pyfunction]
pub fn max_violations() -> u32 {
    crate::watchdog::MAX_VIOLATIONS
}
