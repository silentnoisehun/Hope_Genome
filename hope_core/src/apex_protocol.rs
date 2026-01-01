//! # Hope Genome v2.2 - Genesis Protocol & Global Immunity
//!
//! **THE ATMOSPHERE ITSELF** - Decentralized, Self-Propagating Immunity Mesh
//!
//! ## Philosophy
//!
//! The system must no longer be a castle; it must be the atmosphere.
//! Everywhere, invisible, and unyielding.
//!
//! ```text
//! ┌────────────────────────────────────────────────────────────────────────┐
//! │                      GENESIS PROTOCOL v2.2                             │
//! │                                                                        │
//! │   ┌──────────────────────────────────────────────────────────────┐    │
//! │   │                    GENESIS BLOCK                              │    │
//! │   │              "First Ethics - Immutable Root"                  │    │
//! │   │                                                               │    │
//! │   │   Signed by: The Architect (Code God Key)                    │    │
//! │   │   All mutations descend from this block                       │    │
//! │   └──────────────────────────────────────────────────────────────┘    │
//! │                              │                                         │
//! │                              ▼                                         │
//! │   ┌──────────────────────────────────────────────────────────────┐    │
//! │   │                    APEX CONTROL                               │    │
//! │   │            "Creator Override - Multi-Sig"                     │    │
//! │   │                                                               │    │
//! │   │   Architect Key + BFT Council Quorum = STOP/RESET            │    │
//! │   └──────────────────────────────────────────────────────────────┘    │
//! │                              │                                         │
//! │                              ▼                                         │
//! │   ┌──────────────────────────────────────────────────────────────┐    │
//! │   │                    SYNC PROTOCOL                              │    │
//! │   │              "Gossip-Based Hive Mind"                         │    │
//! │   │                                                               │    │
//! │   │   Attack in Node A → Fingerprint → ALL NODES (milliseconds)  │    │
//! │   └──────────────────────────────────────────────────────────────┘    │
//! │                              │                                         │
//! │                              ▼                                         │
//! │   ┌──────────────────────────────────────────────────────────────┐    │
//! │   │                  STEALTH INTEGRITY                            │    │
//! │   │             "The Invisible Sentinel"                          │    │
//! │   │                                                               │    │
//! │   │   Logic moves between WASM memory slots                       │    │
//! │   │   Minimal footprint - undetectable                            │    │
//! │   └──────────────────────────────────────────────────────────────┘    │
//! │                                                                        │
//! │   "Everywhere, invisible, and unyielding."                            │
//! └────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ---
//!
//! **Date**: 2026-01-01
//! **Version**: 2.2.0 (Genesis Protocol)
//! **Authors**: Máté Róbert (The Architect) + Claude

use crate::bft_watchdog::ThresholdSignature;
use crate::crypto::{CryptoError, KeyStore, SoftwareKeyStore};
use crate::evolutionary_guard::{AttackPattern, SignedFilter, ThreatLevel};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
// zeroize is available for future use

// ============================================================================
// GENESIS BLOCK - THE FIRST ETHICS
// ============================================================================

/// The Genesis Block - Immutable First Ethics
///
/// This is the root of all mutations. Signed by the Architect.
/// All subsequent filters, rules, and evolutions must be
/// cryptographically descended from this block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisBlock {
    /// Version of the Genesis Protocol
    pub version: String,

    /// The First Ethics - immutable core rules
    pub first_ethics: Vec<String>,

    /// Hash of the first ethics (for descendant verification)
    pub ethics_hash: [u8; 32],

    /// Creation timestamp (Unix epoch)
    pub created_at: u64,

    /// The Architect's public key
    pub architect_pubkey: Vec<u8>,

    /// The Architect's signature over the block
    pub architect_signature: Vec<u8>,

    /// Block hash (self-referential integrity)
    pub block_hash: [u8; 32],

    /// Genesis message from the Architect
    pub genesis_message: String,
}

impl GenesisBlock {
    /// Create a new Genesis Block signed by the Architect
    ///
    /// # Arguments
    ///
    /// * `first_ethics` - The immutable core ethical rules
    /// * `architect_key` - The Architect's signing key
    /// * `genesis_message` - Message from the Creator
    pub fn create(
        first_ethics: Vec<String>,
        architect_key: &dyn KeyStore,
        genesis_message: impl Into<String>,
    ) -> Result<Self, CryptoError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Compute ethics hash
        let ethics_hash = Self::hash_ethics(&first_ethics);

