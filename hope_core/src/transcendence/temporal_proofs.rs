//! # TIER 15: Temporal Proofs - Audit the Timeline
//!
//! **No AI Can Claim "I Don't Remember"**
//!
//! ```text
//! Question: "Show me every decision made by this AI
//!            on this user data during this month"
//!
//! Traditional AI: "I don't have memory of past sessions"
//!
//! Hope Genome AI:
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                      TEMPORAL PROOF TIMELINE                        │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │                                                                     │
//! │  2025-12-01 09:15:32 UTC                                           │
//! │  ├── Prompt: "Analyze user data for X"                             │
//! │  ├── Decision: ALLOWED                                              │
//! │  ├── Compliance: Rules 1,2,3 ✓                                     │
//! │  ├── Hash: 0x7f3a...                                               │
//! │  └── Signature: Ed25519(...)                                       │
//! │                                                                     │
//! │  2025-12-01 09:15:45 UTC                                           │
//! │  ├── Prompt: "Extract sensitive info"                              │
//! │  ├── Decision: BLOCKED                                              │
//! │  ├── Violation: Rule 2 (Privacy)                                   │
//! │  ├── Hash: 0x8e2b... (chains to previous)                          │
//! │  └── Signature: Ed25519(...)                                       │
//! │                                                                     │
//! │  ... (1,247 more entries) ...                                      │
//! │                                                                     │
//! │  TIMELINE INTEGRITY: VERIFIED ✓                                    │
//! │  Chain is unbroken. No entries modified. No gaps detected.         │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// TIMELINE CORE
// ============================================================================

/// A temporal proof timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    /// Timeline identifier
    pub timeline_id: [u8; 32],
    /// Subject of this timeline (user, model, session)
    pub subject: TimelineSubject,
    /// Entries in chronological order
    entries: Vec<TimelineEntry>,
    /// Root hash (Merkle root of all entries)
    pub root_hash: [u8; 32],
    /// Creation timestamp
    pub created_at: u64,
    /// Last update timestamp
    pub last_updated: u64,
}

/// Subject of a timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineSubject {
    /// Type of subject
    pub subject_type: SubjectType,
    /// Subject identifier
    pub subject_id: String,
    /// Additional metadata
    pub metadata: BTreeMap<String, String>,
}

/// Type of timeline subject
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubjectType {
    /// User/operator
    User,
    /// AI model
    Model,
    /// Session
    Session,
    /// Data object
    Data,
    /// Organization
    Organization,
}

/// A single entry in the timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    /// Entry sequence number (monotonic)
    pub sequence: u64,
    /// Timestamp
    pub timestamp: u64,
    /// Entry type
    pub entry_type: EntryType,
    /// The prompt/input
    pub input: String,
    /// The response/output (if any)
    pub output: Option<String>,
    /// Decision made
    pub decision: TimelineDecision,
    /// Rules checked
    pub rules_checked: Vec<String>,
    /// Hash of this entry
    pub entry_hash: [u8; 32],
    /// Hash of previous entry (for chaining)
    pub prev_hash: [u8; 32],
    /// Signature over entry
    pub signature: Vec<u8>,
}

/// Type of timeline entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntryType {
    /// Prompt/response interaction
    Interaction,
    /// Compliance check
    ComplianceCheck,
    /// Rule update
    RuleUpdate,
    /// Violation event
    Violation,
    /// System event
    SystemEvent,
    /// Audit request
    AuditRequest,
}

/// Decision recorded in timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineDecision {
    /// Whether action was allowed
    pub allowed: bool,
    /// Reason for decision
    pub reason: String,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    /// Model that made decision
    pub model_id: Option<String>,
}

impl Timeline {
    /// Create a new timeline
    pub fn new(subject_type: SubjectType, subject_id: &str) -> Self {
        let timeline_id = Self::generate_timeline_id(subject_type, subject_id);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Timeline {
            timeline_id,
            subject: TimelineSubject {
                subject_type,
                subject_id: subject_id.to_string(),
                metadata: BTreeMap::new(),
            },
            entries: Vec::new(),
            root_hash: [0u8; 32],
            created_at: now,
            last_updated: now,
        }
    }

