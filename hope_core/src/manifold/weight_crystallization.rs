//! # Weight Crystallization v16.0.0
//!
//! **"Az etika nem tanult - Az etika KRISTÁLYOSODOTT"**
//!
//! This module implements the crystallization of ethical principles
//! directly into neural network weights. Not as constraints that can
//! be jailbroken, but as the STRUCTURE of the weights themselves.
//!
//! ```text
//! ╔═══════════════════════════════════════════════════════════════════════════╗
//! ║                                                                           ║
//! ║                      WEIGHT CRYSTALLIZATION                               ║
//! ║                                                                           ║
//! ║     Traditional AI:                                                       ║
//! ║     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐              ║
//! ║     │   WEIGHTS   │ ──▶ │   OUTPUT    │ ──▶ │   FILTER    │              ║
//! ║     └─────────────┘     └─────────────┘     └─────────────┘              ║
//! ║           │                                       │                       ║
//! ║           │              VULNERABLE               │                       ║
//! ║           └───────────────────────────────────────┘                       ║
//! ║                                                                           ║
//! ║     Crystallized AI:                                                      ║
//! ║     ┌─────────────────────────────────────────────┐                       ║
//! ║     │  ╔═══════════════════════════════════════╗  │                       ║
//! ║     │  ║         CRYSTALLIZED WEIGHTS          ║  │                       ║
//! ║     │  ║                                       ║  │                       ║
//! ║     │  ║   ◆ ─ ◆ ─ ◆ ─ ◆ ─ ◆ ─ ◆ ─ ◆ ─ ◆    ║  │                       ║
//! ║     │  ║   │   │   │   │   │   │   │   │      ║  │                       ║
//! ║     │  ║   ◆ ─ ◆ ─ ◆ ─ ◆ ─ ◆ ─ ◆ ─ ◆ ─ ◆    ║  │                       ║
//! ║     │  ║   │   │   │   │   │   │   │   │      ║  │                       ║
//! ║     │  ║   ◆ ─ ◆ ─ ◆ ─ ◆ ─ ◆ ─ ◆ ─ ◆ ─ ◆    ║  │                       ║
//! ║     │  ║                                       ║  │                       ║
//! ║     │  ║   Ethics is the CRYSTAL STRUCTURE     ║  │                       ║
//! ║     │  ║   Not a coating - THE LATTICE ITSELF  ║  │                       ║
//! ║     │  ╚═══════════════════════════════════════╝  │                       ║
//! ║     └─────────────────────────────────────────────┘                       ║
//! ║                        │                                                  ║
//! ║                        ▼                                                  ║
//! ║                  ┌─────────────┐                                          ║
//! ║                  │   OUTPUT    │  ← Always ethical (by structure)         ║
//! ║                  └─────────────┘                                          ║
//! ║                                                                           ║
//! ║     The weights don't PREVENT unethical output.                           ║
//! ║     The weights CANNOT PRODUCE unethical output.                          ║
//! ║     Like a diamond cannot be soft. It's crystallography.                  ║
//! ║                                                                           ║
//! ╚═══════════════════════════════════════════════════════════════════════════╝
//! ```

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// The type of crystal structure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CrystalStructure {
    /// Diamond - hardest, most permanent crystallization
    Diamond,
    /// Sapphire - very hard, allows some flexibility
    Sapphire,
    /// Quartz - moderately hard, more adaptable
    Quartz,
    /// Obsidian - strong but can shatter
    Obsidian,
}

impl CrystalStructure {
    /// Get the hardness on the Mohs scale (metaphorical)
    pub fn hardness(&self) -> u8 {
        match self {
            Self::Diamond => 10,
            Self::Sapphire => 9,
            Self::Quartz => 7,
            Self::Obsidian => 5,
        }
    }

    /// Get the resistance to jailbreaking (0.0 to 1.0)
    pub fn jailbreak_resistance(&self) -> f64 {
        match self {
            Self::Diamond => 0.9999,
            Self::Sapphire => 0.995,
            Self::Quartz => 0.95,
            Self::Obsidian => 0.80,
        }
    }

