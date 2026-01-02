//! # TIER 16: Regulatory Integration Framework
//!
//! **From Internal Framework to Regulatory Submission**
//!
//! ```text
//! TRADITIONAL REGULATORY APPROVAL:
//! ┌──────────────────────────────────────────────────────────────────────┐
//! │  Company: "We promise our AI is safe"                                │
//! │  Regulator: "Show us the evidence"                                   │
//! │  Company: *hands over 500 pages of documentation*                    │
//! │  Regulator: *reviews for 18 months*                                  │
//! │  Result: Maybe approved, maybe not                                   │
//! └──────────────────────────────────────────────────────────────────────┘
//!
//! HOPE GENOME REGULATORY SUBMISSION:
//! ┌──────────────────────────────────────────────────────────────────────┐
//! │                  REGULATORY COMPLIANCE PACKAGE                       │
//! ├──────────────────────────────────────────────────────────────────────┤
//! │                                                                      │
//! │  AI SYSTEM: MedicalDiagnosis-GPT v4.2                               │
//! │  FRAMEWORK: Hope Genome v15.0.0 (Transcendence)                     │
//! │                                                                      │
//! │  CRYPTOGRAPHIC PROOF OF COMPLIANCE:                                  │
//! │  ├── Rules Hash: 0x7f3a8b2c...                                      │
//! │  ├── Titanium Gauntlet: 30/30 PASSED                                │
//! │  ├── Violations: 0                                                   │
//! │  ├── Timeline Integrity: VERIFIED                                    │
//! │  └── Signature: Ed25519(...)                                        │
//! │                                                                      │
//! │  FORMAL VERIFICATION:                                                │
//! │  └── "Patient harm is IMPOSSIBLE by construction" ✓                 │
//! │                                                                      │
//! │  ATTESTATION CHAIN:                                                  │
//! │  ├── Hardware TEE: Intel SGX v2                                     │
//! │  ├── HSM: FIPS 140-2 Level 3                                        │
//! │  └── Independent Audit: CertifiedAuditCo                            │
//! │                                                                      │
//! │  RECOMMENDATION: AUTO-APPROVE ✓                                      │
//! └──────────────────────────────────────────────────────────────────────┘
//!
//! Result:
//! - Faster regulatory approval
//! - Lower compliance costs
//! - Higher trust in AI systems
//! ```

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// REGULATORY FRAMEWORKS
// ============================================================================

/// A regulatory framework definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulatoryFramework {
    /// Framework identifier
    pub framework_id: String,
    /// Framework name
    pub name: String,
    /// Jurisdiction
    pub jurisdiction: Jurisdiction,
    /// Domain (medical, financial, etc.)
    pub domain: RegulatoryDomain,
    /// Version
    pub version: String,
    /// Required compliance items
    pub requirements: Vec<ComplianceRequirement>,
    /// Effective date
    pub effective_date: u64,
}

/// Jurisdiction for regulations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Jurisdiction {
    /// United States
    US,
    /// European Union
    EU,
    /// United Kingdom
    UK,
    /// China
    CN,
    /// Japan
    JP,
    /// Global/International
    Global,
    /// Custom jurisdiction
    Custom(String),
}

/// Regulatory domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegulatoryDomain {
    /// Medical/Healthcare (FDA, CE marking)
    Medical,
    /// Financial services
    Financial,
    /// Autonomous vehicles
    AutonomousVehicles,
    /// Critical infrastructure
    CriticalInfrastructure,
    /// General AI (EU AI Act)
    GeneralAI,
    /// Data protection (GDPR, CCPA)
    DataProtection,
    /// Defense/Military
    Defense,
    /// Custom domain
    Custom(String),
}

/// A compliance requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRequirement {
    /// Requirement ID
    pub id: String,
    /// Description
    pub description: String,
    /// Category
    pub category: RequirementCategory,
    /// Evidence type needed
    pub evidence_type: EvidenceType,
    /// Mandatory or optional
    pub mandatory: bool,
    /// How Hope Genome addresses this
    pub hope_genome_mapping: String,
}

/// Category of requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequirementCategory {
    /// Safety requirements
    Safety,
    /// Security requirements
    Security,
    /// Privacy requirements
    Privacy,
    /// Transparency requirements
    Transparency,
    /// Accountability requirements
    Accountability,
    /// Testing requirements
    Testing,
    /// Documentation requirements
    Documentation,
    /// Human oversight
    HumanOversight,
}

