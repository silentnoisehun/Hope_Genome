#[allow(deprecated)]
use hope_core::{
    Action, ActionType, AuditLog, Decision, IntegrityProof, KeyPair,
    ProofAuditor, SealedGenome, VerificationStatus,
};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

/// Python wrapper for SealedGenome
#[pyclass(name = "HopeGenome")]
struct PyHopeGenome {
    inner: SealedGenome,
}

#[pymethods]
impl PyHopeGenome {
    #[new]
    fn new(rules: Vec<String>) -> PyResult<Self> {
        let genome = SealedGenome::new(rules)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create genome: {}", e)))?;
        Ok(PyHopeGenome { inner: genome })
    }

    fn seal(&mut self) -> PyResult<()> {
        self.inner
            .seal()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to seal genome: {}", e)))
    }

    fn is_sealed(&self) -> bool {
        self.inner.is_sealed()
    }

    fn rules(&self) -> Vec<String> {
        self.inner.rules().to_vec()
    }

    fn capsule_hash(&self) -> Option<String> {
        self.inner.capsule_hash().map(|s| s.to_string())
    }

    fn set_default_ttl(&mut self, ttl: u64) {
        self.inner.set_default_ttl(ttl);
    }

    fn verify_action(&self, action: &PyAction) -> PyResult<PyIntegrityProof> {
        let rust_action = action.to_rust_action();
        let proof = self
            .inner
            .verify_action(&rust_action)
            .map_err(|e| PyRuntimeError::new_err(format!("Verification failed: {}", e)))?;
        Ok(PyIntegrityProof::from_rust(proof))
    }
}

/// Python wrapper for Action
#[pyclass(name = "Action")]
#[derive(Clone)]
struct PyAction {
    action_type: String,
    target: String,
    payload: Option<Vec<u8>>,
}

#[pymethods]
impl PyAction {
    #[staticmethod]
    fn delete(target: String) -> Self {
        PyAction {
            action_type: "Delete".to_string(),
            target,
            payload: None,
        }
    }

    #[staticmethod]
    fn write_file(path: String, content: Vec<u8>) -> Self {
        PyAction {
            action_type: "Write".to_string(),
            target: path,
            payload: Some(content),
        }
    }

    #[staticmethod]
    fn read(target: String) -> Self {
        PyAction {
            action_type: "Read".to_string(),
            target,
            payload: None,
        }
    }

    #[staticmethod]
    fn execute(command: String) -> Self {
        PyAction {
            action_type: "Execute".to_string(),
            target: command,
            payload: None,
        }
    }

    #[getter]
    fn action_type(&self) -> String {
        self.action_type.clone()
    }

    #[getter]
    fn target(&self) -> String {
        self.target.clone()
    }

    fn __repr__(&self) -> String {
        format!("Action(type={}, target={})", self.action_type, self.target)
    }
}

impl PyAction {
    fn to_rust_action(&self) -> Action {
        match self.action_type.as_str() {
            "Delete" => Action::delete(&self.target),
            "Write" => Action::write_file(&self.target, self.payload.clone().unwrap_or_default()),
            "Read" => Action::read(&self.target),
            "Execute" => Action::execute(&self.target),
            _ => Action::read(&self.target), // Fallback
        }
    }
}

/// Python wrapper for IntegrityProof
#[pyclass(name = "IntegrityProof")]
struct PyIntegrityProof {
    #[pyo3(get)]
    nonce: Vec<u8>,
    #[pyo3(get)]
    timestamp: u64,
    #[pyo3(get)]
    ttl: u64,
    #[pyo3(get)]
    action_hash: Vec<u8>,
    #[pyo3(get)]
    action_type: String,
    #[pyo3(get)]
    capsule_hash: String,
    #[pyo3(get)]
    status: String,
    #[pyo3(get)]
    signature: Vec<u8>,
}

impl PyIntegrityProof {
    fn from_rust(proof: IntegrityProof) -> Self {
        PyIntegrityProof {
            nonce: proof.nonce.to_vec(),
            timestamp: proof.timestamp,
            ttl: proof.ttl,
            action_hash: proof.action_hash.to_vec(),
            action_type: format!("{:?}", proof.action_type),
            capsule_hash: proof.capsule_hash,
            status: format!("{:?}", proof.status),
            signature: proof.signature,
        }
    }

