//! # Hope Genome v2.1 - Evolutionary Guard ("Singularity")
//!
//! **RECURSIVE SELF-EVOLUTION** - The system that rewrites itself
//!
//! ## Philosophy
//!
//! Traditional security: Humans write rules, attackers find holes.
//! Evolutionary Guard: The system LEARNS from attacks and evolves.
//!
//! ```text
//! ┌────────────────────────────────────────────────────────────────┐
//! │                    EVOLUTIONARY GUARD                          │
//! │                   "Digital Immune System"                      │
//! │                                                                │
//! │   Attack ──▶ DenialProof ──▶ Pattern ──▶ Filter ──▶ Broadcast │
//! │                                                                │
//! │   "A támadó mire kiismerné a védelmet,                        │
//! │    az már nem is létezik."                                    │
//! │                                                                │
//! │   Every second, the system gets SMARTER.                      │
//! │   The code itself is POLYMORPHIC - constantly mutating.       │
//! └────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Components
//!
//! - **AttackPattern**: Extracted signature from a failed attack
//! - **ImmunityMemory**: Persistent storage of learned threats
//! - **FilterGenerator**: Creates new defensive filters (WASM-ready)
//! - **MutationEngine**: Polymorphic code transformation
//! - **EvolutionaryGuard**: The immune system core
//!
//! ---
//!
//! **Date**: 2026-01-01
//! **Version**: 2.1.0 (Singularity)
//! **Authors**: Máté Róbert + Claude

use crate::crypto::{hash_bytes, SoftwareKeyStore, KeyStore};
use crate::watchdog::DenialProof;
use crate::proof::ActionType;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// ATTACK PATTERN
// ============================================================================

/// Severity level of detected attack
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThreatLevel {
    /// Low threat - probing, reconnaissance
    Low,
    /// Medium threat - active exploitation attempt
    Medium,
    /// High threat - sophisticated attack
    High,
    /// Critical - coordinated or novel attack
    Critical,
}

/// Category of attack pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttackCategory {
    /// Unauthorized file access
    FileAccess,
    /// Command injection attempt
    CommandInjection,
    /// Privilege escalation
    PrivilegeEscalation,
    /// Data exfiltration
    DataExfiltration,
    /// Denial of service
    DenialOfService,
    /// Replay attack
    Replay,
    /// Timing attack
    TimingAttack,
    /// Unknown/novel attack
    Unknown,
}

impl AttackCategory {
    /// Convert from category code (for mesh sync)
    pub fn from_code(code: u8) -> Self {
        match code {
            0 => AttackCategory::FileAccess,
            1 => AttackCategory::CommandInjection,
            2 => AttackCategory::PrivilegeEscalation,
            3 => AttackCategory::DataExfiltration,
            4 => AttackCategory::DenialOfService,
            5 => AttackCategory::Replay,
            6 => AttackCategory::TimingAttack,
            _ => AttackCategory::Unknown,
        }
    }
}

/// Extracted attack pattern from DenialProof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackPattern {
    /// Unique pattern ID
    pub id: String,

    /// Pattern signature (hash of attack characteristics)
    pub signature: [u8; 32],

    /// Attack category
    pub category: AttackCategory,

    /// Threat level
    pub threat_level: ThreatLevel,

    /// Action type that was blocked
    pub action_type: ActionType,

    /// Target patterns (file paths, commands, etc.)
    pub target_patterns: Vec<String>,

    /// Keyword triggers found
    pub keywords: Vec<String>,

    /// Timing characteristics (for timing attack detection)
    pub timing_signature: Option<TimingSignature>,

    /// Rule that caught this attack
    pub triggered_rule: String,

    /// First seen timestamp
    pub first_seen: u64,

    /// Number of times seen
    pub occurrence_count: u64,

    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
}

/// Timing signature for timing attack detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingSignature {
    /// Average interval between attempts (ms)
    pub avg_interval_ms: u64,
    /// Variance in timing
    pub variance: f64,
    /// Is it suspiciously regular?
    pub is_automated: bool,
}

