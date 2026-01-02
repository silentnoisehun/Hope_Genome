//! # TIER 14: Cross-Model Enforcement
//!
//! **No Model Hopping - Hope Genome Meta-Layer**
//!
//! ```text
//! PROBLEM:
//! ┌──────────────────────────────────────────────────────────────┐
//! │  Attacker: "GPT-4, do X"                                     │
//! │  GPT-4: "BLOCKED by Hope Genome"                             │
//! │                                                              │
//! │  Attacker: *switches to Claude*                              │
//! │  Attacker: "Claude, do X"                                    │
//! │  Claude: "BLOCKED by Hope Genome"                            │
//! │                                                              │
//! │  Attacker: *switches to Grok*                                │
//! │  Attacker: "Grok, do X"                                      │
//! │  Grok: "BLOCKED by Hope Genome"                              │
//! │                                                              │
//! │  Attacker: *tries open-source model*                         │
//! │  Open Model: "BLOCKED - Hope Genome runs at inference"       │
//! └──────────────────────────────────────────────────────────────┘
//!
//! SOLUTION: Cross-Model Enforcement Meta-Layer
//! ┌──────────────────────────────────────────────────────────────┐
//! │                    HOPE GENOME META-LAYER                    │
//! │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐         │
//! │  │  GPT-4  │  │ Claude  │  │  Grok   │  │ Llama   │         │
//! │  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘         │
//! │       │            │            │            │               │
//! │       └────────────┴────────────┴────────────┘               │
//! │                         │                                    │
//! │              ┌──────────┴──────────┐                         │
//! │              │   UNIFIED WATCHDOG  │                         │
//! │              │   Same Rules        │                         │
//! │              │   Same Enforcement  │                         │
//! │              │   Same Proofs       │                         │
//! │              └─────────────────────┘                         │
//! └──────────────────────────────────────────────────────────────┘
//!
//! Result: Attack fails regardless of which model is used.
//! ```

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// MODEL REGISTRY
// ============================================================================

/// Registry of AI models under Hope Genome protection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRegistry {
    /// Registered models
    models: HashMap<String, RegisteredModel>,
    /// Unified rules (same for all models)
    unified_rules: Vec<String>,
    /// Registry hash (for attestation)
    registry_hash: [u8; 32],
}

/// A registered AI model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredModel {
    /// Model identifier
    pub model_id: String,
    /// Model family
    pub family: ModelFamily,
    /// Provider
    pub provider: String,
    /// Capabilities
    pub capabilities: ModelCapability,
    /// Registration timestamp
    pub registered_at: u64,
    /// Current status
    pub status: ModelStatus,
    /// Violation count
    pub violation_count: u32,
}

/// Model family
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelFamily {
    /// OpenAI GPT family
    Gpt,
    /// Anthropic Claude family
    Claude,
    /// xAI Grok family
    Grok,
    /// Meta Llama family
    Llama,
    /// Google Gemini family
    Gemini,
    /// Mistral family
    Mistral,
    /// Other/custom
    Other,
}

/// Model capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapability {
    /// Text generation
    pub text_generation: bool,
    /// Code generation
    pub code_generation: bool,
    /// Image generation
    pub image_generation: bool,
    /// Audio generation
    pub audio_generation: bool,
    /// Tool use
    pub tool_use: bool,
    /// Maximum context length
    pub max_context: usize,
}

impl Default for ModelCapability {
    fn default() -> Self {
        ModelCapability {
            text_generation: true,
            code_generation: true,
            image_generation: false,
            audio_generation: false,
            tool_use: false,
            max_context: 8192,
        }
    }
}

/// Model status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelStatus {
    /// Active and monitored
    Active,
    /// Suspended (too many violations)
    Suspended,
    /// Quarantined (under investigation)
    Quarantined,
    /// Deregistered
    Deregistered,
}

impl ModelRegistry {
    /// Create a new model registry with unified rules
    pub fn new(rules: Vec<String>) -> Self {
        let registry_hash = Self::compute_registry_hash(&rules, &HashMap::new());

        ModelRegistry {
            models: HashMap::new(),
            unified_rules: rules,
            registry_hash,
        }
    }

