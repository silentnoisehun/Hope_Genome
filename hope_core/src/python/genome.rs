//! SealedGenome Python wrapper

use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::sync::{Arc, Mutex};

use super::action::PyAction;
use super::errors::to_py_result;
use super::proof::PyProof;
use crate::genome::SealedGenome;

/// Python wrapper for SealedGenome
///
/// Represents a cryptographically sealed set of ethical rules that governs
/// AI decision-making with tamper-evident accountability.
///
/// # Example
/// ```python
/// genome = SealedGenome(rules=["Do no harm", "Respect privacy"])
/// genome.seal()
///
/// action = Action.delete_file("data.txt")
/// proof = genome.verify_action(action)
/// ```
#[pyclass(name = "SealedGenome")]
pub struct PySealedGenome {
    inner: Arc<Mutex<SealedGenome>>,
}

#[pymethods]
impl PySealedGenome {
    /// Create a new SealedGenome with the given rules
    ///
    /// Args:
    ///     rules (list[str]): Ethical rules governing AI behavior
    ///
    /// Returns:
    ///     SealedGenome: Unsealed genome instance
    ///
    /// Raises:
    ///     GenomeError: If rules are empty
    ///
    /// Example:
    ///     >>> genome = SealedGenome(rules=["Do no harm"])
    #[new]
    fn new(rules: Vec<String>) -> PyResult<Self> {
        let genome = to_py_result(SealedGenome::new(rules))?;

        Ok(PySealedGenome {
            inner: Arc::new(Mutex::new(genome)),
        })
    }

    /// Seal the genome, making it immutable
    ///
    /// After sealing, the genome's rules cannot be modified.
    /// This is a cryptographically enforced guarantee.
    ///
    /// Raises:
    ///     GenomeError: If already sealed
    ///
    /// Example:
    ///     >>> genome.seal()
    fn seal(&mut self) -> PyResult<()> {
        let mut genome = self.inner.lock().unwrap();
        to_py_result(genome.seal())
    }

    /// Check if the genome is sealed
    ///
    /// Returns:
    ///     bool: True if sealed, False otherwise
    ///
    /// Example:
    ///     >>> genome.is_sealed()
    ///     False
    ///     >>> genome.seal()
    ///     >>> genome.is_sealed()
    ///     True
    fn is_sealed(&self) -> bool {
        self.inner.lock().unwrap().is_sealed()
    }

    /// Get the genome's rules
    ///
    /// Returns:
    ///     list[str]: The immutable rule set
    ///
    /// Example:
    ///     >>> genome.rules()
    ///     ['Do no harm', 'Respect privacy']
    fn rules(&self) -> Vec<String> {
        self.inner.lock().unwrap().rules().to_vec()
    }

    /// Get the genome's cryptographic hash
    ///
    /// Returns:
    ///     str: Hex-encoded SHA-256 hash of the sealed genome
    ///
    /// Example:
    ///     >>> genome.genome_hash()
    ///     'a3f2e1...'
    fn genome_hash(&self) -> Option<String> {
        let genome = self.inner.lock().unwrap();
        genome.capsule_hash().map(|s| s.to_string())
    }

    /// Verify an action against the genome rules
    ///
    /// Args:
    ///     action (Action): The action to verify
    ///
    /// Returns:
    ///     Proof: Cryptographic proof of verification result
    ///
    /// Raises:
    ///     GenomeError: If genome is not sealed
    ///     CryptoError: If signature generation fails
    ///
    /// Example:
    ///     >>> action = Action.delete_file("data.txt")
    ///     >>> proof = genome.verify_action(action)
    ///     >>> print(proof.approved)
    ///     True
    fn verify_action(&mut self, action: &PyAction) -> PyResult<PyProof> {
        let genome = self.inner.lock().unwrap();
        let proof = to_py_result(genome.verify_action(&action.inner))?;
        Ok(PyProof { inner: proof })
    }

    /// Python repr
    fn __repr__(&self) -> String {
        let genome = self.inner.lock().unwrap();
        format!(
            "SealedGenome(rules={}, sealed={})",
            genome.rules().len(),
            genome.is_sealed()
        )
    }

    /// Python str
    fn __str__(&self) -> String {
        self.__repr__()
    }

    /// Context manager support: __enter__
    fn __enter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    /// Context manager support: __exit__
    #[pyo3(signature = (_exc_type=None, _exc_value=None, _traceback=None))]
    fn __exit__(
        &mut self,
        _exc_type: Option<Bound<PyAny>>,
        _exc_value: Option<Bound<PyAny>>,
        _traceback: Option<Bound<PyAny>>,
    ) -> PyResult<bool> {
        // Cleanup if needed
        Ok(false) // Don't suppress exceptions
    }

    /// Get diagnostic information
    ///
    /// Returns:
    ///     dict: Diagnostic data including version, sealed status, rule count
    fn diagnostics(&self) -> PyResult<Py<PyDict>> {
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            let genome = self.inner.lock().unwrap();

            dict.set_item("version", "1.5.0")?;
            dict.set_item("sealed", genome.is_sealed())?;
            dict.set_item("rule_count", genome.rules().len())?;
            dict.set_item("rules", genome.rules())?;

            if let Some(hash) = genome.capsule_hash() {
                dict.set_item("genome_hash", hash)?;
            }

            Ok(dict.into())
        })
    }
}

// Thread-safe Clone implementation
impl Clone for PySealedGenome {
    fn clone(&self) -> Self {
        PySealedGenome {
            inner: Arc::clone(&self.inner),
        }
    }
}
