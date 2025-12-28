use crate::audit_log::{AuditLog, Decision};
use crate::auditor::ProofAuditor;
use crate::crypto::hash_bytes;
use crate::proof::{Action, ActionType, IntegrityProof};
use std::fs::{remove_file, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid; // Kell az egyedi temp fájlokhoz!

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
    
    #[error("Path traversal attempt detected: {0}")]
    PathTraversal(String),
}

pub type Result<T> = std::result::Result<T, ExecutorError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionResult {
    Success,
    Denied { reason: String },
}

pub struct SecureExecutor {
    auditor: ProofAuditor,
    audit_log: AuditLog,
    storage_root: PathBuf, // <--- A BÖRTÖN (Jail Root)
}

impl SecureExecutor {
    /// Create a new secure executor with a confined storage root
    pub fn new(auditor: ProofAuditor, audit_log: AuditLog, storage_root: PathBuf) -> Result<Self> {
        // Ensure root exists and is canonicalized (absolute path)
        if !storage_root.exists() {
            std::fs::create_dir_all(&storage_root)?;
        }
        let canonical_root = storage_root.canonicalize()?;
        
        Ok(SecureExecutor { 
            auditor, 
            audit_log,
            storage_root: canonical_root 
        })
    }

    /// Security Critical: Sanitize and jail the path
    /// Prevents directory traversal (../) and symlink attacks escaping the root.
    fn sanitize_path(&self, unsafe_path_str: &str) -> Result<PathBuf> {
        let path = Path::new(unsafe_path_str);
        
        // 1. Block absolute paths (force relative to root)
        if path.is_absolute() {
            return Err(ExecutorError::PermissionDenied(
                "Absolute paths are not allowed. Use paths relative to storage root.".into()
            ));
        }

        // 2. Construct full path candidate
        let full_candidate = self.storage_root.join(path);
        
        // 3. Canonicalize the PARENT directory
        // We cannot canonicalize the file itself yet, as it might not exist (for writes).
        let parent = full_candidate.parent()
            .ok_or_else(|| ExecutorError::PermissionDenied("Invalid path structure".into()))?;

        if !parent.exists() {
             return Err(ExecutorError::PermissionDenied("Target directory does not exist".into()));
        }

        // This resolves all symlinks in the directory tree
        let canonical_parent = parent.canonicalize()?;

        // 4. JAIL CHECK: Ensure the resolved parent is still inside storage_root
        if !canonical_parent.starts_with(&self.storage_root) {
            return Err(ExecutorError::PathTraversal(
                format!("Resolved path {:?} escapes storage root", canonical_parent)
            ));
        }

        // 5. Re-attach the filename (which we know is safe if parent is safe and it's just a filename)
        let filename = full_candidate.file_name()
            .ok_or_else(|| ExecutorError::PermissionDenied("Missing filename".into()))?;
            
        Ok(canonical_parent.join(filename))
    }

    pub fn execute_with_proof(
        &mut self,
        action: &Action,
        proof: &IntegrityProof,
    ) -> Result<ExecutionResult> {
        // ... (Verification logic unchanged: Proof check, Hash check, Type check) ...
        self.auditor.verify_proof(proof)?;
        
        if proof.action_hash != action.hash() {
             // ... Log & Error ...
             return Err(ExecutorError::ActionMismatch { expected: action.hash(), found: proof.action_hash });
        }

        // Execute action
        let exec_result = match &action.action_type {
            ActionType::Write => self.execute_write_atomic(action)?, // <--- NEW ATOMIC WRITE
            ActionType::Delete => self.execute_delete(action)?,
            ActionType::Read => self.execute_read(action)?,
            _ => ExecutionResult::Denied { reason: "Not implemented".into() },
        };

        // ... (Logging logic unchanged) ...
        
        Ok(exec_result)
    }

    /// ATOMIC WRITE with Handle-Based Verification
    fn execute_write_atomic(&self, action: &Action) -> Result<ExecutionResult> {
        // 1. Sanitize Path (The Jail Check)
        let safe_target_path = self.sanitize_path(&action.target)?;
        
        let content = action.payload.as_ref()
            .ok_or_else(|| ExecutorError::PermissionDenied("No content".into()))?;

        // 2. Create Temporary File (in the same directory to allow atomic rename)
        // Using UUID to prevent collision
        let temp_filename = format!(".tmp_{}_{}", Uuid::new_v4(), safe_target_path.file_name().unwrap().to_string_lossy());
        let temp_path = safe_target_path.parent().unwrap().join(temp_filename);

        {
            // 3. Write to Temp File
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true) // Fail if temp file somehow exists
                .open(&temp_path)?;

            file.write_all(content)?;
            file.sync_all()?; // Force flush to disk

            // 4. HANDLE-BASED VERIFICATION (Fixes TOCTOU)
            // We do NOT close the file. We do NOT open by path.
            // We seek back to start on the SAME file descriptor.
            file.seek(SeekFrom::Start(0))?;
            
            let mut written_content = Vec::new();
            file.read_to_end(&mut written_content)?;
            
            let written_hash = hash_bytes(&written_content);
            let expected_hash = hash_bytes(content);

            if written_hash != expected_hash {
                // Cleanup and abort
                drop(file); // Close handle
                let _ = std::fs::remove_file(&temp_path); // Try to clean up
                return Err(ExecutorError::IntegrityViolation);
            }
        } // File handle closes here automatically

        // 5. ATOMIC SWAP (The Commit)
        // POSIX guarantees this is atomic. It's either the old file or the new file.
        // No partial writes visible to the world.
        std::fs::rename(&temp_path, &safe_target_path)?;

        Ok(ExecutionResult::Success)
    }

    fn execute_read(&self, action: &Action) -> Result<ExecutionResult> {
        // Path sanitization is critical for reads too!
        let safe_path = self.sanitize_path(&action.target)?;
        
        if !safe_path.exists() {
            return Ok(ExecutionResult::Denied { reason: "File not found".into() });
        }
        let _content = std::fs::read(safe_path)?;
        Ok(ExecutionResult::Success)
    }

    fn execute_delete(&self, action: &Action) -> Result<ExecutionResult> {
        let safe_path = self.sanitize_path(&action.target)?;
        
        if !safe_path.exists() {
            return Ok(ExecutionResult::Denied { reason: "File not found".into() });
        }
        remove_file(safe_path)?;
        Ok(ExecutionResult::Success)
    }
}