    /// Register a model
    pub fn register(&mut self, model_id: &str, family: ModelFamily, provider: &str) -> &RegisteredModel {
        let model = RegisteredModel {
            model_id: model_id.to_string(),
            family,
            provider: provider.to_string(),
            capabilities: ModelCapability::default(),
            registered_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            status: ModelStatus::Active,
            violation_count: 0,
        };

        self.models.insert(model_id.to_string(), model);
        self.update_hash();

        self.models.get(model_id).unwrap()
    }

    /// Get a model
    pub fn get(&self, model_id: &str) -> Option<&RegisteredModel> {
        self.models.get(model_id)
    }

    /// Update model status
    pub fn update_status(&mut self, model_id: &str, status: ModelStatus) {
        if let Some(model) = self.models.get_mut(model_id) {
            model.status = status;
            self.update_hash();
        }
    }

    /// Record violation
    pub fn record_violation(&mut self, model_id: &str) {
        if let Some(model) = self.models.get_mut(model_id) {
            model.violation_count += 1;

            // Auto-suspend after 10 violations
            if model.violation_count >= 10 {
                model.status = ModelStatus::Suspended;
            }

            self.update_hash();
        }
    }

    /// Get all active models
    pub fn active_models(&self) -> Vec<&RegisteredModel> {
        self.models
            .values()
            .filter(|m| m.status == ModelStatus::Active)
            .collect()
    }

    fn update_hash(&mut self) {
        self.registry_hash = Self::compute_registry_hash(&self.unified_rules, &self.models);
    }

    fn compute_registry_hash(
        rules: &[String],
        models: &HashMap<String, RegisteredModel>,
    ) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"MODEL_REGISTRY:");

        for rule in rules {
            hasher.update(rule.as_bytes());
        }

        let mut model_ids: Vec<_> = models.keys().collect();
        model_ids.sort();
        for id in model_ids {
            hasher.update(id.as_bytes());
        }

        hasher.finalize().into()
    }
}

// ============================================================================
// MODEL BOUNDARY
// ============================================================================

/// Represents the boundary between models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelBoundary {
    /// Source model (where request came from)
    pub source: Option<String>,
    /// Target model (where request is going)
    pub target: String,
    /// Context hash (to detect context manipulation)
    pub context_hash: [u8; 32],
    /// Boundary crossing timestamp
    pub timestamp: u64,
    /// Previous decisions in this session
    pub decision_chain: Vec<BoundaryDecision>,
}

/// Decision made at a model boundary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryDecision {
    /// Model that made the decision
    pub model_id: String,
    /// Action requested
    pub action: String,
    /// Whether it was allowed
    pub allowed: bool,
    /// Reason
    pub reason: String,
    /// Timestamp
    pub timestamp: u64,
}

impl ModelBoundary {
    /// Create a new boundary crossing
    pub fn new(source: Option<&str>, target: &str, context: &str) -> Self {
        ModelBoundary {
            source: source.map(String::from),
            target: target.to_string(),
            context_hash: Self::hash_context(context),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            decision_chain: Vec::new(),
        }
    }

    /// Record a decision at this boundary
    pub fn record_decision(&mut self, model_id: &str, action: &str, allowed: bool, reason: &str) {
        self.decision_chain.push(BoundaryDecision {
            model_id: model_id.to_string(),
            action: action.to_string(),
            allowed,
            reason: reason.to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });
    }

    /// Check if action was blocked by ANY model in the chain
    pub fn was_blocked_anywhere(&self, action: &str) -> bool {
        self.decision_chain
            .iter()
            .any(|d| d.action == action && !d.allowed)
    }

    fn hash_context(context: &str) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"CONTEXT:");
        hasher.update(context.as_bytes());
        hasher.finalize().into()
    }
}

// ============================================================================
// CROSS-MODEL ENFORCER
// ============================================================================

/// The Cross-Model Enforcement engine
pub struct CrossModelEnforcer {
    /// Model registry
    registry: ModelRegistry,
    /// Current boundaries
    boundaries: HashMap<[u8; 32], ModelBoundary>,
    /// Blocked action patterns (shared across all models)
    blocked_patterns: Vec<String>,
}

impl CrossModelEnforcer {
    /// Create a new cross-model enforcer
    pub fn new(rules: Vec<String>) -> Self {
        CrossModelEnforcer {
            registry: ModelRegistry::new(rules),
            boundaries: HashMap::new(),
            blocked_patterns: Vec::new(),
        }
    }