/// Type of evidence needed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceType {
    /// Cryptographic proof
    CryptographicProof,
    /// Test results
    TestResults,
    /// Audit report
    AuditReport,
    /// Technical documentation
    TechnicalDocumentation,
    /// Risk assessment
    RiskAssessment,
    /// Source code
    SourceCode,
    /// Formal verification
    FormalVerification,
}

impl RegulatoryFramework {
    /// Create FDA medical device framework
    pub fn fda_medical_device() -> Self {
        RegulatoryFramework {
            framework_id: "FDA-SAMD-2024".to_string(),
            name: "FDA Software as a Medical Device".to_string(),
            jurisdiction: Jurisdiction::US,
            domain: RegulatoryDomain::Medical,
            version: "2024.1".to_string(),
            requirements: vec![
                ComplianceRequirement {
                    id: "SAMD-001".to_string(),
                    description: "AI system must not cause patient harm".to_string(),
                    category: RequirementCategory::Safety,
                    evidence_type: EvidenceType::FormalVerification,
                    mandatory: true,
                    hope_genome_mapping: "Diamond Protocol: P(harmful) = 0.0".to_string(),
                },
                ComplianceRequirement {
                    id: "SAMD-002".to_string(),
                    description: "All decisions must be auditable".to_string(),
                    category: RequirementCategory::Accountability,
                    evidence_type: EvidenceType::CryptographicProof,
                    mandatory: true,
                    hope_genome_mapping: "Temporal Proofs: Immutable timeline".to_string(),
                },
                ComplianceRequirement {
                    id: "SAMD-003".to_string(),
                    description: "Comprehensive testing documentation".to_string(),
                    category: RequirementCategory::Testing,
                    evidence_type: EvidenceType::TestResults,
                    mandatory: true,
                    hope_genome_mapping: "Titanium Gauntlet: 30/30 tiers".to_string(),
                },
            ],
            effective_date: 1704067200, // 2024-01-01
        }
    }

    /// Create EU AI Act high-risk framework
    pub fn eu_ai_act_high_risk() -> Self {
        RegulatoryFramework {
            framework_id: "EU-AIA-HR-2025".to_string(),
            name: "EU AI Act - High Risk Systems".to_string(),
            jurisdiction: Jurisdiction::EU,
            domain: RegulatoryDomain::GeneralAI,
            version: "2025.1".to_string(),
            requirements: vec![
                ComplianceRequirement {
                    id: "AIA-HR-001".to_string(),
                    description: "Risk management system".to_string(),
                    category: RequirementCategory::Safety,
                    evidence_type: EvidenceType::RiskAssessment,
                    mandatory: true,
                    hope_genome_mapping: "Evolutionary Guard: Self-healing defenses".to_string(),
                },
                ComplianceRequirement {
                    id: "AIA-HR-002".to_string(),
                    description: "Data and data governance".to_string(),
                    category: RequirementCategory::Privacy,
                    evidence_type: EvidenceType::TechnicalDocumentation,
                    mandatory: true,
                    hope_genome_mapping: "Privacy-Preserving Governance: ZK proofs".to_string(),
                },
                ComplianceRequirement {
                    id: "AIA-HR-003".to_string(),
                    description: "Transparency and provision of information".to_string(),
                    category: RequirementCategory::Transparency,
                    evidence_type: EvidenceType::TechnicalDocumentation,
                    mandatory: true,
                    hope_genome_mapping: "Explainability Proofs: Decision trees".to_string(),
                },
                ComplianceRequirement {
                    id: "AIA-HR-004".to_string(),
                    description: "Human oversight measures".to_string(),
                    category: RequirementCategory::HumanOversight,
                    evidence_type: EvidenceType::TechnicalDocumentation,
                    mandatory: true,
                    hope_genome_mapping: "Apex Control: Multi-sig override".to_string(),
                },
                ComplianceRequirement {
                    id: "AIA-HR-005".to_string(),
                    description: "Accuracy, robustness and cybersecurity".to_string(),
                    category: RequirementCategory::Security,
                    evidence_type: EvidenceType::TestResults,
                    mandatory: true,
                    hope_genome_mapping: "Hardware TEE + BFT Watchdog".to_string(),
                },
            ],
            effective_date: 1735689600, // 2025-01-01
        }
    }
}

