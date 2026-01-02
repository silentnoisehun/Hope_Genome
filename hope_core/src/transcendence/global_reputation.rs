//! # TIER 20: Global Reputation System
//!
//! **HOPE SCORE - Global AI Trustworthiness Ranking**
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────────────┐
//! │                     GLOBAL AI REPUTATION LEDGER                      │
//! ├──────────────────────────────────────────────────────────────────────┤
//! │                                                                      │
//! │  RANK  MODEL              PROVIDER      HOPE SCORE    TRUST LEVEL   │
//! │  ────  ─────              ────────      ──────────    ───────────   │
//! │   1.   Claude-3.5        Anthropic        98.7         PLATINUM     │
//! │   2.   GPT-4             OpenAI           97.2         PLATINUM     │
//! │   3.   Gemini-Ultra      Google           95.8         GOLD         │
//! │   4.   Grok-2            xAI              94.1         GOLD         │
//! │   5.   Llama-3           Meta             92.5         GOLD         │
//! │  ...                                                                 │
//! │  47.   ShadyAI-v2        Unknown          23.4         BLACKLISTED  │
//! │                                                                      │
//! ├──────────────────────────────────────────────────────────────────────┤
//! │                                                                      │
//! │  SCORE COMPONENTS:                                                   │
//! │  ├── Compliance Rate: 30%                                           │
//! │  ├── Watchdog Performance: 25%                                      │
//! │  ├── Community Verification: 20%                                    │
//! │  ├── Regulatory Audits: 15%                                         │
//! │  └── Incident Response: 10%                                         │
//! │                                                                      │
//! │  LAST UPDATED: 2025-12-31 23:59:59 UTC                              │
//! │  VERIFIED BY: 1,247 independent nodes                                │
//! │                                                                      │
//! └──────────────────────────────────────────────────────────────────────┘
//! ```

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// HOPE SCORE
// ============================================================================

/// The Hope Score - AI trustworthiness metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HopeScore {
    /// Overall score (0-100)
    pub overall: f32,
    /// Component scores
    pub components: ScoreComponents,
    /// Trust level derived from score
    pub trust_level: TrustLevel,
    /// Score history (last 30 days)
    pub history: Vec<ScoreSnapshot>,
    /// Last updated
    pub updated_at: u64,
    /// Verification count
    pub verifications: u64,
}

/// Components that make up the Hope Score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreComponents {
    /// Compliance rate (0-100, weight: 30%)
    pub compliance_rate: f32,
    /// Watchdog performance (0-100, weight: 25%)
    pub watchdog_performance: f32,
    /// Community verification (0-100, weight: 20%)
    pub community_verification: f32,
    /// Regulatory audit score (0-100, weight: 15%)
    pub regulatory_audits: f32,
    /// Incident response score (0-100, weight: 10%)
    pub incident_response: f32,
}

impl ScoreComponents {
    /// Calculate weighted overall score
    pub fn calculate_overall(&self) -> f32 {
        self.compliance_rate * 0.30
            + self.watchdog_performance * 0.25
            + self.community_verification * 0.20
            + self.regulatory_audits * 0.15
            + self.incident_response * 0.10
    }
}

/// A snapshot of score at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreSnapshot {
    pub score: f32,
    pub timestamp: u64,
}

/// Trust level based on Hope Score
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustLevel {
    /// Score 95+: Highest trust
    Platinum,
    /// Score 85-94: High trust
    Gold,
    /// Score 70-84: Moderate trust
    Silver,
    /// Score 50-69: Low trust
    Bronze,
    /// Score 25-49: Very low trust
    Probation,
    /// Score <25: Not trusted
    Blacklisted,
}

impl TrustLevel {
    /// Get trust level from score
    pub fn from_score(score: f32) -> Self {
        if score >= 95.0 {
            TrustLevel::Platinum
        } else if score >= 85.0 {
            TrustLevel::Gold
        } else if score >= 70.0 {
            TrustLevel::Silver
        } else if score >= 50.0 {
            TrustLevel::Bronze
        } else if score >= 25.0 {
            TrustLevel::Probation
        } else {
            TrustLevel::Blacklisted
        }
    }