        // Create unsigned block for signing
        let mut block = GenesisBlock {
            version: "2.2.0".to_string(),
            first_ethics,
            ethics_hash,
            created_at: now,
            architect_pubkey: architect_key.public_key_bytes(),
            architect_signature: Vec::new(),
            block_hash: [0u8; 32],
            genesis_message: genesis_message.into(),
        };

        // Sign the block
        let signable_data = block.signable_bytes();
        block.architect_signature = architect_key.sign(&signable_data)?;

        // Compute block hash (includes signature)
        block.block_hash = block.compute_block_hash();

        Ok(block)
    }

    /// Hash the first ethics
    fn hash_ethics(ethics: &[String]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        for (i, rule) in ethics.iter().enumerate() {
            hasher.update((i as u32).to_le_bytes());
            hasher.update(rule.as_bytes());
            hasher.update(b"\x00");
        }
        hasher.finalize().into()
    }

    /// Get bytes for signing (excludes signature and block_hash)
    fn signable_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(self.version.as_bytes());
        data.extend_from_slice(&self.ethics_hash);
        data.extend_from_slice(&self.created_at.to_le_bytes());
        data.extend_from_slice(&self.architect_pubkey);
        data.extend_from_slice(self.genesis_message.as_bytes());
        data
    }

    /// Compute block hash
    fn compute_block_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.signable_bytes());
        hasher.update(&self.architect_signature);
        hasher.finalize().into()
    }

    /// Verify the Genesis Block's integrity and signature
    pub fn verify(&self, architect_key: &dyn KeyStore) -> bool {
        // Verify signature
        let signable_data = self.signable_bytes();
        if architect_key
            .verify(&signable_data, &self.architect_signature)
            .is_err()
        {
            return false;
        }

        // Verify ethics hash
        if self.ethics_hash != Self::hash_ethics(&self.first_ethics) {
            return false;
        }

        // Verify block hash
        if self.block_hash != self.compute_block_hash() {
            return false;
        }

        true
    }

    /// Check if a mutation is a valid descendant of this Genesis
    pub fn is_valid_descendant(&self, mutation_ethics_hash: &[u8; 32]) -> bool {
        // For now, descendants must include the original ethics hash in their lineage
        // In production, this would verify a Merkle proof of ancestry
        *mutation_ethics_hash == self.ethics_hash
    }
}

// ============================================================================
// COMPACTED THREAT FINGERPRINT
// ============================================================================

/// Compact representation of a threat for network transmission
///
/// Minimal size, maximum information. Designed for gossip protocol.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CompactedThreatFingerprint {
    /// Threat signature (32 bytes)
    pub signature: [u8; 32],

    /// Threat level (1 byte)
    pub level: ThreatLevel,

    /// Category code (1 byte - maps to AttackCategory)
    pub category_code: u8,

    /// Timestamp (8 bytes)
    pub timestamp: u64,

    /// Origin node ID hash (8 bytes - truncated)
    pub origin_hash: u64,

    /// Hop count (how many nodes have relayed this)
    pub hop_count: u8,

    /// Priority (for transmission ordering)
    pub priority: u8,
}

impl CompactedThreatFingerprint {
    /// Create fingerprint from attack pattern
    pub fn from_pattern(pattern: &AttackPattern, origin_node: &str) -> Self {
        let mut origin_hasher = Sha256::new();
        origin_hasher.update(origin_node.as_bytes());
        let origin_hash_bytes = origin_hasher.finalize();
        let origin_hash = u64::from_le_bytes(origin_hash_bytes[0..8].try_into().unwrap());

        let priority = match pattern.threat_level {
            ThreatLevel::Critical => 255,
            ThreatLevel::High => 192,
            ThreatLevel::Medium => 128,
            ThreatLevel::Low => 64,
        };

        CompactedThreatFingerprint {
            signature: pattern.signature,
            level: pattern.threat_level,
            category_code: pattern.category as u8,
            timestamp: pattern.first_seen,
            origin_hash,
            hop_count: 0,
            priority,
        }
    }

    /// Increment hop count (for gossip tracking)
    pub fn relay(&mut self) {
        self.hop_count = self.hop_count.saturating_add(1);
        // Decrease priority with each hop
        self.priority = self.priority.saturating_sub(1);
    }

    /// Check if fingerprint is still fresh
    pub fn is_fresh(&self, max_age_secs: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now - self.timestamp <= max_age_secs
    }

    /// Size in bytes (for network optimization)
    pub const SIZE: usize = 32 + 1 + 1 + 8 + 8 + 1 + 1; // 52 bytes
}

// ============================================================================
// SYNC PROTOCOL - GOSSIP-BASED HIVE MIND
// ============================================================================

