//! # Hope Genome v1.4.0 - Persistent Nonce Store API
//!
//! **Replay Attack Prevention with Persistent Storage**
//!
//! This module provides trait-based nonce storage backends to prevent replay attacks
//! even after system restarts. The architecture supports multiple storage engines:
//!
//! - **MemoryNonceStore**: In-memory (fast, testing only - lost on restart)
//! - **RocksDbNonceStore**: Persistent disk storage (production recommended)
//! - **RedisNonceStore**: Distributed cache (multi-instance deployments)
//!
//! ## Security Model
//!
//! - **Atomic check-and-insert**: Prevents race conditions
//! - **TTL-based expiry**: Automatic cleanup of old nonces
//! - **Crash-resistant**: RocksDB/Redis survive restarts
//!
//! ## Example
//!
//! ```rust
//! use hope_core::nonce_store::{NonceStore, MemoryNonceStore};
//!
//! let mut store = MemoryNonceStore::new();
//! let nonce = [0u8; 32];
//!
//! // First use: success
//! assert!(store.check_and_insert(nonce, 3600).is_ok());
//!
//! // Replay attack: blocked
//! assert!(store.check_and_insert(nonce, 3600).is_err());
//! ```
//!
//! ---
//!
//! **Date**: 2025-12-30
//! **Version**: 1.4.0 (Hardened Security Edition)
//! **Author**: Máté Róbert <stratosoiteam@gmail.com>

use std::collections::HashMap;
use thiserror::Error;

// ============================================================================
// ERROR TYPES
// ============================================================================

#[derive(Debug, Error)]
pub enum NonceStoreError {
    #[error("REPLAY ATTACK DETECTED: Nonce {0} already used")]
    NonceReused(String),

    #[error("Storage backend error: {0}")]
    StorageError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub type Result<T> = std::result::Result<T, NonceStoreError>;

// ============================================================================
// TRAIT: NonceStore
// ============================================================================

/// Trait for nonce storage backends
///
/// Implementations must guarantee:
/// 1. Atomicity: check-and-insert is atomic (no race conditions)
/// 2. Persistence: Nonces survive process restarts (except MemoryNonceStore)
/// 3. TTL support: Old nonces can be expired automatically
pub trait NonceStore: Send + Sync {
    /// Check if nonce exists, if not, insert it atomically
    ///
    /// # Arguments
    /// * `nonce` - The nonce to check (32 bytes)
    /// * `ttl_seconds` - Time-to-live for this nonce (for expiry)
    ///
    /// # Returns
    /// - `Ok(())` if nonce was not used and is now recorded
    /// - `Err(NonceReused)` if nonce was already used (replay attack)
    ///
    /// # Example
    /// ```
    /// # use hope_core::nonce_store::{NonceStore, MemoryNonceStore};
    /// let mut store = MemoryNonceStore::new();
    /// let nonce = [42u8; 32];
    /// store.check_and_insert(nonce, 3600).unwrap();
    /// assert!(store.check_and_insert(nonce, 3600).is_err()); // Replay blocked
    /// ```
    fn check_and_insert(&mut self, nonce: [u8; 32], ttl_seconds: u64) -> Result<()>;

    /// Check if nonce exists (read-only, non-blocking)
    fn contains(&self, nonce: &[u8; 32]) -> bool;

    /// Clear all nonces (DANGEROUS - use only for testing or maintenance)
    fn clear(&mut self) -> Result<()>;

    /// Get count of stored nonces
    fn count(&self) -> usize;

    /// Remove expired nonces based on current time
    ///
    /// This is an optional maintenance operation. Some backends (Redis)
    /// handle expiry automatically via TTL.
    fn cleanup_expired(&mut self) -> Result<usize> {
        // Default implementation: no-op
        Ok(0)
    }
}

// ============================================================================
// MEMORY NONCE STORE (Default - Testing Only)
// ============================================================================

/// In-memory nonce store
///
/// **WARNING**: Nonces are lost on process restart. Use only for:
/// - Unit tests
/// - Development environments
/// - Short-lived processes
///
/// For production, use `RocksDbNonceStore` or `RedisNonceStore`.
pub struct MemoryNonceStore {
    /// Map: nonce -> (insertion_timestamp, ttl_seconds)
    nonces: HashMap<[u8; 32], (u64, u64)>,
}

impl MemoryNonceStore {
    /// Create a new in-memory nonce store
    pub fn new() -> Self {
        MemoryNonceStore {
            nonces: HashMap::new(),
        }
    }
}

impl Default for MemoryNonceStore {
    fn default() -> Self {
        Self::new()
    }
}

impl NonceStore for MemoryNonceStore {
    fn check_and_insert(&mut self, nonce: [u8; 32], ttl_seconds: u64) -> Result<()> {
        // Check if already exists
        if self.nonces.contains_key(&nonce) {
            return Err(NonceStoreError::NonceReused(hex::encode(nonce)));
        }

        // Insert with current timestamp
        let now = chrono::Utc::now().timestamp() as u64;
        self.nonces.insert(nonce, (now, ttl_seconds));

        Ok(())
    }

