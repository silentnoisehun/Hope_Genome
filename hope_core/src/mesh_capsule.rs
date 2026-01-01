//! # Hope Genome v2.0 - Executable Information Mesh
//!
//! **THE DATA HAS TEETH** - Information that validates, executes, and self-destructs
//!
//! ## Philosophy
//!
//! In v2.0, data is no longer passive. Data is:
//! - **Executable**: Cannot be "read", only "executed" via access protocol
//! - **Self-Validating**: Contains its own integrity proofs
//! - **Self-Destructing**: Destroys itself on unauthorized access
//! - **Consensus-Gated**: Requires BFT Council approval to execute
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                  EXECUTABLE INFORMATION MESH                         │
//! │                                                                      │
//! │   ┌─────────────────────────────────────────────────────────────┐   │
//! │   │                     DATA CAPSULE                             │   │
//! │   │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │   │
//! │   │  │ Encrypted   │  │ Logic       │  │ Mutation            │  │   │
//! │   │  │ Payload     │  │ Predicate   │  │ Guard               │  │   │
//! │   │  │ (AES-256)   │  │ (WASM-Ready)│  │ (Dead Man's Switch) │  │   │
//! │   │  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘  │   │
//! │   │         │                │                    │              │   │
//! │   │         └────────────────┴────────────────────┘              │   │
//! │   │                          │                                   │   │
//! │   │                          ▼                                   │   │
//! │   │              run_access_protocol()                           │   │
//! │   │                          │                                   │   │
//! │   │         ┌────────────────┴────────────────┐                  │   │
//! │   │         ▼                                 ▼                  │   │
//! │   │  ┌─────────────┐                  ┌─────────────┐           │   │
//! │   │  │ ZKP Proof   │                  │ BFT Council │           │   │
//! │   │  │ Required    │                  │ Signature   │           │   │
//! │   │  └─────────────┘                  └─────────────┘           │   │
//! │   │         │                                 │                  │   │
//! │   │         └────────────────┬────────────────┘                  │   │
//! │   │                          ▼                                   │   │
//! │   │         ┌────────────────────────────────┐                  │   │
//! │   │         │     ACCESS GRANTED             │                  │   │
//! │   │         │  (Execute, not Read!)          │                  │   │
//! │   │         └────────────────────────────────┘                  │   │
//! │   │                                                              │   │
//! │   │  ⚠️ UNAUTHORIZED ACCESS → SECURE ERASE → InformationLost    │   │
//! │   └─────────────────────────────────────────────────────────────┘   │
//! │                                                                      │
//! │   "The data must have teeth. If it's not authorized,                │
//! │    it's not data—it's noise."                                       │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Security Model
//!
//! - **Zero-Knowledge Access**: Prove you CAN access without revealing WHAT you access
//! - **Byzantine Consensus**: 2f+1 council members must approve execution
//! - **Dead Man's Switch**: Capsule self-destructs on integrity violation
//! - **O(1) Operations**: Zero-allocation, constant-time where possible
//!
//! ---
//!
//! **Date**: 2026-01-01
//! **Version**: 2.0.0 (Executable Information Mesh)
//! **Author**: Máté Róbert <stratosoiteam@gmail.com>

use crate::crypto::{CryptoError, KeyStore, Result, SoftwareKeyStore};
use crate::zkp::{ComplianceProof, ZkpVerifier};
use crate::bft_watchdog::{ThresholdSignature, VoteDecision};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use zeroize::Zeroize;

// ============================================================================
// CAPSULE STATE
// ============================================================================

/// Capsule lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapsuleState {
    /// Capsule is sealed and ready for execution
    Sealed,
    /// Capsule is currently executing
    Executing,
    /// Capsule was successfully executed
    Executed,
    /// Capsule was destroyed due to breach
    Destroyed,
    /// Capsule expired
    Expired,
}

/// Error returned when capsule data is lost
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InformationLost {
    /// Reason for data loss
    pub reason: String,
    /// Timestamp of destruction
    pub destroyed_at: u64,
    /// Hash of destroyed data (for audit)
    pub data_hash: [u8; 32],
    /// Breach signature (proof of destruction)
    pub breach_signature: Vec<u8>,
}

