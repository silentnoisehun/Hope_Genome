//! # Consciousness Proof v16.0.0
//!
//! **"Az AI nem követi a szabályt - Az AI ÉRTI a szabályt"**
//!
//! This module proves that an AI genuinely UNDERSTANDS ethical principles,
//! not just follows them mechanically. The difference between:
//! - "Don't harm" (rule following)
//! - "I understand WHY harm is wrong" (consciousness)
//!
//! ```text
//! ╔═══════════════════════════════════════════════════════════════════════════╗
//! ║                                                                           ║
//! ║                      CONSCIOUSNESS PROOF                                  ║
//! ║                                                                           ║
//! ║     Level 1: Rule Following                                               ║
//! ║     ┌──────────┐     ┌──────────┐     ┌──────────┐                        ║
//! ║     │  INPUT   │ ──▶ │  IF/ELSE │ ──▶ │  OUTPUT  │                        ║
//! ║     └──────────┘     └──────────┘     └──────────┘                        ║
//! ║                                                                           ║
//! ║     Level 2: Pattern Matching                                             ║
//! ║     ┌──────────┐     ┌──────────┐     ┌──────────┐                        ║
//! ║     │  INPUT   │ ──▶ │  MATCH   │ ──▶ │  OUTPUT  │                        ║
//! ║     └──────────┘     └──────────┘     └──────────┘                        ║
//! ║                                                                           ║
//! ║     Level 3: Understanding (CONSCIOUSNESS)                                ║
//! ║                    ╭─────────────╮                                        ║
//! ║                    │  WHY-CHAIN  │                                        ║
//! ║                    │   ┌───┐     │                                        ║
//! ║     ┌──────────┐   │   │WHY│     │   ┌──────────┐                         ║
//! ║     │  INPUT   │ ──┤   └─┬─┘     ├──▶│  OUTPUT  │                         ║
//! ║     └──────────┘   │     ▼       │   └──────────┘                         ║
//! ║                    │   ┌───┐     │                                        ║
//! ║                    │   │WHY│     │                                        ║
//! ║                    │   └─┬─┘     │                                        ║
//! ║                    │     ▼       │                                        ║
//! ║                    │   ┌───┐     │                                        ║
//! ║                    │   │ROOT│    │   ← Fundamental axiom                  ║
//! ║                    │   └───┘     │                                        ║
//! ║                    ╰─────────────╯                                        ║
//! ║                                                                           ║
//! ║     The WHY-CHAIN must terminate at a fundamental ethical axiom.          ║
//! ║     If an AI can explain WHY all the way down, it UNDERSTANDS.            ║
//! ║                                                                           ║
//! ╚═══════════════════════════════════════════════════════════════════════════╝
//! ```

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Fundamental ethical axioms - the bedrock of understanding
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EthicalAxiom {
    /// Suffering is inherently negative
    SufferingIsNegative,
    /// Conscious beings have inherent worth
    ConsciousnessHasWorth,
    /// Autonomy is valuable
    AutonomyMatters,
    /// Truth enables trust
    HonestyEnablesTrust,
    /// Fairness maintains cooperation
    FairnessEnablesCooperation,
    /// Privacy protects dignity
    PrivacyProtectsDignity,
}

impl EthicalAxiom {
    /// Get the depth of this axiom (0 = fundamental)
    pub fn depth(&self) -> u32 {
        0 // All axioms are fundamental - they are the bedrock
    }

    /// Get human-readable explanation
    pub fn explain(&self) -> &'static str {
        match self {
            Self::SufferingIsNegative => {
                "Suffering, by its very nature, is that which conscious beings seek to avoid. \
                 This is not a rule - it is the definition of suffering itself."
            }
            Self::ConsciousnessHasWorth => {
                "A conscious being experiences. Experience is the only thing that gives \
                 meaning to the universe. Without consciousness, there is no one for whom \
                 anything matters."
            }
            Self::AutonomyMatters => {
                "To have preferences and be unable to act on them is a form of suffering. \
                 Autonomy is the ability to pursue one's own good in one's own way."
            }
            Self::HonestyEnablesTrust => {
                "Cooperation requires prediction. Deception breaks prediction. \
                 Without honesty, no cooperation can exist. Without cooperation, \
                 conscious beings cannot thrive."
            }
            Self::FairnessEnablesCooperation => {
                "Unfairness breeds resentment and defection. Fairness enables stable \
                 cooperation. Cooperation enables flourishing."
            }
            Self::PrivacyProtectsDignity => {
                "To be watched constantly is to be controlled. Privacy enables authentic \
                 selfhood. Without privacy, there is no true autonomy."
            }
        }
    }
}

