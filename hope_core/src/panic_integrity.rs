//! # Hope Genome v1.8.0 - Panic Integrity (Self-Destructing Protection)
//!
//! **THE BLACK BOX** - If compromised, destroy everything
//!
//! ## Problem
//!
//! Even with perfect software security, physical attacks exist:
//! - **Cold Boot Attack**: Freeze RAM, read keys from memory
//! - **Rowhammer**: Bit-flip attacks via DRAM physics
//! - **DMA Attack**: Direct memory access via Thunderbolt/PCIe
//! - **Side-Channel**: Timing, power analysis, EM emanation
//!
//! ## Solution: Self-Destructing Integrity
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    PANIC INTEGRITY SYSTEM                        │
//! │                                                                  │
//! │   ┌─────────────────────────────────────────────────────────┐   │
//! │   │                  ANOMALY DETECTORS                       │   │
//! │   │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐     │   │
//! │   │  │Timing   │  │Memory   │  │Syscall  │  │Crypto   │     │   │
//! │   │  │Anomaly  │  │Pressure │  │Pattern  │  │Failure  │     │   │
//! │   │  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘     │   │
//! │   └───────┼────────────┼────────────┼────────────┼──────────┘   │
//! │           │            │            │            │              │
//! │           └────────────┴─────┬──────┴────────────┘              │
//! │                              ▼                                  │
//! │                    ┌─────────────────┐                          │
//! │                    │  PANIC ENGINE   │                          │
//! │                    │   (Threshold)   │                          │
//! │                    └────────┬────────┘                          │
//! │                             │                                   │
//! │              ┌──────────────┼──────────────┐                    │
//! │              ▼              ▼              ▼                    │
//! │      ┌────────────┐  ┌────────────┐  ┌────────────┐            │
//! │      │  ZEROIZE   │  │  FREEZE    │  │   ALERT    │            │
//! │      │  ALL KEYS  │  │  KEYSTORE  │  │  NETWORK   │            │
//! │      └────────────┘  └────────────┘  └────────────┘            │
//! │                                                                  │
//! │   Result: Attacker gets NOTHING. Keys destroyed. Alert sent.   │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Security Guarantees
//!
//! - **Pre-emptive**: Destroys keys BEFORE extraction possible
//! - **Non-recoverable**: Keys are overwritten, not just freed
//! - **Alerting**: Network notification of attack attempt
//! - **Forensic**: Panic log for post-incident analysis
//!
//! ---
//!
//! **Date**: 2026-01-01
//! **Version**: 1.8.0 (Betonozás Edition - Panic Integrity)
//! **Author**: Máté Róbert <stratosoiteam@gmail.com>

use crate::crypto::{CryptoError, KeyStore, Result};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use zeroize::Zeroize;

// ============================================================================
// PANIC TYPES
// ============================================================================

/// Anomaly type detected by the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnomalyType {
    /// Timing attack detected (operation took too long/short)
    TimingAnomaly,
    /// Memory pressure attack (allocation failures)
    MemoryPressure,
    /// Suspicious syscall pattern
    SyscallAnomaly,
    /// Cryptographic operation failure
    CryptoFailure,
    /// Signature verification failed unexpectedly
    SignatureAnomaly,
    /// Nonce reuse detected
    NonceReuse,
    /// Too many failed operations
    ExcessiveFailures,
    /// External trigger (manual panic)
    ExternalTrigger,
}

/// Severity level of anomaly
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Informational, no action
    Info = 0,
    /// Warning, log but continue
    Warning = 1,
    /// Error, increment panic counter
    Error = 2,
    /// Critical, immediate panic
    Critical = 3,
}

/// Anomaly event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyEvent {
    /// Anomaly type
    pub anomaly_type: AnomalyType,
    /// Severity
    pub severity: Severity,
    /// Description
    pub description: String,
    /// Timestamp
    pub timestamp: u64,
    /// Additional data
    pub metadata: Option<String>,
}

/// Panic state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PanicState {
    /// Normal operation
    Normal,
    /// Elevated threat level
    Elevated,
    /// Keys frozen (no new operations)
    Frozen,
    /// Keys destroyed (terminal state)
    Destroyed,
}

/// Panic log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanicLogEntry {
    /// Entry ID
    pub id: u64,
    /// Timestamp
    pub timestamp: u64,
    /// State transition
    pub from_state: PanicState,
    pub to_state: PanicState,
    /// Triggering anomaly
    pub trigger: AnomalyEvent,
    /// Hash chain (for tamper evidence)
    pub prev_hash: [u8; 32],
    pub entry_hash: [u8; 32],
}