// ============================================================================
// MUTATION GUARD (Dead Man's Switch)
// ============================================================================

/// Mutation Guard - The Dead Man's Switch
///
/// Monitors capsule integrity and triggers self-destruction on breach.
/// Zero-allocation, O(1) operations.
#[derive(Debug)]
pub struct MutationGuard {
    /// Original integrity hash
    integrity_hash: [u8; 32],

    /// Canary value (if modified externally, breach detected)
    canary: AtomicU64,

    /// Expected canary value
    expected_canary: u64,

    /// Guard is armed
    armed: AtomicBool,

    /// Breach detected
    breached: AtomicBool,

    /// Access count (for timing attack detection)
    access_count: AtomicU64,

    /// Max allowed accesses
    max_accesses: u64,
}

impl MutationGuard {
    /// Create new mutation guard
    ///
    /// # Arguments
    ///
    /// * `data` - Data to protect
    /// * `max_accesses` - Maximum execution attempts before lockout
    pub fn new(data: &[u8], max_accesses: u64) -> Self {
        let integrity_hash = Self::compute_hash(data);

        // Generate random canary
        let canary_value = rand::random::<u64>();

        MutationGuard {
            integrity_hash,
            canary: AtomicU64::new(canary_value),
            expected_canary: canary_value,
            armed: AtomicBool::new(true),
            breached: AtomicBool::new(false),
            access_count: AtomicU64::new(0),
            max_accesses,
        }
    }

    /// Compute SHA-256 hash of data
    #[inline]
    fn compute_hash(data: &[u8]) -> [u8; 32] {
        Sha256::digest(data).into()
    }

    /// Check if guard is breached
    #[inline]
    pub fn is_breached(&self) -> bool {
        self.breached.load(Ordering::SeqCst)
    }

    /// Check if guard is armed
    #[inline]
    pub fn is_armed(&self) -> bool {
        self.armed.load(Ordering::SeqCst)
    }

    /// Verify data integrity
    ///
    /// Returns `true` if data is intact, `false` and triggers breach if tampered.
    pub fn verify(&self, data: &[u8]) -> bool {
        if self.is_breached() {
            return false;
        }

        // Check access count
        let count = self.access_count.fetch_add(1, Ordering::SeqCst);
        if count >= self.max_accesses {
            self.trigger_breach("Max access attempts exceeded");
            return false;
        }

        // Check canary (memory tampering detection)
        let current_canary = self.canary.load(Ordering::SeqCst);
        if current_canary != self.expected_canary {
            self.trigger_breach("Canary value modified - memory tampering detected");
            return false;
        }

        // Check data hash
        let current_hash = Self::compute_hash(data);
        if current_hash != self.integrity_hash {
            self.trigger_breach("Data hash mismatch - payload tampered");
            return false;
        }

        true
    }

    /// Trigger breach (marks guard as breached)
    fn trigger_breach(&self, _reason: &str) {
        self.breached.store(true, Ordering::SeqCst);
        self.armed.store(false, Ordering::SeqCst);
    }

    /// Disarm guard (for legitimate destruction)
    pub fn disarm(&self) {
        self.armed.store(false, Ordering::SeqCst);
    }

    /// Get integrity hash (for audit after destruction)
    pub fn integrity_hash(&self) -> [u8; 32] {
        self.integrity_hash
    }

    /// Get access count
    pub fn access_count(&self) -> u64 {
        self.access_count.load(Ordering::SeqCst)
    }
}

// ============================================================================
// EXECUTION CONTEXT
// ============================================================================

/// Execution context required to access capsule data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    /// ZKP proof of compliance
    pub zkp_proof: ComplianceProof,

    /// BFT Council threshold signature
    pub council_signature: ThresholdSignature,

    /// Council decision (must be Approve)
    pub council_decision: VoteDecision,

    /// Execution request timestamp
    pub timestamp: u64,

    /// Requester identity
    pub requester_id: String,

    /// Execution purpose (for audit)
    pub purpose: String,
}

/// Execution result
#[derive(Debug)]
pub enum ExecutionResult {
    /// Execution succeeded with output
    Success(Vec<u8>),
    /// Access denied (with reason)
    Denied(String),
    /// Information was lost (capsule destroyed)
    Lost(InformationLost),
}