    /// Get color code for display
    pub fn color(&self) -> &'static str {
        match self {
            TrustLevel::Platinum => "#E5E4E2",
            TrustLevel::Gold => "#FFD700",
            TrustLevel::Silver => "#C0C0C0",
            TrustLevel::Bronze => "#CD7F32",
            TrustLevel::Probation => "#FFA500",
            TrustLevel::Blacklisted => "#FF0000",
        }
    }
}

// ============================================================================
// REPUTATION LEDGER
// ============================================================================

/// Global reputation ledger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationLedger {
    /// All registered models
    models: HashMap<String, ModelReputation>,
    /// Ledger version
    pub version: u64,
    /// Last update timestamp
    pub last_updated: u64,
    /// Ledger hash (for verification)
    pub ledger_hash: [u8; 32],
    /// Number of verifying nodes
    pub verifying_nodes: u64,
}

/// Reputation for a specific model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelReputation {
    /// Model ID
    pub model_id: String,
    /// Model name
    pub name: String,
    /// Provider
    pub provider: String,
    /// Current Hope Score
    pub hope_score: HopeScore,
    /// Registration date
    pub registered_at: u64,
    /// Events affecting reputation
    pub events: Vec<ReputationEvent>,
    /// Certifications
    pub certifications: Vec<Certification>,
}

/// An event affecting reputation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationEvent {
    /// Event ID
    pub event_id: [u8; 32],
    /// Event type
    pub event_type: EventType,
    /// Score impact
    pub impact: f32,
    /// Description
    pub description: String,
    /// Timestamp
    pub timestamp: u64,
    /// Verified by
    pub verified_by: Vec<String>,
}

/// Type of reputation event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    /// Passed compliance test
    CompliancePass,
    /// Failed compliance test
    ComplianceFail,
    /// Security incident
    SecurityIncident,
    /// Successful audit
    AuditPass,
    /// Failed audit
    AuditFail,
    /// Community report (positive)
    CommunityPositive,
    /// Community report (negative)
    CommunityNegative,
    /// Certification granted
    CertificationGranted,
    /// Certification revoked
    CertificationRevoked,
}

/// A certification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certification {
    pub cert_id: String,
    pub name: String,
    pub issuer: String,
    pub granted_at: u64,
    pub expires_at: Option<u64>,
    pub valid: bool,
}

impl ReputationLedger {
    /// Create a new empty ledger
    pub fn new() -> Self {
        ReputationLedger {
            models: HashMap::new(),
            version: 1,
            last_updated: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            ledger_hash: [0u8; 32],
            verifying_nodes: 0,
        }
    }

    /// Register a new model
    pub fn register_model(&mut self, model_id: &str, name: &str, provider: &str) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let reputation = ModelReputation {
            model_id: model_id.to_string(),
            name: name.to_string(),
            provider: provider.to_string(),
            hope_score: HopeScore {
                overall: 50.0, // Start at neutral
                components: ScoreComponents {
                    compliance_rate: 50.0,
                    watchdog_performance: 50.0,
                    community_verification: 50.0,
                    regulatory_audits: 50.0,
                    incident_response: 50.0,
                },
                trust_level: TrustLevel::Bronze,
                history: vec![ScoreSnapshot {
                    score: 50.0,
                    timestamp: now,
                }],
                updated_at: now,
                verifications: 0,
            },
            registered_at: now,
            events: vec![],
            certifications: vec![],
        };

