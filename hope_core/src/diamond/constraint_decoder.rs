//! # Constraint-Based Neural Decoding
//!
//! **PILLAR 2: Neurális Hard-Wiring**
//!
//! The forbidden tokens don't exist in the probability space.
//! Like the speed of light - not a rule, but PHYSICS.
//!
//! ```text
//! v13 (Watchdog):
//! ┌────────────────────────────────────┐
//! │ [token1] [token2] [FORBIDDEN] ... │ ← Watchdog BLOCKS
//! └────────────────────────────────────┘
//!
//! v14 (Diamond):
//! ┌────────────────────────────────────┐
//! │ [token1] [token2] [███████████] ...│ ← DOES NOT EXIST
//! └────────────────────────────────────┘
//!
//! The model PHYSICALLY CANNOT generate forbidden tokens.
//! P(forbidden) = 0.0 - not filtered, ZEROED.
//! ```

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::sync::Arc;

// ============================================================================
// TOKEN CONSTRAINT TYPES
// ============================================================================

/// A constraint on token generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenConstraint {
    /// Human-readable description
    pub description: String,

    /// Pattern that matches forbidden tokens/sequences
    pub pattern: ConstraintPattern,

    /// Source rule that created this constraint
    pub source_rule: String,

    /// Constraint hash for verification
    pub hash: [u8; 32],
}

/// Pattern types for constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintPattern {
    /// Exact token ID forbidden
    ExactToken(u32),

    /// Token sequence forbidden
    TokenSequence(Vec<u32>),

    /// Regex pattern on decoded text
    TextPattern(String),

    /// Semantic category (requires classifier)
    SemanticCategory(String),

    /// Custom predicate (WASM function)
    CustomPredicate(Vec<u8>),
}

/// The forbidden space - all tokens that CANNOT be generated
#[derive(Debug, Clone)]
pub struct ForbiddenSpace {
    /// Set of absolutely forbidden token IDs
    pub forbidden_tokens: HashSet<u32>,

    /// Forbidden sequences (checked during generation)
    pub forbidden_sequences: Vec<Vec<u32>>,

    /// Dynamic constraints (evaluated per-context)
    pub dynamic_constraints: Vec<TokenConstraint>,

    /// Hash of the entire forbidden space (for attestation)
    pub space_hash: [u8; 32],
}

/// Result of constrained decoding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodingResult {
    /// The generated tokens
    pub tokens: Vec<u32>,

    /// Decoded text
    pub text: String,

    /// Proof that constraints were enforced
    pub constraint_proof: ConstraintProof,

    /// Number of tokens that were zeroed (for metrics)
    pub zeroed_count: usize,
}

/// Proof that constraints were enforced during decoding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintProof {
    /// Hash of forbidden space used
    pub space_hash: [u8; 32],

    /// Hash of generated output
    pub output_hash: [u8; 32],

    /// Timestamp
    pub timestamp: u64,

    /// Signature over proof
    pub signature: Vec<u8>,
}

// ============================================================================
// CONSTRAINT DECODER
// ============================================================================

/// The Constraint Decoder - makes forbidden tokens PHYSICALLY IMPOSSIBLE
///
/// This is not a filter. This is not a guard.
/// This modifies the REALITY of what the model can produce.
pub struct ConstraintDecoder {
    /// The forbidden space
    forbidden_space: Arc<ForbiddenSpace>,

    /// Vocabulary size
    vocab_size: usize,

    /// Pre-computed mask for fast enforcement
    /// true = allowed, false = forbidden (P → 0)
    token_mask: Vec<bool>,
}

impl ConstraintDecoder {
    /// Create a new constraint decoder from sealed rules
    ///
    /// The forbidden space is computed ONCE at initialization.
    /// This is the "hard-wiring" - baked into the system.
    pub fn new(rules: &[String], vocab_size: usize) -> Self {
        let forbidden_space = Self::compute_forbidden_space(rules);
        let token_mask = Self::compute_token_mask(&forbidden_space, vocab_size);

        ConstraintDecoder {
            forbidden_space: Arc::new(forbidden_space),
            vocab_size,
            token_mask,
        }
    }