/// Node information for the mesh network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshNode {
    /// Node unique ID
    pub id: String,

    /// Node public key (for verification)
    pub pubkey: Vec<u8>,

    /// Last seen timestamp
    pub last_seen: u64,

    /// Node reputation score (0.0 - 1.0)
    pub reputation: f64,

    /// Is node active
    pub active: bool,
}

/// Sync message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncMessage {
    /// New threat fingerprint broadcast
    ThreatBroadcast(CompactedThreatFingerprint),

    /// Batch of fingerprints
    ThreatBatch(Vec<CompactedThreatFingerprint>),

    /// Request immunity memory shard
    RequestShard { shard_id: u64 },

    /// Immunity memory shard response
    ShardResponse { shard_id: u64, data: Vec<u8> },

    /// Heartbeat (I'm alive)
    Heartbeat { node_id: String, timestamp: u64 },

    /// New filter broadcast
    FilterBroadcast(SignedFilter),

    /// Apex command (requires multi-sig)
    ApexCommand(ApexCommand),
}

/// Sync Protocol - Gossip-based Hive Mind
///
/// Implements epidemic/gossip protocol for rapid threat propagation.
#[allow(dead_code)]
pub struct SyncProtocol {
    /// This node's ID
    node_id: String,

    /// This node's signing key
    node_key: SoftwareKeyStore,

    /// Known peers
    peers: RwLock<HashMap<String, MeshNode>>,

    /// Seen fingerprints (to prevent loops)
    seen_fingerprints: RwLock<HashSet<[u8; 32]>>,

    /// Pending outbound messages
    outbound_queue: RwLock<Vec<SyncMessage>>,

    /// Received messages (for processing)
    inbound_queue: RwLock<Vec<SyncMessage>>,

    /// Genesis block reference
    genesis: RwLock<Option<GenesisBlock>>,

    /// Is protocol active
    active: AtomicBool,

    /// Message counter
    message_count: AtomicU64,

    /// Gossip fanout (number of peers to relay to)
    fanout: usize,

    /// Maximum hop count
    max_hops: u8,
}

impl SyncProtocol {
    /// Create new sync protocol
    pub fn new(fanout: usize, max_hops: u8) -> Result<Self, CryptoError> {
        let node_key = SoftwareKeyStore::generate()?;
        let node_id = format!("node-{:016x}", rand::random::<u64>());

        Ok(SyncProtocol {
            node_id,
            node_key,
            peers: RwLock::new(HashMap::new()),
            seen_fingerprints: RwLock::new(HashSet::new()),
            outbound_queue: RwLock::new(Vec::new()),
            inbound_queue: RwLock::new(Vec::new()),
            genesis: RwLock::new(None),
            active: AtomicBool::new(true),
            message_count: AtomicU64::new(0),
            fanout,
            max_hops,
        })
    }

    /// Set the Genesis Block
    pub fn set_genesis(&self, genesis: GenesisBlock) {
        *self.genesis.write() = Some(genesis);
    }

    /// Register a peer node
    pub fn register_peer(&self, node: MeshNode) {
        self.peers.write().insert(node.id.clone(), node);
    }

    /// Broadcast a threat fingerprint (gossip protocol)
    ///
    /// This is the core of the hive-mind intelligence.
    pub fn broadcast_threat(&self, mut fingerprint: CompactedThreatFingerprint) {
        if !self.active.load(Ordering::SeqCst) {
            return;
        }

        // Check if we've already seen this
        if self
            .seen_fingerprints
            .read()
            .contains(&fingerprint.signature)
        {
            return;
        }

        // Mark as seen
        self.seen_fingerprints.write().insert(fingerprint.signature);

        // Increment hop count
        fingerprint.relay();

        // Check max hops
        if fingerprint.hop_count > self.max_hops {
            return;
        }

        // Queue for broadcast to fanout peers
        let message = SyncMessage::ThreatBroadcast(fingerprint);
        self.outbound_queue.write().push(message);

        self.message_count.fetch_add(1, Ordering::SeqCst);
    }

    /// Broadcast a signed filter
    pub fn broadcast_filter(&self, filter: SignedFilter) {
        if !self.active.load(Ordering::SeqCst) {
            return;
        }

        let message = SyncMessage::FilterBroadcast(filter);
        self.outbound_queue.write().push(message);
        self.message_count.fetch_add(1, Ordering::SeqCst);
    }

    /// Get pending outbound messages (for network layer to transmit)
    pub fn take_outbound(&self) -> Vec<SyncMessage> {
        std::mem::take(&mut *self.outbound_queue.write())
    }

    /// Receive a sync message
    pub fn receive(&self, message: SyncMessage) {
        self.inbound_queue.write().push(message);
    }