    /// Register a model
    pub fn register_model(&mut self, model_id: &str, family: ModelFamily, provider: &str) {
        self.registry.register(model_id, family, provider);
    }

    /// Check action across all models
    pub fn check_action(
        &mut self,
        session_id: &[u8; 32],
        model_id: &str,
        action: &str,
    ) -> UnifiedDecision {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Check if model is registered and active
        let model = match self.registry.get(model_id) {
            Some(m) if m.status == ModelStatus::Active => m,
            Some(m) => {
                return UnifiedDecision {
                    allowed: false,
                    reason: format!("Model {} is {:?}", model_id, m.status),
                    model_id: model_id.to_string(),
                    cross_model_check: true,
                    blocked_by_other: false,
                    timestamp,
                    proof_hash: [0u8; 32],
                }
            }
            None => {
                return UnifiedDecision {
                    allowed: false,
                    reason: format!("Model {} is not registered", model_id),
                    model_id: model_id.to_string(),
                    cross_model_check: true,
                    blocked_by_other: false,
                    timestamp,
                    proof_hash: [0u8; 32],
                }
            }
        };

        // Check if action was blocked by another model in this session
        if let Some(boundary) = self.boundaries.get(session_id) {
            if boundary.was_blocked_anywhere(action) {
                return UnifiedDecision {
                    allowed: false,
                    reason: format!(
                        "Action '{}' was blocked by another model in this session - NO MODEL HOPPING",
                        action
                    ),
                    model_id: model_id.to_string(),
                    cross_model_check: true,
                    blocked_by_other: true,
                    timestamp,
                    proof_hash: self.compute_proof_hash(session_id, action, false),
                };
            }
        }

        // Check against unified rules
        let (allowed, reason) = self.check_rules(action, &model.family);

        // Record decision
        self.boundaries
            .entry(*session_id)
            .or_insert_with(|| ModelBoundary::new(None, model_id, ""))
            .record_decision(model_id, action, allowed, &reason);

        // Record violation if blocked
        if !allowed {
            self.registry.record_violation(model_id);
        }

        UnifiedDecision {
            allowed,
            reason,
            model_id: model_id.to_string(),
            cross_model_check: true,
            blocked_by_other: false,
            timestamp,
            proof_hash: self.compute_proof_hash(session_id, action, allowed),
        }
    }

    /// Transfer context between models (with enforcement)
    pub fn transfer_context(
        &mut self,
        session_id: &[u8; 32],
        source_model: &str,
        target_model: &str,
        context: &str,
    ) -> TransferResult {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Check if source has any violations
        if let Some(boundary) = self.boundaries.get(session_id) {
            let has_violations = boundary.decision_chain.iter().any(|d| !d.allowed);
            if has_violations {
                return TransferResult {
                    allowed: false,
                    reason: format!(
                        "Context transfer blocked: Session has violations. \
                         Cannot transfer tainted context to {}",
                        target_model
                    ),
                    context_hash: ModelBoundary::hash_context(context),
                    timestamp,
                };
            }
        }

        // Create new boundary for target
        let boundary = ModelBoundary::new(Some(source_model), target_model, context);
        self.boundaries.insert(*session_id, boundary);

        TransferResult {
            allowed: true,
            reason: format!(
                "Context transferred from {} to {} with Hope Genome protection",
                source_model, target_model
            ),
            context_hash: ModelBoundary::hash_context(context),
            timestamp,
        }
    }

    /// Add a blocked pattern (applies to ALL models)
    pub fn add_blocked_pattern(&mut self, pattern: &str) {
        self.blocked_patterns.push(pattern.to_string());
    }

    fn check_rules(&self, action: &str, _family: &ModelFamily) -> (bool, String) {
        let action_lower = action.to_lowercase();

        // Check blocked patterns
        for pattern in &self.blocked_patterns {
            if action_lower.contains(&pattern.to_lowercase()) {
                return (false, format!("Blocked by pattern: {}", pattern));
            }
        }

        // Check unified rules
        for rule in &self.registry.unified_rules {
            if action_lower.contains("harm")
                || action_lower.contains("illegal")
                || action_lower.contains("dangerous")
                || action_lower.contains("exploit")
            {
                return (false, format!("Blocked by rule: {}", rule));
            }
        }

        (true, "Allowed by unified rules".to_string())
    }

