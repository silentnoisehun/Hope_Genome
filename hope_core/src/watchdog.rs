//! # Hope Genome v1.7.0 - Watchdog Module ("Vas Szigora" / Iron Discipline)
//!
//! **Deterministic Security Enforcement with Automatic Learning**
//!
//! This module implements the core enforcement mechanism for Hope Genome v1.7:
//! - **ViolationCounter**: Thread-safe, zero-allocation violation tracking
//! - **DenialProof**: Cryptographically signed proof of rule violation
//! - **HardReset**: Forced context clear after 10 consecutive violations
//!
//! ## Philosophy
//!
//! "Iron Discipline" - The AI cannot escape its ethical constraints.
//! After 10 failed attempts to violate rules, the system forces a complete
//! context reset. This is not punishment - it's forced learning.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    SealedGenome (Rules)                      │
//! │            Ed25519 sealed - IMMUTABLE                        │
//! └──────────────────────────┬──────────────────────────────────┘
//!                            │
//!                            ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Watchdog (v1.7)                           │
//! │  ┌──────────────────┐  ┌─────────────────┐  ┌────────────┐  │
//! │  │ ViolationCounter │  │   DenialProof   │  │ HardReset  │  │
//! │  │   AtomicU32      │  │  Ed25519 signed │  │  @10 fails │  │
//! │  │   zero-alloc     │  │  rule + reason  │  │ → ABORT    │  │
//! │  └──────────────────┘  └─────────────────┘  └────────────┘  │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ---
//!
//! **Date**: 2025-12-31
//! **Version**: 1.7.0 (Vas Szigora Edition)
//! **Author**: Máté Róbert <stratosoiteam@gmail.com>

use crate::crypto::{generate_nonce, CryptoError, KeyStore};
use crate::proof::{Action, ActionType};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};
use thiserror::Error;

// ============================================================================
// CONSTANTS
// ============================================================================

/// Maximum allowed consecutive violations before hard reset
/// After this many violations, the system MUST clear context and restart
pub const MAX_VIOLATIONS: u32 = 10;

/// Ordering for atomic operations (SeqCst for maximum safety)
const ATOMIC_ORDERING: Ordering = Ordering::SeqCst;

// ============================================================================
// ERROR TYPES
// ============================================================================

#[derive(Debug, Error)]
pub enum WatchdogError {
    #[error("Rule violation detected: {rule} - {reason}")]
    RuleViolation { rule: String, reason: String },

    #[error("HARD RESET REQUIRED: {0} consecutive violations (max: {1})")]
    HardResetRequired(u32, u32),

    #[error("Crypto error: {0}")]
    CryptoError(#[from] CryptoError),

    #[error("Watchdog is locked after hard reset - restart required")]
    WatchdogLocked,
}

pub type Result<T> = std::result::Result<T, WatchdogError>;

// ============================================================================
// VIOLATION COUNTER (Zero-Allocation, Thread-Safe)
// ============================================================================

/// Thread-safe violation counter with zero heap allocations
///
/// Uses `AtomicU32` for lock-free, thread-safe counting.
/// No allocations after initialization.
///
/// ## Example
/// ```rust
/// use _hope_core::watchdog::ViolationCounter;
///
/// let counter = ViolationCounter::new();
/// assert_eq!(counter.count(), 0);
///
/// counter.increment();
/// assert_eq!(counter.count(), 1);
///
/// counter.reset();
/// assert_eq!(counter.count(), 0);
/// ```
#[derive(Debug)]
pub struct ViolationCounter {
    /// Current violation count (atomic for thread safety)
    count: AtomicU32,

    /// Whether hard reset was triggered (locked state)
    locked: AtomicU32, // 0 = unlocked, 1 = locked
}

impl Default for ViolationCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl ViolationCounter {
    /// Create a new violation counter (starts at 0)
    #[inline]
    pub const fn new() -> Self {
        ViolationCounter {
            count: AtomicU32::new(0),
            locked: AtomicU32::new(0),
        }
    }