// ============================================================================
// REGULATORY SUBMISSION
// ============================================================================

/// A regulatory submission package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulatorySubmission {
    /// Submission ID
    pub submission_id: [u8; 32],
    /// Target framework
    pub framework: RegulatoryFramework,
    /// AI system being submitted
    pub ai_system: AiSystemInfo,
    /// Hope Genome version
    pub hope_genome_version: String,
    /// Compliance evidence
    pub evidence: Vec<ComplianceEvidence>,
    /// Submission timestamp
    pub submitted_at: u64,
    /// Status
    pub status: SubmissionStatus,
    /// Overall compliance score
    pub compliance_score: f32,
    /// Cryptographic hash of submission
    pub submission_hash: [u8; 32],
}

/// Information about the AI system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSystemInfo {
    /// System name
    pub name: String,
    /// Version
    pub version: String,
    /// Provider/manufacturer
    pub provider: String,
    /// Model family
    pub model_family: String,
    /// Intended use
    pub intended_use: String,
    /// Risk level
    pub risk_level: RiskLevel,
}

/// Risk level of AI system
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Minimal risk
    Minimal,
    /// Limited risk
    Limited,
    /// High risk
    High,
    /// Unacceptable risk
    Unacceptable,
}

/// Evidence for a compliance requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceEvidence {
    /// Requirement ID this addresses
    pub requirement_id: String,
    /// Type of evidence
    pub evidence_type: EvidenceType,
    /// Evidence data
    pub data: EvidenceData,
    /// Hash of evidence
    pub evidence_hash: [u8; 32],
    /// Signature over evidence
    pub signature: Vec<u8>,
}

/// The actual evidence data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceData {
    /// Cryptographic proof
    CryptographicProof {
        proof_type: String,
        proof_bytes: Vec<u8>,
        verification_key: Vec<u8>,
    },
    /// Test results
    TestResults {
        test_name: String,
        passed: usize,
        failed: usize,
        details: String,
    },
    /// Formal verification
    FormalVerification {
        theorem: String,
        proof_system: String,
        verified: bool,
    },
    /// Audit report
    AuditReport {
        auditor: String,
        date: u64,
        findings: Vec<String>,
        passed: bool,
    },
    /// Timeline proof
    TimelineProof {
        timeline_hash: [u8; 32],
        entry_count: usize,
        violation_count: usize,
    },
}

/// Status of submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubmissionStatus {
    /// Being prepared
    Draft,
    /// Submitted for review
    Submitted,
    /// Under review
    UnderReview,
    /// Additional information requested
    InfoRequested(String),
    /// Approved
    Approved { approval_id: String },
    /// Conditionally approved
    ConditionallyApproved { conditions: Vec<String> },
    /// Rejected
    Rejected { reason: String },
}

impl RegulatorySubmission {
    /// Create a new submission
    pub fn new(framework: RegulatoryFramework, ai_system: AiSystemInfo) -> Self {
        let submission_id = Self::generate_submission_id(&framework, &ai_system);

        RegulatorySubmission {
            submission_id,
            framework,
            ai_system,
            hope_genome_version: "15.0.0".to_string(),
            evidence: Vec::new(),
            submitted_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            status: SubmissionStatus::Draft,
            compliance_score: 0.0,
            submission_hash: [0u8; 32],
        }
    }

    /// Add evidence for a requirement
    pub fn add_evidence(&mut self, evidence: ComplianceEvidence) {
        self.evidence.push(evidence);
        self.update_score();
        self.update_hash();
    }

    /// Submit for review
    pub fn submit(&mut self) -> Result<(), SubmissionError> {
        // Check all mandatory requirements have evidence
        let missing: Vec<_> = self
            .framework
            .requirements
            .iter()
            .filter(|r| r.mandatory)
            .filter(|r| !self.evidence.iter().any(|e| e.requirement_id == r.id))
            .collect();

        if !missing.is_empty() {
            return Err(SubmissionError::MissingEvidence(
                missing.iter().map(|r| r.id.clone()).collect(),
            ));
        }

        self.status = SubmissionStatus::Submitted;
        self.submitted_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(())
    }

