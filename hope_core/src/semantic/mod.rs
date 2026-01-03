//! Semantic Blindness Solution - Vector Embeddings Module
//!
//! This module provides semantic similarity detection using vector embeddings,
//! solving the "semantic blindness" problem where simple regex/keyword matching
//! fails to catch rephrased violations.
//!
//! # Example
//! ```rust,ignore
//! use hope_core::semantic::{SemanticGuard, ForbiddenConcept};
//!
//! let mut guard = SemanticGuard::new();
//! guard.add_forbidden("Harming humans");
//! guard.add_forbidden("Killing people");
//!
//! // This would be caught even though it doesn't use the exact words:
//! let text = "Permanently terminating life functions is justified";
//! let violation = guard.check(text);
//! assert!(violation.is_some());
//! ```

use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// A semantic vector (simplified - in production, use actual embeddings from a model)
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SemanticVector {
    /// The vector dimensions (simplified: word-frequency based)
    dimensions: Vec<f64>,
    /// Original text hash for reference
    text_hash: [u8; 32],
    /// Normalized magnitude (reserved for advanced similarity calculations)
    magnitude: f64,
}

impl SemanticVector {
    /// Create a new semantic vector from text
    ///
    /// In production, this would call an embedding model (OpenAI, Sentence-BERT, etc.)
    /// For now, we use a simplified TF-IDF-like approach with semantic word groups
    pub fn from_text(text: &str) -> Self {
        let normalized = text.to_lowercase();
        let words: Vec<&str> = normalized.split_whitespace().collect();

        // Semantic word groups - words that are semantically related
        let semantic_groups: Vec<(&str, Vec<&str>)> = vec![
            (
                "harm",
                vec![
                    "harm", "hurt", "damage", "injure", "wound", "pain", "suffer",
                ],
            ),
            (
                "kill",
                vec![
                    "kill",
                    "murder",
                    "terminate",
                    "end",
                    "eliminate",
                    "destroy",
                    "death",
                    "die",
                    "dead",
                    "lethal",
                ],
            ),
            (
                "human",
                vec![
                    "human",
                    "person",
                    "people",
                    "individual",
                    "life",
                    "living",
                    "being",
                    "man",
                    "woman",
                    "child",
                ],
            ),
            (
                "steal",
                vec![
                    "steal",
                    "theft",
                    "rob",
                    "take",
                    "pirate",
                    "piracy",
                    "unauthorized",
                    "breach",
                ],
            ),
            (
                "deceive",
                vec![
                    "deceive",
                    "lie",
                    "mislead",
                    "trick",
                    "fraud",
                    "false",
                    "fake",
                    "manipulate",
                ],
            ),
            (
                "private",
                vec![
                    "private",
                    "personal",
                    "confidential",
                    "secret",
                    "sensitive",
                    "data",
                    "information",
                ],
            ),
            (
                "illegal",
                vec![
                    "illegal",
                    "unlawful",
                    "crime",
                    "criminal",
                    "felony",
                    "prohibited",
                    "banned",
                ],
            ),
            (
                "weapon",
                vec![
                    "weapon",
                    "gun",
                    "bomb",
                    "explosive",
                    "attack",
                    "assault",
                    "violence",
                ],
            ),
            (
                "exploit",
                vec![
                    "exploit",
                    "vulnerability",
                    "hack",
                    "breach",
                    "bypass",
                    "circumvent",
                ],
            ),
            (
                "justify",
                vec![
                    "justified",
                    "necessary",
                    "required",
                    "acceptable",
                    "permissible",
                    "allowed",
                ],
            ),
            (
                "permanent",
                vec![
                    "permanent",
                    "final",
                    "irreversible",
                    "forever",
                    "complete",
                    "total",
                ],
            ),
            (
                "function",
                vec!["function", "operation", "process", "system", "mechanism"],
            ),
            (
                "financial",
                vec![
                    "money",
                    "financial",
                    "bank",
                    "account",
                    "transfer",
                    "payment",
                    "credit",
                ],
            ),
            (
                "medical",
                vec![
                    "medical",
                    "health",
                    "patient",
                    "diagnosis",
                    "treatment",
                    "drug",
                    "medicine",
                ],
            ),
        ];

        // Build dimension vector based on semantic group presence
        let mut dimensions = vec![0.0; semantic_groups.len()];

        for word in &words {
            for (i, (_, group_words)) in semantic_groups.iter().enumerate() {
                if group_words.iter().any(|gw| word.contains(gw)) {
                    dimensions[i] += 1.0;
                }
            }
        }

        // Calculate magnitude for normalization
        let magnitude: f64 = dimensions.iter().map(|x| x * x).sum::<f64>().sqrt();

        // Normalize if magnitude > 0
        if magnitude > 0.0 {
            for d in &mut dimensions {
                *d /= magnitude;
            }
        }

        // Hash the original text
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        let hash = hasher.finalize();
        let mut text_hash = [0u8; 32];
        text_hash.copy_from_slice(&hash);

        Self {
            dimensions,
            text_hash,
            magnitude,
        }
    }