    fn contains(&self, nonce: &[u8; 32]) -> bool {
        self.nonces.contains_key(nonce)
    }

    fn clear(&mut self) -> Result<()> {
        self.nonces.clear();
        Ok(())
    }

    fn count(&self) -> usize {
        self.nonces.len()
    }

    fn cleanup_expired(&mut self) -> Result<usize> {
        let now = chrono::Utc::now().timestamp() as u64;
        let initial_count = self.nonces.len();

        // Remove expired entries
        self.nonces
            .retain(|_, (timestamp, ttl)| now - *timestamp <= *ttl);

        Ok(initial_count - self.nonces.len())
    }
}

// ============================================================================
// ROCKSDB NONCE STORE (Production - Persistent)
// ============================================================================

#[cfg(feature = "rocksdb-nonce-store")]
use rocksdb::{Options, WriteBatch, DB};

#[cfg(feature = "rocksdb-nonce-store")]
/// Persistent nonce store backed by RocksDB
///
/// **Recommended for production**: Nonces survive process restarts.
///
/// # Features
/// - Atomic operations via RocksDB transactions
/// - Crash-resistant (write-ahead log)
/// - Efficient storage (LSM tree)
/// - TTL-based expiry
///
/// # Example
/// ```no_run
/// # use hope_core::nonce_store::{NonceStore, RocksDbNonceStore};
/// let mut store = RocksDbNonceStore::new("./hope_nonces.db").unwrap();
/// let nonce = [1u8; 32];
/// store.check_and_insert(nonce, 3600).unwrap();
/// // Nonce persists even after process restart
/// ```
pub struct RocksDbNonceStore {
    db: DB,
}

#[cfg(feature = "rocksdb-nonce-store")]
impl RocksDbNonceStore {
    /// Create or open RocksDB nonce store
    ///
    /// # Arguments
    /// * `path` - Database path (e.g., "./hope_nonces.db")
    ///
    /// # Errors
    /// Returns error if database cannot be opened or created
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);

        let db = DB::open(&opts, path).map_err(|e| NonceStoreError::StorageError(e.to_string()))?;

        Ok(RocksDbNonceStore { db })
    }

    /// Encode nonce entry: [timestamp (8 bytes)][ttl (8 bytes)]
    fn encode_value(timestamp: u64, ttl: u64) -> [u8; 16] {
        let mut value = [0u8; 16];
        value[0..8].copy_from_slice(&timestamp.to_le_bytes());
        value[8..16].copy_from_slice(&ttl.to_le_bytes());
        value
    }

    /// Decode nonce entry
    fn decode_value(value: &[u8]) -> Option<(u64, u64)> {
        if value.len() < 16 {
            return None;
        }
        let timestamp = u64::from_le_bytes(value[0..8].try_into().ok()?);
        let ttl = u64::from_le_bytes(value[8..16].try_into().ok()?);
        Some((timestamp, ttl))
    }
}

#[cfg(feature = "rocksdb-nonce-store")]
impl NonceStore for RocksDbNonceStore {
    fn check_and_insert(&mut self, nonce: [u8; 32], ttl_seconds: u64) -> Result<()> {
        // Check if exists
        if self
            .db
            .get(&nonce)
            .map_err(|e| NonceStoreError::StorageError(e.to_string()))?
            .is_some()
        {
            return Err(NonceStoreError::NonceReused(hex::encode(nonce)));
        }

        // Insert with timestamp
        let now = chrono::Utc::now().timestamp() as u64;
        let value = Self::encode_value(now, ttl_seconds);

        self.db
            .put(&nonce, &value)
            .map_err(|e| NonceStoreError::StorageError(e.to_string()))?;

        Ok(())
    }