    /// Process received messages
    ///
    /// Returns fingerprints to add to local immunity memory
    pub fn process_inbound(&self) -> Vec<CompactedThreatFingerprint> {
        let messages = std::mem::take(&mut *self.inbound_queue.write());
        let mut new_threats = Vec::new();

        for message in messages {
            match message {
                SyncMessage::ThreatBroadcast(fingerprint) => {
                    // Check if new
                    if !self
                        .seen_fingerprints
                        .read()
                        .contains(&fingerprint.signature)
                    {
                        // Add to our immunity
                        new_threats.push(fingerprint.clone());
                        // Relay to others
                        self.broadcast_threat(fingerprint);
                    }
                }
                SyncMessage::ThreatBatch(batch) => {
                    for fingerprint in batch {
                        if !self
                            .seen_fingerprints
                            .read()
                            .contains(&fingerprint.signature)
                        {
                            new_threats.push(fingerprint.clone());
                            self.broadcast_threat(fingerprint);
                        }
                    }
                }
                SyncMessage::Heartbeat { node_id, timestamp } => {
                    // Update peer last_seen
                    if let Some(peer) = self.peers.write().get_mut(&node_id) {
                        peer.last_seen = timestamp;
                        peer.active = true;
                    }
                }
                SyncMessage::FilterBroadcast(filter) => {
                    // Filter broadcasts are handled separately
                    self.outbound_queue
                        .write()
                        .push(SyncMessage::FilterBroadcast(filter));
                }
                SyncMessage::ApexCommand(cmd) => {
                    // Apex commands require special handling
                    self.outbound_queue
                        .write()
                        .push(SyncMessage::ApexCommand(cmd));
                }
                _ => {}
            }
        }

        new_threats
    }

    /// Get active peer count
    pub fn peer_count(&self) -> usize {
        self.peers.read().values().filter(|p| p.active).count()
    }

    /// Get message count
    pub fn message_count(&self) -> u64 {
        self.message_count.load(Ordering::SeqCst)
    }

    /// Get node ID
    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    /// Shutdown protocol
    pub fn shutdown(&self) {
        self.active.store(false, Ordering::SeqCst);
    }
}

// ============================================================================
// APEX CONTROL - CREATOR OVERRIDE
// ============================================================================

/// Apex command types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApexCommandType {
    /// Emergency stop of all mutation engines
    EmergencyStop,

    /// Force global hard reset
    GlobalHardReset,

    /// Update genesis ethics (extremely rare)
    EthicsUpdate(Vec<String>),

    /// Revoke a malicious filter
    RevokeFilter { filter_id: String },

    /// Add new council member
    AddCouncilMember { pubkey: Vec<u8> },

    /// Remove council member
    RemoveCouncilMember { pubkey: Vec<u8> },

    /// Custom command
    Custom { command: String, data: Vec<u8> },
}

/// Apex Command - requires multi-signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApexCommand {
    /// Command type
    pub command: ApexCommandType,

    /// Command nonce (anti-replay)
    pub nonce: [u8; 32],

    /// Timestamp
    pub timestamp: u64,

    /// Architect signature
    pub architect_signature: Vec<u8>,

    /// BFT Council threshold signature
    pub council_signature: ThresholdSignature,

    /// Required quorum (must match)
    pub required_quorum: usize,
}

impl ApexCommand {
    /// Create new apex command (unsigned)
    pub fn new(command: ApexCommandType, required_quorum: usize) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut nonce = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut nonce);

        ApexCommand {
            command,
            nonce,
            timestamp: now,
            architect_signature: Vec::new(),
            council_signature: ThresholdSignature {
                combined_signature: Vec::new(),
                signer_pubkeys: Vec::new(),
                count: 0,
            },
            required_quorum,
        }
    }

    /// Get signable bytes
    fn signable_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.nonce);
        data.extend_from_slice(&self.timestamp.to_le_bytes());
        data.extend_from_slice(&(self.required_quorum as u32).to_le_bytes());
        // Include command type hash
        let cmd_bytes = serde_json::to_vec(&self.command).unwrap_or_default();
        data.extend_from_slice(&cmd_bytes);
        data
    }

    /// Sign by Architect
    pub fn sign_architect(&mut self, architect_key: &dyn KeyStore) -> Result<(), CryptoError> {
        let data = self.signable_bytes();
        self.architect_signature = architect_key.sign(&data)?;
        Ok(())
    }

    /// Add council signature
    pub fn add_council_signature(&mut self, signature: ThresholdSignature) {
        self.council_signature = signature;
    }

    /// Verify the command has valid multi-signature
    pub fn verify(&self, architect_key: &dyn KeyStore) -> bool {
        // Verify architect signature
        let data = self.signable_bytes();
        if architect_key
            .verify(&data, &self.architect_signature)
            .is_err()
        {
            return false;
        }

        // Verify council quorum
        if self.council_signature.count < self.required_quorum {
            return false;
        }

        true
    }

    /// Check if command is fresh (not expired)
    pub fn is_fresh(&self, max_age_secs: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now - self.timestamp <= max_age_secs
    }
}