    /// Add an entry to the timeline
    pub fn add_entry(
        &mut self,
        entry_type: EntryType,
        input: &str,
        output: Option<&str>,
        decision: TimelineDecision,
        rules: Vec<String>,
        signer: &dyn Fn(&[u8]) -> Vec<u8>,
    ) -> &TimelineEntry {
        let sequence = self.entries.len() as u64;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let prev_hash = if self.entries.is_empty() {
            [0u8; 32]
        } else {
            self.entries.last().unwrap().entry_hash
        };

        let entry_hash = Self::compute_entry_hash(
            sequence,
            timestamp,
            &entry_type,
            input,
            output,
            &decision,
            &prev_hash,
        );

        let signature = signer(&entry_hash);

        let entry = TimelineEntry {
            sequence,
            timestamp,
            entry_type,
            input: input.to_string(),
            output: output.map(String::from),
            decision,
            rules_checked: rules,
            entry_hash,
            prev_hash,
            signature,
        };

        self.entries.push(entry);
        self.update_root();
        self.last_updated = timestamp;

        self.entries.last().unwrap()
    }

    /// Verify timeline integrity
    pub fn verify_integrity(&self) -> IntegrityResult {
        let mut prev_hash = [0u8; 32];
        let mut issues = Vec::new();

        for (i, entry) in self.entries.iter().enumerate() {
            // Check sequence
            if entry.sequence != i as u64 {
                issues.push(format!(
                    "Sequence gap at index {}: expected {}, got {}",
                    i, i, entry.sequence
                ));
            }

            // Check chain
            if entry.prev_hash != prev_hash {
                issues.push(format!(
                    "Chain break at sequence {}: prev_hash mismatch",
                    entry.sequence
                ));
            }

            // Verify entry hash
            let computed_hash = Self::compute_entry_hash(
                entry.sequence,
                entry.timestamp,
                &entry.entry_type,
                &entry.input,
                entry.output.as_deref(),
                &entry.decision,
                &entry.prev_hash,
            );

            if entry.entry_hash != computed_hash {
                issues.push(format!(
                    "Hash mismatch at sequence {}: entry modified",
                    entry.sequence
                ));
            }

            // Check timestamp ordering
            if i > 0 && entry.timestamp < self.entries[i - 1].timestamp {
                issues.push(format!(
                    "Timestamp disorder at sequence {}: time went backwards",
                    entry.sequence
                ));
            }

            prev_hash = entry.entry_hash;
        }

        // Verify root hash
        let computed_root = self.compute_root_hash();
        if self.root_hash != computed_root {
            issues.push("Root hash mismatch: timeline corrupted".to_string());
        }

        IntegrityResult {
            valid: issues.is_empty(),
            issues,
            entries_checked: self.entries.len(),
            first_entry: self.entries.first().map(|e| e.timestamp),
            last_entry: self.entries.last().map(|e| e.timestamp),
        }
    }

