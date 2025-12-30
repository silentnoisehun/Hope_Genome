//! ConsensusEngine Python wrapper

use crate::consensus::ConsensusVerifier;
use pyo3::prelude::*;

/// Python wrapper for ConsensusEngine
#[pyclass(name = "ConsensusEngine")]
#[allow(dead_code)]
pub struct PyConsensusEngine {
    inner: ConsensusVerifier,
}

#[pymethods]
impl PyConsensusEngine {
    /// Create a new ConsensusEngine
    #[new]
    #[pyo3(signature = (required_sources=3, tolerance=0.1))]
    fn new(required_sources: usize, tolerance: f64) -> Self {
        PyConsensusEngine {
            inner: ConsensusVerifier::new(required_sources, tolerance),
        }
    }

    /// Python repr
    fn __repr__(&self) -> String {
        "ConsensusEngine(backend='Rust')".to_string()
    }
}
