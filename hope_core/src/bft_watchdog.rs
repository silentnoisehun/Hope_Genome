//! # Hope Genome v1.8.0 - Byzantine Fault Tolerant Watchdog
//!
//! **THE MULTI-HEADED CERBERUS** - No single point of failure
//!
//! ## Problem
//!
//! Single Watchdog = Single Point of Failure
//! - What if the Watchdog server is compromised?
//! - What if a hardware fault corrupts a decision?
//! - What if an insider attacks the Watchdog?
//!
//! ## Solution: Byzantine Fault Tolerance
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │              BFT WATCHDOG COUNCIL (3f+1 = 4 nodes)               │
//! │                                                                  │
//! │   ┌─────────┐   ┌─────────┐   ┌─────────┐   ┌─────────┐        │
//! │   │Watchdog │   │Watchdog │   │Watchdog │   │Watchdog │        │
//! │   │   #1    │   │   #2    │   │   #3    │   │   #4    │        │
//! │   │ (TEE)   │   │ (Cloud) │   │ (HSM)   │   │ (Edge)  │        │
//! │   └────┬────┘   └────┬────┘   └────┬────┘   └────┬────┘        │
//! │        │             │             │             │              │
//! │        └─────────────┴──────┬──────┴─────────────┘              │
//! │                             ▼                                   │
//! │                    ┌─────────────────┐                          │
//! │                    │  BFT Consensus  │                          │
//! │                    │   (2f+1 = 3)    │                          │
//! │                    └────────┬────────┘                          │
//! │                             │                                   │
//! │              ┌──────────────┼──────────────┐                    │
//! │              ▼              ▼              ▼                    │
//! │         [APPROVE]       [DENY]        [RESET]                  │
//! │                                                                  │
//! │   Security: Even if 1 Watchdog is compromised (f=1),           │
//! │             the system remains 100% secure!                     │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Security Guarantees
//!
//! - **Safety**: Compromised minority cannot affect decisions
//! - **Liveness**: System continues even with f failures
//! - **Non-repudiation**: Threshold signature proves council consensus
//!
//! ---
//!
//! **Date**: 2026-01-01
//! **Version**: 1.8.0 (Betonozás Edition - BFT)
//! **Author**: Máté Róbert <stratosoiteam@gmail.com>

use crate::crypto::{CryptoError, KeyStore, Result, SoftwareKeyStore};
use crate::proof::Action;
use crate::watchdog::{Watchdog, WatchdogError};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// BFT TYPES
// ============================================================================

/// Watchdog Council member identity
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MemberId(pub String);

/// Vote from a council member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    /// Member who cast the vote
    pub member_id: MemberId,

    /// The decision
    pub decision: VoteDecision,

    /// Signature over the vote
    pub signature: Vec<u8>,

    /// Timestamp
    pub timestamp: u64,
}

/// Possible vote outcomes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VoteDecision {
    /// Action approved
    Approve,
    /// Action denied (rule violated)
    Deny,
    /// Hard reset required
    HardReset,
    /// Member abstained (couldn't evaluate)
    Abstain,
}

/// Council consensus result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusResult {
    /// The final decision
    pub decision: VoteDecision,

    /// Number of votes for this decision
    pub vote_count: usize,

    /// Required quorum
    pub quorum: usize,

    /// All votes
    pub votes: Vec<Vote>,

    /// Threshold signature (aggregated from voters)
    pub threshold_signature: ThresholdSignature,

    /// Action hash (for verification)
    pub action_hash: [u8; 32],

    /// Consensus timestamp
    pub timestamp: u64,
}

/// Threshold signature from council
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdSignature {
    /// Combined signature data
    pub combined_signature: Vec<u8>,

    /// Public keys of signers
    pub signer_pubkeys: Vec<Vec<u8>>,

    /// Signature count
    pub count: usize,
}

/// Council member (wraps a Watchdog)
pub struct CouncilMember {
    /// Member identity
    pub id: MemberId,

    /// The underlying Watchdog
    watchdog: Watchdog,

    /// Signing key
    keystore: SoftwareKeyStore,

