//! AuditLogger Python wrapper

use pyo3::prelude::*;
use std::sync::{Arc, Mutex};

use crate::audit_log::{AuditLog, AuditEntry};
use crate::crypto::KeyPair;
use super::errors::to_py_result;

/// Python wrapper for AuditLogger
#[pyclass(name = "AuditLogger")]
pub struct PyAuditLogger {
    inner: Arc<Mutex<AuditLog>>,
}

#[pymethods]
impl PyAuditLogger {
    /// Create a new audit logger
    #[new]
    fn new(log_path: String) -> PyResult<Self> {
        let keypair = to_py_result(KeyPair::generate())?;
        let inner = to_py_result(AuditLog::with_storage(keypair, &log_path))?;

        Ok(PyAuditLogger {
            inner: Arc::new(Mutex::new(inner)),
        })
    }

    /// Get the total number of audit entries
    fn entry_count(&self) -> usize {
        self.inner.lock().unwrap().len()
    }

    /// Python repr
    fn __repr__(&self) -> String {
        let logger = self.inner.lock().unwrap();
        format!("AuditLogger(entries={})", logger.len())
    }
}

/// Python wrapper for AuditEntry
#[pyclass(name = "AuditEntry")]
#[derive(Clone)]
pub struct PyAuditEntry {
    inner: AuditEntry,
}

#[pymethods]
impl PyAuditEntry {
    /// Get the entry index
    #[getter]
    fn index(&self) -> usize {
        self.inner.index as usize
    }

    /// Get the entry timestamp
    #[getter]
    fn timestamp(&self) -> u64 {
        self.inner.timestamp
    }

    /// Python repr
    fn __repr__(&self) -> String {
        format!(
            "AuditEntry(index={}, timestamp={})",
            self.inner.index,
            self.inner.timestamp
        )
    }
}