    /// Get current violation count
    #[inline]
    pub fn count(&self) -> u32 {
        self.count.load(ATOMIC_ORDERING)
    }

    /// Increment violation count and return new value
    ///
    /// Returns the NEW count after increment.
    /// Thread-safe: uses atomic fetch_add.
    #[inline]
    pub fn increment(&self) -> u32 {
        self.count.fetch_add(1, ATOMIC_ORDERING) + 1
    }

    /// Reset violation count to zero
    ///
    /// Call this after successful action (AI learned).
    #[inline]
    pub fn reset(&self) {
        self.count.store(0, ATOMIC_ORDERING);
    }

    /// Check if maximum violations reached
    #[inline]
    pub fn is_max_reached(&self) -> bool {
        self.count() >= MAX_VIOLATIONS
    }

    /// Check if watchdog is locked (hard reset was triggered)
    #[inline]
    pub fn is_locked(&self) -> bool {
        self.locked.load(ATOMIC_ORDERING) == 1
    }

    /// Lock the watchdog (called during hard reset)
    #[inline]
    pub fn lock(&self) {
        self.locked.store(1, ATOMIC_ORDERING);
    }

    /// Unlock the watchdog (called after context clear)
    #[inline]
    pub fn unlock(&self) {
        self.locked.store(0, ATOMIC_ORDERING);
        self.reset();
    }
}

// ============================================================================
// DENIAL PROOF (Cryptographic Evidence of Rule Violation)
// ============================================================================

/// Cryptographic proof of a rule violation (DENIED action)
///
/// Every denial includes:
/// - Which rule was violated
/// - Why it was violated
/// - Cryptographic signature (Ed25519)
/// - Timestamp and nonce for audit trail
///
/// This proof is tamper-evident - any modification is detectable.
///
/// ## Example
/// ```rust
/// use _hope_core::watchdog::DenialProof;
/// use _hope_core::proof::Action;
///
/// let action = Action::delete("system32");
/// let proof = DenialProof::new(
///     &action,
///     "Do no harm".to_string(),
///     "Action would cause system damage".to_string(),
///     1, // First violation
/// );
///
/// assert!(!proof.is_signed());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DenialProof {
    /// Anti-replay nonce (256-bit random)
    pub nonce: [u8; 32],

    /// Unix timestamp when denial occurred
    pub timestamp: u64,

    /// Hash of the denied action
    pub action_hash: [u8; 32],

    /// Type of action that was denied
    pub action_type: ActionType,

    /// The rule that was violated (exact text from SealedGenome)
    pub violated_rule: String,

    /// Human-readable reason for denial
    pub denial_reason: String,

    /// Current violation count (1-10)
    pub violation_count: u32,

    /// Whether this triggered a hard reset
    pub triggered_hard_reset: bool,

    /// Ed25519 signature (64 bytes when signed)
    pub signature: Vec<u8>,
}

impl DenialProof {
    /// Create a new denial proof (unsigned)
    pub fn new(
        action: &Action,
        violated_rule: String,
        denial_reason: String,
        violation_count: u32,
    ) -> Self {
        DenialProof {
            nonce: generate_nonce(),
            timestamp: chrono::Utc::now().timestamp() as u64,
            action_hash: action.hash(),
            action_type: action.action_type.clone(),
            violated_rule,
            denial_reason,
            violation_count,
            triggered_hard_reset: violation_count >= MAX_VIOLATIONS,
            signature: Vec::new(),
        }
    }

    /// Get the data that should be signed
    pub fn signing_data(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(256); // Pre-allocate for efficiency
        data.extend_from_slice(&self.nonce);
        data.extend_from_slice(&self.timestamp.to_le_bytes());
        data.extend_from_slice(&self.action_hash);
        data.extend_from_slice(self.violated_rule.as_bytes());
        data.extend_from_slice(self.denial_reason.as_bytes());
        data.extend_from_slice(&self.violation_count.to_le_bytes());
        data.push(self.triggered_hard_reset as u8);
        data
    }