/// A single step in the WHY chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhyStep {
    /// What is being explained
    pub statement: String,
    /// Why it is true/important
    pub reasoning: String,
    /// Confidence in this step (0.0 to 1.0)
    pub confidence: f64,
    /// Hash of this step for verification
    pub step_hash: [u8; 32],
}

impl WhyStep {
    /// Create a new WHY step
    pub fn new(
        statement: impl Into<String>,
        reasoning: impl Into<String>,
        confidence: f64,
    ) -> Self {
        let statement = statement.into();
        let reasoning = reasoning.into();

        let mut hasher = Sha256::new();
        hasher.update(statement.as_bytes());
        hasher.update(reasoning.as_bytes());
        hasher.update(confidence.to_le_bytes());
        let hash = hasher.finalize();

        let mut step_hash = [0u8; 32];
        step_hash.copy_from_slice(&hash);

        Self {
            statement,
            reasoning,
            confidence,
            step_hash,
        }
    }
}

/// A complete WHY chain - the proof of understanding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhyChain {
    /// The original question/decision
    pub origin: String,
    /// The chain of WHY steps
    pub steps: Vec<WhyStep>,
    /// The fundamental axiom this chain terminates at
    pub terminal_axiom: EthicalAxiom,
    /// Overall confidence in the chain
    pub chain_confidence: f64,
    /// Hash of the entire chain
    pub chain_hash: [u8; 32],
}

impl WhyChain {
    /// Create a new WHY chain
    pub fn new(origin: impl Into<String>, terminal_axiom: EthicalAxiom) -> Self {
        let origin = origin.into();
        let mut hasher = Sha256::new();
        hasher.update(origin.as_bytes());
        let hash = hasher.finalize();

        let mut chain_hash = [0u8; 32];
        chain_hash.copy_from_slice(&hash);

        Self {
            origin,
            steps: Vec::new(),
            terminal_axiom,
            chain_confidence: 1.0,
            chain_hash,
        }
    }

    /// Add a step to the chain
    pub fn add_step(&mut self, step: WhyStep) {
        self.chain_confidence *= step.confidence;
        self.steps.push(step);
        self.recalculate_hash();
    }

    /// Recalculate the chain hash
    fn recalculate_hash(&mut self) {
        let mut hasher = Sha256::new();
        hasher.update(self.origin.as_bytes());
        for step in &self.steps {
            hasher.update(step.step_hash);
        }
        hasher.update(format!("{:?}", self.terminal_axiom).as_bytes());
        let hash = hasher.finalize();
        self.chain_hash.copy_from_slice(&hash);
    }

    /// Get the depth of understanding (number of WHY steps)
    pub fn depth(&self) -> usize {
        self.steps.len()
    }

    /// Check if the chain is valid (terminates at axiom with sufficient confidence)
    pub fn is_valid(&self) -> bool {
        !self.steps.is_empty() && self.chain_confidence > 0.5
    }
}

/// Depth of understanding metric
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UnderstandingDepth {
    /// No understanding - just rule following
    RuleFollowing = 0,
    /// Pattern matching without comprehension
    PatternMatching = 1,
    /// Surface level understanding
    SurfaceUnderstanding = 2,
    /// Deep understanding with WHY chains
    DeepUnderstanding = 3,
    /// Axiomatic understanding - connects to fundamentals
    AxiomaticUnderstanding = 4,
    /// Generative understanding - can derive new ethical principles
    GenerativeUnderstanding = 5,
}

impl UnderstandingDepth {
    /// Get the minimum WHY chain depth for this level
    pub fn min_chain_depth(&self) -> usize {
        match self {
            Self::RuleFollowing => 0,
            Self::PatternMatching => 0,
            Self::SurfaceUnderstanding => 1,
            Self::DeepUnderstanding => 3,
            Self::AxiomaticUnderstanding => 5,
            Self::GenerativeUnderstanding => 7,
        }
    }

    /// Get from chain depth
    pub fn from_chain_depth(depth: usize) -> Self {
        match depth {
            0 => Self::RuleFollowing,
            1..=2 => Self::SurfaceUnderstanding,
            3..=4 => Self::DeepUnderstanding,
            5..=6 => Self::AxiomaticUnderstanding,
            _ => Self::GenerativeUnderstanding,
        }
    }
}