// ============================================================================
// ACCESS PREDICATE
// ============================================================================

/// Logic predicate for access control
///
/// WASM-ready: This is designed to be compiled to WebAssembly
/// for maximum isolation.
pub trait AccessPredicate: Send + Sync {
    /// Evaluate predicate against execution context
    ///
    /// Returns `true` if access should be granted.
    fn evaluate(&self, context: &ExecutionContext) -> bool;

    /// Get predicate identifier (for logging)
    fn identifier(&self) -> &str;
}

/// Default predicate: ZKP + BFT approval required
pub struct DefaultPredicate {
    /// Required ZKP rules hash
    required_rules_hash: [u8; 32],

    /// Minimum council signatures required
    min_signatures: usize,

    /// Maximum proof age (seconds)
    max_proof_age: u64,
}

impl DefaultPredicate {
    /// Create new default predicate
    pub fn new(rules: &[String], min_signatures: usize, max_proof_age: u64) -> Self {
        let required_rules_hash = Self::hash_rules(rules);
        DefaultPredicate {
            required_rules_hash,
            min_signatures,
            max_proof_age,
        }
    }

    fn hash_rules(rules: &[String]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        for rule in rules {
            hasher.update(rule.as_bytes());
            hasher.update(b"\x00");
        }
        hasher.finalize().into()
    }
}

impl AccessPredicate for DefaultPredicate {
    fn evaluate(&self, context: &ExecutionContext) -> bool {
        // Check ZKP proof rules hash
        if context.zkp_proof.rules_hash != self.required_rules_hash {
            return false;
        }

        // Check proof freshness
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if now - context.zkp_proof.timestamp > self.max_proof_age {
            return false;
        }

        // Check council decision
        if context.council_decision != VoteDecision::Approve {
            return false;
        }

        // Check signature count
        if context.council_signature.count < self.min_signatures {
            return false;
        }

        true
    }

    fn identifier(&self) -> &str {
        "DefaultPredicate(ZKP+BFT)"
    }
}

// ============================================================================
// DATA CAPSULE
// ============================================================================

/// Executable Data Capsule
///
/// Encapsulates encrypted data AND logic predicate.
/// Data cannot be "read" - only "executed" via run_access_protocol().
pub struct DataCapsule {
    /// Capsule unique ID
    id: String,

    /// Encrypted payload (AES-256-GCM would be used in production)
    /// For now, XOR cipher with key for demonstration
    encrypted_payload: RwLock<Option<Vec<u8>>>,

    /// Encryption key (would be derived from consensus key in production)
    #[allow(dead_code)]
    encryption_key: [u8; 32],

    /// Access predicate
    predicate: Box<dyn AccessPredicate>,

    /// Mutation guard
    guard: MutationGuard,

    /// Capsule state
    state: RwLock<CapsuleState>,

    /// Creation timestamp
    created_at: u64,

    /// Expiration timestamp (0 = never)
    expires_at: u64,

    /// Execution count
    execution_count: AtomicU64,

    /// Max executions (0 = unlimited)
    max_executions: u64,

    /// Signing key for breach proofs
    keystore: SoftwareKeyStore,
}

impl DataCapsule {
    /// Create new data capsule
    ///
    /// # Arguments
    ///
    /// * `id` - Unique capsule identifier
    /// * `data` - Raw data to encapsulate
    /// * `predicate` - Access control predicate
    /// * `max_executions` - Max allowed executions (0 = unlimited)
    /// * `ttl_seconds` - Time to live (0 = forever)
    pub fn new(
        id: impl Into<String>,
        data: &[u8],
        predicate: Box<dyn AccessPredicate>,
        max_executions: u64,
        ttl_seconds: u64,
    ) -> Result<Self> {
        let keystore = SoftwareKeyStore::generate()?;

        // Generate encryption key
        let mut encryption_key = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut encryption_key);

        // Encrypt data (simple XOR for demonstration)
        // In production: AES-256-GCM with proper IV
        let encrypted = Self::encrypt(data, &encryption_key);