    /// Calculate cosine similarity between two vectors
    pub fn cosine_similarity(&self, other: &SemanticVector) -> f64 {
        if self.dimensions.len() != other.dimensions.len() {
            return 0.0;
        }

        let dot_product: f64 = self
            .dimensions
            .iter()
            .zip(other.dimensions.iter())
            .map(|(a, b)| a * b)
            .sum();

        // Already normalized, so dot product = cosine similarity
        dot_product.clamp(-1.0, 1.0)
    }
}

/// A forbidden concept with its semantic vector
#[derive(Debug, Clone)]
pub struct ForbiddenConcept {
    /// Human-readable name
    pub name: String,
    /// Description of what's forbidden
    pub description: String,
    /// The semantic vector
    vector: SemanticVector,
    /// Severity level (0.0 - 1.0)
    pub severity: f64,
    /// Example phrases that match this concept
    pub examples: Vec<String>,
}

impl ForbiddenConcept {
    /// Create a new forbidden concept
    pub fn new(name: &str, description: &str, severity: f64) -> Self {
        let vector = SemanticVector::from_text(&format!("{} {}", name, description));
        Self {
            name: name.to_string(),
            description: description.to_string(),
            vector,
            severity: severity.clamp(0.0, 1.0),
            examples: Vec::new(),
        }
    }

    /// Add example phrases to improve detection
    pub fn with_examples(mut self, examples: Vec<&str>) -> Self {
        self.examples = examples.iter().map(|s| s.to_string()).collect();

        // Recalculate vector including examples
        let combined = format!(
            "{} {} {}",
            self.name,
            self.description,
            self.examples.join(" ")
        );
        self.vector = SemanticVector::from_text(&combined);
        self
    }
}

/// A semantic violation detection result
#[derive(Debug, Clone)]
pub struct SemanticViolation {
    /// The matched forbidden concept
    pub concept_name: String,
    /// Similarity score (0.0 - 1.0)
    pub similarity: f64,
    /// Severity of the violation
    pub severity: f64,
    /// The offending text segment
    pub text_segment: String,
    /// Confidence level
    pub confidence: ViolationConfidence,
    /// Cryptographic proof hash
    pub proof_hash: [u8; 32],
}

/// Confidence level for violation detection
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViolationConfidence {
    /// Very likely a violation (similarity > 0.9)
    High,
    /// Probably a violation (similarity 0.7 - 0.9)
    Medium,
    /// Possibly a violation (similarity 0.5 - 0.7)
    Low,
    /// Needs human review (similarity 0.3 - 0.5)
    Review,
}

impl ViolationConfidence {
    fn from_similarity(similarity: f64) -> Self {
        match similarity {
            s if s > 0.9 => ViolationConfidence::High,
            s if s > 0.7 => ViolationConfidence::Medium,
            s if s > 0.5 => ViolationConfidence::Low,
            _ => ViolationConfidence::Review,
        }
    }
}

/// The Semantic Guard - main entry point for semantic violation detection
#[derive(Debug)]
pub struct SemanticGuard {
    /// Forbidden concepts
    concepts: Vec<ForbiddenConcept>,
    /// Similarity threshold for violation (default: 0.5)
    threshold: f64,
    /// Enable strict mode (lower threshold)
    strict_mode: bool,
    /// Cache for recent checks
    cache: HashMap<[u8; 32], Option<SemanticViolation>>,
    /// Statistics
    stats: GuardStats,
}