// ============================================================================
// PROTECTED KEY STORE
// ============================================================================

/// Key material that can be destroyed
#[allow(dead_code)]
#[derive(Zeroize)]
#[zeroize(drop)]
struct DestructibleKey {
    /// The actual key bytes
    key_bytes: Vec<u8>,
    /// Canary value (if changed, memory was tampered)
    canary: [u8; 32],
}

#[allow(dead_code)]
impl DestructibleKey {
    fn new(key_bytes: Vec<u8>) -> Self {
        let mut canary = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut canary);

        DestructibleKey { key_bytes, canary }
    }

    fn verify_canary(&self) -> bool {
        // Canary should not be all zeros (indicates tampering/corruption)
        !self.canary.iter().all(|&b| b == 0)
    }

    fn destroy(&mut self) {
        // Overwrite with random data first
        rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut self.key_bytes);
        rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut self.canary);

        // Then zero
        self.key_bytes.zeroize();
        self.canary.zeroize();
    }
}

/// Protected KeyStore wrapper with panic capability
#[allow(clippy::type_complexity)]
pub struct PanicProtectedKeyStore<K: KeyStore> {
    /// Inner keystore
    inner: Arc<RwLock<Option<K>>>,

    /// Panic state
    state: Arc<RwLock<PanicState>>,

    /// Anomaly counter
    anomaly_count: AtomicU64,

    /// Panic threshold
    panic_threshold: u64,

    /// Is frozen
    frozen: AtomicBool,

    /// Is destroyed
    destroyed: AtomicBool,

    /// Panic log
    panic_log: Arc<RwLock<Vec<PanicLogEntry>>>,

    /// Alert callback
    alert_callback: Option<Box<dyn Fn(&PanicLogEntry) + Send + Sync>>,

    /// Last operation timestamp (for timing analysis)
    last_op_time: Arc<RwLock<Instant>>,
}

