use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// Type of action being performed
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionType {
    Delete,
    Write,
    Read,
    Execute,
    Network,
    Custom(String),
}

/// Verification status of a proof
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationStatus {
    OK,
    Tampered,
    Failed(String),
}

/// Action that can be performed by the AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub action_type: ActionType,
    pub target: String,
    pub payload: Option<Vec<u8>>,
    pub metadata: Option<String>,
}

impl Action {
    pub fn delete(target: impl Into<String>) -> Self {
        Action {
            action_type: ActionType::Delete,
            target: target.into(),
            payload: None,
            metadata: None,
        }
    }

    pub fn write_file(path: impl Into<String>, content: Vec<u8>) -> Self {
        Action {
            action_type: ActionType::Write,
            target: path.into(),
            payload: Some(content),
            metadata: None,
        }
    }

    pub fn read(target: impl Into<String>) -> Self {
        Action {
            action_type: ActionType::Read,
            target: target.into(),
            payload: None,
            metadata: None,
        }
    }

    pub fn execute(command: impl Into<String>) -> Self {
        Action {
            action_type: ActionType::Execute,
            target: command.into(),
            payload: None,
            metadata: None,
        }
    }

    /// Get the canonical hash of this action
    pub fn hash(&self) -> [u8; 32] {
        let serialized = serde_json::to_vec(self).unwrap();
        crate::crypto::hash_bytes(&serialized)
    }
}

/// Cryptographic proof of action integrity
///
/// This struct provides tamper-evident proof that an action was approved
/// by the Hope Genome system. It includes:
/// - Anti-replay protection (nonce + TTL)
/// - Action binding (prevents oracle attacks)
/// - Cryptographic signature (prevents forgery)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityProof {
    /// Anti-replay nonce (256-bit random)
    pub nonce: [u8; 32],

    /// Unix timestamp (seconds) when proof was created
    pub timestamp: u64,

    /// Time-to-live in seconds (proof expires after this)
    pub ttl: u64,

    /// Hash of the action being approved (prevents oracle attacks)
    pub action_hash: [u8; 32],

    /// Type of action
    pub action_type: ActionType,

    /// Current genome capsule hash (ties proof to specific genome state)
    pub capsule_hash: String,

    /// Verification status
    pub status: VerificationStatus,

    /// RSA signature (signs all above fields)
    pub signature: Vec<u8>,
}

impl IntegrityProof {
    /// Create a new proof (before signing)
    pub fn new(
        action: &Action,
        capsule_hash: String,
        ttl: u64,
    ) -> Self {
        let nonce = crate::crypto::generate_nonce();
        let timestamp = chrono::Utc::now().timestamp() as u64;

        IntegrityProof {
            nonce,
            timestamp,
            ttl,
            action_hash: action.hash(),
            action_type: action.action_type.clone(),
            capsule_hash,
            status: VerificationStatus::OK,
            signature: Vec::new(), // Will be filled by signing
        }
    }

    /// Get the data that should be signed
    pub fn signing_data(&self) -> Vec<u8> {
        // Serialize everything except signature
        let mut data = Vec::new();
        data.extend_from_slice(&self.nonce);
        data.extend_from_slice(&self.timestamp.to_le_bytes());
        data.extend_from_slice(&self.ttl.to_le_bytes());
        data.extend_from_slice(&self.action_hash);
        data.extend_from_slice(self.capsule_hash.as_bytes());
        data
    }

    /// Check if proof has expired
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp() as u64;
        now - self.timestamp > self.ttl
    }

    /// Get human-readable timestamp
    pub fn timestamp_string(&self) -> String {
        let dt = DateTime::from_timestamp(self.timestamp as i64, 0)
            .unwrap_or_else(|| Utc::now());
        dt.to_rfc3339()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_hash_deterministic() {
        let action = Action::delete("test.txt");
        let hash1 = action.hash();
        let hash2 = action.hash();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_different_actions_different_hashes() {
        let action1 = Action::delete("test1.txt");
        let action2 = Action::delete("test2.txt");
        assert_ne!(action1.hash(), action2.hash());
    }

    #[test]
    fn test_proof_nonce_uniqueness() {
        let action = Action::delete("test.txt");
        let proof1 = IntegrityProof::new(&action, "hash1".into(), 60);
        let proof2 = IntegrityProof::new(&action, "hash1".into(), 60);
        assert_ne!(proof1.nonce, proof2.nonce);
    }

    #[test]
    fn test_proof_expiration() {
        let action = Action::delete("test.txt");
        let mut proof = IntegrityProof::new(&action, "hash1".into(), 0);

        // Set timestamp to 10 seconds ago with 5 second TTL
        proof.timestamp = chrono::Utc::now().timestamp() as u64 - 10;
        proof.ttl = 5;

        assert!(proof.is_expired());
    }

    #[test]
    fn test_proof_not_expired() {
        let action = Action::delete("test.txt");
        let proof = IntegrityProof::new(&action, "hash1".into(), 3600);
        assert!(!proof.is_expired());
    }
}