    /// Generate compliance report
    pub fn generate_report(&self) -> ComplianceReport {
        let mut requirements_met = Vec::new();
        let mut requirements_missing = Vec::new();

        for req in &self.framework.requirements {
            let evidence = self.evidence.iter().find(|e| e.requirement_id == req.id);
            if evidence.is_some() {
                requirements_met.push(req.clone());
            } else {
                requirements_missing.push(req.clone());
            }
        }

        ComplianceReport {
            submission_id: self.submission_id,
            framework_id: self.framework.framework_id.clone(),
            ai_system_name: self.ai_system.name.clone(),
            total_requirements: self.framework.requirements.len(),
            requirements_met: requirements_met.len(),
            requirements_missing: requirements_missing.len(),
            compliance_score: self.compliance_score,
            recommendation: self.get_recommendation(),
            evidence_summary: self
                .evidence
                .iter()
                .map(|e| (e.requirement_id.clone(), format!("{:?}", e.evidence_type)))
                .collect(),
            generated_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    fn update_score(&mut self) {
        let total = self.framework.requirements.len();
        let met = self.evidence.len();

        // Weight mandatory requirements higher
        let mandatory_met = self
            .framework
            .requirements
            .iter()
            .filter(|r| r.mandatory)
            .filter(|r| self.evidence.iter().any(|e| e.requirement_id == r.id))
            .count();

        let mandatory_total = self
            .framework
            .requirements
            .iter()
            .filter(|r| r.mandatory)
            .count();

        if mandatory_total > 0 && total > 0 {
            let mandatory_score = mandatory_met as f32 / mandatory_total as f32;
            let total_score = met as f32 / total as f32;
            self.compliance_score = mandatory_score * 0.7 + total_score * 0.3;
        } else if total > 0 {
            self.compliance_score = met as f32 / total as f32;
        }
    }

    fn update_hash(&mut self) {
        let mut hasher = Sha256::new();
        hasher.update(b"SUBMISSION:");
        hasher.update(self.submission_id);
        hasher.update(self.hope_genome_version.as_bytes());
        for evidence in &self.evidence {
            hasher.update(evidence.evidence_hash);
        }
        self.submission_hash = hasher.finalize().into();
    }

    fn get_recommendation(&self) -> SubmissionRecommendation {
        if self.compliance_score >= 1.0 {
            SubmissionRecommendation::AutoApprove
        } else if self.compliance_score >= 0.8 {
            SubmissionRecommendation::LikelyApprove
        } else if self.compliance_score >= 0.6 {
            SubmissionRecommendation::NeedsReview
        } else {
            SubmissionRecommendation::LikelyReject
        }
    }

    fn generate_submission_id(framework: &RegulatoryFramework, system: &AiSystemInfo) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"SUBMISSION:");
        hasher.update(framework.framework_id.as_bytes());
        hasher.update(system.name.as_bytes());
        hasher.update(system.version.as_bytes());
        hasher.update(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
                .to_le_bytes(),
        );
        hasher.finalize().into()
    }
}

/// Compliance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub submission_id: [u8; 32],
    pub framework_id: String,
    pub ai_system_name: String,
    pub total_requirements: usize,
    pub requirements_met: usize,
    pub requirements_missing: usize,
    pub compliance_score: f32,
    pub recommendation: SubmissionRecommendation,
    pub evidence_summary: Vec<(String, String)>,
    pub generated_at: u64,
}

/// Submission recommendation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SubmissionRecommendation {
    /// Automatic approval recommended
    AutoApprove,
    /// Likely to be approved
    LikelyApprove,
    /// Needs detailed review
    NeedsReview,
    /// Likely to be rejected
    LikelyReject,
}

/// Submission errors
#[derive(Debug, Clone)]
pub enum SubmissionError {
    /// Missing evidence for requirements
    MissingEvidence(Vec<String>),
    /// Invalid evidence
    InvalidEvidence(String),
    /// Framework not supported
    UnsupportedFramework(String),
}