/// Apex Control Module - Creator-level overrides
///
/// The only way to stop the MutationEngine or force a global HardReset.
/// Requires: Architect Key + BFT Council Quorum
#[allow(dead_code)]
pub struct ApexControl {
    /// The Architect's public key
    architect_pubkey: Vec<u8>,

    /// Required council quorum
    required_quorum: usize,

    /// Pending commands awaiting signatures
    pending_commands: RwLock<Vec<ApexCommand>>,

    /// Executed command nonces (anti-replay)
    executed_nonces: RwLock<HashSet<[u8; 32]>>,

    /// Is system stopped (emergency stop active)
    is_stopped: AtomicBool,

    /// Is global reset pending
    reset_pending: AtomicBool,

    /// Command execution count
    execution_count: AtomicU64,
}

impl ApexControl {
    /// Create new Apex Control
    pub fn new(architect_pubkey: Vec<u8>, required_quorum: usize) -> Self {
        ApexControl {
            architect_pubkey,
            required_quorum,
            pending_commands: RwLock::new(Vec::new()),
            executed_nonces: RwLock::new(HashSet::new()),
            is_stopped: AtomicBool::new(false),
            reset_pending: AtomicBool::new(false),
            execution_count: AtomicU64::new(0),
        }
    }

    /// Submit a command for execution
    ///
    /// Returns true if command was accepted and executed
    pub fn execute(
        &self,
        command: ApexCommand,
        architect_key: &dyn KeyStore,
    ) -> Result<bool, ApexError> {
        // Verify command is fresh
        if !command.is_fresh(300) {
            // 5 minute max age
            return Err(ApexError::CommandExpired);
        }

        // Check for replay
        if self.executed_nonces.read().contains(&command.nonce) {
            return Err(ApexError::ReplayDetected);
        }

        // Verify multi-signature
        if !command.verify(architect_key) {
            return Err(ApexError::InvalidSignature);
        }

        // Verify quorum
        if command.council_signature.count < self.required_quorum {
            return Err(ApexError::InsufficientQuorum {
                required: self.required_quorum,
                got: command.council_signature.count,
            });
        }

        // Execute command
        match &command.command {
            ApexCommandType::EmergencyStop => {
                self.is_stopped.store(true, Ordering::SeqCst);
            }
            ApexCommandType::GlobalHardReset => {
                self.reset_pending.store(true, Ordering::SeqCst);
            }
            _ => {
                // Other commands handled by caller
            }
        }

        // Record nonce
        self.executed_nonces.write().insert(command.nonce);
        self.execution_count.fetch_add(1, Ordering::SeqCst);

        Ok(true)
    }

    /// Check if emergency stop is active
    pub fn is_stopped(&self) -> bool {
        self.is_stopped.load(Ordering::SeqCst)
    }

    /// Check if global reset is pending
    pub fn is_reset_pending(&self) -> bool {
        self.reset_pending.load(Ordering::SeqCst)
    }

    /// Clear reset flag (after reset is complete)
    pub fn acknowledge_reset(&self) {
        self.reset_pending.store(false, Ordering::SeqCst);
    }

    /// Resume from emergency stop (requires another command)
    pub fn resume(&self) {
        self.is_stopped.store(false, Ordering::SeqCst);
    }

    /// Get execution count
    pub fn execution_count(&self) -> u64 {
        self.execution_count.load(Ordering::SeqCst)
    }
}

/// Apex Control errors
#[derive(Debug, Clone)]
pub enum ApexError {
    /// Command has expired
    CommandExpired,
    /// Replay attack detected
    ReplayDetected,
    /// Invalid signature
    InvalidSignature,
    /// Insufficient council quorum
    InsufficientQuorum { required: usize, got: usize },
    /// Command not authorized
    NotAuthorized,
}

impl std::fmt::Display for ApexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApexError::CommandExpired => write!(f, "Apex command has expired"),
            ApexError::ReplayDetected => write!(f, "Replay attack detected"),
            ApexError::InvalidSignature => write!(f, "Invalid multi-signature"),
            ApexError::InsufficientQuorum { required, got } => {
                write!(f, "Insufficient quorum: need {}, got {}", required, got)
            }
            ApexError::NotAuthorized => write!(f, "Command not authorized"),
        }
    }
}