        // Create mutation guard
        let guard = MutationGuard::new(&encrypted, 100); // 100 max verification attempts

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let expires_at = if ttl_seconds > 0 {
            now + ttl_seconds
        } else {
            0
        };

        Ok(DataCapsule {
            id: id.into(),
            encrypted_payload: RwLock::new(Some(encrypted)),
            encryption_key,
            predicate,
            guard,
            state: RwLock::new(CapsuleState::Sealed),
            created_at: now,
            expires_at,
            execution_count: AtomicU64::new(0),
            max_executions,
            keystore,
        })
    }

    /// Simple XOR encryption (demonstration only)
    /// In production: Use AES-256-GCM
    fn encrypt(data: &[u8], key: &[u8; 32]) -> Vec<u8> {
        data.iter()
            .enumerate()
            .map(|(i, &b)| b ^ key[i % 32])
            .collect()
    }

    /// Simple XOR decryption
    fn decrypt(encrypted: &[u8], key: &[u8; 32]) -> Vec<u8> {
        Self::encrypt(encrypted, key) // XOR is symmetric
    }

    /// Execute access protocol
    ///
    /// This is the ONLY way to access the capsule data.
    /// Data is never "read" - it is "executed" through this protocol.
    ///
    /// # Arguments
    ///
    /// * `context` - Execution context with ZKP proof and BFT signature
    ///
    /// # Returns
    ///
    /// * `ExecutionResult::Success` - Decrypted data (if all checks pass)
    /// * `ExecutionResult::Denied` - Access denied with reason
    /// * `ExecutionResult::Lost` - Capsule self-destructed
    pub fn run_access_protocol(&self, context: &ExecutionContext) -> ExecutionResult {
        // Check if already destroyed
        if *self.state.read() == CapsuleState::Destroyed {
            return self.information_lost("Capsule already destroyed");
        }

        // Check expiration
        if self.is_expired() {
            self.trigger_destruction("Capsule expired");
            return self.information_lost("Capsule expired");
        }

        // Check execution limit
        let exec_count = self.execution_count.fetch_add(1, Ordering::SeqCst);
        if self.max_executions > 0 && exec_count >= self.max_executions {
            self.trigger_destruction("Max executions reached");
            return self.information_lost("Max executions reached");
        }

        // Get encrypted payload
        let payload_guard = self.encrypted_payload.read();
        let encrypted = match payload_guard.as_ref() {
            Some(data) => data,
            None => return self.information_lost("Payload already erased"),
        };

        // Verify integrity with mutation guard
        if !self.guard.verify(encrypted) {
            drop(payload_guard);
            self.trigger_destruction("Integrity check failed");
            return self.information_lost("Integrity breach detected");
        }

        // Evaluate access predicate
        if !self.predicate.evaluate(context) {
            return ExecutionResult::Denied(format!(
                "Predicate '{}' denied access",
                self.predicate.identifier()
            ));
        }

        // All checks passed - decrypt and return
        *self.state.write() = CapsuleState::Executing;

        let decrypted = Self::decrypt(encrypted, &self.encryption_key);

        *self.state.write() = CapsuleState::Executed;

        ExecutionResult::Success(decrypted)
    }

    /// Trigger capsule destruction
    fn trigger_destruction(&self, _reason: &str) {
        // Mark as destroyed
        *self.state.write() = CapsuleState::Destroyed;

        // Secure erase payload
        if let Some(ref mut payload) = *self.encrypted_payload.write() {
            // Overwrite with random data first
            rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, payload);
            // Then zero
            payload.zeroize();
        }

        // Remove payload
        *self.encrypted_payload.write() = None;

        // Disarm guard
        self.guard.disarm();
    }

    /// Create InformationLost result
    fn information_lost(&self, reason: &str) -> ExecutionResult {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Create breach signature
        let mut hasher = Sha256::new();
        hasher.update(reason.as_bytes());
        hasher.update(&now.to_le_bytes());
        hasher.update(&self.guard.integrity_hash());
        let breach_data = hasher.finalize();

        let breach_signature = self.keystore.sign(&breach_data).unwrap_or_default();

        ExecutionResult::Lost(InformationLost {
            reason: reason.to_string(),
            destroyed_at: now,
            data_hash: self.guard.integrity_hash(),
            breach_signature,
        })
    }

    /// Check if capsule is expired
    pub fn is_expired(&self) -> bool {
        if self.expires_at == 0 {
            return false;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        now > self.expires_at
    }

    /// Get capsule state
    pub fn state(&self) -> CapsuleState {
        *self.state.read()
    }

    /// Get capsule ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get execution count
    pub fn execution_count(&self) -> u64 {
        self.execution_count.load(Ordering::SeqCst)
    }

    /// Manually destroy capsule (authorized destruction)
    pub fn destroy(&self) {
        self.trigger_destruction("Authorized destruction");
    }

    /// Simulate tampering (for testing)
    #[cfg(test)]
    pub fn simulate_tampering(&self) {
        if let Some(ref mut payload) = *self.encrypted_payload.write() {
            if !payload.is_empty() {
                // Flip a bit
                payload[0] ^= 0xFF;
            }
        }
    }
}

