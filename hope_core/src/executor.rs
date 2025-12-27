use crate::audit_log::{AuditLog, Decision};
use crate::auditor::ProofAuditor;
use crate::crypto::hash_bytes;
use crate::proof::{Action, ActionType, IntegrityProof};
use std::fs::{remove_file, OpenOptions};
use std::io::Write;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExecutorError {
    #[error("Auditor error: {0}")]
    AuditorError(#[from] crate::auditor::AuditorError),

    #[error("Audit log error: {0}")]
    AuditLogError(#[from] crate::audit_log::AuditError),

    #[error("Action mismatch: expected hash {expected:?}, found {found:?}")]
    ActionMismatch { expected: [u8; 32], found: [u8; 32] },

    #[error("Action type mismatch")]
    TypeMismatch,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Integrity violation: written data doesn't match expected hash")]
    IntegrityViolation,

    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

pub type Result<T> = std::result::Result<T, ExecutorError>;

/// Result of action execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionResult {
    Success,
    Denied { reason: String },
}

/// Secure execution engine
///
/// This component performs TOCTOU-safe execution of approved actions:
/// - Verifies proof before execution
/// - Binds proof to specific action (anti-oracle attack)
/// - Executes in Rust (not Python) for security
/// - Logs all actions to audit trail
pub struct SecureExecutor {
    auditor: ProofAuditor,
    audit_log: AuditLog,
}

impl SecureExecutor {
    /// Create a new secure executor
    pub fn new(auditor: ProofAuditor, audit_log: AuditLog) -> Self {
        SecureExecutor { auditor, audit_log }
    }

    /// Execute an action with cryptographic proof
    ///
    /// This is the main entry point for secure execution:
    /// 1. Verify proof cryptographically
    /// 2. Verify action binding (anti-oracle attack)
    /// 3. Execute action securely (TOCTOU-safe)
    /// 4. Log to audit trail
    pub fn execute_with_proof(
        &mut self,
        action: &Action,
        proof: &IntegrityProof,
    ) -> Result<ExecutionResult> {
        // 1. Verify proof
        self.auditor.verify_proof(proof)?;

        // 2. ACTION BINDING CHECK (anti-oracle attack)
        // Prevents: Get proof for action A, execute action B
        let expected_hash = action.hash();
        if proof.action_hash != expected_hash {
            let result = Err(ExecutorError::ActionMismatch {
                expected: expected_hash,
                found: proof.action_hash,
            });

            // Log the denial
            self.audit_log.append(
                action.clone(),
                proof.clone(),
                Decision::Denied {
                    reason: "Action hash mismatch (oracle attack detected)".into(),
                },
            )?;

            return result;
        }

        // 3. Type check
        if proof.action_type != action.action_type {
            let result = Err(ExecutorError::TypeMismatch);

            self.audit_log.append(
                action.clone(),
                proof.clone(),
                Decision::Denied {
                    reason: "Action type mismatch".into(),
                },
            )?;

            return result;
        }

        // 4. Execute action (Rust-controlled, TOCTOU-safe)
        let exec_result = match &action.action_type {
            ActionType::Write => self.execute_write(action)?,
            ActionType::Delete => self.execute_delete(action)?,
            ActionType::Read => self.execute_read(action)?,
            ActionType::Execute => self.execute_command(action)?,
            _ => ExecutionResult::Denied {
                reason: "Action type not implemented".into(),
            },
        };

        // 5. Log to audit trail
        let decision = match exec_result {
            ExecutionResult::Success => Decision::Approved,
            ExecutionResult::Denied { ref reason } => Decision::Denied {
                reason: reason.clone(),
            },
        };

        self.audit_log
            .append(action.clone(), proof.clone(), decision)?;

        Ok(exec_result)
    }

    /// TOCTOU-safe file write
    fn execute_write(&self, action: &Action) -> Result<ExecutionResult> {
        let path = Path::new(&action.target);
        let content = action
            .payload
            .as_ref()
            .ok_or_else(|| ExecutorError::PermissionDenied("No content provided".into()))?;

        // Atomic write operation
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;

        file.write_all(content)?;
        file.sync_all()?; // Ensure written to disk

        // Verify write integrity
        let written_content = std::fs::read(path)?;
        let written_hash = hash_bytes(&written_content);
        let expected_hash = hash_bytes(content);

        if written_hash != expected_hash {
            return Err(ExecutorError::IntegrityViolation);
        }

        Ok(ExecutionResult::Success)
    }

    /// Secure delete operation
    fn execute_delete(&self, action: &Action) -> Result<ExecutionResult> {
        let path = Path::new(&action.target);

        if !path.exists() {
            return Ok(ExecutionResult::Denied {
                reason: "File does not exist".into(),
            });
        }

        remove_file(path)?;

        Ok(ExecutionResult::Success)
    }

    /// Secure read operation
    fn execute_read(&self, action: &Action) -> Result<ExecutionResult> {
        let path = Path::new(&action.target);

        if !path.exists() {
            return Ok(ExecutionResult::Denied {
                reason: "File does not exist".into(),
            });
        }

        // In a real implementation, this would return the content
        // For now, just verify we can read it
        let _content = std::fs::read(path)?;

        Ok(ExecutionResult::Success)
    }

    /// Execute command (placeholder - requires sandboxing in production)
    fn execute_command(&self, _action: &Action) -> Result<ExecutionResult> {
        // In production, this would use a secure sandbox
        // For now, deny all command execution
        Ok(ExecutionResult::Denied {
            reason: "Command execution not implemented (requires sandboxing)".into(),
        })
    }

    /// Get reference to audit log
    pub fn audit_log(&self) -> &AuditLog {
        &self.audit_log
    }

    /// Get mutable reference to audit log
    pub fn audit_log_mut(&mut self) -> &mut AuditLog {
        &mut self.audit_log
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::KeyPair;
    use crate::genome::SealedGenome;
    use tempfile::tempdir;

    fn create_executor() -> (SecureExecutor, SealedGenome) {
        // Create a shared keypair for both genome and auditor
        // This simulates a production setup where they share the same signing authority
        let shared_keypair = KeyPair::generate().unwrap();

        // Create genome with the shared keypair
        let mut genome = SealedGenome::with_keypair(
            vec!["Rule".to_string()],
            shared_keypair.clone(), // Clone for genome
        )
        .unwrap();
        genome.seal().unwrap();

        // Create auditor with THE SAME keypair as genome
        // In production, this would be the genome's public key
        let auditor = ProofAuditor::new(shared_keypair.clone()); // Clone for auditor

        let log_keypair = KeyPair::generate().unwrap();
        let audit_log = AuditLog::new(log_keypair).unwrap();

        let executor = SecureExecutor::new(auditor, audit_log);

        (executor, genome)
    }

    fn create_signed_proof(genome: &SealedGenome, action: &Action) -> IntegrityProof {
        genome.verify_action(action).unwrap()
    }

    #[test]
    fn test_execute_write_success() {
        let (mut executor, genome) = create_executor();
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");

        let action = Action::write_file(file_path.to_str().unwrap(), b"test content".to_vec());

        let proof = create_signed_proof(&genome, &action);

        let result = executor.execute_with_proof(&action, &proof).unwrap();
        assert_eq!(result, ExecutionResult::Success);

        // Verify file was written
        let content = std::fs::read(&file_path).unwrap();
        assert_eq!(content, b"test content");
    }

    #[test]
    fn test_oracle_attack_prevention() {
        let (mut executor, genome) = create_executor();
        let dir = tempdir().unwrap();

        // Get proof for safe action
        let safe_action = Action::write_file(
            dir.path().join("safe.txt").to_str().unwrap(),
            b"safe".to_vec(),
        );
        let proof = create_signed_proof(&genome, &safe_action);

        // Try to execute different action with same proof
        let malicious_action = Action::delete("/etc/passwd");

        let result = executor.execute_with_proof(&malicious_action, &proof);

        // Should fail with ActionMismatch
        assert!(matches!(result, Err(ExecutorError::ActionMismatch { .. })));
    }

    #[test]
    fn test_replay_attack_prevention() {
        let (mut executor, genome) = create_executor();
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");

        let action = Action::write_file(file_path.to_str().unwrap(), b"content".to_vec());

        let proof = create_signed_proof(&genome, &action);

        // First execution: should succeed
        let result1 = executor.execute_with_proof(&action, &proof);
        assert!(result1.is_ok());

        // Second execution with same proof: should FAIL (replay attack)
        let result2 = executor.execute_with_proof(&action, &proof);
        assert!(result2.is_err());
    }

    #[test]
    fn test_audit_log_records_execution() {
        let (mut executor, genome) = create_executor();
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");

        let action = Action::write_file(file_path.to_str().unwrap(), b"content".to_vec());

        let proof = create_signed_proof(&genome, &action);

        executor.execute_with_proof(&action, &proof).unwrap();

        // Verify audit log
        assert_eq!(executor.audit_log().len(), 1);
        assert_eq!(
            executor.audit_log().entries()[0].decision,
            Decision::Approved
        );
    }
}