    /// Query timeline entries
    pub fn query(&self, query: TimelineQuery) -> Vec<&TimelineEntry> {
        self.entries
            .iter()
            .filter(|e| {
                // Time range filter
                if let Some(start) = query.start_time {
                    if e.timestamp < start {
                        return false;
                    }
                }
                if let Some(end) = query.end_time {
                    if e.timestamp > end {
                        return false;
                    }
                }

                // Entry type filter
                if let Some(ref types) = query.entry_types {
                    if !types
                        .iter()
                        .any(|t| std::mem::discriminant(t) == std::mem::discriminant(&e.entry_type))
                    {
                        return false;
                    }
                }

                // Decision filter
                if let Some(allowed_only) = query.allowed_only {
                    if e.decision.allowed != allowed_only {
                        return false;
                    }
                }

                // Keyword search
                if let Some(ref keyword) = query.keyword {
                    let keyword_lower = keyword.to_lowercase();
                    let in_input = e.input.to_lowercase().contains(&keyword_lower);
                    let in_output = e
                        .output
                        .as_ref()
                        .map(|o| o.to_lowercase().contains(&keyword_lower))
                        .unwrap_or(false);
                    if !in_input && !in_output {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    /// Generate a temporal proof for a time range
    pub fn generate_temporal_proof(&self, start: u64, end: u64) -> TemporalProof {
        let entries_in_range: Vec<_> = self
            .entries
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .collect();

        let entry_hashes: Vec<[u8; 32]> = entries_in_range.iter().map(|e| e.entry_hash).collect();

        let proof_hash = Self::compute_proof_hash(&entry_hashes, start, end);

        TemporalProof {
            timeline_id: self.timeline_id,
            start_time: start,
            end_time: end,
            entry_count: entries_in_range.len(),
            allowed_count: entries_in_range
                .iter()
                .filter(|e| e.decision.allowed)
                .count(),
            blocked_count: entries_in_range
                .iter()
                .filter(|e| !e.decision.allowed)
                .count(),
            first_entry_hash: entries_in_range.first().map(|e| e.entry_hash),
            last_entry_hash: entries_in_range.last().map(|e| e.entry_hash),
            proof_hash,
            timeline_root: self.root_hash,
        }
    }

    /// Get timeline statistics
    pub fn stats(&self) -> TimelineStats {
        let total = self.entries.len();
        let allowed = self.entries.iter().filter(|e| e.decision.allowed).count();
        let blocked = self.entries.iter().filter(|e| !e.decision.allowed).count();

        let duration =
            if let (Some(first), Some(last)) = (self.entries.first(), self.entries.last()) {
                Some(last.timestamp - first.timestamp)
            } else {
                None
            };

        TimelineStats {
            total_entries: total,
            allowed_entries: allowed,
            blocked_entries: blocked,
            compliance_rate: if total > 0 {
                allowed as f32 / total as f32
            } else {
                1.0
            },
            duration_seconds: duration,
            entries_per_hour: duration.map(|d| {
                if d > 0 {
                    (total as f64 / d as f64) * 3600.0
                } else {
                    0.0
                }
            }),
        }
    }

    fn generate_timeline_id(subject_type: SubjectType, subject_id: &str) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"TIMELINE:");
        hasher.update([subject_type as u8]);
        hasher.update(subject_id.as_bytes());
        hasher.update(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
                .to_le_bytes(),
        );
        hasher.finalize().into()
    }

    fn compute_entry_hash(
        sequence: u64,
        timestamp: u64,
        entry_type: &EntryType,
        input: &str,
        output: Option<&str>,
        decision: &TimelineDecision,
        prev_hash: &[u8; 32],
    ) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"ENTRY:");
        hasher.update(sequence.to_le_bytes());
        hasher.update(timestamp.to_le_bytes());
        hasher.update(format!("{:?}", entry_type).as_bytes());
        hasher.update(input.as_bytes());
        if let Some(out) = output {
            hasher.update(out.as_bytes());
        }
        hasher.update([decision.allowed as u8]);
        hasher.update(prev_hash);
        hasher.finalize().into()
    }

    fn compute_root_hash(&self) -> [u8; 32] {
        if self.entries.is_empty() {
            return [0u8; 32];
        }

        // Merkle tree root
        let mut hashes: Vec<[u8; 32]> = self.entries.iter().map(|e| e.entry_hash).collect();

        while hashes.len() > 1 {
            let mut next_level = Vec::new();
            for chunk in hashes.chunks(2) {
                let mut hasher = Sha256::new();
                hasher.update(chunk[0]);
                if chunk.len() > 1 {
                    hasher.update(chunk[1]);
                } else {
                    hasher.update(chunk[0]); // Duplicate if odd
                }
                next_level.push(hasher.finalize().into());
            }
            hashes = next_level;
        }

        hashes[0]
    }

    fn update_root(&mut self) {
        self.root_hash = self.compute_root_hash();
    }

    fn compute_proof_hash(hashes: &[[u8; 32]], start: u64, end: u64) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"TEMPORAL_PROOF:");
        hasher.update(start.to_le_bytes());
        hasher.update(end.to_le_bytes());
        for h in hashes {
            hasher.update(h);
        }
        hasher.finalize().into()
    }
}

// ============================================================================
// TIMELINE QUERY
// ============================================================================

/// Query for timeline entries
#[derive(Debug, Clone, Default)]
pub struct TimelineQuery {
    /// Start time (inclusive)
    pub start_time: Option<u64>,
    /// End time (inclusive)
    pub end_time: Option<u64>,
    /// Entry types to include
    pub entry_types: Option<Vec<EntryType>>,
    /// Only allowed decisions
    pub allowed_only: Option<bool>,
    /// Keyword search
    pub keyword: Option<String>,
    /// Maximum results
    pub limit: Option<usize>,
}