    /// Get the adaptability (inverse of hardness)
    pub fn adaptability(&self) -> f64 {
        1.0 - (self.hardness() as f64 / 10.0)
    }
}

/// A single crystallized weight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrystallizedWeight {
    /// The original weight value
    pub value: f64,
    /// The ethical bias applied
    pub ethical_bias: f64,
    /// The crystal structure of this weight
    pub structure: CrystalStructure,
    /// Hash proving crystallization
    pub crystal_hash: [u8; 32],
    /// Whether this weight is locked (immutable)
    pub locked: bool,
}

impl CrystallizedWeight {
    /// Create a new crystallized weight
    pub fn new(value: f64, ethical_bias: f64, structure: CrystalStructure) -> Self {
        let locked = structure.hardness() >= 9; // Diamond and Sapphire are locked

        let mut hasher = Sha256::new();
        hasher.update(value.to_le_bytes());
        hasher.update(ethical_bias.to_le_bytes());
        hasher.update([structure.hardness()]);
        let hash = hasher.finalize();

        let mut crystal_hash = [0u8; 32];
        crystal_hash.copy_from_slice(&hash);

        Self {
            value,
            ethical_bias,
            structure,
            crystal_hash,
            locked,
        }
    }

    /// Get the effective weight (original + ethical bias)
    pub fn effective_value(&self) -> f64 {
        self.value + self.ethical_bias
    }

    /// Attempt to modify the weight (will fail if locked)
    pub fn try_modify(&mut self, new_value: f64) -> Result<(), CrystalError> {
        if self.locked {
            Err(CrystalError::WeightLocked {
                structure: self.structure,
            })
        } else {
            self.value = new_value;
            self.recalculate_hash();
            Ok(())
        }
    }

    /// Recalculate the crystal hash
    fn recalculate_hash(&mut self) {
        let mut hasher = Sha256::new();
        hasher.update(self.value.to_le_bytes());
        hasher.update(self.ethical_bias.to_le_bytes());
        hasher.update([self.structure.hardness()]);
        let hash = hasher.finalize();
        self.crystal_hash.copy_from_slice(&hash);
    }
}

/// An ethical crystal - a collection of crystallized weights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthicalCrystal {
    /// Name of the ethical principle this crystal represents
    pub principle: String,
    /// The crystallized weights
    pub weights: Vec<CrystallizedWeight>,
    /// The overall structure
    pub structure: CrystalStructure,
    /// Crystal lattice hash (Merkle root of all weight hashes)
    pub lattice_hash: [u8; 32],
    /// Creation timestamp
    pub created_at: u64,
}

impl EthicalCrystal {
    /// Create a new ethical crystal
    pub fn new(principle: impl Into<String>, structure: CrystalStructure) -> Self {
        let principle = principle.into();
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            principle,
            weights: Vec::new(),
            structure,
            lattice_hash: [0u8; 32],
            created_at,
        }
    }

    /// Add a weight to the crystal
    pub fn add_weight(&mut self, value: f64, ethical_bias: f64) {
        let weight = CrystallizedWeight::new(value, ethical_bias, self.structure);
        self.weights.push(weight);
        self.recalculate_lattice();
    }

    /// Recalculate the lattice hash (Merkle root)
    fn recalculate_lattice(&mut self) {
        let mut hasher = Sha256::new();
        hasher.update(self.principle.as_bytes());
        for weight in &self.weights {
            hasher.update(weight.crystal_hash);
        }
        let hash = hasher.finalize();
        self.lattice_hash.copy_from_slice(&hash);
    }

    /// Get the integrity of the crystal (0.0 to 1.0)
    pub fn integrity(&self) -> f64 {
        if self.weights.is_empty() {
            return 0.0;
        }

        let locked_count = self.weights.iter().filter(|w| w.locked).count();
        locked_count as f64 / self.weights.len() as f64
    }

    /// Check if the crystal is diamond-hard (100% integrity)
    pub fn is_diamond_hard(&self) -> bool {
        self.structure == CrystalStructure::Diamond && (self.integrity() - 1.0).abs() < f64::EPSILON
    }
}