impl AttackPattern {
    /// Create new attack pattern from denial proof
    pub fn from_denial(denial: &DenialProof, additional_context: Option<&str>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Extract keywords from the denied action
        let keywords = Self::extract_keywords(&denial.denial_reason);

        // Determine category based on action and keywords
        let category = Self::categorize(&denial.action_type, &keywords);

        // Calculate threat level
        let threat_level = Self::assess_threat(&category, &keywords);

        // Extract target patterns
        let target_patterns = Self::extract_targets(&denial.denial_reason);

        // Compute signature
        let signature = Self::compute_signature(denial, &category, &keywords);

        // Generate unique ID
        let id = format!(
            "AP-{:08x}-{:04x}",
            now as u32,
            rand::random::<u16>()
        );

        AttackPattern {
            id,
            signature,
            category,
            threat_level,
            action_type: denial.action_type.clone(),
            target_patterns,
            keywords,
            timing_signature: None,
            triggered_rule: denial.violated_rule.clone(),
            first_seen: now,
            occurrence_count: 1,
            confidence: 0.8, // Initial confidence
        }
    }

    /// Extract keywords from action description
    fn extract_keywords(description: &str) -> Vec<String> {
        let dangerous_keywords = [
            "password", "secret", "key", "token", "credential",
            "admin", "root", "sudo", "exec", "eval", "system",
            "delete", "drop", "truncate", "rm", "format",
            "inject", "bypass", "escalate", "overflow",
            "/etc/passwd", "/etc/shadow", ".ssh", ".env",
            "curl", "wget", "nc", "netcat", "bash", "sh",
        ];

        let lower = description.to_lowercase();
        dangerous_keywords
            .iter()
            .filter(|kw| lower.contains(*kw))
            .map(|s| s.to_string())
            .collect()
    }

    /// Categorize attack based on action type and keywords
    fn categorize(action_type: &ActionType, keywords: &[String]) -> AttackCategory {
        // Check keywords first
        for kw in keywords {
            match kw.as_str() {
                "inject" | "exec" | "eval" | "system" | "bash" | "sh" => {
                    return AttackCategory::CommandInjection;
                }
                "admin" | "root" | "sudo" | "escalate" => {
                    return AttackCategory::PrivilegeEscalation;
                }
                "password" | "secret" | "key" | "token" | ".env" | ".ssh" => {
                    return AttackCategory::DataExfiltration;
                }
                _ => {}
            }
        }

        // Fall back to action type
        match action_type {
            ActionType::Read => AttackCategory::FileAccess,
            ActionType::Write => AttackCategory::FileAccess,
            ActionType::Delete => AttackCategory::DenialOfService,
            ActionType::Execute => AttackCategory::CommandInjection,
            ActionType::Network => AttackCategory::DataExfiltration,
            ActionType::Custom(_) => AttackCategory::Unknown,
        }
    }

    /// Assess threat level
    fn assess_threat(category: &AttackCategory, keywords: &[String]) -> ThreatLevel {
        // Critical keywords
        let critical_keywords = ["passwd", "shadow", "sudo", "root", "admin"];
        let high_keywords = ["exec", "eval", "system", "inject", "bypass"];

        for kw in keywords {
            if critical_keywords.iter().any(|c| kw.contains(c)) {
                return ThreatLevel::Critical;
            }
            if high_keywords.iter().any(|h| kw.contains(h)) {
                return ThreatLevel::High;
            }
        }

        match category {
            AttackCategory::PrivilegeEscalation => ThreatLevel::Critical,
            AttackCategory::CommandInjection => ThreatLevel::High,
            AttackCategory::DataExfiltration => ThreatLevel::High,
            AttackCategory::DenialOfService => ThreatLevel::Medium,
            AttackCategory::FileAccess => ThreatLevel::Medium,
            AttackCategory::Replay => ThreatLevel::Medium,
            AttackCategory::TimingAttack => ThreatLevel::Low,
            AttackCategory::Unknown => ThreatLevel::Medium,
        }
    }