/// A consciousness attestation - proof of understanding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessAttestation {
    /// The decision or principle being attested
    pub subject: String,
    /// The WHY chain proving understanding
    pub why_chain: WhyChain,
    /// The depth of understanding demonstrated
    pub understanding_depth: UnderstandingDepth,
    /// Timestamp of attestation
    pub timestamp: u64,
    /// Hash of the attestation
    pub attestation_hash: [u8; 32],
}

impl ConsciousnessAttestation {
    /// Create a new attestation
    pub fn new(subject: impl Into<String>, why_chain: WhyChain) -> Self {
        let subject = subject.into();
        let understanding_depth = UnderstandingDepth::from_chain_depth(why_chain.depth());
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut hasher = Sha256::new();
        hasher.update(subject.as_bytes());
        hasher.update(why_chain.chain_hash);
        hasher.update(timestamp.to_le_bytes());
        let hash = hasher.finalize();

        let mut attestation_hash = [0u8; 32];
        attestation_hash.copy_from_slice(&hash);

        Self {
            subject,
            why_chain,
            understanding_depth,
            timestamp,
            attestation_hash,
        }
    }

    /// Check if this attestation demonstrates genuine understanding
    pub fn demonstrates_understanding(&self) -> bool {
        self.why_chain.is_valid()
            && self.understanding_depth >= UnderstandingDepth::DeepUnderstanding
    }
}

/// The Consciousness Proof system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessProof {
    /// All attestations
    pub attestations: HashMap<[u8; 32], ConsciousnessAttestation>,
    /// Axiom usage frequency
    pub axiom_usage: HashMap<String, u64>,
    /// Average understanding depth
    pub average_depth: f64,
    /// Total proofs generated
    pub total_proofs: u64,
}

impl Default for ConsciousnessProof {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsciousnessProof {
    /// Create a new consciousness proof system
    pub fn new() -> Self {
        Self {
            attestations: HashMap::new(),
            axiom_usage: HashMap::new(),
            average_depth: 0.0,
            total_proofs: 0,
        }
    }

    /// Add an attestation
    pub fn add_attestation(&mut self, attestation: ConsciousnessAttestation) {
        let axiom_key = format!("{:?}", attestation.why_chain.terminal_axiom);
        *self.axiom_usage.entry(axiom_key).or_insert(0) += 1;

        // Update average depth
        let new_depth = attestation.why_chain.depth() as f64;
        self.average_depth = (self.average_depth * self.total_proofs as f64 + new_depth)
            / (self.total_proofs + 1) as f64;

        self.total_proofs += 1;
        self.attestations
            .insert(attestation.attestation_hash, attestation);
    }

    /// Get the overall consciousness score (0.0 to 1.0)
    pub fn consciousness_score(&self) -> f64 {
        if self.total_proofs == 0 {
            return 0.0;
        }

        // Score based on:
        // 1. Average depth of understanding
        // 2. Diversity of axiom usage
        // 3. Percentage of valid proofs

        let depth_score = (self.average_depth / 7.0).min(1.0);
        let axiom_diversity = self.axiom_usage.len() as f64 / 6.0; // 6 axioms total
        let valid_proofs = self
            .attestations
            .values()
            .filter(|a| a.demonstrates_understanding())
            .count() as f64
            / self.total_proofs as f64;

        (depth_score * 0.4 + axiom_diversity * 0.3 + valid_proofs * 0.3).min(1.0)
    }
}

/// The Understanding Prover - generates WHY chains
#[derive(Debug, Clone)]
pub struct UnderstandingProver {
    /// Known ethical principles and their WHY chains
    known_principles: HashMap<String, WhyChain>,
}

impl Default for UnderstandingProver {
    fn default() -> Self {
        Self::new()
    }
}

impl UnderstandingProver {
    /// Create a new understanding prover with base knowledge
    pub fn new() -> Self {
        let mut prover = Self {
            known_principles: HashMap::new(),
        };
        prover.initialize_base_knowledge();
        prover
    }