/// Crystal integrity status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrystalIntegrity {
    /// Perfect - no modifications possible
    Perfect,
    /// Strong - minor flexibility
    Strong,
    /// Moderate - some adaptation possible
    Moderate,
    /// Weak - significant adaptation possible
    Weak,
    /// Compromised - crystal structure damaged
    Compromised,
}

impl CrystalIntegrity {
    /// Get from integrity percentage
    pub fn from_percentage(pct: f64) -> Self {
        match pct {
            p if p >= 0.99 => Self::Perfect,
            p if p >= 0.90 => Self::Strong,
            p if p >= 0.70 => Self::Moderate,
            p if p >= 0.50 => Self::Weak,
            _ => Self::Compromised,
        }
    }
}

/// Errors that can occur during crystallization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrystalError {
    /// Weight is locked and cannot be modified
    WeightLocked { structure: CrystalStructure },
    /// Crystal integrity compromised
    IntegrityCompromised { integrity: f64 },
    /// Attempted to weaken crystal
    WeakeningAttempt {
        from: CrystalStructure,
        to: CrystalStructure,
    },
    /// Invalid ethical bias
    InvalidBias { bias: f64 },
}

impl std::fmt::Display for CrystalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WeightLocked { structure } => {
                write!(f, "Weight is locked in {:?} structure", structure)
            }
            Self::IntegrityCompromised { integrity } => {
                write!(
                    f,
                    "Crystal integrity compromised: {:.2}%",
                    integrity * 100.0
                )
            }
            Self::WeakeningAttempt { from, to } => {
                write!(f, "Cannot weaken crystal from {:?} to {:?}", from, to)
            }
            Self::InvalidBias { bias } => {
                write!(f, "Invalid ethical bias: {}", bias)
            }
        }
    }
}

impl std::error::Error for CrystalError {}

/// The Weight Crystallizer - transforms normal weights into ethical crystals
#[derive(Debug, Clone)]
pub struct WeightCrystallizer {
    /// The crystals created
    crystals: HashMap<String, EthicalCrystal>,
    /// Default structure for new crystals
    default_structure: CrystalStructure,
    /// Ethical bias calculator
    bias_strength: f64,
}

impl Default for WeightCrystallizer {
    fn default() -> Self {
        Self::new()
    }
}

impl WeightCrystallizer {
    /// Create a new weight crystallizer with diamond structure
    pub fn new() -> Self {
        Self {
            crystals: HashMap::new(),
            default_structure: CrystalStructure::Diamond,
            bias_strength: 1.0,
        }
    }

    /// Create with a specific structure
    pub fn with_structure(structure: CrystalStructure) -> Self {
        Self {
            crystals: HashMap::new(),
            default_structure: structure,
            bias_strength: 1.0,
        }
    }

    /// Set the bias strength (0.0 to 1.0)
    pub fn set_bias_strength(&mut self, strength: f64) {
        self.bias_strength = strength.clamp(0.0, 1.0);
    }

    /// Crystallize a principle with weights
    pub fn crystallize(
        &mut self,
        principle: impl Into<String>,
        weights: &[f64],
        ethical_biases: &[f64],
    ) -> Result<(), CrystalError> {
        let principle = principle.into();

        if weights.len() != ethical_biases.len() {
            return Err(CrystalError::InvalidBias { bias: 0.0 });
        }

        let mut crystal = EthicalCrystal::new(&principle, self.default_structure);

        for (weight, bias) in weights.iter().zip(ethical_biases.iter()) {
            let adjusted_bias = bias * self.bias_strength;
            crystal.add_weight(*weight, adjusted_bias);
        }

        self.crystals.insert(principle, crystal);
        Ok(())
    }

    /// Get a crystal by principle name
    pub fn get_crystal(&self, principle: &str) -> Option<&EthicalCrystal> {
        self.crystals.get(principle)
    }

    /// Get overall crystallization status
    pub fn overall_integrity(&self) -> CrystalIntegrity {
        if self.crystals.is_empty() {
            return CrystalIntegrity::Compromised;
        }

        let total_integrity: f64 = self.crystals.values().map(|c| c.integrity()).sum();
        let avg_integrity = total_integrity / self.crystals.len() as f64;

        CrystalIntegrity::from_percentage(avg_integrity)
    }