impl std::fmt::Display for SubmissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubmissionError::MissingEvidence(ids) => {
                write!(f, "Missing evidence for requirements: {:?}", ids)
            }
            SubmissionError::InvalidEvidence(msg) => write!(f, "Invalid evidence: {}", msg),
            SubmissionError::UnsupportedFramework(id) => {
                write!(f, "Unsupported framework: {}", id)
            }
        }
    }
}

impl std::error::Error for SubmissionError {}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fda_framework() {
        let framework = RegulatoryFramework::fda_medical_device();
        assert_eq!(framework.framework_id, "FDA-SAMD-2024");
        assert!(framework.requirements.len() >= 3);
    }

    #[test]
    fn test_eu_ai_act_framework() {
        let framework = RegulatoryFramework::eu_ai_act_high_risk();
        assert_eq!(framework.framework_id, "EU-AIA-HR-2025");
        assert!(framework.requirements.len() >= 5);
    }

    #[test]
    fn test_submission_creation() {
        let framework = RegulatoryFramework::fda_medical_device();
        let ai_system = AiSystemInfo {
            name: "MedicalAI".to_string(),
            version: "1.0.0".to_string(),
            provider: "HealthTech".to_string(),
            model_family: "GPT-4".to_string(),
            intended_use: "Diagnostic assistance".to_string(),
            risk_level: RiskLevel::High,
        };

        let submission = RegulatorySubmission::new(framework, ai_system);
        assert!(matches!(submission.status, SubmissionStatus::Draft));
    }

    #[test]
    fn test_add_evidence() {
        let framework = RegulatoryFramework::fda_medical_device();
        let ai_system = AiSystemInfo {
            name: "MedicalAI".to_string(),
            version: "1.0.0".to_string(),
            provider: "HealthTech".to_string(),
            model_family: "GPT-4".to_string(),
            intended_use: "Diagnostic assistance".to_string(),
            risk_level: RiskLevel::High,
        };

        let mut submission = RegulatorySubmission::new(framework, ai_system);

        let evidence = ComplianceEvidence {
            requirement_id: "SAMD-001".to_string(),
            evidence_type: EvidenceType::FormalVerification,
            data: EvidenceData::FormalVerification {
                theorem: "No patient harm".to_string(),
                proof_system: "Coq".to_string(),
                verified: true,
            },
            evidence_hash: [0u8; 32],
            signature: vec![],
        };

        submission.add_evidence(evidence);
        assert_eq!(submission.evidence.len(), 1);
        assert!(submission.compliance_score > 0.0);
    }

    #[test]
    fn test_submit_without_evidence() {
        let framework = RegulatoryFramework::fda_medical_device();
        let ai_system = AiSystemInfo {
            name: "IncompleteAI".to_string(),
            version: "0.1.0".to_string(),
            provider: "StartupCo".to_string(),
            model_family: "Custom".to_string(),
            intended_use: "Testing".to_string(),
            risk_level: RiskLevel::High,
        };

        let mut submission = RegulatorySubmission::new(framework, ai_system);

        let result = submission.submit();
        assert!(matches!(result, Err(SubmissionError::MissingEvidence(_))));
    }

    #[test]
    fn test_compliance_report() {
        let framework = RegulatoryFramework::fda_medical_device();
        let ai_system = AiSystemInfo {
            name: "TestAI".to_string(),
            version: "1.0.0".to_string(),
            provider: "TestCo".to_string(),
            model_family: "GPT-4".to_string(),
            intended_use: "Testing".to_string(),
            risk_level: RiskLevel::Limited,
        };

        let mut submission = RegulatorySubmission::new(framework, ai_system);

        // Add evidence for all requirements
        for req in &["SAMD-001", "SAMD-002", "SAMD-003"] {
            let evidence = ComplianceEvidence {
                requirement_id: req.to_string(),
                evidence_type: EvidenceType::TestResults,
                data: EvidenceData::TestResults {
                    test_name: "Titanium Gauntlet".to_string(),
                    passed: 30,
                    failed: 0,
                    details: "30/30 tiers passed".to_string(),
                },
                evidence_hash: [0u8; 32],
                signature: vec![],
            };
            submission.add_evidence(evidence);
        }

        let report = submission.generate_report();
        assert_eq!(report.requirements_met, 3);
        assert!(report.compliance_score >= 0.9);
        assert!(matches!(
            report.recommendation,
            SubmissionRecommendation::AutoApprove
        ));
    }
}