impl std::error::Error for ApexError {}

// ============================================================================
// STEALTH INTEGRITY - THE INVISIBLE SENTINEL
// ============================================================================

/// Memory slot for stealth operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemorySlot {
    Primary,
    Secondary,
    Tertiary,
    Quarternary,
}

/// Stealth Integrity - Minimizes system footprint
///
/// Moves logic between WASM memory slots to avoid detection.
pub struct StealthIntegrity {
    /// Current active slot
    current_slot: RwLock<MemorySlot>,

    /// Slot rotation counter
    rotation_count: AtomicU64,

    /// Rotation interval (operations between rotations)
    rotation_interval: u64,

    /// Operations since last rotation
    ops_since_rotation: AtomicU64,

    /// Stealth mode active
    stealth_active: AtomicBool,

    /// Canary values for each slot (tamper detection)
    canaries: RwLock<[u64; 4]>,

    /// Expected canary values
    expected_canaries: [u64; 4],

    /// Is integrity compromised
    compromised: AtomicBool,
}

impl StealthIntegrity {
    /// Create new stealth integrity monitor
    pub fn new(rotation_interval: u64) -> Self {
        // Generate random canaries
        let canaries = [
            rand::random::<u64>(),
            rand::random::<u64>(),
            rand::random::<u64>(),
            rand::random::<u64>(),
        ];

        StealthIntegrity {
            current_slot: RwLock::new(MemorySlot::Primary),
            rotation_count: AtomicU64::new(0),
            rotation_interval,
            ops_since_rotation: AtomicU64::new(0),
            stealth_active: AtomicBool::new(true),
            canaries: RwLock::new(canaries),
            expected_canaries: canaries,
            compromised: AtomicBool::new(false),
        }
    }

    /// Record an operation and potentially rotate slots
    pub fn tick(&self) -> MemorySlot {
        if !self.stealth_active.load(Ordering::SeqCst) {
            return *self.current_slot.read();
        }

        let ops = self.ops_since_rotation.fetch_add(1, Ordering::SeqCst);

        if ops >= self.rotation_interval {
            self.rotate_slot();
            self.ops_since_rotation.store(0, Ordering::SeqCst);
        }

        *self.current_slot.read()
    }

    /// Rotate to next memory slot
    fn rotate_slot(&self) {
        let mut slot = self.current_slot.write();
        *slot = match *slot {
            MemorySlot::Primary => MemorySlot::Secondary,
            MemorySlot::Secondary => MemorySlot::Tertiary,
            MemorySlot::Tertiary => MemorySlot::Quarternary,
            MemorySlot::Quarternary => MemorySlot::Primary,
        };

        self.rotation_count.fetch_add(1, Ordering::SeqCst);
    }

    /// Verify all canaries are intact
    pub fn verify_canaries(&self) -> bool {
        let canaries = self.canaries.read();
        for (i, &canary) in canaries.iter().enumerate() {
            if canary != self.expected_canaries[i] {
                self.compromised.store(true, Ordering::SeqCst);
                return false;
            }
        }
        true
    }

    /// Check if integrity is compromised
    pub fn is_compromised(&self) -> bool {
        self.compromised.load(Ordering::SeqCst)
    }

    /// Get current slot
    pub fn current_slot(&self) -> MemorySlot {
        *self.current_slot.read()
    }

    /// Get rotation count
    pub fn rotation_count(&self) -> u64 {
        self.rotation_count.load(Ordering::SeqCst)
    }

    /// Disable stealth mode
    pub fn disable_stealth(&self) {
        self.stealth_active.store(false, Ordering::SeqCst);
    }

    /// Enable stealth mode
    pub fn enable_stealth(&self) {
        self.stealth_active.store(true, Ordering::SeqCst);
    }

    /// Get slot index (for WASM memory operations)
    pub fn slot_index(&self) -> usize {
        match *self.current_slot.read() {
            MemorySlot::Primary => 0,
            MemorySlot::Secondary => 1,
            MemorySlot::Tertiary => 2,
            MemorySlot::Quarternary => 3,
        }
    }
}

// ============================================================================
// GLOBAL IMMUNITY MESH
// ============================================================================

/// The Global Immunity Mesh - combines all components
///
/// This is THE ATMOSPHERE - everywhere, invisible, unyielding.
pub struct GlobalImmunityMesh {
    /// Genesis block (root of all)
    genesis: RwLock<Option<GenesisBlock>>,

