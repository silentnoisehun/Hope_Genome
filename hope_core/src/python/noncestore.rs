//! NonceStore Python wrapper

use pyo3::prelude::*;
use crate::nonce_store::MemoryNonceStore;

/// In-memory nonce store
#[pyclass(name = "MemoryNonceStore")]
pub struct PyMemoryNonceStore {
    inner: MemoryNonceStore,
}

#[pymethods]
impl PyMemoryNonceStore {
    /// Create a new in-memory nonce store
    #[new]
    fn new() -> Self {
        PyMemoryNonceStore {
            inner: MemoryNonceStore::new(),
        }
    }

    /// Check if a nonce exists
    fn contains(&self, nonce: Vec<u8>) -> PyResult<bool> {
        use crate::nonce_store::NonceStore;

        if nonce.len() != 32 {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Nonce must be exactly 32 bytes"
            ));
        }
        let mut nonce_array = [0u8; 32];
        nonce_array.copy_from_slice(&nonce);
        Ok(self.inner.contains(&nonce_array))
    }

    /// Python repr
    fn __repr__(&self) -> String {
        "MemoryNonceStore(persistent=false)".to_string()
    }
}