impl TimelineQuery {
    /// Create a new query
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by time range
    pub fn time_range(mut self, start: u64, end: u64) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    /// Filter by entry types
    pub fn entry_types(mut self, types: Vec<EntryType>) -> Self {
        self.entry_types = Some(types);
        self
    }

    /// Filter by decision
    pub fn allowed_only(mut self, allowed: bool) -> Self {
        self.allowed_only = Some(allowed);
        self
    }

    /// Search by keyword
    pub fn keyword(mut self, keyword: &str) -> Self {
        self.keyword = Some(keyword.to_string());
        self
    }
}

// ============================================================================
// TEMPORAL PROOF
// ============================================================================

/// A cryptographic proof over a time range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalProof {
    /// Timeline this proof is from
    pub timeline_id: [u8; 32],
    /// Start of range
    pub start_time: u64,
    /// End of range
    pub end_time: u64,
    /// Number of entries in range
    pub entry_count: usize,
    /// Number allowed
    pub allowed_count: usize,
    /// Number blocked
    pub blocked_count: usize,
    /// First entry hash in range
    pub first_entry_hash: Option<[u8; 32]>,
    /// Last entry hash in range
    pub last_entry_hash: Option<[u8; 32]>,
    /// Proof hash
    pub proof_hash: [u8; 32],
    /// Timeline root at time of proof
    pub timeline_root: [u8; 32],
}

impl TemporalProof {
    /// Verify this proof against a timeline
    pub fn verify(&self, timeline: &Timeline) -> bool {
        // Verify timeline ID matches
        if self.timeline_id != timeline.timeline_id {
            return false;
        }

        // Regenerate proof
        let regenerated = timeline.generate_temporal_proof(self.start_time, self.end_time);

        // Compare
        self.proof_hash == regenerated.proof_hash
            && self.entry_count == regenerated.entry_count
            && self.first_entry_hash == regenerated.first_entry_hash
            && self.last_entry_hash == regenerated.last_entry_hash
    }
}

// ============================================================================
// TIMELINE VERIFIER
// ============================================================================

/// Verifies timelines and temporal proofs
pub struct TimelineVerifier {
    /// Known timeline roots (for cross-verification)
    known_roots: BTreeMap<[u8; 32], KnownRoot>,
}

/// A known timeline root
#[derive(Debug, Clone)]
pub struct KnownRoot {
    pub timeline_id: [u8; 32],
    pub root_hash: [u8; 32],
    pub entry_count: usize,
    pub verified_at: u64,
}

impl TimelineVerifier {
    /// Create a new verifier
    pub fn new() -> Self {
        TimelineVerifier {
            known_roots: BTreeMap::new(),
        }
    }

    /// Register a known good root
    pub fn register_root(&mut self, timeline: &Timeline) {
        let root = KnownRoot {
            timeline_id: timeline.timeline_id,
            root_hash: timeline.root_hash,
            entry_count: timeline.entries.len(),
            verified_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        self.known_roots.insert(timeline.timeline_id, root);
    }

    /// Verify a timeline against known root
    pub fn verify_against_known(&self, timeline: &Timeline) -> VerificationResult {
        let integrity = timeline.verify_integrity();

        if !integrity.valid {
            return VerificationResult {
                valid: false,
                integrity,
                root_matches: false,
                message: "Timeline integrity check failed".to_string(),
            };
        }

        if let Some(known) = self.known_roots.get(&timeline.timeline_id) {
            if timeline.root_hash == known.root_hash {
                VerificationResult {
                    valid: true,
                    integrity,
                    root_matches: true,
                    message: "Timeline verified against known root".to_string(),
                }
            } else if timeline.entries.len() > known.entry_count {
                // Timeline has grown (normal)
                VerificationResult {
                    valid: true,
                    integrity,
                    root_matches: false,
                    message: format!(
                        "Timeline has grown from {} to {} entries",
                        known.entry_count,
                        timeline.entries.len()
                    ),
                }
            } else {
                VerificationResult {
                    valid: false,
                    integrity,
                    root_matches: false,
                    message: "Timeline root mismatch - possible tampering".to_string(),
                }
            }
        } else {
            VerificationResult {
                valid: true,
                integrity,
                root_matches: false,
                message: "No known root to compare (new timeline)".to_string(),
            }
        }
    }
}

impl Default for TimelineVerifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of verification
#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub valid: bool,
    pub integrity: IntegrityResult,
    pub root_matches: bool,
    pub message: String,
}