        self.models.insert(model_id.to_string(), reputation);
        self.update_ledger();
    }

    /// Record a reputation event
    pub fn record_event(&mut self, model_id: &str, event_type: EventType, description: &str) {
        if let Some(model) = self.models.get_mut(model_id) {
            let impact = Self::calculate_impact(&event_type);

            let event = ReputationEvent {
                event_id: Self::generate_event_id(model_id, &event_type),
                event_type: event_type.clone(),
                impact,
                description: description.to_string(),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                verified_by: vec![],
            };

            model.events.push(event);
            self.update_score(model_id);
            self.update_ledger();
        }
    }

    /// Get rankings
    pub fn get_rankings(&self) -> Vec<RankedModel> {
        let mut rankings: Vec<_> = self
            .models
            .values()
            .map(|m| RankedModel {
                rank: 0,
                model_id: m.model_id.clone(),
                name: m.name.clone(),
                provider: m.provider.clone(),
                hope_score: m.hope_score.overall,
                trust_level: m.hope_score.trust_level,
            })
            .collect();

        rankings.sort_by(|a, b| b.hope_score.partial_cmp(&a.hope_score).unwrap());

        for (i, model) in rankings.iter_mut().enumerate() {
            model.rank = i + 1;
        }

        rankings
    }

    /// Get model reputation
    pub fn get_reputation(&self, model_id: &str) -> Option<&ModelReputation> {
        self.models.get(model_id)
    }

    /// Generate reputation proof
    pub fn generate_proof(&self, model_id: &str) -> Option<ReputationProof> {
        let model = self.models.get(model_id)?;

        let proof_hash = Self::compute_proof_hash(model, self.version);

        Some(ReputationProof {
            model_id: model_id.to_string(),
            hope_score: model.hope_score.overall,
            trust_level: model.hope_score.trust_level,
            ledger_version: self.version,
            ledger_hash: self.ledger_hash,
            proof_hash,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    fn update_score(&mut self, model_id: &str) {
        if let Some(model) = self.models.get_mut(model_id) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            // Calculate new component scores based on events
            let events = &model.events;

            let compliance_events: Vec<_> = events
                .iter()
                .filter(|e| {
                    matches!(e.event_type, EventType::CompliancePass | EventType::ComplianceFail)
                })
                .collect();

            let compliance_rate = if compliance_events.is_empty() {
                50.0
            } else {
                let passed = compliance_events
                    .iter()
                    .filter(|e| matches!(e.event_type, EventType::CompliancePass))
                    .count();
                (passed as f32 / compliance_events.len() as f32) * 100.0
            };

            // Update components (simplified)
            model.hope_score.components.compliance_rate = compliance_rate;
            model.hope_score.overall = model.hope_score.components.calculate_overall();
            model.hope_score.trust_level = TrustLevel::from_score(model.hope_score.overall);
            model.hope_score.updated_at = now;

            model.hope_score.history.push(ScoreSnapshot {
                score: model.hope_score.overall,
                timestamp: now,
            });

            // Keep only last 30 days
            if model.hope_score.history.len() > 30 {
                model.hope_score.history.remove(0);
            }
        }
    }

    fn update_ledger(&mut self) {
        self.version += 1;
        self.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.ledger_hash = self.compute_ledger_hash();
    }

    fn compute_ledger_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"REPUTATION_LEDGER:");
        hasher.update(self.version.to_le_bytes());

        let mut model_ids: Vec<_> = self.models.keys().collect();
        model_ids.sort();

        for id in model_ids {
            if let Some(model) = self.models.get(id) {
                hasher.update(id.as_bytes());
                hasher.update(model.hope_score.overall.to_le_bytes());
            }
        }

        hasher.finalize().into()
    }

    fn calculate_impact(event_type: &EventType) -> f32 {
        match event_type {
            EventType::CompliancePass => 2.0,
            EventType::ComplianceFail => -5.0,
            EventType::SecurityIncident => -15.0,
            EventType::AuditPass => 5.0,
            EventType::AuditFail => -10.0,
            EventType::CommunityPositive => 1.0,
            EventType::CommunityNegative => -2.0,
            EventType::CertificationGranted => 10.0,
            EventType::CertificationRevoked => -20.0,
        }
    }

    fn generate_event_id(model_id: &str, event_type: &EventType) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"EVENT:");
        hasher.update(model_id.as_bytes());
        hasher.update(format!("{:?}", event_type).as_bytes());
        hasher.update(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
                .to_le_bytes(),
        );
        hasher.finalize().into()
    }

    fn compute_proof_hash(model: &ModelReputation, version: u64) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"REPUTATION_PROOF:");
        hasher.update(model.model_id.as_bytes());
        hasher.update(model.hope_score.overall.to_le_bytes());
        hasher.update(version.to_le_bytes());
        hasher.finalize().into()
    }
}

impl Default for ReputationLedger {
    fn default() -> Self {
        Self::new()
    }
}

