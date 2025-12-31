//! Action Python wrapper

use crate::proof::Action;
use pyo3::prelude::*;

/// Python wrapper for Action
#[pyclass(name = "Action")]
#[derive(Clone)]
pub struct PyAction {
    pub(crate) inner: Action,
}

impl PyAction {
    /// Convert to Rust Action (for internal use)
    pub fn to_rust_action(&self) -> Action {
        self.inner.clone()
    }
}

#[pymethods]
impl PyAction {
    /// Create a file deletion action
    #[staticmethod]
    fn delete_file(file_path: String) -> Self {
        PyAction {
            inner: Action::delete(&file_path),
        }
    }

    /// Create a file read action
    #[staticmethod]
    fn read_file(file_path: String) -> Self {
        PyAction {
            inner: Action::read(&file_path),
        }
    }

    /// Create a file write action
    #[staticmethod]
    fn write_file(file_path: String, content: Vec<u8>) -> Self {
        PyAction {
            inner: Action::write_file(&file_path, content),
        }
    }

    /// Create a command execution action
    #[staticmethod]
    fn execute_command(command: String) -> Self {
        PyAction {
            inner: Action::execute(&command),
        }
    }

    /// Get the action hash
    fn action_hash(&self) -> String {
        hex::encode(self.inner.hash())
    }

    /// Python repr
    fn __repr__(&self) -> String {
        format!(
            "Action(type='{:?}', target='{}')",
            self.inner.action_type, self.inner.target
        )
    }

    /// Python equality
    fn __eq__(&self, other: &Self) -> bool {
        self.inner.hash() == other.inner.hash()
    }

    /// Python hash
    fn __hash__(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.inner.hash().hash(&mut hasher);
        hasher.finish()
    }
}