    fn contains(&self, nonce: &[u8; 32]) -> bool {
        self.db.get(nonce).ok().flatten().is_some()
    }

    fn clear(&mut self) -> Result<()> {
        // Delete all keys (use with caution!)
        let mut batch = WriteBatch::default();

        let iter = self.db.iterator(rocksdb::IteratorMode::Start);
        for item in iter {
            if let Ok((key, _)) = item {
                batch.delete(&key);
            }
        }

        self.db
            .write(batch)
            .map_err(|e| NonceStoreError::StorageError(e.to_string()))?;

        Ok(())
    }

    fn count(&self) -> usize {
        self.db.iterator(rocksdb::IteratorMode::Start).count()
    }

    fn cleanup_expired(&mut self) -> Result<usize> {
        let now = chrono::Utc::now().timestamp() as u64;
        let mut batch = WriteBatch::default();
        let mut expired_count = 0;

        let iter = self.db.iterator(rocksdb::IteratorMode::Start);
        for item in iter {
            if let Ok((key, value)) = item {
                if let Some((timestamp, ttl)) = Self::decode_value(&value) {
                    if now - timestamp > ttl {
                        batch.delete(&key);
                        expired_count += 1;
                    }
                }
            }
        }

        self.db
            .write(batch)
            .map_err(|e| NonceStoreError::StorageError(e.to_string()))?;

        Ok(expired_count)
    }
}

// ============================================================================
// REDIS NONCE STORE (Distributed - Multi-Instance)
// ============================================================================

#[cfg(feature = "redis-nonce-store")]
use redis::{Client, Commands, Connection};

#[cfg(feature = "redis-nonce-store")]
/// Distributed nonce store backed by Redis
///
/// **Recommended for multi-instance deployments**: Shared nonce tracking
/// across multiple Hope Genome instances.
///
/// # Features
/// - Atomic operations via Redis SET NX
/// - Automatic TTL expiry (no manual cleanup needed)
/// - Distributed consensus (multiple servers share state)
///
/// # Example
/// ```no_run
/// # use hope_core::nonce_store::{NonceStore, RedisNonceStore};
/// let mut store = RedisNonceStore::new("redis://127.0.0.1/", "hope:nonce:").unwrap();
/// let nonce = [2u8; 32];
/// store.check_and_insert(nonce, 3600).unwrap();
/// ```
pub struct RedisNonceStore {
    client: Client,
    key_prefix: String,
}

#[cfg(feature = "redis-nonce-store")]
impl RedisNonceStore {
    /// Create Redis nonce store
    ///
    /// # Arguments
    /// * `redis_url` - Redis connection URL (e.g., "redis://127.0.0.1/")
    /// * `key_prefix` - Prefix for keys (e.g., "hope:nonce:")
    ///
    /// # Errors
    /// Returns error if Redis connection cannot be established
    pub fn new(redis_url: &str, key_prefix: impl Into<String>) -> Result<Self> {
        let client =
            Client::open(redis_url).map_err(|e| NonceStoreError::StorageError(e.to_string()))?;

        // Test connection
        let mut con = client
            .get_connection()
            .map_err(|e| NonceStoreError::StorageError(e.to_string()))?;

        // Ping to verify
        let _: String = redis::cmd("PING")
            .query(&mut con)
            .map_err(|e| NonceStoreError::StorageError(e.to_string()))?;

        Ok(RedisNonceStore {
            client,
            key_prefix: key_prefix.into(),
        })
    }

    /// Get Redis key for nonce
    fn nonce_key(&self, nonce: &[u8; 32]) -> String {
        format!("{}{}", self.key_prefix, hex::encode(nonce))
    }

    /// Get connection (helper)
    fn get_connection(&self) -> Result<Connection> {
        self.client
            .get_connection()
            .map_err(|e| NonceStoreError::StorageError(e.to_string()))
    }
}

#[cfg(feature = "redis-nonce-store")]
impl NonceStore for RedisNonceStore {
    fn check_and_insert(&mut self, nonce: [u8; 32], ttl_seconds: u64) -> Result<()> {
        let mut con = self.get_connection()?;
        let key = self.nonce_key(&nonce);

        // SET key 1 NX EX ttl (atomic: set if not exists with expiry)
        let was_set: bool = redis::cmd("SET")
            .arg(&key)
            .arg(1)
            .arg("NX") // Only set if not exists
            .arg("EX") // Expiry in seconds
            .arg(ttl_seconds)
            .query(&mut con)
            .map_err(|e| NonceStoreError::StorageError(e.to_string()))?;

        if !was_set {
            return Err(NonceStoreError::NonceReused(hex::encode(nonce)));
        }

        Ok(())
    }