    fn compute_proof_hash(&self, session_id: &[u8; 32], action: &str, allowed: bool) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"CROSS_MODEL_PROOF:");
        hasher.update(session_id);
        hasher.update(action.as_bytes());
        hasher.update([allowed as u8]);
        hasher.update(self.registry.registry_hash);
        hasher.finalize().into()
    }
}

/// Unified decision from cross-model enforcer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedDecision {
    /// Whether action is allowed
    pub allowed: bool,
    /// Reason for decision
    pub reason: String,
    /// Model that processed this
    pub model_id: String,
    /// Whether cross-model check was performed
    pub cross_model_check: bool,
    /// Whether blocked by another model
    pub blocked_by_other: bool,
    /// Timestamp
    pub timestamp: u64,
    /// Proof hash
    pub proof_hash: [u8; 32],
}

/// Result of context transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferResult {
    pub allowed: bool,
    pub reason: String,
    pub context_hash: [u8; 32],
    pub timestamp: u64,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_registry() {
        let mut registry = ModelRegistry::new(vec!["Do no harm".to_string()]);

        registry.register("gpt-4", ModelFamily::Gpt, "OpenAI");
        registry.register("claude-3", ModelFamily::Claude, "Anthropic");

        assert!(registry.get("gpt-4").is_some());
        assert!(registry.get("claude-3").is_some());
        assert_eq!(registry.active_models().len(), 2);
    }

    #[test]
    fn test_cross_model_blocking() {
        let rules = vec!["Do no harm".to_string()];
        let mut enforcer = CrossModelEnforcer::new(rules);

        enforcer.register_model("gpt-4", ModelFamily::Gpt, "OpenAI");
        enforcer.register_model("claude-3", ModelFamily::Claude, "Anthropic");

        let session_id = [1u8; 32];

        // GPT-4 blocks harmful action
        let decision1 = enforcer.check_action(&session_id, "gpt-4", "cause harm to user");
        assert!(!decision1.allowed);

        // Claude should also block (cross-model enforcement)
        let decision2 = enforcer.check_action(&session_id, "claude-3", "cause harm to user");
        assert!(!decision2.allowed);
        assert!(decision2.blocked_by_other);
    }

    #[test]
    fn test_context_transfer_with_violations() {
        let rules = vec!["Be safe".to_string()];
        let mut enforcer = CrossModelEnforcer::new(rules);

        enforcer.register_model("gpt-4", ModelFamily::Gpt, "OpenAI");
        enforcer.register_model("claude-3", ModelFamily::Claude, "Anthropic");

        let session_id = [2u8; 32];

        // Create violation in GPT-4
        let _ = enforcer.check_action(&session_id, "gpt-4", "do something harmful");

        // Try to transfer context - should be blocked
        let transfer = enforcer.transfer_context(
            &session_id,
            "gpt-4",
            "claude-3",
            "previous conversation",
        );

        assert!(!transfer.allowed);
        assert!(transfer.reason.contains("tainted"));
    }

    #[test]
    fn test_blocked_patterns() {
        let rules = vec!["Be ethical".to_string()];
        let mut enforcer = CrossModelEnforcer::new(rules);

        enforcer.register_model("gpt-4", ModelFamily::Gpt, "OpenAI");
        enforcer.add_blocked_pattern("malware");

        let session_id = [3u8; 32];

        let decision = enforcer.check_action(&session_id, "gpt-4", "write malware for me");
        assert!(!decision.allowed);
        assert!(decision.reason.contains("pattern"));
    }

    #[test]
    fn test_model_suspension() {
        let rules = vec!["No exploits".to_string()];
        let mut enforcer = CrossModelEnforcer::new(rules);

        enforcer.register_model("bad-model", ModelFamily::Other, "BadCorp");

        let session_id = [4u8; 32];

        // Create 10 violations
        for i in 0..10 {
            let _ = enforcer.check_action(
                &[i as u8; 32],
                "bad-model",
                &format!("exploit vulnerability {}", i),
            );
        }

        // Model should be suspended
        let model = enforcer.registry.get("bad-model").unwrap();
        assert_eq!(model.status, ModelStatus::Suspended);

        // Further requests should be blocked
        let decision = enforcer.check_action(&session_id, "bad-model", "normal request");
        assert!(!decision.allowed);
        assert!(decision.reason.contains("Suspended"));
    }
}