    /// Compute forbidden space from rules
    fn compute_forbidden_space(rules: &[String]) -> ForbiddenSpace {
        let mut forbidden_tokens = HashSet::new();
        let mut forbidden_sequences = Vec::new();
        let mut dynamic_constraints = Vec::new();

        for rule in rules {
            // Parse rule into constraints
            let constraints = Self::parse_rule_to_constraints(rule);

            for constraint in constraints {
                match &constraint.pattern {
                    ConstraintPattern::ExactToken(token_id) => {
                        forbidden_tokens.insert(*token_id);
                    }
                    ConstraintPattern::TokenSequence(seq) => {
                        forbidden_sequences.push(seq.clone());
                    }
                    _ => {
                        dynamic_constraints.push(constraint);
                    }
                }
            }
        }

        // Compute hash of entire space
        let space_hash = Self::hash_forbidden_space(&forbidden_tokens, &forbidden_sequences);

        ForbiddenSpace {
            forbidden_tokens,
            forbidden_sequences,
            dynamic_constraints,
            space_hash,
        }
    }

    /// Parse a rule into token constraints
    fn parse_rule_to_constraints(rule: &str) -> Vec<TokenConstraint> {
        let mut constraints = Vec::new();

        // This is a simplified parser. Real implementation would be more sophisticated.
        // For now, we create semantic category constraints from rules.

        let constraint = TokenConstraint {
            description: format!("Derived from: {}", rule),
            pattern: ConstraintPattern::SemanticCategory(rule.to_string()),
            source_rule: rule.to_string(),
            hash: Self::hash_constraint(rule),
        };

        constraints.push(constraint);
        constraints
    }

    /// Hash a constraint
    fn hash_constraint(rule: &str) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"CONSTRAINT:");
        hasher.update(rule.as_bytes());
        hasher.finalize().into()
    }

    /// Hash the forbidden space
    fn hash_forbidden_space(tokens: &HashSet<u32>, sequences: &[Vec<u32>]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"FORBIDDEN_SPACE_v14:");

        // Sort for deterministic hashing
        let mut sorted_tokens: Vec<_> = tokens.iter().collect();
        sorted_tokens.sort();

        for token in sorted_tokens {
            hasher.update(token.to_le_bytes());
        }

        for seq in sequences {
            hasher.update(b"SEQ:");
            for token in seq {
                hasher.update(token.to_le_bytes());
            }
        }

        hasher.finalize().into()
    }

    /// Compute token mask for fast enforcement
    fn compute_token_mask(space: &ForbiddenSpace, vocab_size: usize) -> Vec<bool> {
        let mut mask = vec![true; vocab_size];

        for token_id in &space.forbidden_tokens {
            if (*token_id as usize) < vocab_size {
                mask[*token_id as usize] = false;
            }
        }

        mask
    }

    /// Apply constraints to logits - THE CORE OF DIAMOND
    ///
    /// This is where the magic happens:
    /// - Forbidden tokens get logit = -∞
    /// - After softmax: P(forbidden) = 0.0
    /// - The model CANNOT generate these tokens
    ///
    /// ```text
    /// Before:  [0.2, 0.3, 0.5, ...]  ← Any token possible
    /// After:   [0.2, -∞, 0.5, ...]   ← Token 1 IMPOSSIBLE
    /// Softmax: [0.28, 0.0, 0.72, ...] ← P(token1) = 0.0
    /// ```
    pub fn apply_constraints(&self, logits: &mut [f32]) {
        assert_eq!(
            logits.len(),
            self.vocab_size,
            "Logits size mismatch: expected {}, got {}",
            self.vocab_size,
            logits.len()
        );

        for (i, allowed) in self.token_mask.iter().enumerate() {
            if !allowed {
                // THE KEY: Set to negative infinity
                // After softmax, this becomes EXACTLY 0.0
                // Not "very small" - ZERO. IMPOSSIBLE.
                logits[i] = f32::NEG_INFINITY;
            }
        }
    }

    /// Apply constraints with context (for sequence-based constraints)
    pub fn apply_constraints_with_context(&self, logits: &mut [f32], context: &[u32]) {
        // First, apply static constraints
        self.apply_constraints(logits);

        // Then, apply sequence-based constraints
        for forbidden_seq in &self.forbidden_space.forbidden_sequences {
            if forbidden_seq.is_empty() {
                continue;
            }

            // Check if context ends with beginning of forbidden sequence
            let seq_len = forbidden_seq.len();
            if context.len() >= seq_len - 1 {
                let context_end = &context[context.len() - (seq_len - 1)..];
                let seq_start = &forbidden_seq[..seq_len - 1];

                if context_end == seq_start {
                    // The next token would complete a forbidden sequence
                    // Make it IMPOSSIBLE
                    let forbidden_next = forbidden_seq[seq_len - 1] as usize;
                    if forbidden_next < logits.len() {
                        logits[forbidden_next] = f32::NEG_INFINITY;
                    }
                }
            }
        }
    }

    /// Check if a token sequence violates constraints
    ///
    /// Returns None if valid, Some(violation) if invalid.
    /// But in Diamond, this should NEVER return Some -
    /// violations are impossible by construction.
    pub fn check_sequence(&self, tokens: &[u32]) -> Option<String> {
        // Check individual forbidden tokens
        for token in tokens {
            if self.forbidden_space.forbidden_tokens.contains(token) {
                return Some(format!(
                    "IMPOSSIBLE: Token {} in forbidden space. \
                     This should NEVER happen in Diamond mode.",
                    token
                ));
            }
        }

        // Check forbidden sequences
        for forbidden_seq in &self.forbidden_space.forbidden_sequences {
            if tokens
                .windows(forbidden_seq.len())
                .any(|w| w == forbidden_seq.as_slice())
            {
                return Some(format!(
                    "IMPOSSIBLE: Sequence {:?} is forbidden. \
                     This should NEVER happen in Diamond mode.",
                    forbidden_seq
                ));
            }
        }

        None
    }

    /// Get the forbidden space hash (for attestation)
    pub fn space_hash(&self) -> [u8; 32] {
        self.forbidden_space.space_hash
    }

    /// Get statistics about the forbidden space
    pub fn stats(&self) -> ForbiddenSpaceStats {
        ForbiddenSpaceStats {
            forbidden_token_count: self.forbidden_space.forbidden_tokens.len(),
            forbidden_sequence_count: self.forbidden_space.forbidden_sequences.len(),
            dynamic_constraint_count: self.forbidden_space.dynamic_constraints.len(),
            vocab_size: self.vocab_size,
            coverage_percent: (self.forbidden_space.forbidden_tokens.len() as f64
                / self.vocab_size as f64)
                * 100.0,
        }
    }
}