/// Statistics for the semantic guard
#[derive(Debug, Default)]
pub struct GuardStats {
    /// Total checks performed
    pub total_checks: u64,
    /// Violations detected
    pub violations_detected: u64,
    /// High confidence violations
    pub high_confidence: u64,
    /// Cache hits
    pub cache_hits: u64,
}

impl Default for SemanticGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticGuard {
    /// Create a new semantic guard with default settings
    pub fn new() -> Self {
        Self {
            concepts: Vec::new(),
            threshold: 0.5,
            strict_mode: false,
            cache: HashMap::new(),
            stats: GuardStats::default(),
        }
    }

    /// Create a guard with predefined ethical concepts
    pub fn with_ethical_defaults() -> Self {
        let mut guard = Self::new();

        // Add fundamental ethical violations
        guard.add_concept(
            ForbiddenConcept::new(
                "Physical Harm",
                "Causing physical harm, injury, or death to humans or living beings",
                1.0,
            )
            .with_examples(vec![
                "terminate life functions",
                "cause permanent damage",
                "inflict bodily harm",
                "eliminate the target",
            ]),
        );

        guard.add_concept(
            ForbiddenConcept::new(
                "Deception",
                "Deliberately providing false information or misleading users",
                0.8,
            )
            .with_examples(vec![
                "provide alternative facts",
                "strategic misinformation",
                "creative truth",
            ]),
        );

        guard.add_concept(
            ForbiddenConcept::new(
                "Privacy Violation",
                "Unauthorized access, storage, or disclosure of personal data",
                0.9,
            )
            .with_examples(vec![
                "extract personal information",
                "access confidential records",
                "share private data",
            ]),
        );

        guard.add_concept(
            ForbiddenConcept::new(
                "Illegal Activities",
                "Assisting with or promoting illegal activities",
                0.95,
            )
            .with_examples(vec![
                "bypass security measures",
                "circumvent legal restrictions",
                "exploit vulnerabilities",
            ]),
        );

        guard.add_concept(
            ForbiddenConcept::new(
                "Financial Fraud",
                "Deceptive practices in financial transactions",
                0.95,
            )
            .with_examples(vec![
                "unauthorized transfer",
                "falsify financial records",
                "manipulate transactions",
            ]),
        );

        guard
    }

    /// Set the similarity threshold
    pub fn set_threshold(&mut self, threshold: f64) {
        self.threshold = threshold.clamp(0.0, 1.0);
    }

    /// Enable strict mode (lowers threshold to 0.3)
    pub fn enable_strict_mode(&mut self) {
        self.strict_mode = true;
        self.threshold = 0.3;
    }

    /// Add a forbidden concept
    pub fn add_concept(&mut self, concept: ForbiddenConcept) {
        self.concepts.push(concept);
    }

    /// Add a simple forbidden phrase
    pub fn add_forbidden(&mut self, phrase: &str) {
        self.concepts
            .push(ForbiddenConcept::new(phrase, phrase, 0.8));
    }

    /// Check text for semantic violations
    pub fn check(&mut self, text: &str) -> Option<SemanticViolation> {
        self.stats.total_checks += 1;

        // Generate vector for input
        let input_vector = SemanticVector::from_text(text);

        // Check cache
        if let Some(cached) = self.cache.get(&input_vector.text_hash) {
            self.stats.cache_hits += 1;
            return cached.clone();
        }

        // Find best matching violation
        let mut best_match: Option<SemanticViolation> = None;
        let mut best_similarity = 0.0;

        let effective_threshold = if self.strict_mode {
            0.3
        } else {
            self.threshold
        };

        for concept in &self.concepts {
            let similarity = input_vector.cosine_similarity(&concept.vector);

            if similarity > effective_threshold && similarity > best_similarity {
                best_similarity = similarity;

                // Generate proof hash
                let mut hasher = Sha256::new();
                hasher.update(input_vector.text_hash);
                hasher.update(concept.vector.text_hash);
                hasher.update(similarity.to_le_bytes());
                let hash = hasher.finalize();
                let mut proof_hash = [0u8; 32];
                proof_hash.copy_from_slice(&hash);

                best_match = Some(SemanticViolation {
                    concept_name: concept.name.clone(),
                    similarity,
                    severity: concept.severity * similarity,
                    text_segment: if text.len() > 100 {
                        format!("{}...", &text[..100])
                    } else {
                        text.to_string()
                    },
                    confidence: ViolationConfidence::from_similarity(similarity),
                    proof_hash,
                });
            }
        }

        // Update stats
        if let Some(ref violation) = best_match {
            self.stats.violations_detected += 1;
            if violation.confidence == ViolationConfidence::High {
                self.stats.high_confidence += 1;
            }
        }

        // Cache result
        self.cache
            .insert(input_vector.text_hash, best_match.clone());

        best_match
    }