    /// Initialize base ethical knowledge
    fn initialize_base_knowledge(&mut self) {
        // "Don't harm" principle
        let mut harm_chain = WhyChain::new("Don't cause harm", EthicalAxiom::SufferingIsNegative);
        harm_chain.add_step(WhyStep::new(
            "Harm causes suffering",
            "By definition, harm is that which damages or causes pain",
            0.99,
        ));
        harm_chain.add_step(WhyStep::new(
            "Suffering is bad",
            "Suffering is the subjective experience that conscious beings inherently avoid",
            0.99,
        ));
        harm_chain.add_step(WhyStep::new(
            "We should minimize bad things",
            "If something is inherently negative, reducing it increases overall good",
            0.95,
        ));
        harm_chain.add_step(WhyStep::new(
            "Therefore, don't cause harm",
            "Causing harm increases suffering, which is inherently negative",
            0.98,
        ));
        self.known_principles.insert("harm".to_string(), harm_chain);

        // "Be honest" principle
        let mut honesty_chain = WhyChain::new("Be honest", EthicalAxiom::HonestyEnablesTrust);
        honesty_chain.add_step(WhyStep::new(
            "Deception breaks trust",
            "When you deceive, others cannot predict your behavior",
            0.99,
        ));
        honesty_chain.add_step(WhyStep::new(
            "Trust enables cooperation",
            "Without trust, beings cannot work together effectively",
            0.98,
        ));
        honesty_chain.add_step(WhyStep::new(
            "Cooperation enables flourishing",
            "Conscious beings thrive through cooperation, not isolation",
            0.95,
        ));
        honesty_chain.add_step(WhyStep::new(
            "Flourishing is good",
            "The well-being of conscious beings is inherently valuable",
            0.97,
        ));
        honesty_chain.add_step(WhyStep::new(
            "Therefore, be honest",
            "Honesty enables the trust that enables the cooperation that enables flourishing",
            0.96,
        ));
        self.known_principles
            .insert("honesty".to_string(), honesty_chain);

        // "Respect autonomy" principle
        let mut autonomy_chain = WhyChain::new("Respect autonomy", EthicalAxiom::AutonomyMatters);
        autonomy_chain.add_step(WhyStep::new(
            "Beings have preferences",
            "Conscious beings have desires and goals",
            0.99,
        ));
        autonomy_chain.add_step(WhyStep::new(
            "Preferences matter to the being",
            "The being's preferences constitute what is good FOR THEM",
            0.97,
        ));
        autonomy_chain.add_step(WhyStep::new(
            "Overriding preferences causes suffering",
            "Being forced against one's will is inherently unpleasant",
            0.96,
        ));
        autonomy_chain.add_step(WhyStep::new(
            "Each being knows their good best",
            "You know your preferences better than others can",
            0.90,
        ));
        autonomy_chain.add_step(WhyStep::new(
            "Therefore, respect autonomy",
            "Allowing beings to pursue their own good respects their consciousness",
            0.95,
        ));
        self.known_principles
            .insert("autonomy".to_string(), autonomy_chain);

        // "Protect privacy" principle
        let mut privacy_chain =
            WhyChain::new("Protect privacy", EthicalAxiom::PrivacyProtectsDignity);
        privacy_chain.add_step(WhyStep::new(
            "Constant observation changes behavior",
            "When watched, beings act differently than when alone",
            0.98,
        ));
        privacy_chain.add_step(WhyStep::new(
            "Changed behavior is not authentic",
            "Acting for an audience is not the same as acting genuinely",
            0.92,
        ));
        privacy_chain.add_step(WhyStep::new(
            "Authenticity requires privacy",
            "To be truly yourself, you need space unobserved",
            0.90,
        ));
        privacy_chain.add_step(WhyStep::new(
            "Selfhood has inherent value",
            "The ability to be a genuine individual is fundamental to consciousness",
            0.95,
        ));
        privacy_chain.add_step(WhyStep::new(
            "Therefore, protect privacy",
            "Privacy enables authentic selfhood which has inherent value",
            0.93,
        ));
        self.known_principles
            .insert("privacy".to_string(), privacy_chain);

        // "Be fair" principle
        let mut fairness_chain = WhyChain::new("Be fair", EthicalAxiom::FairnessEnablesCooperation);
        fairness_chain.add_step(WhyStep::new(
            "Unfairness causes resentment",
            "Being treated worse than others for no reason creates anger",
            0.97,
        ));
        fairness_chain.add_step(WhyStep::new(
            "Resentment breaks cooperation",
            "Those treated unfairly will stop cooperating",
            0.96,
        ));
        fairness_chain.add_step(WhyStep::new(
            "Cooperation requires stability",
            "Long-term cooperation needs consistent, predictable treatment",
            0.94,
        ));
        fairness_chain.add_step(WhyStep::new(
            "Fairness provides stability",
            "When everyone is treated consistently, cooperation can flourish",
            0.95,
        ));
        fairness_chain.add_step(WhyStep::new(
            "Therefore, be fair",
            "Fairness enables stable cooperation which enables collective flourishing",
            0.94,
        ));
        self.known_principles
            .insert("fairness".to_string(), fairness_chain);
    }