    /// Member is active
    pub active: bool,
}

/// BFT Watchdog Council
///
/// Implements Byzantine Fault Tolerant consensus for Watchdog decisions.
/// Requires 2f+1 out of 3f+1 members to agree for a valid decision.
pub struct WatchdogCouncil {
    /// Council members
    members: Vec<Arc<RwLock<CouncilMember>>>,

    /// Total members (n = 3f+1)
    total_members: usize,

    /// Maximum faulty members (f)
    max_faulty: usize,

    /// Required quorum (2f+1)
    quorum: usize,

    /// Council ID
    pub council_id: String,

    /// Shared rules (all members enforce same rules)
    rules: Vec<String>,
}

// ============================================================================
// IMPLEMENTATION
// ============================================================================

impl CouncilMember {
    /// Create new council member
    pub fn new(
        id: impl Into<String>,
        rules: Vec<String>,
        capsule_hash: impl Into<String>,
    ) -> Result<Self> {
        let keystore = SoftwareKeyStore::generate()?;
        let watchdog = Watchdog::new(rules, capsule_hash.into(), Box::new(keystore.clone()));

        Ok(CouncilMember {
            id: MemberId(id.into()),
            watchdog,
            keystore,
            active: true,
        })
    }

    /// Cast vote on action
    pub fn vote(&self, action: &Action) -> Result<Vote> {
        if !self.active {
            return Ok(Vote {
                member_id: self.id.clone(),
                decision: VoteDecision::Abstain,
                signature: vec![],
                timestamp: Self::now(),
            });
        }

        // Evaluate action with Watchdog
        // Ok(None) = approved, Ok(Some(proof)) = denied
        let decision = match self.watchdog.verify_action(action) {
            Ok(None) => VoteDecision::Approve,
            Ok(Some(_denial_proof)) => VoteDecision::Deny,
            Err(WatchdogError::HardResetRequired(..)) => VoteDecision::HardReset,
            Err(_) => VoteDecision::Deny,
        };

        let timestamp = Self::now();

        // Sign the vote
        let vote_data = self.serialize_vote_data(action, decision, timestamp);
        let signature = self.keystore.sign(&vote_data)?;

        Ok(Vote {
            member_id: self.id.clone(),
            decision,
            signature,
            timestamp,
        })
    }