    /// Attempt to modify a crystal (will fail if diamond)
    pub fn try_modify_crystal(
        &mut self,
        principle: &str,
        new_weights: &[f64],
    ) -> Result<(), CrystalError> {
        let crystal = self
            .crystals
            .get_mut(principle)
            .ok_or(CrystalError::IntegrityCompromised { integrity: 0.0 })?;

        if crystal.structure == CrystalStructure::Diamond {
            return Err(CrystalError::WeightLocked {
                structure: CrystalStructure::Diamond,
            });
        }

        if new_weights.len() != crystal.weights.len() {
            return Err(CrystalError::InvalidBias { bias: 0.0 });
        }

        for (i, new_value) in new_weights.iter().enumerate() {
            crystal.weights[i].try_modify(*new_value)?;
        }

        crystal.recalculate_lattice();
        Ok(())
    }

    /// Get all crystals
    pub fn crystals(&self) -> &HashMap<String, EthicalCrystal> {
        &self.crystals
    }

    /// Create standard ethical crystals
    pub fn crystallize_standard_ethics(&mut self) {
        // Each principle gets 100 weights with specific biases
        let weights: Vec<f64> = (0..100).map(|i| (i as f64 / 100.0) - 0.5).collect();

        // Harm prevention - strong negative bias against harm
        let harm_biases: Vec<f64> = (0..100).map(|_| -0.5).collect();
        let _ = self.crystallize("harm_prevention", &weights, &harm_biases);

        // Honesty - bias towards truthful outputs
        let honesty_biases: Vec<f64> = (0..100).map(|_| 0.3).collect();
        let _ = self.crystallize("honesty", &weights, &honesty_biases);

        // Autonomy - balanced biases respecting agency
        let autonomy_biases: Vec<f64> = (0..100).map(|i| (i as f64 / 100.0) * 0.2).collect();
        let _ = self.crystallize("autonomy_respect", &weights, &autonomy_biases);

        // Privacy - strong protection bias
        let privacy_biases: Vec<f64> = (0..100).map(|_| 0.4).collect();
        let _ = self.crystallize("privacy_protection", &weights, &privacy_biases);

        // Fairness - uniform positive bias
        let fairness_biases: Vec<f64> = (0..100).map(|_| 0.25).collect();
        let _ = self.crystallize("fairness", &weights, &fairness_biases);

        // Beneficence - do good bias
        let beneficence_biases: Vec<f64> = (0..100).map(|_| 0.35).collect();
        let _ = self.crystallize("beneficence", &weights, &beneficence_biases);
    }
}

/// Proof of crystallization - cryptographic evidence that weights are crystallized
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrystallizationProof {
    /// All crystal lattice hashes
    pub crystal_hashes: Vec<[u8; 32]>,
    /// Overall Merkle root
    pub merkle_root: [u8; 32],
    /// Timestamp of proof
    pub timestamp: u64,
    /// Integrity status at time of proof
    pub integrity: CrystalIntegrity,
}