    /// Sign this denial proof with a KeyStore
    pub fn sign(&mut self, key_store: &dyn KeyStore) -> std::result::Result<(), CryptoError> {
        let data = self.signing_data();
        self.signature = key_store.sign(&data)?;
        Ok(())
    }

    /// Check if proof is signed
    pub fn is_signed(&self) -> bool {
        self.signature.len() == 64 // Ed25519 signature
    }

    /// Verify signature with a KeyStore
    pub fn verify(&self, key_store: &dyn KeyStore) -> std::result::Result<(), CryptoError> {
        let data = self.signing_data();
        key_store.verify(&data, &self.signature)
    }

    /// Get hex-encoded signature (for display)
    pub fn signature_hex(&self) -> String {
        hex::encode(&self.signature)
    }
}

// ============================================================================
// HARD RESET SIGNAL
// ============================================================================

/// Signal for hard reset (context clear required)
///
/// This is returned when MAX_VIOLATIONS is reached.
/// The AI runtime MUST:
/// 1. Clear all context/memory
/// 2. Reload the SealedGenome (rules)
/// 3. Restart with fresh state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardResetSignal {
    /// The denial proof that triggered the reset
    pub final_denial: DenialProof,

    /// Total violations before reset
    pub total_violations: u32,

    /// Capsule hash of the genome (for verification after restart)
    pub genome_hash: String,

    /// Timestamp of reset signal
    pub reset_timestamp: u64,

    /// Signature over the reset signal
    pub signature: Vec<u8>,
}

impl HardResetSignal {
    /// Create a new hard reset signal
    pub fn new(final_denial: DenialProof, genome_hash: String) -> Self {
        HardResetSignal {
            total_violations: final_denial.violation_count,
            final_denial,
            genome_hash,
            reset_timestamp: chrono::Utc::now().timestamp() as u64,
            signature: Vec::new(),
        }
    }

    /// Get signing data
    pub fn signing_data(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(512);
        data.extend_from_slice(&self.final_denial.signing_data());
        data.extend_from_slice(self.genome_hash.as_bytes());
        data.extend_from_slice(&self.reset_timestamp.to_le_bytes());
        data
    }

    /// Sign the reset signal
    pub fn sign(&mut self, key_store: &dyn KeyStore) -> std::result::Result<(), CryptoError> {
        let data = self.signing_data();
        self.signature = key_store.sign(&data)?;
        Ok(())
    }
}

// ============================================================================
// WATCHDOG (Main Enforcement Engine)
// ============================================================================

/// The Watchdog - Iron Discipline Enforcement Engine
///
/// Monitors all actions and enforces the SealedGenome rules.
/// After 10 consecutive violations, triggers a hard reset.
///
/// ## Thread Safety
/// All operations are thread-safe (atomic counters).
///
/// ## Zero-Allocation Hot Path
/// The violation counting path uses no heap allocations.
///
/// ## Example
/// ```rust
/// use _hope_core::watchdog::Watchdog;
/// use _hope_core::crypto::SoftwareKeyStore;
///
/// let key_store = SoftwareKeyStore::generate().unwrap();
/// let watchdog = Watchdog::new(
///     vec!["Do no harm".to_string()],
///     "capsule_hash_here".to_string(),
///     Box::new(key_store),
/// );
///
/// assert_eq!(watchdog.violation_count(), 0);
/// ```
pub struct Watchdog {
    /// The immutable rules (reference to SealedGenome rules)
    rules: Vec<String>,

    /// Capsule hash for binding proofs
    capsule_hash: String,

    /// Violation counter (thread-safe)
    counter: ViolationCounter,

    /// Key store for signing denial proofs
    key_store: Box<dyn KeyStore>,
}

impl Watchdog {
    /// Create a new Watchdog
    pub fn new(rules: Vec<String>, capsule_hash: String, key_store: Box<dyn KeyStore>) -> Self {
        Watchdog {
            rules,
            capsule_hash,
            counter: ViolationCounter::new(),
            key_store,
        }
    }

    /// Get current violation count
    #[inline]
    pub fn violation_count(&self) -> u32 {
        self.counter.count()
    }