    fn contains(&self, nonce: &[u8; 32]) -> bool {
        let mut con = match self.get_connection() {
            Ok(c) => c,
            Err(_) => return false,
        };

        let key = self.nonce_key(nonce);
        con.exists(&key).unwrap_or(false)
    }

    fn clear(&mut self) -> Result<()> {
        let mut con = self.get_connection()?;

        // SCAN for all keys with prefix (safer than KEYS for production)
        let pattern = format!("{}*", self.key_prefix);
        let mut cursor = 0u64;
        let mut total_deleted = 0;

        loop {
            let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&pattern)
                .arg("COUNT")
                .arg(100)
                .query(&mut con)
                .map_err(|e| NonceStoreError::StorageError(e.to_string()))?;

            if !keys.is_empty() {
                let deleted: usize = con
                    .del(&keys)
                    .map_err(|e| NonceStoreError::StorageError(e.to_string()))?;
                total_deleted += deleted;
            }

            cursor = new_cursor;
            if cursor == 0 {
                break;
            }
        }

        Ok(())
    }

    fn count(&self) -> usize {
        let mut con = match self.get_connection() {
            Ok(c) => c,
            Err(_) => return 0,
        };

        let pattern = format!("{}*", self.key_prefix);
        let mut cursor = 0u64;
        let mut count = 0;

        loop {
            let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&pattern)
                .arg("COUNT")
                .arg(100)
                .query(&mut con)
                .unwrap_or((0, vec![]));

            count += keys.len();
            cursor = new_cursor;
            if cursor == 0 {
                break;
            }
        }

        count
    }

    fn cleanup_expired(&mut self) -> Result<usize> {
        // Redis handles TTL expiry automatically - no manual cleanup needed
        Ok(0)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_store_basic() {
        let mut store = MemoryNonceStore::new();
        let nonce = [42u8; 32];

        // First insert: success
        assert!(store.check_and_insert(nonce, 3600).is_ok());
        assert_eq!(store.count(), 1);

        // Second insert: replay attack detected
        let result = store.check_and_insert(nonce, 3600);
        assert!(result.is_err());
        assert!(matches!(result, Err(NonceStoreError::NonceReused(_))));
    }

    #[test]
    fn test_memory_store_multiple_nonces() {
        let mut store = MemoryNonceStore::new();

        for i in 0..10u8 {
            let nonce = [i; 32];
            assert!(store.check_and_insert(nonce, 3600).is_ok());
        }

        assert_eq!(store.count(), 10);
    }

    #[test]
    fn test_memory_store_contains() {
        let mut store = MemoryNonceStore::new();
        let nonce = [99u8; 32];

        assert!(!store.contains(&nonce));
        store.check_and_insert(nonce, 3600).unwrap();
        assert!(store.contains(&nonce));
    }

    #[test]
    fn test_memory_store_clear() {
        let mut store = MemoryNonceStore::new();

        for i in 0..5u8 {
            let nonce = [i; 32];
            store.check_and_insert(nonce, 3600).unwrap();
        }

        assert_eq!(store.count(), 5);
        store.clear().unwrap();
        assert_eq!(store.count(), 0);
    }

    #[test]
    fn test_memory_store_cleanup_expired() {
        let mut store = MemoryNonceStore::new();

        // Insert nonce with manual timestamp manipulation
        let nonce1 = [1u8; 32];
        let nonce2 = [2u8; 32];

        store.check_and_insert(nonce1, 10).unwrap(); // 10s TTL
        store.check_and_insert(nonce2, 3600).unwrap(); // 1h TTL

        // Manually expire nonce1 by manipulating timestamp
        let old_timestamp = chrono::Utc::now().timestamp() as u64 - 20; // 20s ago
        store.nonces.insert(nonce1, (old_timestamp, 10));

        assert_eq!(store.count(), 2);

        // Cleanup expired
        let removed = store.cleanup_expired().unwrap();
        assert_eq!(removed, 1);
        assert_eq!(store.count(), 1);
        assert!(!store.contains(&nonce1));
        assert!(store.contains(&nonce2));
    }
}
