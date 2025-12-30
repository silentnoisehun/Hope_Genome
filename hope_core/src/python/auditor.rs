//! ProofAuditor Python wrapper

use pyo3::prelude::*;
use std::sync::{Arc, Mutex};

use super::errors::to_py_result;
use super::proof::PyProof;
use crate::auditor::ProofAuditor;
use crate::crypto::SoftwareKeyStore;
use crate::nonce_store::MemoryNonceStore;

/// Python wrapper for ProofAuditor
#[pyclass(name = "ProofAuditor")]
pub struct PyProofAuditor {
    inner: Arc<Mutex<ProofAuditor>>,
}

#[pymethods]
impl PyProofAuditor {
    /// Create a new ProofAuditor with default stores
    #[new]
    fn new() -> PyResult<Self> {
        let key_store = Box::new(to_py_result(SoftwareKeyStore::generate())?);
        let nonce_store = Box::new(MemoryNonceStore::new());

        let auditor = ProofAuditor::new(key_store, nonce_store);

        Ok(PyProofAuditor {
            inner: Arc::new(Mutex::new(auditor)),
        })
    }

    /// Verify a proof
    fn verify_proof(&mut self, proof: &PyProof) -> PyResult<()> {
        let mut auditor = self.inner.lock().unwrap();
        to_py_result(auditor.verify_proof(&proof.inner))
    }

    /// Python repr
    fn __repr__(&self) -> String {
        "ProofAuditor(backend='Rust')".to_string()
    }
}