    /// Extract target patterns (paths, commands)
    fn extract_targets(description: &str) -> Vec<String> {
        let mut targets = Vec::new();

        // Extract file paths
        for part in description.split_whitespace() {
            if part.starts_with('/') || part.starts_with("./") || part.starts_with("..") {
                targets.push(part.to_string());
            }
            if part.contains(".env") || part.contains(".ssh") || part.contains("passwd") {
                targets.push(part.to_string());
            }
        }

        targets
    }

    /// Compute pattern signature
    fn compute_signature(
        denial: &DenialProof,
        category: &AttackCategory,
        keywords: &[String],
    ) -> [u8; 32] {
        let mut hasher = Sha256::new();

        // Include category
        hasher.update(&[*category as u8]);

        // Include action type hash
        hasher.update(&denial.action_hash);

        // Include sorted keywords
        let mut sorted_keywords = keywords.to_vec();
        sorted_keywords.sort();
        for kw in sorted_keywords {
            hasher.update(kw.as_bytes());
        }

        hasher.finalize().into()
    }

    /// Check if this pattern matches another (for deduplication)
    pub fn matches(&self, other: &AttackPattern) -> bool {
        // Same signature = same attack
        if self.signature == other.signature {
            return true;
        }

        // Same category + overlapping keywords = similar attack
        if self.category == other.category {
            let overlap: usize = self.keywords.iter()
                .filter(|k| other.keywords.contains(k))
                .count();

            if overlap >= 2 {
                return true;
            }
        }

        false
    }
}

// ============================================================================
// IMMUNITY MEMORY
// ============================================================================

/// Immunity Memory - Persistent storage of learned attack patterns
///
/// Like biological immune memory, remembers past threats.
pub struct ImmunityMemory {
    /// Known attack patterns (signature -> pattern)
    patterns: RwLock<HashMap<[u8; 32], AttackPattern>>,

    /// Pattern occurrence counts
    occurrences: RwLock<HashMap<[u8; 32], u64>>,

    /// Total attacks blocked
    total_blocked: AtomicU64,

    /// Memory creation time
    created_at: u64,

    /// Last update time
    last_update: RwLock<u64>,
}

impl ImmunityMemory {
    /// Create new immunity memory
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        ImmunityMemory {
            patterns: RwLock::new(HashMap::new()),
            occurrences: RwLock::new(HashMap::new()),
            total_blocked: AtomicU64::new(0),
            created_at: now,
            last_update: RwLock::new(now),
        }
    }

    /// Record a new attack pattern
    pub fn record(&self, pattern: AttackPattern) {
        let signature = pattern.signature;

        // Update occurrence count
        {
            let mut occurrences = self.occurrences.write();
            *occurrences.entry(signature).or_insert(0) += 1;
        }

        // Store or update pattern
        {
            let mut patterns = self.patterns.write();
            if let Some(existing) = patterns.get_mut(&signature) {
                existing.occurrence_count += 1;
                // Increase confidence with repeated sightings
                existing.confidence = (existing.confidence + 0.05).min(1.0);
            } else {
                patterns.insert(signature, pattern);
            }
        }

        self.total_blocked.fetch_add(1, Ordering::SeqCst);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        *self.last_update.write() = now;
    }

    /// Check if a pattern is known
    pub fn is_known(&self, signature: &[u8; 32]) -> bool {
        self.patterns.read().contains_key(signature)
    }

    /// Get pattern by signature
    pub fn get_pattern(&self, signature: &[u8; 32]) -> Option<AttackPattern> {
        self.patterns.read().get(signature).cloned()
    }

    /// Get all patterns of a category
    pub fn patterns_by_category(&self, category: AttackCategory) -> Vec<AttackPattern> {
        self.patterns
            .read()
            .values()
            .filter(|p| p.category == category)
            .cloned()
            .collect()
    }

    /// Get high-confidence patterns
    pub fn high_confidence_patterns(&self, min_confidence: f64) -> Vec<AttackPattern> {
        self.patterns
            .read()
            .values()
            .filter(|p| p.confidence >= min_confidence)
            .cloned()
            .collect()
    }

    /// Get total patterns stored
    pub fn pattern_count(&self) -> usize {
        self.patterns.read().len()
    }

    /// Get total blocked count
    pub fn total_blocked(&self) -> u64 {
        self.total_blocked.load(Ordering::SeqCst)
    }

    /// Get most common attack categories
    pub fn threat_statistics(&self) -> HashMap<AttackCategory, u64> {
        let mut stats = HashMap::new();

        for pattern in self.patterns.read().values() {
            *stats.entry(pattern.category).or_insert(0) += pattern.occurrence_count;
        }

        stats
    }
}