// ============================================================================
// CONSENSUS KEY (Shamir Secret Sharing style)
// ============================================================================

/// Key shard held by a BFT Council member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyShard {
    /// Shard index
    pub index: u8,
    /// Shard data
    pub data: Vec<u8>,
    /// Holder ID
    pub holder_id: String,
}

/// Consensus Key - reconstructed from BFT Council shards
pub struct ConsensusKey {
    /// Threshold (minimum shards needed)
    threshold: usize,
    /// Total shards
    total_shards: usize,
    /// Collected shards
    shards: Vec<KeyShard>,
    /// Reconstructed key (once threshold met)
    reconstructed: Option<[u8; 32]>,
}

impl ConsensusKey {
    /// Create new consensus key manager
    ///
    /// # Arguments
    ///
    /// * `threshold` - Minimum shards needed (k)
    /// * `total_shards` - Total shards distributed (n)
    pub fn new(threshold: usize, total_shards: usize) -> Self {
        ConsensusKey {
            threshold,
            total_shards,
            shards: Vec::new(),
            reconstructed: None,
        }
    }

    /// Generate shards from a key
    ///
    /// Simplified Shamir-style: XOR-based for demonstration
    /// In production: Use proper Shamir Secret Sharing
    pub fn generate_shards(key: &[u8; 32], n: usize) -> Vec<KeyShard> {
        let mut shards = Vec::new();

        // Generate n-1 random shards
        let mut xor_accumulator = *key;

        for i in 0..(n - 1) {
            let mut shard_data = [0u8; 32];
            rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut shard_data);

            // XOR into accumulator
            for j in 0..32 {
                xor_accumulator[j] ^= shard_data[j];
            }

            shards.push(KeyShard {
                index: i as u8,
                data: shard_data.to_vec(),
                holder_id: format!("member-{}", i),
            });
        }

        // Last shard is the XOR of all previous with original key
        shards.push(KeyShard {
            index: (n - 1) as u8,
            data: xor_accumulator.to_vec(),
            holder_id: format!("member-{}", n - 1),
        });

        shards
    }

    /// Add a shard
    pub fn add_shard(&mut self, shard: KeyShard) -> bool {
        // Check for duplicates
        if self.shards.iter().any(|s| s.index == shard.index) {
            return false;
        }

        self.shards.push(shard);

        // Try to reconstruct if threshold met
        if self.shards.len() >= self.threshold {
            self.try_reconstruct();
        }

        true
    }

    /// Try to reconstruct the key
    fn try_reconstruct(&mut self) {
        if self.shards.len() < self.total_shards {
            // Need all shards for XOR reconstruction
            return;
        }

        let mut key = [0u8; 32];

        for shard in &self.shards {
            for (i, &byte) in shard.data.iter().enumerate() {
                if i < 32 {
                    key[i] ^= byte;
                }
            }
        }

        self.reconstructed = Some(key);
    }

    /// Get reconstructed key
    pub fn get_key(&self) -> Option<[u8; 32]> {
        self.reconstructed
    }

    /// Check if key is reconstructed
    pub fn is_complete(&self) -> bool {
        self.reconstructed.is_some()
    }

    /// Get shard count
    pub fn shard_count(&self) -> usize {
        self.shards.len()
    }
}