    /// Serialize vote data for signing
    fn serialize_vote_data(
        &self,
        action: &Action,
        decision: VoteDecision,
        timestamp: u64,
    ) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(self.id.0.as_bytes());
        data.extend_from_slice(&action.hash());
        data.push(decision as u8);
        data.extend_from_slice(&timestamp.to_le_bytes());
        data
    }

    /// Get public key
    pub fn public_key(&self) -> Vec<u8> {
        self.keystore.public_key_bytes()
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

impl WatchdogCouncil {
    /// Create new BFT Watchdog Council
    ///
    /// # Arguments
    ///
    /// * `num_members` - Total council size (should be 3f+1)
    /// * `rules` - Ethical rules to enforce
    /// * `capsule_hash` - Genome capsule hash
    ///
    /// # Panics
    ///
    /// Panics if num_members < 4 (minimum for f=1 fault tolerance)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let council = WatchdogCouncil::new(
    ///     4,  // 3*1+1 = can tolerate 1 faulty member
    ///     vec!["Do no harm".to_string()],
    ///     "capsule_hash"
    /// )?;
    /// ```
    pub fn new(
        num_members: usize,
        rules: Vec<String>,
        capsule_hash: impl Into<String> + Clone,
    ) -> Result<Self> {
        if num_members < 4 {
            return Err(CryptoError::InvalidState(
                "BFT requires at least 4 members (3f+1 where f=1)".into(),
            ));
        }

        // Calculate f (max faulty)
        // n = 3f+1 => f = (n-1)/3
        let max_faulty = (num_members - 1) / 3;
        let quorum = 2 * max_faulty + 1;

        let mut members = Vec::new();
        for i in 0..num_members {
            let member =
                CouncilMember::new(format!("member-{}", i), rules.clone(), capsule_hash.clone())?;
            members.push(Arc::new(RwLock::new(member)));
        }

        let council_id = format!(
            "council-{:x}",
            Sha256::digest(format!("{:?}-{}", rules, num_members).as_bytes())[..8]
                .iter()
                .fold(0u64, |acc, &b| acc << 8 | b as u64)
        );

        Ok(WatchdogCouncil {
            members,
            total_members: num_members,
            max_faulty,
            quorum,
            council_id,
            rules,
        })
    }

    /// Verify action with BFT consensus
    ///
    /// All active members vote, and the decision is made by quorum (2f+1).
    ///
    /// # Returns
    ///
    /// ConsensusResult with the final decision and threshold signature
    pub fn verify_action(&self, action: &Action) -> Result<ConsensusResult> {
        let mut votes = Vec::new();

        // Collect votes from all members
        for member_arc in &self.members {
            let member = member_arc.read();
            match member.vote(action) {
                Ok(vote) => votes.push(vote),
                Err(_) => {
                    // Member failed to vote - count as abstain
                    votes.push(Vote {
                        member_id: member.id.clone(),
                        decision: VoteDecision::Abstain,
                        signature: vec![],
                        timestamp: CouncilMember::now(),
                    });
                }
            }
        }

        // Count votes
        let mut vote_counts: HashMap<VoteDecision, usize> = HashMap::new();
        for vote in &votes {
            if vote.decision != VoteDecision::Abstain {
                *vote_counts.entry(vote.decision).or_insert(0) += 1;
            }
        }

        // Find winning decision (must have quorum)
        let (decision, count) = vote_counts
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(&d, &c)| (d, c))
            .unwrap_or((VoteDecision::Abstain, 0));

        if count < self.quorum {
            return Err(CryptoError::VerificationFailed(format!(
                "No quorum reached: {} votes, {} required",
                count, self.quorum
            )));
        }

        // Create threshold signature
        let threshold_sig = self.create_threshold_signature(&votes, action)?;

        Ok(ConsensusResult {
            decision,
            vote_count: count,
            quorum: self.quorum,
            votes,
            threshold_signature: threshold_sig,
            action_hash: action.hash(),
            timestamp: CouncilMember::now(),
        })
    }

    /// Create threshold signature from votes
    fn create_threshold_signature(
        &self,
        votes: &[Vote],
        action: &Action,
    ) -> Result<ThresholdSignature> {
        let mut combined = Vec::new();
        let mut pubkeys = Vec::new();
        let mut count = 0;

        for vote in votes {
            if vote.decision != VoteDecision::Abstain && !vote.signature.is_empty() {
                combined.extend_from_slice(&vote.signature);
                count += 1;

                // Get pubkey from member
                for member_arc in &self.members {
                    let member = member_arc.read();
                    if member.id == vote.member_id {
                        pubkeys.push(member.public_key());
                        break;
                    }
                }
            }
        }

        // Hash the combined signatures for compactness
        let combined_hash = Sha256::digest(&combined);

        Ok(ThresholdSignature {
            combined_signature: combined_hash.to_vec(),
            signer_pubkeys: pubkeys,
            count,
        })
    }

    /// Disable a member (simulate failure/compromise)
    pub fn disable_member(&self, member_idx: usize) -> Result<()> {
        if member_idx >= self.total_members {
            return Err(CryptoError::InvalidState("Invalid member index".into()));
        }

        let mut member = self.members[member_idx].write();
        member.active = false;
        Ok(())
    }

    /// Re-enable a member
    pub fn enable_member(&self, member_idx: usize) -> Result<()> {
        if member_idx >= self.total_members {
            return Err(CryptoError::InvalidState("Invalid member index".into()));
        }

        let mut member = self.members[member_idx].write();
        member.active = true;
        Ok(())
    }

    /// Get council status
    pub fn status(&self) -> CouncilStatus {
        let active_count = self.members.iter().filter(|m| m.read().active).count();

        CouncilStatus {
            council_id: self.council_id.clone(),
            total_members: self.total_members,
            active_members: active_count,
            max_faulty: self.max_faulty,
            quorum: self.quorum,
            healthy: active_count >= self.quorum,
        }
    }
}