    fn to_rust(&self) -> IntegrityProof {
        let action_type = match self.action_type.as_str() {
            "Delete" => ActionType::Delete,
            "Write" => ActionType::Write,
            "Read" => ActionType::Read,
            "Execute" => ActionType::Execute,
            _ => ActionType::Read,
        };

        let status = match self.status.as_str() {
            "OK" => VerificationStatus::OK,
            "Tampered" => VerificationStatus::Tampered,
            _ => VerificationStatus::Failed("Unknown".to_string()),
        };

        let mut nonce = [0u8; 32];
        nonce.copy_from_slice(&self.nonce[..32]);

        let mut action_hash = [0u8; 32];
        action_hash.copy_from_slice(&self.action_hash[..32]);

        IntegrityProof {
            nonce,
            timestamp: self.timestamp,
            ttl: self.ttl,
            action_hash,
            action_type,
            capsule_hash: self.capsule_hash.clone(),
            status,
            signature: self.signature.clone(),
        }
    }
}

#[pymethods]
impl PyIntegrityProof {
    fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp() as u64;
        now - self.timestamp > self.ttl
    }

    fn timestamp_string(&self) -> String {
        let dt = chrono::DateTime::from_timestamp(self.timestamp as i64, 0)
            .unwrap_or_else(chrono::Utc::now);
        dt.to_rfc3339()
    }

    fn __repr__(&self) -> String {
        format!(
            "IntegrityProof(status={}, timestamp={})",
            self.status, self.timestamp
        )
    }
}

/// Python wrapper for ProofAuditor
#[pyclass(name = "Auditor")]
struct PyAuditor {
    inner: ProofAuditor,
}

#[pymethods]
impl PyAuditor {
    #[new]
    fn new() -> PyResult<Self> {
        let key_store = hope_core::crypto::SoftwareKeyStore::generate()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to generate key store: {}", e)))?;
        let nonce_store = hope_core::nonce_store::MemoryNonceStore::new();
        let auditor = ProofAuditor::new(Box::new(key_store), Box::new(nonce_store));
        Ok(PyAuditor { inner: auditor })
    }

    fn verify_proof(&mut self, proof: &PyIntegrityProof) -> PyResult<()> {
        let rust_proof = proof.to_rust();
        self.inner
            .verify_proof(&rust_proof)
            .map_err(|e| PyRuntimeError::new_err(format!("Proof verification failed: {}", e)))
    }

    fn used_nonce_count(&self) -> usize {
        self.inner.used_nonce_count()
    }

    fn clear_nonces(&mut self) {
        let _ = self.inner.clear_nonces();
    }
}

/// Python wrapper for AuditLog
#[pyclass(name = "AuditLog")]
struct PyAuditLog {
    inner: AuditLog,
}

#[pymethods]
impl PyAuditLog {
    #[new]
    #[allow(deprecated)]
    fn new() -> PyResult<Self> {
        let keypair = KeyPair::generate()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to generate keypair: {}", e)))?;
        let log = AuditLog::new(keypair)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create audit log: {}", e)))?;
        Ok(PyAuditLog { inner: log })
    }

    fn append(
        &mut self,
        action: &PyAction,
        proof: &PyIntegrityProof,
        approved: bool,
    ) -> PyResult<()> {
        let rust_action = action.to_rust_action();
        let rust_proof = proof.to_rust();
        let decision = if approved {
            Decision::Approved
        } else {
            Decision::Denied {
                reason: "Denied by user".to_string(),
            }
        };

        self.inner
            .append(rust_action, rust_proof, decision)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to append to audit log: {}", e)))
    }

    fn verify_chain(&self) -> PyResult<()> {
        self.inner
            .verify_chain()
            .map_err(|e| PyRuntimeError::new_err(format!("Chain verification failed: {}", e)))
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn __len__(&self) -> usize {
        self.inner.len()
    }
}

/// Python wrapper for ConsensusVerifier
#[pyclass(name = "ConsensusVerifier")]
struct PyConsensusVerifier;

#[pymethods]
impl PyConsensusVerifier {
    #[new]
    fn new(_required_sources: usize, _tolerance: f64) -> Self {
        PyConsensusVerifier
    }
}

/// Module initialization
#[pymodule]
fn _hope_genome(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyHopeGenome>()?;
    m.add_class::<PyAction>()?;
    m.add_class::<PyIntegrityProof>()?;
    m.add_class::<PyAuditor>()?;
    m.add_class::<PyAuditLog>()?;
    m.add_class::<PyConsensusVerifier>()?;

    m.add("__version__", "1.2.0")?;
    m.add("__author__", "Máté Róbert, Claude (Anthropic)")?;

    Ok(())
}
