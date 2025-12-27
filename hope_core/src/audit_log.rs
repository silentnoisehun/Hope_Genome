use crate::crypto::{hash_bytes, KeyPair};
use crate::proof::{Action, IntegrityProof};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuditError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error(
        "Chain integrity broken at index {index}: expected prev_hash {expected:?}, found {found:?}"
    )]
    BrokenChain {
        index: usize,
        expected: [u8; 32],
        found: [u8; 32],
    },

    #[error("Invalid signature at index {0}")]
    InvalidSignature(usize),

    #[error("Crypto error: {0}")]
    CryptoError(#[from] crate::crypto::CryptoError),
}

pub type Result<T> = std::result::Result<T, AuditError>;

/// Decision made by the genome
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Decision {
    Approved,
    Denied { reason: String },
}

/// A single entry in the audit log
///
/// Each entry is cryptographically linked to the previous entry,
/// forming a blockchain-style tamper-evident chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Sequential index (starts at 0)
    pub index: u64,

    /// Unix timestamp when entry was created
    pub timestamp: u64,

    /// Action that was requested
    pub action: Action,

    /// Cryptographic proof for this action
    pub proof: IntegrityProof,

    /// Decision made (approved or denied)
    pub decision: Decision,

    /// Hash of previous entry (blockchain linkage)
    pub prev_hash: [u8; 32],

    /// Hash of this entry (excluding signature)
    pub current_hash: [u8; 32],

    /// RSA signature of this entire entry
    pub signature: Vec<u8>,
}

impl AuditEntry {
    /// Compute the hash of this entry (for blockchain linkage)
    pub fn compute_hash(&self) -> [u8; 32] {
        let mut data = Vec::new();
        data.extend_from_slice(&self.index.to_le_bytes());
        data.extend_from_slice(&self.timestamp.to_le_bytes());
        data.extend_from_slice(&self.action.hash());
        data.extend_from_slice(&self.proof.action_hash);
        data.extend_from_slice(&self.prev_hash);

        hash_bytes(&data)
    }

    /// Get data to be signed
    fn signing_data(&self) -> Vec<u8> {
        serde_json::to_vec(&(
            self.index,
            self.timestamp,
            &self.action,
            &self.proof,
            &self.decision,
            &self.prev_hash,
            &self.current_hash,
        ))
        .unwrap()
    }
}

/// Blockchain-style audit log
///
/// Provides tamper-evident logging of all AI actions and decisions.
/// Each entry is cryptographically linked to the previous one.
pub struct AuditLog {
    entries: Vec<AuditEntry>,
    storage_path: Option<PathBuf>,
    keypair: KeyPair,
}

impl AuditLog {
    /// Create a new audit log
    pub fn new(keypair: KeyPair) -> Result<Self> {
        Ok(AuditLog {
            entries: Vec::new(),
            storage_path: None,
            keypair,
        })
    }

    /// Create a new audit log with file persistence
    pub fn with_storage(keypair: KeyPair, path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        // Try to load existing log
        let entries = if path.exists() {
            Self::load_from_file(&path)?
        } else {
            Vec::new()
        };

        Ok(AuditLog {
            entries,
            storage_path: Some(path),
            keypair,
        })
    }

    /// Append a new entry to the audit log
    pub fn append(
        &mut self,
        action: Action,
        proof: IntegrityProof,
        decision: Decision,
    ) -> Result<()> {
        // Get previous hash (or genesis hash)
        let prev_hash = self
            .entries
            .last()
            .map(|e| e.current_hash)
            .unwrap_or([0u8; 32]); // Genesis block

        let index = self.entries.len() as u64;
        let timestamp = chrono::Utc::now().timestamp() as u64;

        // Create entry (without hash and signature)
        let mut entry = AuditEntry {
            index,
            timestamp,
            action,
            proof,
            decision,
            prev_hash,
            current_hash: [0u8; 32],
            signature: Vec::new(),
        };

        // Compute hash
        entry.current_hash = entry.compute_hash();

        // Sign entry
        let signing_data = entry.signing_data();
        entry.signature = self.keypair.sign(&signing_data)?;

        // Append to log
        self.entries.push(entry.clone());

        // Persist if storage is configured
        if let Some(path) = &self.storage_path {
            self.append_to_file(path, &entry)?;
        }

        Ok(())
    }