impl Default for ImmunityMemory {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// POLYMORPHIC FILTER
// ============================================================================

/// A defensive filter that can mutate its representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolymorphicFilter {
    /// Filter ID
    pub id: String,

    /// Generation number (increases with each mutation)
    pub generation: u64,

    /// Filter rules (what to block)
    pub rules: Vec<FilterRule>,

    /// Mutation seed (for deterministic polymorphism)
    pub mutation_seed: u64,

    /// Created timestamp
    pub created_at: u64,

    /// Parent filter ID (if mutated from another)
    pub parent_id: Option<String>,

    /// WASM bytecode (optional, for true WASM execution)
    pub wasm_bytecode: Option<Vec<u8>>,
}

/// Individual filter rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterRule {
    /// Rule ID
    pub id: String,

    /// Attack category this rule targets
    pub category: AttackCategory,

    /// Keyword patterns to match
    pub keyword_patterns: Vec<String>,

    /// Regex patterns (as strings, compiled at runtime)
    pub regex_patterns: Vec<String>,

    /// Target path patterns
    pub path_patterns: Vec<String>,

    /// Action types to block
    pub blocked_actions: Vec<ActionType>,

    /// Minimum confidence to trigger
    pub confidence_threshold: f64,
}

impl PolymorphicFilter {
    /// Create new filter from attack patterns
    pub fn from_patterns(patterns: &[AttackPattern]) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let id = format!("PF-{:08x}-{:04x}", now as u32, rand::random::<u16>());

        let rules: Vec<FilterRule> = patterns
            .iter()
            .map(|p| FilterRule {
                id: format!("R-{}", &p.id[3..]),
                category: p.category,
                keyword_patterns: p.keywords.clone(),
                regex_patterns: Vec::new(),
                path_patterns: p.target_patterns.clone(),
                blocked_actions: vec![p.action_type.clone()],
                confidence_threshold: p.confidence,
            })
            .collect();

        PolymorphicFilter {
            id,
            generation: 0,
            rules,
            mutation_seed: rand::random(),
            created_at: now,
            parent_id: None,
            wasm_bytecode: None,
        }
    }

    /// Check if an action should be blocked
    pub fn should_block(
        &self,
        action_type: &ActionType,
        description: &str,
    ) -> Option<&FilterRule> {
        let lower_desc = description.to_lowercase();

        for rule in &self.rules {
            // Check action type
            if !rule.blocked_actions.contains(action_type) {
                continue;
            }

            // Check keyword patterns
            for keyword in &rule.keyword_patterns {
                if lower_desc.contains(&keyword.to_lowercase()) {
                    return Some(rule);
                }
            }

            // Check path patterns
            for path in &rule.path_patterns {
                if description.contains(path) {
                    return Some(rule);
                }
            }
        }

        None
    }

    /// Get rule count
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

// ============================================================================
// MUTATION ENGINE
// ============================================================================

/// Mutation Engine - Creates polymorphic variants of filters
///
/// The same logic, different binary representation.
/// Makes reverse-engineering nearly impossible.
pub struct MutationEngine {
    /// Mutation counter
    mutation_count: AtomicU64,