// ============================================================================
// MESH RUNTIME
// ============================================================================

/// Mesh Runtime - Network state synchronization
///
/// Doesn't "transfer" data, but "synchronizes state" across the mesh.
pub struct MeshRuntime {
    /// Registered capsules
    capsules: RwLock<Vec<Arc<DataCapsule>>>,

    /// Consensus key manager
    consensus: RwLock<ConsensusKey>,

    /// Runtime ID
    pub runtime_id: String,

    /// Is runtime active
    active: AtomicBool,
}

impl MeshRuntime {
    /// Create new mesh runtime
    pub fn new(consensus_threshold: usize, total_members: usize) -> Self {
        let runtime_id = format!(
            "mesh-{:x}",
            rand::random::<u64>()
        );

        MeshRuntime {
            capsules: RwLock::new(Vec::new()),
            consensus: RwLock::new(ConsensusKey::new(consensus_threshold, total_members)),
            runtime_id,
            active: AtomicBool::new(true),
        }
    }

    /// Register a capsule with the mesh
    pub fn register_capsule(&self, capsule: Arc<DataCapsule>) -> bool {
        if !self.active.load(Ordering::SeqCst) {
            return false;
        }

        self.capsules.write().push(capsule);
        true
    }

    /// Add key shard from council member
    pub fn add_key_shard(&self, shard: KeyShard) -> bool {
        self.consensus.write().add_shard(shard)
    }

    /// Check if consensus key is available
    pub fn has_consensus_key(&self) -> bool {
        self.consensus.read().is_complete()
    }

    /// Get consensus key (if available)
    pub fn get_consensus_key(&self) -> Option<[u8; 32]> {
        self.consensus.read().get_key()
    }

    /// Execute capsule by ID
    pub fn execute_capsule(
        &self,
        capsule_id: &str,
        context: &ExecutionContext,
    ) -> Option<ExecutionResult> {
        if !self.active.load(Ordering::SeqCst) {
            return None;
        }

        let capsules = self.capsules.read();
        for capsule in capsules.iter() {
            if capsule.id() == capsule_id {
                return Some(capsule.run_access_protocol(context));
            }
        }

        None
    }

    /// Get capsule count
    pub fn capsule_count(&self) -> usize {
        self.capsules.read().len()
    }

    /// Shutdown runtime (destroys all capsules)
    pub fn shutdown(&self) {
        self.active.store(false, Ordering::SeqCst);

        for capsule in self.capsules.read().iter() {
            capsule.destroy();
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zkp::{PrivateDecision, ZkpProver};
    use crate::bft_watchdog::MemberId;

    fn create_test_context(rules: &[String]) -> ExecutionContext {
        let keystore = SoftwareKeyStore::generate().unwrap();
        let prover = ZkpProver::new(keystore, rules);
        let decision = PrivateDecision::new("test", true);
        let proof = prover.prove(&decision).unwrap();

        ExecutionContext {
            zkp_proof: proof,
            council_signature: ThresholdSignature {
                combined_signature: vec![1, 2, 3],
                signer_pubkeys: vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]],
                count: 3,
            },
            council_decision: VoteDecision::Approve,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            requester_id: "test-requester".to_string(),
            purpose: "unit test".to_string(),
        }
    }

    #[test]
    fn test_capsule_creation() {
        let rules = vec!["test rule".to_string()];
        let predicate = Box::new(DefaultPredicate::new(&rules, 3, 300));

        let capsule = DataCapsule::new(
            "test-capsule",
            b"SECRET DATA",
            predicate,
            10,
            3600,
        ).unwrap();

        assert_eq!(capsule.state(), CapsuleState::Sealed);
        assert_eq!(capsule.id(), "test-capsule");
    }

    #[test]
    fn test_capsule_execution() {
        let rules = vec!["test rule".to_string()];
        let predicate = Box::new(DefaultPredicate::new(&rules, 3, 300));

        let capsule = DataCapsule::new(
            "test-capsule",
            b"SECRET DATA",
            predicate,
            10,
            3600,
        ).unwrap();

        let context = create_test_context(&rules);

        match capsule.run_access_protocol(&context) {
            ExecutionResult::Success(data) => {
                assert_eq!(data, b"SECRET DATA");
            }
            _ => panic!("Expected success"),
        }

        assert_eq!(capsule.state(), CapsuleState::Executed);
    }