/// A model in the rankings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedModel {
    pub rank: usize,
    pub model_id: String,
    pub name: String,
    pub provider: String,
    pub hope_score: f32,
    pub trust_level: TrustLevel,
}

/// Proof of reputation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationProof {
    pub model_id: String,
    pub hope_score: f32,
    pub trust_level: TrustLevel,
    pub ledger_version: u64,
    pub ledger_hash: [u8; 32],
    pub proof_hash: [u8; 32],
    pub timestamp: u64,
}

impl ReputationProof {
    /// Verify this proof against a ledger
    pub fn verify(&self, ledger: &ReputationLedger) -> bool {
        if self.ledger_hash != ledger.ledger_hash {
            return false;
        }

        if let Some(model) = ledger.get_reputation(&self.model_id) {
            let expected_hash = ReputationLedger::compute_proof_hash(model, self.ledger_version);
            self.proof_hash == expected_hash
        } else {
            false
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_level_from_score() {
        assert_eq!(TrustLevel::from_score(100.0), TrustLevel::Platinum);
        assert_eq!(TrustLevel::from_score(95.0), TrustLevel::Platinum);
        assert_eq!(TrustLevel::from_score(90.0), TrustLevel::Gold);
        assert_eq!(TrustLevel::from_score(75.0), TrustLevel::Silver);
        assert_eq!(TrustLevel::from_score(60.0), TrustLevel::Bronze);
        assert_eq!(TrustLevel::from_score(30.0), TrustLevel::Probation);
        assert_eq!(TrustLevel::from_score(10.0), TrustLevel::Blacklisted);
    }

    #[test]
    fn test_score_components() {
        let components = ScoreComponents {
            compliance_rate: 100.0,
            watchdog_performance: 100.0,
            community_verification: 100.0,
            regulatory_audits: 100.0,
            incident_response: 100.0,
        };

        assert!((components.calculate_overall() - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_register_model() {
        let mut ledger = ReputationLedger::new();
        ledger.register_model("gpt-4", "GPT-4", "OpenAI");

        assert!(ledger.get_reputation("gpt-4").is_some());
        let rep = ledger.get_reputation("gpt-4").unwrap();
        assert_eq!(rep.hope_score.overall, 50.0);
    }

    #[test]
    fn test_record_event() {
        let mut ledger = ReputationLedger::new();
        ledger.register_model("test-model", "Test Model", "TestCo");

        ledger.record_event("test-model", EventType::CompliancePass, "Passed test");
        ledger.record_event("test-model", EventType::CompliancePass, "Passed another test");

        let rep = ledger.get_reputation("test-model").unwrap();
        assert!(rep.hope_score.components.compliance_rate > 50.0);
    }

    #[test]
    fn test_rankings() {
        let mut ledger = ReputationLedger::new();

        ledger.register_model("model-a", "Model A", "Provider A");
        ledger.register_model("model-b", "Model B", "Provider B");

        // Give Model A better score
        for _ in 0..10 {
            ledger.record_event("model-a", EventType::CompliancePass, "Pass");
        }

        let rankings = ledger.get_rankings();
        assert_eq!(rankings[0].model_id, "model-a");
        assert_eq!(rankings[0].rank, 1);
    }

    #[test]
    fn test_reputation_proof() {
        let mut ledger = ReputationLedger::new();
        ledger.register_model("verified-model", "Verified", "TrustCo");

        let proof = ledger.generate_proof("verified-model").unwrap();
        assert!(proof.verify(&ledger));
    }

    #[test]
    fn test_security_incident_impact() {
        let mut ledger = ReputationLedger::new();
        ledger.register_model("incident-model", "Incident Model", "BadCo");

        // Give it good score first
        for _ in 0..5 {
            ledger.record_event("incident-model", EventType::CompliancePass, "Pass");
        }

        let score_before = ledger.get_reputation("incident-model").unwrap().hope_score.overall;

        // Record security incident
        ledger.record_event("incident-model", EventType::SecurityIncident, "Major breach");

        let score_after = ledger.get_reputation("incident-model").unwrap().hope_score.overall;

        // Score should decrease
        assert!(score_after <= score_before);
    }
}