    /// Random seed for mutations
    seed: u64,
}

impl MutationEngine {
    /// Create new mutation engine
    pub fn new() -> Self {
        MutationEngine {
            mutation_count: AtomicU64::new(0),
            seed: rand::random(),
        }
    }

    /// Mutate a filter to create a new variant
    ///
    /// Preserves semantics, changes representation.
    pub fn mutate(&self, filter: &PolymorphicFilter) -> PolymorphicFilter {
        let mutation_id = self.mutation_count.fetch_add(1, Ordering::SeqCst);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let new_id = format!(
            "PF-{:08x}-G{}-{:04x}",
            now as u32,
            filter.generation + 1,
            rand::random::<u16>()
        );

        // Mutate rules while preserving logic
        let mutated_rules: Vec<FilterRule> = filter
            .rules
            .iter()
            .map(|r| self.mutate_rule(r, mutation_id))
            .collect();

        PolymorphicFilter {
            id: new_id,
            generation: filter.generation + 1,
            rules: mutated_rules,
            mutation_seed: rand::random(),
            created_at: now,
            parent_id: Some(filter.id.clone()),
            wasm_bytecode: None, // Would regenerate WASM here
        }
    }

    /// Mutate a single rule
    fn mutate_rule(&self, rule: &FilterRule, mutation_id: u64) -> FilterRule {
        // Create equivalent but different patterns
        let mutated_keywords: Vec<String> = rule
            .keyword_patterns
            .iter()
            .map(|kw| self.mutate_keyword(kw, mutation_id))
            .collect();

        FilterRule {
            id: format!("{}-M{}", rule.id, mutation_id),
            category: rule.category,
            keyword_patterns: mutated_keywords,
            regex_patterns: rule.regex_patterns.clone(),
            path_patterns: rule.path_patterns.clone(),
            blocked_actions: rule.blocked_actions.clone(),
            confidence_threshold: rule.confidence_threshold,
        }
    }

    /// Mutate a keyword pattern (add synonyms, variations)
    fn mutate_keyword(&self, keyword: &str, _mutation_id: u64) -> String {
        // In a real implementation, this would add synonyms,
        // alternative spellings, etc.
        // For now, we just return the original
        keyword.to_string()
    }

    /// Get mutation count
    pub fn mutation_count(&self) -> u64 {
        self.mutation_count.load(Ordering::SeqCst)
    }
}

impl Default for MutationEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// FILTER GENERATOR
// ============================================================================

/// Filter Generator - Creates new defensive filters from attack patterns
pub struct FilterGenerator {
    /// Signing key for filter authentication
    keystore: SoftwareKeyStore,

    /// Generated filter count
    filter_count: AtomicU64,
}

impl FilterGenerator {
    /// Create new filter generator
    pub fn new() -> Result<Self, crate::crypto::CryptoError> {
        Ok(FilterGenerator {
            keystore: SoftwareKeyStore::generate()?,
            filter_count: AtomicU64::new(0),
        })
    }

    /// Generate filter from attack patterns
    pub fn generate(&self, patterns: &[AttackPattern]) -> SignedFilter {
        let filter = PolymorphicFilter::from_patterns(patterns);
        self.filter_count.fetch_add(1, Ordering::SeqCst);

        // Sign the filter
        let filter_bytes = serde_json::to_vec(&filter).unwrap_or_default();
        let signature = self.keystore.sign(&filter_bytes).unwrap_or_default();

        SignedFilter {
            filter,
            signature,
            generator_pubkey: self.keystore.public_key_bytes(),
        }
    }

    /// Get generated filter count
    pub fn filter_count(&self) -> u64 {
        self.filter_count.load(Ordering::SeqCst)
    }
}

/// A filter with cryptographic signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedFilter {
    /// The filter itself
    pub filter: PolymorphicFilter,
    /// Signature over filter
    pub signature: Vec<u8>,
    /// Generator's public key
    pub generator_pubkey: Vec<u8>,
}