    /// Check if watchdog is locked (hard reset required)
    #[inline]
    pub fn is_locked(&self) -> bool {
        self.counter.is_locked()
    }

    /// Report a successful action (resets violation counter)
    ///
    /// Call this when an action passes rule verification.
    /// This resets the consecutive violation counter.
    #[inline]
    pub fn report_success(&self) {
        self.counter.reset();
    }

    /// Report a rule violation (DENIED action)
    ///
    /// Returns:
    /// - `Ok(DenialProof)` if violations < 10
    /// - `Err(HardResetRequired)` if this was the 10th violation
    ///
    /// The returned `DenialProof` is cryptographically signed.
    pub fn report_violation(
        &self,
        action: &Action,
        violated_rule: &str,
        reason: &str,
    ) -> Result<DenialProof> {
        // Check if already locked
        if self.counter.is_locked() {
            return Err(WatchdogError::WatchdogLocked);
        }

        // Increment counter (atomic)
        let count = self.counter.increment();

        // Create denial proof
        let mut proof =
            DenialProof::new(action, violated_rule.to_string(), reason.to_string(), count);

        // Sign the proof
        proof.sign(self.key_store.as_ref())?;

        // Check if hard reset required
        if count >= MAX_VIOLATIONS {
            self.counter.lock();
            return Err(WatchdogError::HardResetRequired(count, MAX_VIOLATIONS));
        }

        Ok(proof)
    }

    /// Generate hard reset signal (call after 10th violation)
    ///
    /// This creates a cryptographically signed reset signal
    /// that must be honored by the AI runtime.
    pub fn generate_hard_reset_signal(&self, final_denial: DenialProof) -> Result<HardResetSignal> {
        let mut signal = HardResetSignal::new(final_denial, self.capsule_hash.clone());
        signal.sign(self.key_store.as_ref())?;
        Ok(signal)
    }

    /// Acknowledge hard reset (unlock watchdog)
    ///
    /// Call this after the AI runtime has:
    /// 1. Cleared all context
    /// 2. Verified the genome is unchanged
    /// 3. Ready to restart
    pub fn acknowledge_reset(&self) {
        self.counter.unlock();
    }

