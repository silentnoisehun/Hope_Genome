//! Proof Python wrapper

use pyo3::prelude::*;
use crate::proof::{IntegrityProof, VerificationStatus};

/// Python wrapper for Proof
#[pyclass(name = "Proof")]
#[derive(Clone)]
pub struct PyProof {
    pub(crate) inner: IntegrityProof,
}

#[pymethods]
impl PyProof {
    /// Check if the action was approved
    #[getter]
    fn approved(&self) -> bool {
        matches!(self.inner.status, VerificationStatus::OK)
    }

    /// Get the genome/capsule hash
    #[getter]
    fn genome_hash(&self) -> String {
        self.inner.capsule_hash.clone()
    }

    /// Get the action hash
    #[getter]
    fn action_hash(&self) -> String {
        hex::encode(&self.inner.action_hash)
    }

    /// Get the cryptographic signature
    fn signature_hex(&self) -> String {
        hex::encode(&self.inner.signature)
    }

    /// Get the signature as bytes
    fn signature_bytes(&self) -> Vec<u8> {
        self.inner.signature.clone()
    }

    /// Get the cryptographic nonce
    fn nonce_hex(&self) -> String {
        hex::encode(&self.inner.nonce)
    }

    /// Get the nonce as bytes
    fn nonce_bytes(&self) -> Vec<u8> {
        self.inner.nonce.to_vec()
    }

    /// Get the timestamp as Unix epoch seconds
    fn timestamp(&self) -> i64 {
        self.inner.timestamp as i64
    }

    /// Get the timestamp as ISO 8601 string
    fn timestamp_string(&self) -> String {
        use chrono::{DateTime, Utc, TimeZone};
        let dt: DateTime<Utc> = Utc.timestamp_opt(self.inner.timestamp as i64, 0).unwrap();
        dt.to_rfc3339()
    }

    /// Get the time-to-live in seconds
    #[getter]
    fn ttl_seconds(&self) -> u64 {
        self.inner.ttl
    }

    /// Check if the proof has expired
    fn is_expired(&self) -> bool {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now > self.inner.timestamp + self.inner.ttl
    }

    /// Get the denial reason (if denied)
    fn denial_reason(&self) -> Option<String> {
        match &self.inner.status {
            VerificationStatus::Failed(reason) => Some(reason.clone()),
            VerificationStatus::Tampered => Some("Proof was tampered with".to_string()),
            VerificationStatus::OK => None,
        }
    }

    /// Python repr
    fn __repr__(&self) -> String {
        format!(
            "Proof(approved={}, timestamp={}, nonce='{}')",
            self.approved(),
            self.inner.timestamp,
            hex::encode(&self.inner.nonce[..4])
        )
    }

    /// Python str
    fn __str__(&self) -> String {
        if self.approved() {
            format!(
                "✅ APPROVED | Timestamp: {} | Nonce: {}...",
                self.timestamp_string(),
                hex::encode(&self.inner.nonce[..4])
            )
        } else {
            format!(
                "❌ DENIED: {} | Timestamp: {}",
                self.denial_reason().unwrap_or_else(|| "Unknown".to_string()),
                self.timestamp_string()
            )
        }
    }

    /// Python equality
    fn __eq__(&self, other: &Self) -> bool {
        self.inner.nonce == other.inner.nonce
            && self.inner.signature == other.inner.signature
    }

    /// Python hash
    fn __hash__(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.inner.nonce.hash(&mut hasher);
        hasher.finish()
    }
}