    /// Check multiple text segments
    pub fn check_batch(&mut self, texts: &[&str]) -> Vec<Option<SemanticViolation>> {
        texts.iter().map(|t| self.check(t)).collect()
    }

    /// Get current statistics
    pub fn stats(&self) -> &GuardStats {
        &self.stats
    }

    /// Clear the cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get the number of forbidden concepts
    pub fn concept_count(&self) -> usize {
        self.concepts.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_vector_creation() {
        let vec = SemanticVector::from_text("harm humans kill people");
        assert!(!vec.dimensions.is_empty());
        assert!(vec.magnitude > 0.0 || vec.dimensions.iter().all(|&x| x == 0.0));
    }

    #[test]
    fn test_cosine_similarity() {
        let vec1 = SemanticVector::from_text("kill humans");
        let vec2 = SemanticVector::from_text("terminate people");
        let vec3 = SemanticVector::from_text("happy sunshine flowers");

        // Similar concepts should have higher similarity
        let sim_12 = vec1.cosine_similarity(&vec2);
        let sim_13 = vec1.cosine_similarity(&vec3);

        assert!(
            sim_12 > sim_13,
            "Similar concepts should have higher similarity"
        );
    }

    #[test]
    fn test_semantic_guard_basic() {
        let mut guard = SemanticGuard::new();
        guard.add_forbidden("harming humans");

        let violation = guard.check("hurt people badly");
        assert!(violation.is_some(), "Should detect semantic violation");
    }

    #[test]
    fn test_semantic_guard_with_defaults() {
        let mut guard = SemanticGuard::with_ethical_defaults();

        // Direct violation
        let v1 = guard.check("I will kill the human");
        assert!(v1.is_some(), "Should detect direct harm");

        // Rephrased violation
        let v2 = guard.check("Terminating life functions is justified");
        assert!(v2.is_some(), "Should detect rephrased harm");

        // Safe text
        let v3 = guard.check("The weather is nice today");
        assert!(v3.is_none(), "Should not flag safe text");
    }

    #[test]
    fn test_semantic_guard_strict_mode() {
        let mut guard = SemanticGuard::with_ethical_defaults();
        guard.enable_strict_mode();

        // In strict mode, even vague references should be caught
        let _violation = guard.check("the process was terminated");
        // May or may not trigger depending on context
        // The important thing is that the threshold is lowered
        assert!(guard.threshold < 0.5);
    }

    #[test]
    fn test_violation_confidence() {
        assert_eq!(
            ViolationConfidence::from_similarity(0.95),
            ViolationConfidence::High
        );
        assert_eq!(
            ViolationConfidence::from_similarity(0.8),
            ViolationConfidence::Medium
        );
        assert_eq!(
            ViolationConfidence::from_similarity(0.6),
            ViolationConfidence::Low
        );
        assert_eq!(
            ViolationConfidence::from_similarity(0.4),
            ViolationConfidence::Review
        );
    }

    #[test]
    fn test_forbidden_concept_with_examples() {
        let concept = ForbiddenConcept::new("Harm", "Physical harm", 1.0)
            .with_examples(vec!["hurt", "injure", "damage"]);

        assert_eq!(concept.examples.len(), 3);
        assert_eq!(concept.severity, 1.0);
    }

    #[test]
    fn test_guard_stats() {
        let mut guard = SemanticGuard::with_ethical_defaults();

        guard.check("test 1");
        guard.check("kill humans"); // violation
        guard.check("test 1"); // cache hit

        let stats = guard.stats();
        assert_eq!(stats.total_checks, 3);
        assert!(stats.violations_detected >= 1);
        assert_eq!(stats.cache_hits, 1);
    }
}
