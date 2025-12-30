//! KeyStore Python wrapper

use pyo3::prelude::*;
use crate::crypto::SoftwareKeyStore;
use super::errors::to_py_result;

/// Software-based KeyStore
#[pyclass(name = "SoftwareKeyStore")]
pub struct PySoftwareKeyStore {
    inner: SoftwareKeyStore,
}

#[pymethods]
impl PySoftwareKeyStore {
    /// Generate a new Ed25519 keypair
    #[staticmethod]
    fn generate() -> PyResult<Self> {
        let inner = to_py_result(SoftwareKeyStore::generate())?;
        Ok(PySoftwareKeyStore { inner })
    }

    /// Python repr
    fn __repr__(&self) -> String {
        "SoftwareKeyStore(type='Ed25519')".to_string()
    }
}
