//! AIBOM (AI Bill of Materials) Python wrapper

use pyo3::prelude::*;
use pyo3::types::PyDict;
use crate::compliance::{AiBom, Component};
use super::errors::to_py_result;

/// Python wrapper for AibomVerifier
///
/// Implements OWASP AI-SBOM compliance with Fort Knox integrity enforcement.
/// Verifies that AI models and components match their declared hashes,
/// preventing supply chain attacks and unauthorized model substitution.
///
/// # Example
/// ```python
/// verifier = AibomVerifier.from_file("aibom.xml")
/// verifier.verify_component("medical-diagnosis-model-v2", model_bytes)
/// ```
#[pyclass(name = "AibomVerifier")]
pub struct PyAibomVerifier {
    inner: AibomVerifier,
}

#[pymethods]
impl PyAibomVerifier {
    /// Load AIBOM from CycloneDX XML file
    ///
    /// Args:
    ///     file_path (str): Path to aibom.xml file
    ///
    /// Returns:
    ///     AibomVerifier: Verifier instance
    ///
    /// Raises:
    ///     AibomError: If file cannot be loaded
    #[staticmethod]
    fn from_file(file_path: String) -> PyResult<Self> {
        let inner = to_py_result(AibomVerifier::from_file(&file_path))?;
        Ok(PyAibomVerifier { inner })
    }

    /// Load AIBOM from CycloneDX JSON file
    ///
    /// Args:
    ///     file_path (str): Path to aibom.json file
    ///
    /// Returns:
    ///     AibomVerifier: Verifier instance
    ///
    /// Raises:
    ///     AibomError: If file cannot be loaded
    #[staticmethod]
    fn from_json_file(file_path: String) -> PyResult<Self> {
        let inner = to_py_result(AibomVerifier::from_json_file(&file_path))?;
        Ok(PyAibomVerifier { inner })
    }

    /// Create AIBOM from components
    ///
    /// Args:
    ///     components (list[AibomComponent]): List of components
    ///
    /// Returns:
    ///     AibomVerifier: Verifier instance
    #[staticmethod]
    fn from_components(components: Vec<PyAibomComponent>) -> Self {
        let inner_components: Vec<AibomComponent> = components
            .into_iter()
            .map(|c| c.inner)
            .collect();
        PyAibomVerifier {
            inner: AibomVerifier::from_components(inner_components),
        }
    }

    /// Verify a component against its declared hash
    ///
    /// Fort Knox policy: Hash mismatch = HALT execution (no fallback)
    ///
    /// Args:
    ///     component_name (str): Name of the component to verify
    ///     component_data (bytes): Actual component bytes
    ///
    /// Raises:
    ///     AibomError: If hash mismatch detected (CRITICAL)
    fn verify_component(&self, component_name: String, component_data: Vec<u8>) -> PyResult<()> {
        to_py_result(self.inner.verify_component(&component_name, &component_data))
    }

    /// Get all components
    ///
    /// Returns:
    ///     list[AibomComponent]: All registered components
    fn get_components(&self) -> Vec<PyAibomComponent> {
        self.inner
            .get_components()
            .iter()
            .map(|c| PyAibomComponent { inner: c.clone() })
            .collect()
    }

    /// Get a specific component by name
    ///
    /// Args:
    ///     name (str): Component name
    ///
    /// Returns:
    ///     AibomComponent | None: Component if found
    fn get_component(&self, name: String) -> Option<PyAibomComponent> {
        self.inner
            .get_component(&name)
            .map(|c| PyAibomComponent { inner: c.clone() })
    }

    /// Export to CycloneDX XML
    ///
    /// Returns:
    ///     str: XML representation
    fn to_xml(&self) -> PyResult<String> {
        to_py_result(self.inner.to_xml())
    }

    /// Export to CycloneDX JSON
    ///
    /// Returns:
    ///     str: JSON representation
    fn to_json(&self) -> PyResult<String> {
        to_py_result(self.inner.to_json())
    }

    /// Python repr
    fn __repr__(&self) -> String {
        format!(
            "AibomVerifier(components={}, standard='OWASP AI-SBOM')",
            self.inner.get_components().len()
        )
    }
}

/// Python wrapper for AibomComponent
///
/// Represents a single AI model or component in the AIBOM.
#[pyclass(name = "AibomComponent")]
#[derive(Clone)]
pub struct PyAibomComponent {
    inner: AibomComponent,
}

#[pymethods]
impl PyAibomComponent {
    /// Create a new AIBOM component
    ///
    /// Args:
    ///     name (str): Component name
    ///     version (str): Component version
    ///     hash_sha256 (str): Hex-encoded SHA-256 hash
    ///     component_type (str, optional): Type (default: "ML-Model")
    ///
    /// Returns:
    ///     AibomComponent: New component
    #[new]
    #[pyo3(signature = (name, version, hash_sha256, component_type="ML-Model"))]
    fn new(name: String, version: String, hash_sha256: String, component_type: &str) -> PyResult<Self> {
        let hash_bytes = hex::decode(&hash_sha256).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid hex hash: {}", e))
        })?;

        if hash_bytes.len() != 32 {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "SHA-256 hash must be exactly 32 bytes (64 hex characters)"
            ));
        }

        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&hash_bytes);

        Ok(PyAibomComponent {
            inner: AibomComponent::new(&name, &version, hash_array, component_type),
        })
    }

    /// Get component name
    #[getter]
    fn name(&self) -> String {
        self.inner.name.clone()
    }

    /// Get component version
    #[getter]
    fn version(&self) -> String {
        self.inner.version.clone()
    }

    /// Get component hash
    #[getter]
    fn hash_sha256(&self) -> String {
        hex::encode(&self.inner.hash_sha256)
    }

    /// Get component type
    #[getter]
    fn component_type(&self) -> String {
        self.inner.component_type.clone()
    }

    /// Convert to dictionary
    fn to_dict(&self) -> PyResult<Py<PyDict>> {
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            dict.set_item("name", &self.inner.name)?;
            dict.set_item("version", &self.inner.version)?;
            dict.set_item("hash_sha256", hex::encode(&self.inner.hash_sha256))?;
            dict.set_item("component_type", &self.inner.component_type)?;
            Ok(dict.into())
        })
    }

    /// Python repr
    fn __repr__(&self) -> String {
        format!(
            "AibomComponent(name='{}', version='{}', type='{}')",
            self.inner.name, self.inner.version, self.inner.component_type
        )
    }
}