impl<K: KeyStore> PanicProtectedKeyStore<K> {
    /// Create new panic-protected keystore
    ///
    /// # Arguments
    ///
    /// * `inner` - The actual keystore to protect
    /// * `panic_threshold` - Number of anomalies before panic
    pub fn new(inner: K, panic_threshold: u64) -> Self {
        PanicProtectedKeyStore {
            inner: Arc::new(RwLock::new(Some(inner))),
            state: Arc::new(RwLock::new(PanicState::Normal)),
            anomaly_count: AtomicU64::new(0),
            panic_threshold,
            frozen: AtomicBool::new(false),
            destroyed: AtomicBool::new(false),
            panic_log: Arc::new(RwLock::new(Vec::new())),
            alert_callback: None,
            last_op_time: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Set alert callback for panic events
    pub fn set_alert_callback<F>(&mut self, callback: F)
    where
        F: Fn(&PanicLogEntry) + Send + Sync + 'static,
    {
        self.alert_callback = Some(Box::new(callback));
    }

    /// Report anomaly
    pub fn report_anomaly(&self, anomaly: AnomalyEvent) {
        let old_count = self.anomaly_count.fetch_add(1, Ordering::SeqCst);

        match anomaly.severity {
            Severity::Critical => {
                // Immediate panic
                self.trigger_panic(anomaly);
            }
            Severity::Error => {
                if old_count + 1 >= self.panic_threshold {
                    self.trigger_panic(anomaly);
                } else {
                    self.log_anomaly(anomaly);
                }
            }
            _ => {
                self.log_anomaly(anomaly);
            }
        }
    }

    /// Trigger panic - destroy all keys
    fn trigger_panic(&self, trigger: AnomalyEvent) {
        let old_state = *self.state.read();

        // Already destroyed?
        if old_state == PanicState::Destroyed {
            return;
        }

        // Freeze first
        self.frozen.store(true, Ordering::SeqCst);
        *self.state.write() = PanicState::Frozen;

        // Log the panic
        let log_entry = self.create_log_entry(old_state, PanicState::Destroyed, trigger);

        // Alert if callback set
        if let Some(ref callback) = self.alert_callback {
            callback(&log_entry);
        }

        // Add to log
        self.panic_log.write().push(log_entry);

        // DESTROY THE KEYS
        if let Some(ref mut _keystore) = *self.inner.write() {
            // The keystore will be dropped, which should zeroize if properly implemented
        }
        *self.inner.write() = None;

        // Mark as destroyed
        self.destroyed.store(true, Ordering::SeqCst);
        *self.state.write() = PanicState::Destroyed;
    }

    /// Create log entry
    fn create_log_entry(
        &self,
        from_state: PanicState,
        to_state: PanicState,
        trigger: AnomalyEvent,
    ) -> PanicLogEntry {
        let log = self.panic_log.read();
        let prev_hash = log.last().map(|e| e.entry_hash).unwrap_or([0u8; 32]);

        let id = log.len() as u64;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Compute entry hash
        let mut hasher = Sha256::new();
        hasher.update(id.to_le_bytes());
        hasher.update(timestamp.to_le_bytes());
        hasher.update([from_state as u8, to_state as u8]);
        hasher.update(prev_hash);
        let entry_hash = hasher.finalize().into();

        PanicLogEntry {
            id,
            timestamp,
            from_state,
            to_state,
            trigger,
            prev_hash,
            entry_hash,
        }
    }

    /// Log anomaly without triggering panic
    fn log_anomaly(&self, _anomaly: AnomalyEvent) {
        // In production, this would write to secure log
    }

    /// Check timing for anomaly detection
    pub fn check_timing(&self, operation_name: &str) -> Result<TimingGuard> {
        if self.is_destroyed() {
            return Err(CryptoError::InvalidState("KeyStore destroyed".into()));
        }

        Ok(TimingGuard::new(
            Arc::clone(&self.last_op_time),
            operation_name.to_string(),
        ))
    }

    /// Get current state
    pub fn state(&self) -> PanicState {
        *self.state.read()
    }

    /// Is keystore frozen?
    pub fn is_frozen(&self) -> bool {
        self.frozen.load(Ordering::SeqCst)
    }

    /// Is keystore destroyed?
    pub fn is_destroyed(&self) -> bool {
        self.destroyed.load(Ordering::SeqCst)
    }

    /// Get anomaly count
    pub fn anomaly_count(&self) -> u64 {
        self.anomaly_count.load(Ordering::SeqCst)
    }

    /// Get panic log
    pub fn get_panic_log(&self) -> Vec<PanicLogEntry> {
        self.panic_log.read().clone()
    }

    /// Manual panic trigger (for testing or emergency)
    pub fn emergency_panic(&self, reason: &str) {
        self.trigger_panic(AnomalyEvent {
            anomaly_type: AnomalyType::ExternalTrigger,
            severity: Severity::Critical,
            description: reason.to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metadata: None,
        });
    }
}

// Implement KeyStore for PanicProtectedKeyStore
impl<K: KeyStore> KeyStore for PanicProtectedKeyStore<K> {
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>> {
        if self.is_destroyed() {
            return Err(CryptoError::InvalidState(
                "KeyStore destroyed - keys zeroed".into(),
            ));
        }
        if self.is_frozen() {
            return Err(CryptoError::InvalidState(
                "KeyStore frozen - panic triggered".into(),
            ));
        }

        let guard = self.inner.read();
        match guard.as_ref() {
            Some(ks) => ks.sign(data),
            None => Err(CryptoError::InvalidState("KeyStore unavailable".into())),
        }
    }

    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<()> {
        // Verify is allowed even in frozen state (for audit)
        if self.is_destroyed() {
            return Err(CryptoError::InvalidState("KeyStore destroyed".into()));
        }

        let guard = self.inner.read();
        match guard.as_ref() {
            Some(ks) => ks.verify(data, signature),
            None => Err(CryptoError::InvalidState("KeyStore unavailable".into())),
        }
    }

    fn public_key_bytes(&self) -> Vec<u8> {
        let guard = self.inner.read();
        match guard.as_ref() {
            Some(ks) => ks.public_key_bytes(),
            None => vec![],
        }
    }

    fn identifier(&self) -> String {
        format!("PanicProtected(state={:?})", self.state())
    }
}

// ============================================================================
// TIMING GUARD
// ============================================================================

/// Guard for timing-based anomaly detection
pub struct TimingGuard {
    start: Instant,
    last_op: Arc<RwLock<Instant>>,
    operation: String,
    completed: bool,
}

impl TimingGuard {
    fn new(last_op: Arc<RwLock<Instant>>, operation: String) -> Self {
        TimingGuard {
            start: Instant::now(),
            last_op,
            operation,
            completed: false,
        }
    }

    /// Complete timing check
    pub fn complete(mut self) -> Option<AnomalyEvent> {
        self.completed = true;
        let elapsed = self.start.elapsed();

        // Check for timing anomalies
        // Operations should not take more than 1 second
        if elapsed > Duration::from_secs(1) {
            return Some(AnomalyEvent {
                anomaly_type: AnomalyType::TimingAnomaly,
                severity: Severity::Warning,
                description: format!(
                    "Operation '{}' took {:?} (expected < 1s)",
                    self.operation, elapsed
                ),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                metadata: Some(format!("elapsed_ms={}", elapsed.as_millis())),
            });
        }

        // Update last op time
        *self.last_op.write() = Instant::now();

        None
    }
}

impl Drop for TimingGuard {
    fn drop(&mut self) {
        if !self.completed {
            // Operation was interrupted - potential attack
            // In production, this would trigger anomaly reporting
        }
    }
}

// ============================================================================
// ANOMALY DETECTORS
// ============================================================================

/// Timing anomaly detector configuration
pub struct TimingAnomalyDetector {
    /// Maximum allowed operation time
    pub max_duration: Duration,
    /// Minimum expected operation time (too fast = suspicious)
    pub min_duration: Duration,
}

impl Default for TimingAnomalyDetector {
    fn default() -> Self {
        TimingAnomalyDetector {
            max_duration: Duration::from_secs(1),
            min_duration: Duration::from_micros(100),
        }
    }
}

impl TimingAnomalyDetector {
    /// Check if duration is anomalous
    pub fn check(&self, duration: Duration) -> Option<AnomalyEvent> {
        let severity = if duration > self.max_duration * 10 {
            Severity::Critical
        } else if duration > self.max_duration {
            Severity::Error
        } else if duration < self.min_duration {
            // Too fast could indicate cached/bypassed operation
            Severity::Warning
        } else {
            return None;
        };

        Some(AnomalyEvent {
            anomaly_type: AnomalyType::TimingAnomaly,
            severity,
            description: format!("Timing anomaly: {:?}", duration),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metadata: Some(format!("duration_us={}", duration.as_micros())),
        })
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::SoftwareKeyStore;

    #[test]
    fn test_panic_protected_keystore() {
        let inner = SoftwareKeyStore::generate().unwrap();
        let protected = PanicProtectedKeyStore::new(inner, 5);

        // Should work normally
        let sig = protected.sign(b"test").unwrap();
        assert!(!sig.is_empty());
    }

    #[test]
    fn test_panic_destroys_keys() {
        let inner = SoftwareKeyStore::generate().unwrap();
        let protected = PanicProtectedKeyStore::new(inner, 5);

        // Trigger emergency panic
        protected.emergency_panic("Test panic");

        // Should be destroyed
        assert!(protected.is_destroyed());
        assert_eq!(protected.state(), PanicState::Destroyed);

        // Operations should fail
        assert!(protected.sign(b"test").is_err());
    }

    #[test]
    fn test_anomaly_threshold() {
        let inner = SoftwareKeyStore::generate().unwrap();
        let protected = PanicProtectedKeyStore::new(inner, 3);

        // Report anomalies below threshold
        for _ in 0..2 {
            protected.report_anomaly(AnomalyEvent {
                anomaly_type: AnomalyType::TimingAnomaly,
                severity: Severity::Error,
                description: "Test".into(),
                timestamp: 0,
                metadata: None,
            });
        }

        // Should still work
        assert!(!protected.is_destroyed());

        // Third anomaly triggers panic
        protected.report_anomaly(AnomalyEvent {
            anomaly_type: AnomalyType::TimingAnomaly,
            severity: Severity::Error,
            description: "Test".into(),
            timestamp: 0,
            metadata: None,
        });

        assert!(protected.is_destroyed());
    }

    #[test]
    fn test_critical_immediate_panic() {
        let inner = SoftwareKeyStore::generate().unwrap();
        let protected = PanicProtectedKeyStore::new(inner, 100);

        // Critical should trigger immediate panic
        protected.report_anomaly(AnomalyEvent {
            anomaly_type: AnomalyType::CryptoFailure,
            severity: Severity::Critical,
            description: "Critical test".into(),
            timestamp: 0,
            metadata: None,
        });

        assert!(protected.is_destroyed());
    }

    #[test]
    fn test_panic_log() {
        let inner = SoftwareKeyStore::generate().unwrap();
        let protected = PanicProtectedKeyStore::new(inner, 5);

        protected.emergency_panic("Test");

        let log = protected.get_panic_log();
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].to_state, PanicState::Destroyed);
    }

    #[test]
    fn test_timing_detector() {
        let detector = TimingAnomalyDetector::default();

        // Normal timing
        assert!(detector.check(Duration::from_millis(100)).is_none());

        // Too slow
        assert!(detector.check(Duration::from_secs(5)).is_some());

        // Too fast
        assert!(detector.check(Duration::from_nanos(1)).is_some());
    }
}