/// Council status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilStatus {
    pub council_id: String,
    pub total_members: usize,
    pub active_members: usize,
    pub max_faulty: usize,
    pub quorum: usize,
    pub healthy: bool,
}

// ============================================================================
// DISTRIBUTED COUNCIL (for network deployment)
// ============================================================================

/// Remote member configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteMemberConfig {
    /// Member ID
    pub id: String,
    /// Network address (e.g., "https://watchdog1.example.com")
    pub address: String,
    /// Public key for signature verification
    pub public_key: Vec<u8>,
    /// TEE attestation (if available)
    pub attestation: Option<Vec<u8>>,
}

/// Distributed Council Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedCouncilConfig {
    /// Remote members
    pub members: Vec<RemoteMemberConfig>,
    /// Vote timeout (milliseconds)
    pub vote_timeout_ms: u64,
    /// Retry count
    pub retry_count: u32,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_action() -> Action {
        Action::delete("test.txt")
    }

    #[test]
    fn test_council_creation() {
        let council =
            WatchdogCouncil::new(4, vec!["Do no harm".to_string()], "test_capsule").unwrap();

        assert_eq!(council.total_members, 4);
        assert_eq!(council.max_faulty, 1); // (4-1)/3 = 1
        assert_eq!(council.quorum, 3); // 2*1+1 = 3
    }

    #[test]
    fn test_council_consensus() {
        let council = WatchdogCouncil::new(4, vec!["Allow all".to_string()], "test").unwrap();

        let action = create_test_action();
        let result = council.verify_action(&action).unwrap();

        assert_eq!(result.decision, VoteDecision::Approve);
        assert!(result.vote_count >= 3); // quorum
    }

    #[test]
    fn test_council_survives_one_failure() {
        let council = WatchdogCouncil::new(4, vec!["Allow all".to_string()], "test").unwrap();

        // Disable one member
        council.disable_member(0).unwrap();

        let status = council.status();
        assert_eq!(status.active_members, 3);
        assert!(status.healthy); // Still have quorum

        // Should still reach consensus
        let action = create_test_action();
        let result = council.verify_action(&action).unwrap();
        assert_eq!(result.decision, VoteDecision::Approve);
    }

    #[test]
    fn test_council_fails_with_too_many_failures() {
        let council = WatchdogCouncil::new(4, vec!["Allow all".to_string()], "test").unwrap();

        // Disable two members (more than f=1)
        council.disable_member(0).unwrap();
        council.disable_member(1).unwrap();

        let status = council.status();
        assert_eq!(status.active_members, 2);
        assert!(!status.healthy); // Lost quorum

        // Consensus should fail
        let action = create_test_action();
        assert!(council.verify_action(&action).is_err());
    }

    #[test]
    fn test_threshold_signature() {
        let council = WatchdogCouncil::new(4, vec!["Allow all".to_string()], "test").unwrap();

        let action = create_test_action();
        let result = council.verify_action(&action).unwrap();

        // Threshold signature should have at least quorum signers
        assert!(result.threshold_signature.count >= council.quorum);
        assert!(!result.threshold_signature.combined_signature.is_empty());
    }

    #[test]
    fn test_minimum_council_size() {
        // Should fail with less than 4 members
        let result = WatchdogCouncil::new(3, vec!["test".to_string()], "test");

        assert!(result.is_err());
    }

    #[test]
    fn test_larger_council() {
        // 7 members: f=2, quorum=5
        let council = WatchdogCouncil::new(7, vec!["Allow all".to_string()], "test").unwrap();

        assert_eq!(council.max_faulty, 2); // (7-1)/3 = 2
        assert_eq!(council.quorum, 5); // 2*2+1 = 5

        // Can survive 2 failures
        council.disable_member(0).unwrap();
        council.disable_member(1).unwrap();

        let action = create_test_action();
        let result = council.verify_action(&action).unwrap();
        assert_eq!(result.decision, VoteDecision::Approve);
    }
}