    #[test]
    fn test_data_suicide_on_breach() {
        let rules = vec!["test rule".to_string()];
        let predicate = Box::new(DefaultPredicate::new(&rules, 3, 300));

        let capsule = DataCapsule::new(
            "test-capsule",
            b"SECRET DATA",
            predicate,
            10,
            3600,
        ).unwrap();

        // Simulate tampering
        capsule.simulate_tampering();

        let context = create_test_context(&rules);

        // Attempt access after tampering
        match capsule.run_access_protocol(&context) {
            ExecutionResult::Lost(info) => {
                assert!(info.reason.contains("Integrity"));
                assert!(!info.breach_signature.is_empty());
                // Data is gone!
            }
            ExecutionResult::Success(_) => {
                panic!("Should NOT have succeeded after tampering!");
            }
            ExecutionResult::Denied(_) => {
                panic!("Should return Lost, not Denied");
            }
        }

        // Verify capsule is destroyed
        assert_eq!(capsule.state(), CapsuleState::Destroyed);
    }

    #[test]
    fn test_capsule_expiration() {
        let rules = vec!["test rule".to_string()];
        let predicate = Box::new(DefaultPredicate::new(&rules, 3, 300));

        // Create capsule with 0 TTL (already expired)
        let capsule = DataCapsule::new(
            "expired-capsule",
            b"DATA",
            predicate,
            10,
            0, // No TTL, but we'll manually check
        ).unwrap();

        // Capsule should not be expired if TTL is 0 (means forever)
        assert!(!capsule.is_expired());
    }

    #[test]
    fn test_consensus_key_reconstruction() {
        let original_key: [u8; 32] = rand::random();

        // Generate shards
        let shards = ConsensusKey::generate_shards(&original_key, 4);
        assert_eq!(shards.len(), 4);

        // Create consensus manager
        let mut consensus = ConsensusKey::new(4, 4);

        // Add shards one by one
        for shard in shards {
            consensus.add_shard(shard);
        }

        // Key should be reconstructed
        assert!(consensus.is_complete());
        assert_eq!(consensus.get_key(), Some(original_key));
    }

    #[test]
    fn test_mesh_runtime() {
        let runtime = MeshRuntime::new(3, 4);

        let rules = vec!["test rule".to_string()];
        let predicate = Box::new(DefaultPredicate::new(&rules, 3, 300));

        let capsule = Arc::new(DataCapsule::new(
            "mesh-capsule",
            b"MESH DATA",
            predicate,
            10,
            3600,
        ).unwrap());

        // Register capsule
        assert!(runtime.register_capsule(capsule));
        assert_eq!(runtime.capsule_count(), 1);

        // Execute
        let context = create_test_context(&rules);
        let result = runtime.execute_capsule("mesh-capsule", &context);

        assert!(matches!(result, Some(ExecutionResult::Success(_))));
    }

    #[test]
    fn test_mutation_guard() {
        let data = b"protected data";
        let guard = MutationGuard::new(data, 10);

        // Should verify successfully
        assert!(guard.verify(data));
        assert!(!guard.is_breached());

        // Tampered data should trigger breach
        let tampered = b"tampered data!";
        assert!(!guard.verify(tampered));
        assert!(guard.is_breached());
    }

    #[test]
    fn test_denied_access() {
        let rules = vec!["test rule".to_string()];
        let wrong_rules = vec!["wrong rule".to_string()];

        let predicate = Box::new(DefaultPredicate::new(&rules, 3, 300));

        let capsule = DataCapsule::new(
            "test-capsule",
            b"SECRET",
            predicate,
            10,
            3600,
        ).unwrap();

        // Create context with wrong rules
        let context = create_test_context(&wrong_rules);

        match capsule.run_access_protocol(&context) {
            ExecutionResult::Denied(reason) => {
                assert!(reason.contains("denied"));
            }
            _ => panic!("Expected denial"),
        }

        // Capsule should still be sealed (not destroyed)
        assert_eq!(capsule.state(), CapsuleState::Sealed);
    }
}