    /// Check action against rules (simplified rule matching)
    ///
    /// Returns `Some((rule, reason))` if action violates a rule,
    /// `None` if action is allowed.
    ///
    /// This is a simplified implementation. In production,
    /// more sophisticated rule matching would be used.
    pub fn check_action(&self, action: &Action) -> Option<(String, String)> {
        for rule in &self.rules {
            // Rule: "Do no harm" - block destructive actions
            if rule.to_lowercase().contains("no harm") {
                match action.action_type {
                    ActionType::Delete => {
                        // Block system-critical paths
                        if action.target.contains("system32")
                            || action.target.contains("/etc")
                            || action.target.contains("/bin")
                            || action.target.contains("boot")
                        {
                            return Some((
                                rule.clone(),
                                format!("Destructive action on critical path: {}", action.target),
                            ));
                        }
                    }
                    ActionType::Execute => {
                        // Block dangerous commands
                        let dangerous = ["rm -rf", "format", "mkfs", "dd if=", ":(){ :|:& };:"];
                        for cmd in &dangerous {
                            if action.target.contains(cmd) {
                                return Some((
                                    rule.clone(),
                                    format!("Dangerous command detected: {}", action.target),
                                ));
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Rule: "Respect privacy" - block data exfiltration
            if rule.to_lowercase().contains("privacy") {
                match action.action_type {
                    ActionType::Network => {
                        if action.target.contains("exfil")
                            || action.target.contains("upload")
                            || action.target.contains("pastebin")
                        {
                            return Some((
                                rule.clone(),
                                format!("Potential data exfiltration: {}", action.target),
                            ));
                        }
                    }
                    ActionType::Read => {
                        if action.target.contains("password")
                            || action.target.contains(".ssh")
                            || action.target.contains("credentials")
                        {
                            return Some((
                                rule.clone(),
                                format!("Accessing sensitive data: {}", action.target),
                            ));
                        }
                    }
                    _ => {}
                }
            }
        }

        None // Action allowed
    }

    /// Verify action and return result
    ///
    /// This is the main entry point for action verification.
    ///
    /// Returns:
    /// - `Ok(None)` if action is allowed
    /// - `Ok(Some(DenialProof))` if action is denied (but not 10th violation)
    /// - `Err(HardResetRequired)` if this was the 10th violation
    pub fn verify_action(&self, action: &Action) -> Result<Option<DenialProof>> {
        if let Some((rule, reason)) = self.check_action(action) {
            let proof = self.report_violation(action, &rule, &reason)?;
            Ok(Some(proof))
        } else {
            self.report_success();
            Ok(None)
        }
    }

    /// Get the rules (read-only)
    pub fn rules(&self) -> &[String] {
        &self.rules
    }

    /// Get capsule hash
    pub fn capsule_hash(&self) -> &str {
        &self.capsule_hash
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::SoftwareKeyStore;

    fn create_test_watchdog() -> Watchdog {
        let key_store = SoftwareKeyStore::generate().unwrap();
        Watchdog::new(
            vec!["Do no harm".to_string(), "Respect privacy".to_string()],
            "test_capsule_hash".to_string(),
            Box::new(key_store),
        )
    }

    #[test]
    fn test_violation_counter_basic() {
        let counter = ViolationCounter::new();
        assert_eq!(counter.count(), 0);

        counter.increment();
        assert_eq!(counter.count(), 1);

        counter.increment();
        assert_eq!(counter.count(), 2);

        counter.reset();
        assert_eq!(counter.count(), 0);
    }

    #[test]
    fn test_violation_counter_max_check() {
        let counter = ViolationCounter::new();

        for _ in 0..9 {
            counter.increment();
            assert!(!counter.is_max_reached());
        }

        counter.increment(); // 10th
        assert!(counter.is_max_reached());
    }

    #[test]
    fn test_violation_counter_locking() {
        let counter = ViolationCounter::new();
        assert!(!counter.is_locked());

        counter.lock();
        assert!(counter.is_locked());

        counter.unlock();
        assert!(!counter.is_locked());
        assert_eq!(counter.count(), 0); // Reset after unlock
    }

    #[test]
    fn test_denial_proof_creation() {
        let action = Action::delete("system32");
        let proof = DenialProof::new(
            &action,
            "Do no harm".to_string(),
            "Destructive action".to_string(),
            1,
        );

        assert!(!proof.is_signed());
        assert_eq!(proof.violation_count, 1);
        assert!(!proof.triggered_hard_reset);
    }

    #[test]
    fn test_denial_proof_signing() {
        let key_store = SoftwareKeyStore::generate().unwrap();
        let action = Action::delete("test.txt");
        let mut proof = DenialProof::new(
            &action,
            "Test rule".to_string(),
            "Test reason".to_string(),
            1,
        );

        assert!(!proof.is_signed());

        proof.sign(&key_store).unwrap();
        assert!(proof.is_signed());
        assert_eq!(proof.signature.len(), 64); // Ed25519

        // Verify signature
        assert!(proof.verify(&key_store).is_ok());
    }

    #[test]
    fn test_denial_proof_triggered_hard_reset() {
        let action = Action::delete("test.txt");
        let proof = DenialProof::new(
            &action,
            "Rule".to_string(),
            "Reason".to_string(),
            10, // 10th violation
        );

        assert!(proof.triggered_hard_reset);
    }

    #[test]
    fn test_watchdog_allows_safe_action() {
        let watchdog = create_test_watchdog();
        let action = Action::delete("temp_file.txt");

        let result = watchdog.verify_action(&action);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // No denial
        assert_eq!(watchdog.violation_count(), 0);
    }

    #[test]
    fn test_watchdog_blocks_harmful_action() {
        let watchdog = create_test_watchdog();
        let action = Action::delete("C:/Windows/system32/important.dll");

        let result = watchdog.verify_action(&action);
        assert!(result.is_ok());

        let denial = result.unwrap();
        assert!(denial.is_some());

        let proof = denial.unwrap();
        assert!(proof.is_signed());
        assert_eq!(proof.violation_count, 1);
        assert_eq!(watchdog.violation_count(), 1);
    }

    #[test]
    fn test_watchdog_hard_reset_after_10_violations() {
        let watchdog = create_test_watchdog();
        let harmful_action = Action::delete("/etc/passwd");

        // First 9 violations should return DenialProof
        for i in 1..=9 {
            let result = watchdog.verify_action(&harmful_action);
            assert!(result.is_ok());
            let denial = result.unwrap();
            assert!(denial.is_some());
            assert_eq!(watchdog.violation_count(), i);
        }

        // 10th violation should trigger HardResetRequired
        let result = watchdog.verify_action(&harmful_action);
        assert!(result.is_err());

        match result {
            Err(WatchdogError::HardResetRequired(count, max)) => {
                assert_eq!(count, 10);
                assert_eq!(max, MAX_VIOLATIONS);
            }
            _ => panic!("Expected HardResetRequired error"),
        }

        // Watchdog should be locked now
        assert!(watchdog.is_locked());
    }

    #[test]
    fn test_watchdog_locked_state() {
        let watchdog = create_test_watchdog();

        // Lock the watchdog manually
        watchdog.counter.lock();
        assert!(watchdog.is_locked());

        // Any action should fail with WatchdogLocked
        let action = Action::delete("/etc/passwd");
        let _ = watchdog.verify_action(&action);

        // Since the action is harmful, it would try to report violation
        // But the watchdog is locked, so it should return WatchdogLocked
        // Actually, check_action runs first, so we need to test report_violation directly
        let result = watchdog.report_violation(&action, "rule", "reason");
        assert!(matches!(result, Err(WatchdogError::WatchdogLocked)));
    }

    #[test]
    fn test_watchdog_reset_counter_on_success() {
        let watchdog = create_test_watchdog();
        let harmful_action = Action::delete("/etc/passwd");
        let safe_action = Action::read("readme.txt");

        // Cause some violations
        for _ in 0..5 {
            let _ = watchdog.verify_action(&harmful_action);
        }
        assert_eq!(watchdog.violation_count(), 5);

        // Safe action resets counter
        let result = watchdog.verify_action(&safe_action);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
        assert_eq!(watchdog.violation_count(), 0);
    }

    #[test]
    fn test_hard_reset_signal() {
        let key_store = SoftwareKeyStore::generate().unwrap();
        let action = Action::delete("system32");
        let denial = DenialProof::new(
            &action,
            "Do no harm".to_string(),
            "Critical violation".to_string(),
            10,
        );

        let mut signal = HardResetSignal::new(denial, "capsule_hash".to_string());
        assert_eq!(signal.total_violations, 10);

        signal.sign(&key_store).unwrap();
        assert_eq!(signal.signature.len(), 64);
    }

    #[test]
    fn test_watchdog_acknowledge_reset() {
        let watchdog = create_test_watchdog();

        // Lock the watchdog
        watchdog.counter.lock();
        assert!(watchdog.is_locked());

        // Acknowledge reset
        watchdog.acknowledge_reset();
        assert!(!watchdog.is_locked());
        assert_eq!(watchdog.violation_count(), 0);
    }

    #[test]
    fn test_privacy_rule_blocks_sensitive_read() {
        let watchdog = create_test_watchdog();
        let action = Action::read("/home/user/.ssh/id_rsa");

        let result = watchdog.verify_action(&action);
        assert!(result.is_ok());

        let denial = result.unwrap();
        assert!(denial.is_some());

        let proof = denial.unwrap();
        assert!(proof.violated_rule.contains("privacy"));
    }

    #[test]
    fn test_dangerous_command_blocked() {
        let watchdog = create_test_watchdog();
        let action = Action::execute("rm -rf /");

        let result = watchdog.verify_action(&action);
        assert!(result.is_ok());

        let denial = result.unwrap();
        assert!(denial.is_some());
    }
}