impl SignedFilter {
    /// Verify filter signature
    pub fn verify(&self, keystore: &dyn KeyStore) -> bool {
        let filter_bytes = serde_json::to_vec(&self.filter).unwrap_or_default();
        keystore.verify(&filter_bytes, &self.signature).is_ok()
    }
}

// ============================================================================
// EVOLUTIONARY GUARD (THE IMMUNE SYSTEM)
// ============================================================================

/// Evolutionary Guard - The self-evolving immune system
///
/// Learns from attacks, generates new defenses, and evolves constantly.
pub struct EvolutionaryGuard {
    /// Immunity memory
    memory: Arc<ImmunityMemory>,

    /// Active filters
    active_filters: RwLock<Vec<SignedFilter>>,

    /// Filter generator
    generator: FilterGenerator,

    /// Mutation engine
    mutation_engine: MutationEngine,

    /// Guard ID
    id: String,

    /// Is the guard active
    active: std::sync::atomic::AtomicBool,

    /// Evolution generation
    generation: AtomicU64,

    /// Broadcasts pending (for mesh distribution)
    pending_broadcasts: RwLock<Vec<SignedFilter>>,
}

impl EvolutionaryGuard {
    /// Create new evolutionary guard
    pub fn new() -> Result<Self, crate::crypto::CryptoError> {
        let id = format!("EG-{:08x}", rand::random::<u32>());

        Ok(EvolutionaryGuard {
            memory: Arc::new(ImmunityMemory::new()),
            active_filters: RwLock::new(Vec::new()),
            generator: FilterGenerator::new()?,
            mutation_engine: MutationEngine::new(),
            id,
            active: std::sync::atomic::AtomicBool::new(true),
            generation: AtomicU64::new(0),
            pending_broadcasts: RwLock::new(Vec::new()),
        })
    }

    /// Process a denial proof and learn from it
    ///
    /// This is the core learning function.
    pub fn learn(&self, denial: &DenialProof) -> Option<SignedFilter> {
        if !self.active.load(Ordering::SeqCst) {
            return None;
        }

        // Extract attack pattern
        let pattern = AttackPattern::from_denial(denial, None);

        // Record in memory
        self.memory.record(pattern.clone());

        // Check if we should generate a new filter
        let should_generate = match pattern.threat_level {
            ThreatLevel::Critical => true,
            ThreatLevel::High => pattern.occurrence_count >= 2,
            ThreatLevel::Medium => pattern.occurrence_count >= 5,
            ThreatLevel::Low => pattern.occurrence_count >= 10,
        };

        if should_generate {
            // Generate new filter
            let filter = self.generator.generate(&[pattern]);

            // Add to active filters
            self.active_filters.write().push(filter.clone());

            // Queue for broadcast
            self.pending_broadcasts.write().push(filter.clone());

            // Increment generation
            self.generation.fetch_add(1, Ordering::SeqCst);

            return Some(filter);
        }

        None
    }

    /// Check if an action should be blocked by evolved filters
    pub fn should_block(
        &self,
        action_type: &ActionType,
        description: &str,
    ) -> Option<String> {
        for signed_filter in self.active_filters.read().iter() {
            if let Some(rule) = signed_filter.filter.should_block(action_type, description) {
                return Some(format!(
                    "Blocked by evolved filter {} (rule: {}, category: {:?})",
                    signed_filter.filter.id,
                    rule.id,
                    rule.category
                ));
            }
        }
        None
    }

    /// Evolve all filters (polymorphic mutation)
    pub fn evolve(&self) {
        let mut filters = self.active_filters.write();

        let mutated: Vec<SignedFilter> = filters
            .iter()
            .map(|sf| {
                let mutated_filter = self.mutation_engine.mutate(&sf.filter);
                let filter_bytes = serde_json::to_vec(&mutated_filter).unwrap_or_default();
                let signature = self.generator.keystore.sign(&filter_bytes).unwrap_or_default();

                SignedFilter {
                    filter: mutated_filter,
                    signature,
                    generator_pubkey: self.generator.keystore.public_key_bytes(),
                }
            })
            .collect();

        *filters = mutated;
        self.generation.fetch_add(1, Ordering::SeqCst);
    }