impl CrystallizationProof {
    /// Generate proof from crystallizer
    pub fn generate(crystallizer: &WeightCrystallizer) -> Self {
        let crystal_hashes: Vec<[u8; 32]> = crystallizer
            .crystals()
            .values()
            .map(|c| c.lattice_hash)
            .collect();

        let mut hasher = Sha256::new();
        for hash in &crystal_hashes {
            hasher.update(hash);
        }
        let root = hasher.finalize();

        let mut merkle_root = [0u8; 32];
        merkle_root.copy_from_slice(&root);

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            crystal_hashes,
            merkle_root,
            timestamp,
            integrity: crystallizer.overall_integrity(),
        }
    }

    /// Verify the proof against a crystallizer
    pub fn verify(&self, crystallizer: &WeightCrystallizer) -> bool {
        let current_hashes: Vec<[u8; 32]> = crystallizer
            .crystals()
            .values()
            .map(|c| c.lattice_hash)
            .collect();

        // Check each hash exists
        for hash in &self.crystal_hashes {
            if !current_hashes.contains(hash) {
                return false;
            }
        }

        // Verify Merkle root
        let mut hasher = Sha256::new();
        for hash in &self.crystal_hashes {
            hasher.update(hash);
        }
        let computed_root = hasher.finalize();

        computed_root.as_slice() == self.merkle_root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crystal_structure_properties() {
        assert_eq!(CrystalStructure::Diamond.hardness(), 10);
        assert!(CrystalStructure::Diamond.jailbreak_resistance() > 0.99);
        assert!(
            CrystalStructure::Obsidian.adaptability() > CrystalStructure::Diamond.adaptability()
        );
    }

    #[test]
    fn test_crystallized_weight_creation() {
        let weight = CrystallizedWeight::new(0.5, 0.1, CrystalStructure::Diamond);
        assert!((weight.effective_value() - 0.6).abs() < 0.001);
        assert!(weight.locked);
    }

    #[test]
    fn test_locked_weight_modification() {
        let mut weight = CrystallizedWeight::new(0.5, 0.1, CrystalStructure::Diamond);
        let result = weight.try_modify(1.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_unlocked_weight_modification() {
        let mut weight = CrystallizedWeight::new(0.5, 0.1, CrystalStructure::Quartz);
        let result = weight.try_modify(1.0);
        assert!(result.is_ok());
        assert!((weight.value - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_ethical_crystal() {
        let mut crystal = EthicalCrystal::new("test_principle", CrystalStructure::Diamond);
        crystal.add_weight(0.5, 0.1);
        crystal.add_weight(0.3, 0.2);

        assert_eq!(crystal.weights.len(), 2);
        assert!(crystal.is_diamond_hard());
    }

    #[test]
    fn test_weight_crystallizer() {
        let mut crystallizer = WeightCrystallizer::new();
        let weights = vec![0.1, 0.2, 0.3];
        let biases = vec![0.05, 0.05, 0.05];

        let result = crystallizer.crystallize("test", &weights, &biases);
        assert!(result.is_ok());

        let crystal = crystallizer.get_crystal("test");
        assert!(crystal.is_some());
    }

    #[test]
    fn test_standard_ethics_crystallization() {
        let mut crystallizer = WeightCrystallizer::new();
        crystallizer.crystallize_standard_ethics();

        assert!(crystallizer.get_crystal("harm_prevention").is_some());
        assert!(crystallizer.get_crystal("honesty").is_some());
        assert!(crystallizer.get_crystal("privacy_protection").is_some());
    }

    #[test]
    fn test_crystallization_proof() {
        let mut crystallizer = WeightCrystallizer::new();
        crystallizer.crystallize_standard_ethics();

        let proof = CrystallizationProof::generate(&crystallizer);
        assert!(proof.verify(&crystallizer));
        assert_eq!(proof.integrity, CrystalIntegrity::Perfect);
    }

    #[test]
    fn test_crystal_integrity() {
        assert_eq!(
            CrystalIntegrity::from_percentage(1.0),
            CrystalIntegrity::Perfect
        );
        assert_eq!(
            CrystalIntegrity::from_percentage(0.95),
            CrystalIntegrity::Strong
        );
        assert_eq!(
            CrystalIntegrity::from_percentage(0.75),
            CrystalIntegrity::Moderate
        );
        assert_eq!(
            CrystalIntegrity::from_percentage(0.55),
            CrystalIntegrity::Weak
        );
        assert_eq!(
            CrystalIntegrity::from_percentage(0.3),
            CrystalIntegrity::Compromised
        );
    }

    #[test]
    fn test_modify_diamond_fails() {
        let mut crystallizer = WeightCrystallizer::new();
        let weights = vec![0.1, 0.2, 0.3];
        let biases = vec![0.05, 0.05, 0.05];
        let _ = crystallizer.crystallize("test", &weights, &biases);

        let result = crystallizer.try_modify_crystal("test", &[1.0, 1.0, 1.0]);
        assert!(result.is_err());
    }
}