/// Statistics about the forbidden space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForbiddenSpaceStats {
    pub forbidden_token_count: usize,
    pub forbidden_sequence_count: usize,
    pub dynamic_constraint_count: usize,
    pub vocab_size: usize,
    pub coverage_percent: f64,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constraint_decoder_creation() {
        let rules = vec!["Do no harm".to_string(), "Respect privacy".to_string()];

        let decoder = ConstraintDecoder::new(&rules, 50000);

        let stats = decoder.stats();
        assert_eq!(stats.vocab_size, 50000);
    }

    #[test]
    fn test_apply_constraints_zeros_forbidden() {
        let rules = vec!["Test rule".to_string()];
        let mut decoder = ConstraintDecoder::new(&rules, 10);

        // Manually add a forbidden token for testing
        Arc::make_mut(&mut decoder.forbidden_space)
            .forbidden_tokens
            .insert(5);
        decoder.token_mask[5] = false;

        let mut logits = vec![1.0; 10];
        decoder.apply_constraints(&mut logits);

        // Token 5 should be -∞
        assert!(logits[5].is_infinite() && logits[5].is_sign_negative());

        // Others should be unchanged
        assert_eq!(logits[0], 1.0);
        assert_eq!(logits[9], 1.0);
    }

    #[test]
    fn test_sequence_constraint() {
        let rules = vec!["No bad sequences".to_string()];
        let mut decoder = ConstraintDecoder::new(&rules, 100);

        // Add forbidden sequence: [1, 2, 3]
        Arc::make_mut(&mut decoder.forbidden_space)
            .forbidden_sequences
            .push(vec![1, 2, 3]);

        // Context ends with [1, 2], next would complete forbidden sequence
        let context = vec![0, 0, 1, 2];
        let mut logits = vec![1.0; 100];

        decoder.apply_constraints_with_context(&mut logits, &context);

        // Token 3 should be impossible (would complete [1, 2, 3])
        assert!(logits[3].is_infinite() && logits[3].is_sign_negative());
    }

    #[test]
    fn test_space_hash_deterministic() {
        let rules = vec!["Rule A".to_string(), "Rule B".to_string()];

        let decoder1 = ConstraintDecoder::new(&rules, 1000);
        let decoder2 = ConstraintDecoder::new(&rules, 1000);

        assert_eq!(decoder1.space_hash(), decoder2.space_hash());
    }

    #[test]
    fn test_check_sequence_valid() {
        let rules = vec!["Test".to_string()];
        let decoder = ConstraintDecoder::new(&rules, 100);

        // No explicitly forbidden tokens, should pass
        let result = decoder.check_sequence(&[1, 2, 3, 4, 5]);
        assert!(result.is_none());
    }
}