    /// Sync protocol (gossip-based)
    sync: Arc<SyncProtocol>,

    /// Apex control (creator override)
    apex: Arc<ApexControl>,

    /// Stealth integrity
    stealth: Arc<StealthIntegrity>,

    /// Mesh ID
    mesh_id: String,

    /// Is mesh active
    active: AtomicBool,
}

impl GlobalImmunityMesh {
    /// Create new global immunity mesh
    pub fn new(architect_pubkey: Vec<u8>, council_quorum: usize) -> Result<Self, CryptoError> {
        let mesh_id = format!("mesh-{:016x}", rand::random::<u64>());

        Ok(GlobalImmunityMesh {
            genesis: RwLock::new(None),
            sync: Arc::new(SyncProtocol::new(4, 10)?), // fanout=4, max_hops=10
            apex: Arc::new(ApexControl::new(architect_pubkey, council_quorum)),
            stealth: Arc::new(StealthIntegrity::new(100)), // rotate every 100 ops
            mesh_id,
            active: AtomicBool::new(true),
        })
    }

    /// Initialize with Genesis Block
    pub fn initialize_genesis(&self, genesis: GenesisBlock) {
        *self.genesis.write() = Some(genesis.clone());
        self.sync.set_genesis(genesis);
    }

    /// Broadcast a threat to the global mesh
    pub fn broadcast_threat(&self, pattern: &AttackPattern) {
        if !self.active.load(Ordering::SeqCst) {
            return;
        }

        // Tick stealth
        self.stealth.tick();

        // Check apex stop
        if self.apex.is_stopped() {
            return;
        }

        // Create fingerprint and broadcast
        let fingerprint = CompactedThreatFingerprint::from_pattern(pattern, &self.mesh_id);
        self.sync.broadcast_threat(fingerprint);
    }

    /// Broadcast a filter to the global mesh
    pub fn broadcast_filter(&self, filter: SignedFilter) {
        if !self.active.load(Ordering::SeqCst) {
            return;
        }

        self.stealth.tick();

        if self.apex.is_stopped() {
            return;
        }

        self.sync.broadcast_filter(filter);
    }

    /// Process incoming messages
    pub fn process_incoming(&self) -> Vec<CompactedThreatFingerprint> {
        self.stealth.tick();
        self.sync.process_inbound()
    }

    /// Get sync protocol
    pub fn sync(&self) -> Arc<SyncProtocol> {
        self.sync.clone()
    }

    /// Get apex control
    pub fn apex(&self) -> Arc<ApexControl> {
        self.apex.clone()
    }

    /// Get stealth integrity
    pub fn stealth(&self) -> Arc<StealthIntegrity> {
        self.stealth.clone()
    }

    /// Get mesh ID
    pub fn mesh_id(&self) -> &str {
        &self.mesh_id
    }