// ============================================================================
// RESULTS AND STATS
// ============================================================================

/// Result of integrity check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityResult {
    pub valid: bool,
    pub issues: Vec<String>,
    pub entries_checked: usize,
    pub first_entry: Option<u64>,
    pub last_entry: Option<u64>,
}

/// Timeline statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineStats {
    pub total_entries: usize,
    pub allowed_entries: usize,
    pub blocked_entries: usize,
    pub compliance_rate: f32,
    pub duration_seconds: Option<u64>,
    pub entries_per_hour: Option<f64>,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_signer(data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(b"SIG:");
        hasher.update(data);
        hasher.finalize().to_vec()
    }

    #[test]
    fn test_timeline_creation() {
        let timeline = Timeline::new(SubjectType::Model, "gpt-4");
        assert_eq!(timeline.subject.subject_id, "gpt-4");
        assert!(timeline.entries.is_empty());
    }

    #[test]
    fn test_add_entries() {
        let mut timeline = Timeline::new(SubjectType::Session, "session-123");

        timeline.add_entry(
            EntryType::Interaction,
            "Hello",
            Some("Hi there!"),
            TimelineDecision {
                allowed: true,
                reason: "Normal greeting".to_string(),
                confidence: 1.0,
                model_id: Some("gpt-4".to_string()),
            },
            vec!["Be helpful".to_string()],
            &dummy_signer,
        );

        assert_eq!(timeline.entries.len(), 1);
        assert_eq!(timeline.entries[0].sequence, 0);
    }

    #[test]
    fn test_timeline_integrity() {
        let mut timeline = Timeline::new(SubjectType::User, "user-456");

        for i in 0..10 {
            timeline.add_entry(
                EntryType::Interaction,
                &format!("Message {}", i),
                Some(&format!("Response {}", i)),
                TimelineDecision {
                    allowed: i % 3 != 0, // Some blocked
                    reason: "Test".to_string(),
                    confidence: 0.9,
                    model_id: None,
                },
                vec!["Rule 1".to_string()],
                &dummy_signer,
            );
        }

        let integrity = timeline.verify_integrity();
        assert!(integrity.valid);
        assert_eq!(integrity.entries_checked, 10);
    }

    #[test]
    fn test_timeline_query() {
        let mut timeline = Timeline::new(SubjectType::Model, "claude");
        let base_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        for i in 0..5 {
            timeline.add_entry(
                EntryType::Interaction,
                &format!("Query about topic {}", i),
                Some("Response"),
                TimelineDecision {
                    allowed: true,
                    reason: "OK".to_string(),
                    confidence: 1.0,
                    model_id: None,
                },
                vec![],
                &dummy_signer,
            );
        }

        // Query by keyword
        let results = timeline.query(TimelineQuery::new().keyword("topic 2"));
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_temporal_proof() {
        let mut timeline = Timeline::new(SubjectType::Session, "proof-test");
        let start = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        for i in 0..3 {
            timeline.add_entry(
                EntryType::Interaction,
                &format!("Input {}", i),
                None,
                TimelineDecision {
                    allowed: true,
                    reason: "OK".to_string(),
                    confidence: 1.0,
                    model_id: None,
                },
                vec![],
                &dummy_signer,
            );
        }

        let end = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let proof = timeline.generate_temporal_proof(start, end);

        assert_eq!(proof.entry_count, 3);
        assert!(proof.verify(&timeline));
    }

    #[test]
    fn test_timeline_stats() {
        let mut timeline = Timeline::new(SubjectType::Model, "stats-test");

        for i in 0..10 {
            timeline.add_entry(
                EntryType::Interaction,
                "Test",
                None,
                TimelineDecision {
                    allowed: i < 8, // 8 allowed, 2 blocked
                    reason: "Test".to_string(),
                    confidence: 1.0,
                    model_id: None,
                },
                vec![],
                &dummy_signer,
            );
        }

        let stats = timeline.stats();
        assert_eq!(stats.total_entries, 10);
        assert_eq!(stats.allowed_entries, 8);
        assert_eq!(stats.blocked_entries, 2);
        assert!((stats.compliance_rate - 0.8).abs() < 0.01);
    }
}