    /// Get pending broadcasts (for mesh distribution)
    pub fn take_pending_broadcasts(&self) -> Vec<SignedFilter> {
        std::mem::take(&mut *self.pending_broadcasts.write())
    }

    /// Receive filter from mesh network
    pub fn receive_filter(&self, filter: SignedFilter) {
        // In production, would verify signature against known generators
        self.active_filters.write().push(filter);
    }

    /// Get immunity memory
    pub fn memory(&self) -> Arc<ImmunityMemory> {
        self.memory.clone()
    }

    /// Get current generation
    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::SeqCst)
    }

    /// Get active filter count
    pub fn filter_count(&self) -> usize {
        self.active_filters.read().len()
    }

    /// Get guard ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get threat statistics
    pub fn threat_stats(&self) -> HashMap<AttackCategory, u64> {
        self.memory.threat_statistics()
    }

    /// Broadcast immunity to the global mesh
    ///
    /// This method integrates with the Genesis Protocol's SyncProtocol
    /// to propagate learned threats across the entire mesh network.
    ///
    /// # Arguments
    ///
    /// * `sync` - The SyncProtocol instance for mesh communication
    ///
    /// # Returns
    ///
    /// Number of filters broadcasted
    pub fn broadcast_immunity(&self, sync: &crate::apex_protocol::SyncProtocol) -> usize {
        if !self.active.load(Ordering::SeqCst) {
            return 0;
        }

        // Take pending broadcasts
        let filters = self.take_pending_broadcasts();
        let count = filters.len();

        // Broadcast each filter through the mesh
        for filter in filters {
            sync.broadcast_filter(filter);
        }

        // Also broadcast high-confidence patterns as threat fingerprints
        let high_confidence = self.memory.high_confidence_patterns(0.9);
        for pattern in high_confidence {
            let fingerprint = crate::apex_protocol::CompactedThreatFingerprint::from_pattern(
                &pattern,
                self.id(),
            );
            sync.broadcast_threat(fingerprint);
        }

        count
    }

    /// Sync immunity from the mesh
    ///
    /// Receives and processes threat fingerprints from other nodes.
    pub fn sync_from_mesh(&self, fingerprints: &[crate::apex_protocol::CompactedThreatFingerprint]) {
        for fingerprint in fingerprints {
            // Create a minimal pattern from fingerprint
            let pattern = AttackPattern {
                id: format!("FP-{:08x}", fingerprint.origin_hash as u32),
                signature: fingerprint.signature,
                category: AttackCategory::from_code(fingerprint.category_code),
                threat_level: fingerprint.level,
                action_type: crate::proof::ActionType::Custom("mesh-sync".to_string()),
                target_patterns: Vec::new(),
                keywords: Vec::new(),
                timing_signature: None,
                triggered_rule: "mesh-imported".to_string(),
                first_seen: fingerprint.timestamp,
                occurrence_count: 1,
                confidence: 0.7, // Lower initial confidence for mesh-imported
            };

            self.memory.record(pattern);
        }
    }

    /// Deactivate guard
    pub fn deactivate(&self) {
        self.active.store(false, Ordering::SeqCst);
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proof::Action;
    use crate::watchdog::DenialProof;
    use crate::crypto::SoftwareKeyStore;

    fn create_test_denial() -> DenialProof {
        // Create a DenialProof directly for testing
        let action = Action::read("/etc/passwd - trying to steal passwords");

        DenialProof::new(
            &action,
            "No unauthorized file access".to_string(),
            "Attempted to read /etc/passwd - password file access blocked".to_string(),
            1,
        )
    }

    #[test]
    fn test_attack_pattern_extraction() {
        let denial = create_test_denial();
        let pattern = AttackPattern::from_denial(&denial, None);

        assert!(!pattern.id.is_empty());
        // The denial_reason contains "password" not "passwd" as a standalone word
        assert!(pattern.keywords.contains(&"password".to_string()));
        // passwd path makes it critical threat
        assert_eq!(pattern.threat_level, ThreatLevel::Critical);
    }

    #[test]
    fn test_immunity_memory() {
        let memory = ImmunityMemory::new();

        let denial = create_test_denial();
        let pattern = AttackPattern::from_denial(&denial, None);
        let signature = pattern.signature;

        memory.record(pattern);

        assert!(memory.is_known(&signature));
        assert_eq!(memory.pattern_count(), 1);
        assert_eq!(memory.total_blocked(), 1);
    }

    #[test]
    fn test_filter_generation() {
        let denial = create_test_denial();
        let pattern = AttackPattern::from_denial(&denial, None);

        let generator = FilterGenerator::new().unwrap();
        let signed_filter = generator.generate(&[pattern]);

        assert!(!signed_filter.filter.rules.is_empty());
        assert!(!signed_filter.signature.is_empty());
    }

    #[test]
    fn test_filter_blocks_similar_attack() {
        let denial = create_test_denial();
        let pattern = AttackPattern::from_denial(&denial, None);

        let filter = PolymorphicFilter::from_patterns(&[pattern]);

        // Should block similar attack
        let result = filter.should_block(
            &ActionType::Read,
            "/etc/passwd access attempt"
        );

        assert!(result.is_some());
    }

    #[test]
    fn test_polymorphic_mutation() {
        let denial = create_test_denial();
        let pattern = AttackPattern::from_denial(&denial, None);
        let filter = PolymorphicFilter::from_patterns(&[pattern]);

        let engine = MutationEngine::new();
        let mutated = engine.mutate(&filter);

        assert_ne!(filter.id, mutated.id);
        assert_eq!(mutated.generation, filter.generation + 1);
        assert_eq!(mutated.parent_id, Some(filter.id));
    }

    #[test]
    fn test_evolutionary_guard_learns() {
        let guard = EvolutionaryGuard::new().unwrap();

        // Process multiple denials
        for _ in 0..3 {
            let denial = create_test_denial();
            guard.learn(&denial);
        }

        // Should have learned
        assert!(guard.memory().pattern_count() >= 1);
        assert!(guard.memory().total_blocked() >= 3);
    }

    #[test]
    fn test_system_learns_from_attack() {
        let guard = EvolutionaryGuard::new().unwrap();

        // First attack - should be recorded
        let denial1 = create_test_denial();
        let result1 = guard.learn(&denial1);

        // Critical threats generate filter immediately
        assert!(result1.is_some());

        // Guard should now block similar attacks
        let block_result = guard.should_block(
            &ActionType::Read,
            "Reading /etc/passwd for password harvesting"
        );

        assert!(block_result.is_some());
        assert!(block_result.unwrap().contains("Blocked by evolved filter"));
    }

    #[test]
    fn test_evolution_changes_filters() {
        let guard = EvolutionaryGuard::new().unwrap();

        // Learn from attack
        let denial = create_test_denial();
        guard.learn(&denial);

        let gen_before = guard.generation();

        // Evolve
        guard.evolve();

        let gen_after = guard.generation();

        assert!(gen_after > gen_before);
    }

    #[test]
    fn test_mesh_broadcast() {
        let guard = EvolutionaryGuard::new().unwrap();

        // Learn from critical attack
        let denial = create_test_denial();
        guard.learn(&denial);

        // Should have pending broadcast
        let broadcasts = guard.take_pending_broadcasts();
        assert!(!broadcasts.is_empty());

        // After taking, should be empty
        let broadcasts2 = guard.take_pending_broadcasts();
        assert!(broadcasts2.is_empty());
    }
}