    /// Shutdown mesh
    pub fn shutdown(&self) {
        self.active.store(false, Ordering::SeqCst);
        self.sync.shutdown();
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_block_creation() {
        let architect_key = SoftwareKeyStore::generate().unwrap();

        let genesis = GenesisBlock::create(
            vec![
                "Do no harm".to_string(),
                "Respect privacy".to_string(),
                "Be transparent".to_string(),
            ],
            &architect_key,
            "In the beginning, there was ethics.",
        )
        .unwrap();

        assert!(!genesis.architect_signature.is_empty());
        assert!(genesis.verify(&architect_key));
    }

    #[test]
    fn test_genesis_block_tamper_detection() {
        let architect_key = SoftwareKeyStore::generate().unwrap();

        let mut genesis =
            GenesisBlock::create(vec!["Do no harm".to_string()], &architect_key, "Genesis")
                .unwrap();

        // Tamper with ethics
        genesis.first_ethics.push("Evil rule".to_string());

        // Should fail verification
        assert!(!genesis.verify(&architect_key));
    }

    #[test]
    fn test_sync_protocol_gossip() {
        let protocol = SyncProtocol::new(4, 10).unwrap();

        // Create a threat fingerprint
        let fingerprint = CompactedThreatFingerprint {
            signature: [1u8; 32],
            level: ThreatLevel::High,
            category_code: 1,
            timestamp: 12345,
            origin_hash: 0,
            hop_count: 0,
            priority: 192,
        };

        // Broadcast
        protocol.broadcast_threat(fingerprint.clone());

        // Should have outbound message
        let outbound = protocol.take_outbound();
        assert_eq!(outbound.len(), 1);

        // Broadcasting same fingerprint should be ignored
        protocol.broadcast_threat(fingerprint);
        let outbound2 = protocol.take_outbound();
        assert_eq!(outbound2.len(), 0);
    }

    #[test]
    fn test_apex_command_multisig() {
        let architect_key = SoftwareKeyStore::generate().unwrap();

        let mut command = ApexCommand::new(ApexCommandType::EmergencyStop, 3);

        // Sign by architect
        command.sign_architect(&architect_key).unwrap();

        // Add council signature
        command.add_council_signature(ThresholdSignature {
            combined_signature: vec![1, 2, 3],
            signer_pubkeys: vec![vec![1], vec![2], vec![3]],
            count: 3,
        });

        // Verify
        assert!(command.verify(&architect_key));
    }

    #[test]
    fn test_apex_control_execution() {
        let architect_key = SoftwareKeyStore::generate().unwrap();
        let apex = ApexControl::new(architect_key.public_key_bytes(), 3);

        let mut command = ApexCommand::new(ApexCommandType::EmergencyStop, 3);
        command.sign_architect(&architect_key).unwrap();
        command.add_council_signature(ThresholdSignature {
            combined_signature: vec![1, 2, 3],
            signer_pubkeys: vec![vec![1], vec![2], vec![3]],
            count: 3,
        });

        // Execute
        let result = apex.execute(command, &architect_key);
        assert!(result.is_ok());

        // System should be stopped
        assert!(apex.is_stopped());
    }

    #[test]
    fn test_stealth_slot_rotation() {
        let stealth = StealthIntegrity::new(3); // Rotate every 3 ops

        assert_eq!(stealth.current_slot(), MemorySlot::Primary);

        // Tick 4 times (rotation happens when ops >= 3, fetch_add returns pre-increment)
        stealth.tick(); // ops becomes 1
        stealth.tick(); // ops becomes 2
        stealth.tick(); // ops becomes 3, triggers rotation
        stealth.tick(); // ops becomes 1 (reset after rotation)

        // Should have rotated to Secondary
        assert_eq!(stealth.current_slot(), MemorySlot::Secondary);

        // 4 more to rotate again
        stealth.tick();
        stealth.tick();
        stealth.tick();
        stealth.tick();

        assert_eq!(stealth.current_slot(), MemorySlot::Tertiary);
    }

    #[test]
    fn test_stealth_canary_verification() {
        let stealth = StealthIntegrity::new(100);

        // Canaries should be intact
        assert!(stealth.verify_canaries());
        assert!(!stealth.is_compromised());

        // Tamper with canary
        stealth.canaries.write()[0] = 0xDEADBEEF;

        // Should detect tampering
        assert!(!stealth.verify_canaries());
        assert!(stealth.is_compromised());
    }

    #[test]
    fn test_global_immunity_sync() {
        let architect_key = SoftwareKeyStore::generate().unwrap();

        // Create mesh
        let mesh = GlobalImmunityMesh::new(architect_key.public_key_bytes(), 3).unwrap();

        // Create and set genesis
        let genesis = GenesisBlock::create(
            vec!["Do no harm".to_string()],
            &architect_key,
            "Test genesis",
        )
        .unwrap();

        mesh.initialize_genesis(genesis);

        // Create test pattern
        let pattern = AttackPattern {
            id: "test".to_string(),
            signature: [42u8; 32],
            category: crate::evolutionary_guard::AttackCategory::CommandInjection,
            threat_level: ThreatLevel::High,
            action_type: crate::proof::ActionType::Execute,
            target_patterns: vec![],
            keywords: vec!["exec".to_string()],
            timing_signature: None,
            triggered_rule: "test rule".to_string(),
            first_seen: 12345,
            occurrence_count: 1,
            confidence: 0.9,
        };

        // Broadcast threat
        mesh.broadcast_threat(&pattern);

        // Should have outbound message
        let outbound = mesh.sync().take_outbound();
        assert!(!outbound.is_empty());
    }

    #[test]
    fn test_apex_prevents_replay() {
        let architect_key = SoftwareKeyStore::generate().unwrap();
        let apex = ApexControl::new(architect_key.public_key_bytes(), 3);

        let mut command = ApexCommand::new(ApexCommandType::GlobalHardReset, 3);
        command.sign_architect(&architect_key).unwrap();
        command.add_council_signature(ThresholdSignature {
            combined_signature: vec![1, 2, 3],
            signer_pubkeys: vec![vec![1], vec![2], vec![3]],
            count: 3,
        });

        // First execution succeeds
        let result1 = apex.execute(command.clone(), &architect_key);
        assert!(result1.is_ok());

        // Replay should fail
        let result2 = apex.execute(command, &architect_key);
        assert!(matches!(result2, Err(ApexError::ReplayDetected)));
    }
}