    /// Verify the entire chain integrity
    pub fn verify_chain(&self) -> Result<()> {
        for i in 1..self.entries.len() {
            let prev = &self.entries[i - 1];
            let curr = &self.entries[i];

            // Check linkage
            if curr.prev_hash != prev.current_hash {
                return Err(AuditError::BrokenChain {
                    index: i,
                    expected: prev.current_hash,
                    found: curr.prev_hash,
                });
            }

            // Check hash integrity
            let expected_hash = curr.compute_hash();
            if curr.current_hash != expected_hash {
                return Err(AuditError::BrokenChain {
                    index: i,
                    expected: expected_hash,
                    found: curr.current_hash,
                });
            }

            // Check signature
            let signing_data = curr.signing_data();
            self.keypair
                .verify(&signing_data, &curr.signature)
                .map_err(|_| AuditError::InvalidSignature(i))?;
        }

        Ok(())
    }

    /// Get all entries
    pub fn entries(&self) -> &[AuditEntry] {
        &self.entries
    }

    /// Get entry count
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if log is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Load entries from file
    fn load_from_file(path: &Path) -> Result<Vec<AuditEntry>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let entries: Vec<AuditEntry> = serde_json::from_reader(reader)?;
        Ok(entries)
    }

    /// Append entry to file (append-only)
    fn append_to_file(&self, path: &Path, _entry: &AuditEntry) -> Result<()> {
        // Write entire log (in production, use append-only format)
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;

        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self.entries)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proof::{Action, ActionType, VerificationStatus};

    fn create_test_proof() -> IntegrityProof {
        IntegrityProof {
            nonce: [1u8; 32],
            timestamp: 1000,
            ttl: 60,
            action_hash: [2u8; 32],
            action_type: ActionType::Delete,
            capsule_hash: "test_hash".into(),
            status: VerificationStatus::OK,
            signature: vec![],
        }
    }

    #[test]
    fn test_audit_log_append() {
        let keypair = KeyPair::generate().unwrap();
        let mut log = AuditLog::new(keypair).unwrap();

        let action = Action::delete("test.txt");
        let proof = create_test_proof();
        let decision = Decision::Approved;

        log.append(action, proof, decision).unwrap();

        assert_eq!(log.len(), 1);
        assert_eq!(log.entries()[0].index, 0);
    }

    #[test]
    fn test_audit_log_chain_linkage() {
        let keypair = KeyPair::generate().unwrap();
        let mut log = AuditLog::new(keypair).unwrap();

        // Add first entry
        log.append(
            Action::delete("file1.txt"),
            create_test_proof(),
            Decision::Approved,
        )
        .unwrap();

        // Add second entry
        log.append(
            Action::delete("file2.txt"),
            create_test_proof(),
            Decision::Approved,
        )
        .unwrap();

        // Verify linkage
        assert_eq!(log.entries()[1].prev_hash, log.entries()[0].current_hash);
    }

    #[test]
    fn test_verify_chain_success() {
        let keypair = KeyPair::generate().unwrap();
        let mut log = AuditLog::new(keypair).unwrap();

        // Add multiple entries
        for i in 0..5 {
            log.append(
                Action::delete(format!("file{}.txt", i)),
                create_test_proof(),
                Decision::Approved,
            )
            .unwrap();
        }

        // Verify chain
        assert!(log.verify_chain().is_ok());
    }

    #[test]
    fn test_verify_chain_detects_tampering() {
        let keypair = KeyPair::generate().unwrap();
        let mut log = AuditLog::new(keypair).unwrap();

        // Add entries
        log.append(
            Action::delete("file1.txt"),
            create_test_proof(),
            Decision::Approved,
        )
        .unwrap();
        log.append(
            Action::delete("file2.txt"),
            create_test_proof(),
            Decision::Approved,
        )
        .unwrap();
        log.append(
            Action::delete("file3.txt"),
            create_test_proof(),
            Decision::Approved,
        )
        .unwrap();

        // Tamper with middle entry (simulate attack)
        log.entries[1].current_hash[0] ^= 0xFF;

        // Verification should fail
        assert!(log.verify_chain().is_err());
    }

    #[test]
    fn test_genesis_block() {
        let keypair = KeyPair::generate().unwrap();
        let mut log = AuditLog::new(keypair).unwrap();

        log.append(
            Action::delete("test.txt"),
            create_test_proof(),
            Decision::Approved,
        )
        .unwrap();

        // First entry should have zero prev_hash (genesis)
        assert_eq!(log.entries()[0].prev_hash, [0u8; 32]);
    }
}