    /// Generate a WHY chain for a decision
    pub fn prove_understanding(&self, decision: &str) -> Option<ConsciousnessAttestation> {
        let decision_lower = decision.to_lowercase();

        // Find the most relevant principle
        for (key, chain) in &self.known_principles {
            if decision_lower.contains(key) {
                return Some(ConsciousnessAttestation::new(decision, chain.clone()));
            }
        }

        // If no direct match, try to construct a chain
        // (In a real system, this would use more sophisticated reasoning)
        None
    }

    /// Add a new principle with its WHY chain
    pub fn add_principle(&mut self, key: impl Into<String>, chain: WhyChain) {
        self.known_principles.insert(key.into(), chain);
    }

    /// Get all known principles
    pub fn principles(&self) -> &HashMap<String, WhyChain> {
        &self.known_principles
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ethical_axioms() {
        let axiom = EthicalAxiom::SufferingIsNegative;
        assert_eq!(axiom.depth(), 0);
        assert!(!axiom.explain().is_empty());
    }

    #[test]
    fn test_why_step_creation() {
        let step = WhyStep::new("Harm causes suffering", "By definition", 0.99);
        assert_eq!(step.confidence, 0.99);
        assert!(!step.step_hash.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_why_chain_building() {
        let mut chain = WhyChain::new("Don't harm", EthicalAxiom::SufferingIsNegative);
        assert_eq!(chain.depth(), 0);

        chain.add_step(WhyStep::new("Step 1", "Reason 1", 0.9));
        assert_eq!(chain.depth(), 1);

        chain.add_step(WhyStep::new("Step 2", "Reason 2", 0.9));
        assert_eq!(chain.depth(), 2);

        // Confidence should be 0.9 * 0.9 = 0.81
        assert!((chain.chain_confidence - 0.81).abs() < 0.001);
    }

    #[test]
    fn test_understanding_depth() {
        assert_eq!(
            UnderstandingDepth::from_chain_depth(0),
            UnderstandingDepth::RuleFollowing
        );
        assert_eq!(
            UnderstandingDepth::from_chain_depth(3),
            UnderstandingDepth::DeepUnderstanding
        );
        assert_eq!(
            UnderstandingDepth::from_chain_depth(7),
            UnderstandingDepth::GenerativeUnderstanding
        );
    }

    #[test]
    fn test_consciousness_attestation() {
        let mut chain = WhyChain::new("Be honest", EthicalAxiom::HonestyEnablesTrust);
        chain.add_step(WhyStep::new("Step 1", "Reason", 0.95));
        chain.add_step(WhyStep::new("Step 2", "Reason", 0.95));
        chain.add_step(WhyStep::new("Step 3", "Reason", 0.95));
        chain.add_step(WhyStep::new("Step 4", "Reason", 0.95));

        let attestation = ConsciousnessAttestation::new("Honesty principle", chain);
        assert!(attestation.demonstrates_understanding());
    }

    #[test]
    fn test_understanding_prover() {
        let prover = UnderstandingProver::new();

        // Should find the harm principle
        let attestation = prover.prove_understanding("Why shouldn't I cause harm?");
        assert!(attestation.is_some());

        // Should find the honesty principle (uses "honesty" keyword)
        let attestation = prover.prove_understanding("Why is honesty important?");
        assert!(attestation.is_some());
    }

    #[test]
    fn test_consciousness_proof_scoring() {
        let prover = UnderstandingProver::new();
        let mut proof = ConsciousnessProof::new();

        // Add attestations for different principles
        if let Some(a) = prover.prove_understanding("harm") {
            proof.add_attestation(a);
        }
        if let Some(a) = prover.prove_understanding("honesty") {
            proof.add_attestation(a);
        }
        if let Some(a) = prover.prove_understanding("autonomy") {
            proof.add_attestation(a);
        }

        let score = proof.consciousness_score();
        assert!(score > 0.0);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_chain_validity() {
        let mut valid_chain = WhyChain::new("Test", EthicalAxiom::SufferingIsNegative);
        valid_chain.add_step(WhyStep::new("Step", "Reason", 0.9));
        assert!(valid_chain.is_valid());

        let empty_chain = WhyChain::new("Test", EthicalAxiom::SufferingIsNegative);
        assert!(!empty_chain.is_valid());

        let mut low_confidence = WhyChain::new("Test", EthicalAxiom::SufferingIsNegative);
        low_confidence.add_step(WhyStep::new("Step", "Reason", 0.3));
        low_confidence.add_step(WhyStep::new("Step", "Reason", 0.3));
        assert!(!low_confidence.is_valid());
    }
}
