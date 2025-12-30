//! Error conversions for Python bindings

use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;

use crate::audit_log::AuditError;
use crate::auditor::AuditorError;
use crate::consensus::ConsensusError;
use crate::crypto::CryptoError;
use crate::genome::GenomeError;

// Custom Python exceptions
create_exception!(_hope_core, PyGenomeError, PyException, "Hope Genome error");
create_exception!(
    _hope_core,
    PyCryptoError,
    PyException,
    "Cryptographic operation error"
);
create_exception!(
    _hope_core,
    PyAuditorError,
    PyException,
    "Proof auditor error"
);
create_exception!(
    _hope_core,
    PyConsensusError,
    PyException,
    "Consensus engine error"
);
create_exception!(
    _hope_core,
    PyAibomError,
    PyException,
    "AI-BOM verification error"
);

// Conversion implementations
impl From<GenomeError> for PyErr {
    fn from(err: GenomeError) -> PyErr {
        match err {
            GenomeError::AlreadySealed => {
                PyGenomeError::new_err("Genome is already sealed and immutable")
            }
            GenomeError::NotSealed => {
                PyGenomeError::new_err("Genome must be sealed before generating proofs")
            }
            GenomeError::RuleViolation(reason) => {
                PyGenomeError::new_err(format!("Action violates genome rules: {}", reason))
            }
            GenomeError::HsmConfigError(var) => {
                PyGenomeError::new_err(format!("HSM configuration error: {} not set", var))
            }
            GenomeError::CryptoError(e) => {
                PyCryptoError::new_err(format!("Cryptographic error: {}", e))
            }
        }
    }
}

impl From<CryptoError> for PyErr {
    fn from(err: CryptoError) -> PyErr {
        match err {
            CryptoError::KeyGeneration(msg) => {
                PyCryptoError::new_err(format!("Key generation failed: {}", msg))
            }
            CryptoError::SigningFailed(msg) => {
                PyCryptoError::new_err(format!("Signature generation failed: {}", msg))
            }
            CryptoError::VerificationFailed(msg) => {
                PyCryptoError::new_err(format!("Signature verification failed: {}", msg))
            }
            CryptoError::InvalidSignature => {
                PyCryptoError::new_err("Invalid cryptographic signature detected")
            }
            CryptoError::InvalidKeyFormat(msg) => {
                PyCryptoError::new_err(format!("Invalid key format: {}", msg))
            }
            CryptoError::PublicKeyMismatch => PyCryptoError::new_err(
                "PublicKey-SecretKey mismatch detected (P0: key leakage attack blocked)",
            ),
            CryptoError::CriticalSecurityFault => PyCryptoError::new_err(
                "CRITICAL SECURITY FAULT: Verify-after-sign failed (P2: fault attack detected)",
            ),
            CryptoError::HsmError(msg) => {
                PyCryptoError::new_err(format!("HSM operation failed: {}", msg))
            }
            CryptoError::HsmKeyNotFound(label) => {
                PyCryptoError::new_err(format!("Key not found in HSM: {}", label))
            }
            CryptoError::TeeError(msg) => {
                PyCryptoError::new_err(format!("TEE operation failed: {}", msg))
            }
            CryptoError::TeeKeyNotFound(label) => {
                PyCryptoError::new_err(format!("Key not found in TEE: {}", label))
            }
        }
    }
}

impl From<AuditorError> for PyErr {
    fn from(err: AuditorError) -> PyErr {
        match err {
            AuditorError::InvalidSignature => {
                PyAuditorError::new_err("Proof signature verification failed")
            }
            AuditorError::ProofExpired { issued, now, ttl } => PyAuditorError::new_err(format!(
                "Proof has expired (issued: {}, now: {}, TTL: {}s)",
                issued, now, ttl
            )),
            AuditorError::NonceReused(nonce) => PyAuditorError::new_err(format!(
                "Replay attack detected: nonce already used ({})",
                nonce
            )),
            AuditorError::NonceStoreError(e) => {
                PyAuditorError::new_err(format!("Nonce store error: {}", e))
            }
            AuditorError::CryptoError(e) => {
                PyCryptoError::new_err(format!("Cryptographic error during audit: {}", e))
            }
        }
    }
}

impl From<ConsensusError> for PyErr {
    fn from(err: ConsensusError) -> PyErr {
        // Generic error conversion for consensus errors
        PyConsensusError::new_err(format!("Consensus error: {}", err))
    }
}

impl From<AuditError> for PyErr {
    fn from(err: AuditError) -> PyErr {
        match err {
            AuditError::IoError(e) => PyException::new_err(format!("IO error in audit log: {}", e)),
            AuditError::SerializationError(e) => {
                PyException::new_err(format!("Serialization error in audit log: {}", e))
            }
            AuditError::BrokenChain {
                index,
                expected,
                found,
            } => PyException::new_err(format!(
                "Audit chain integrity broken at index {}: expected {:?}, found {:?}",
                index, expected, found
            )),
            AuditError::InvalidSignature(index) => {
                PyException::new_err(format!("Invalid signature at audit log index {}", index))
            }
            AuditError::CryptoError(e) => {
                PyCryptoError::new_err(format!("Cryptographic error in audit log: {}", e))
            }
        }
    }
}

// Generic result converter for Python
pub fn to_py_result<T>(result: Result<T, impl Into<PyErr>>) -> PyResult<T> {
    result.map_err(|e| e.into())
}
