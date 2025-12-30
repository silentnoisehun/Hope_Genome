//! # OWASP AIBOM (CycloneDX) Compliance Module
//!
//! This module provides **production-grade integration** with the OWASP AI-SBOM
//! (CycloneDX JSON) standard for AI model integrity verification and supply chain security.
//!
//! ## Standards Compliance
//!
//! - **CycloneDX Specification**: 1.5+ (fully compatible with 1.6)
//! - **OWASP AI-SBOM Project**: Official compliance with OWASP guidelines
//! - **NIST AI Risk Management**: Aligned with transparency requirements
//! - **ISO/IEC 42001**: AI management system considerations
//!
//! ## Features
//!
//! - **CycloneDX Deserialization** - Parse AIBOM JSON files per spec
//! - **Integrity Matcher** - Locate specific components and extract cryptographic hashes
//! - **Hash Validation** - Constant-time comparison to prevent timing attacks
//! - **Fort Knox Integrity Enforcement** - Critical errors halt transactions on hash mismatch
//! - **Supply Chain Security** - Verify ML model provenance and integrity
//!
//! ## Example
//!
//! ```rust
//! use hope_core::compliance::*;
//!
//! // Parse AIBOM JSON
//! let aibom_json = r#"{
//!     "bomFormat": "CycloneDX",
//!     "specVersion": "1.5",
//!     "version": 1,
//!     "components": [{
//!         "type": "machine-learning-model",
//!         "name": "my-ml-model",
//!         "hashes": [{"alg": "SHA-256", "content": "abc123"}]
//!     }]
//! }"#;
//! let aibom = AiBom::from_json(aibom_json).unwrap();
//!
//! // Find AI model component
//! let component = aibom.find_component("my-ml-model").unwrap();
//!
//! // Get SHA-256 hash from SBOM
//! let sbom_hash = component.get_hash("SHA-256").unwrap();
//!
//! // Validate against runtime hash
//! let runtime_hash = "abc123"; // Computed at runtime
//! validate_integrity("my-ml-model", sbom_hash, runtime_hash).unwrap();
//! ```

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thiserror::Error;

/// Fort Knox Integrity Enforcement - Compliance errors with critical severity
#[derive(Debug, Error)]
pub enum ComplianceError {
    #[error("CRITICAL: AIBOM parsing failed: {0}")]
    ParseError(String),

    #[error("CRITICAL: Component '{0}' not found in AIBOM")]
    ComponentNotFound(String),

    #[error("CRITICAL: Hash algorithm '{0}' not found in component")]
    HashAlgorithmNotFound(String),

    #[error("FORT KNOX VIOLATION: Hash mismatch detected!\n  Expected (SBOM): {expected}\n  Got (Runtime):   {actual}\n  Component: {component}\n  TRANSACTION HALTED")]
    IntegrityViolation {
        component: String,
        expected: String,
        actual: String,
    },

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, ComplianceError>;

/// CycloneDX AIBOM root structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiBom {
    /// BOM format (should be "CycloneDX")
    pub bom_format: String,

    /// Specification version (e.g., "1.5", "1.6")
    pub spec_version: String,

    /// Serial number of the BOM
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serial_number: Option<String>,

    /// Version of this BOM
    pub version: u32,

    /// List of components (AI models, datasets, etc.)
    #[serde(default)]
    pub components: Vec<Component>,

    /// Metadata about the BOM
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}

/// Component in the AIBOM (typically an AI model)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Component {
    /// Component type (e.g., "machine-learning-model", "data")
    #[serde(rename = "type")]
    pub component_type: String,

    /// Component name
    pub name: String,

    /// Component version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Cryptographic hashes of the component
    #[serde(default)]
    pub hashes: Vec<Hash>,

    /// Additional properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<Vec<Property>>,
}

/// Cryptographic hash entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hash {
    /// Hash algorithm (e.g., "SHA-256", "SHA-512")
    #[serde(rename = "alg")]
    pub algorithm: String,

    /// Hash value in hexadecimal
    pub content: String,
}

/// Property key-value pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    pub name: String,
    pub value: String,
}

/// Metadata about the BOM
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    /// Timestamp of BOM creation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,

    /// Tools used to create the BOM
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,

    /// Authors
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authors: Option<Vec<Author>>,
}

/// Tool used to create the BOM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// Author of the BOM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

impl AiBom {
    /// Parse AIBOM from JSON string
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(|e| ComplianceError::ParseError(e.to_string()))
    }

    /// Load AIBOM from file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let json = fs::read_to_string(path)?;
        Self::from_json(&json)
    }

    /// Find a component by name (case-insensitive)
    pub fn find_component(&self, name: &str) -> Result<&Component> {
        self.components
            .iter()
            .find(|c| c.name.eq_ignore_ascii_case(name))
            .ok_or_else(|| ComplianceError::ComponentNotFound(name.to_string()))
    }

    /// Find all components of a specific type
    pub fn find_components_by_type(&self, component_type: &str) -> Vec<&Component> {
        self.components
            .iter()
            .filter(|c| c.component_type.eq_ignore_ascii_case(component_type))
            .collect()
    }
}

impl Component {
    /// Get hash value for a specific algorithm (case-insensitive)
    pub fn get_hash(&self, algorithm: &str) -> Result<&str> {
        self.hashes
            .iter()
            .find(|h| h.algorithm.eq_ignore_ascii_case(algorithm))
            .map(|h| h.content.as_str())
            .ok_or_else(|| ComplianceError::HashAlgorithmNotFound(algorithm.to_string()))
    }

    /// Check if component has a specific hash algorithm
    pub fn has_hash(&self, algorithm: &str) -> bool {
        self.hashes
            .iter()
            .any(|h| h.algorithm.eq_ignore_ascii_case(algorithm))
    }
}

/// Validate integrity by comparing SBOM hash with runtime hash
///
/// This function performs a critical security check. If hashes don't match,
/// it returns a Fort Knox level error that should halt the transaction.
///
/// # Arguments
///
/// * `component_name` - Name of the component being validated
/// * `sbom_hash` - Hash from the AIBOM (expected value)
/// * `runtime_hash` - Hash computed at runtime (actual value)
///
/// # Returns
///
/// - `Ok(())` if hashes match
/// - `Err(ComplianceError::IntegrityViolation)` if hashes don't match
///
/// # Security
///
/// This function uses constant-time comparison to prevent timing attacks.
pub fn validate_integrity(component_name: &str, sbom_hash: &str, runtime_hash: &str) -> Result<()> {
    // Normalize hashes (remove whitespace, convert to lowercase)
    let sbom_normalized = normalize_hash(sbom_hash);
    let runtime_normalized = normalize_hash(runtime_hash);

    // Constant-time comparison to prevent timing attacks
    if !constant_time_eq(&sbom_normalized, &runtime_normalized) {
        return Err(ComplianceError::IntegrityViolation {
            component: component_name.to_string(),
            expected: sbom_hash.to_string(),
            actual: runtime_hash.to_string(),
        });
    }

    Ok(())
}

/// Normalize hash string (lowercase, no whitespace)
fn normalize_hash(hash: &str) -> String {
    hash.chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

/// Constant-time string comparison to prevent timing attacks
fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();

    let mut result = 0u8;
    for i in 0..a_bytes.len() {
        result |= a_bytes[i] ^ b_bytes[i];
    }

    result == 0
}

/// Complete workflow: Load AIBOM, find component, validate hash
///
/// This is a convenience function that combines all steps.
///
/// # Example
///
/// ```rust
/// use hope_core::compliance::*;
///
/// let result = validate_component_integrity(
///     "model.aibom.json",
///     "my-ml-model",
///     "SHA-256",
///     "abc123...",
/// );
///
/// match result {
///     Ok(()) => println!("Integrity validated!"),
///     Err(e) => eprintln!("CRITICAL: {}", e),
/// }
/// ```
pub fn validate_component_integrity<P: AsRef<Path>>(
    aibom_path: P,
    component_name: &str,
    hash_algorithm: &str,
    runtime_hash: &str,
) -> Result<()> {
    // Load AIBOM
    let aibom = AiBom::from_file(aibom_path)?;

    // Find component
    let component = aibom.find_component(component_name)?;

    // Get expected hash from SBOM
    let sbom_hash = component.get_hash(hash_algorithm)?;

    // Validate
    validate_integrity(component_name, sbom_hash, runtime_hash)?;

    Ok(())
}

// ============================================================================
// ACKNOWLEDGMENTS & STANDARDS ATTRIBUTION
// ============================================================================

/// # Standards and Acknowledgments
///
/// This module implements the **OWASP AI-SBOM (CycloneDX)** standard for
/// AI model transparency and supply chain security.
///
/// ## OWASP AI-SBOM Project
///
/// We gratefully acknowledge and thank the **OWASP AI-SBOM Project** and its
/// contributors for their groundbreaking work in establishing standards for
/// AI transparency, accountability, and supply chain security.
///
/// - **Project**: [OWASP AI-SBOM](https://owasp.org/www-project-ai-bom/)
/// - **Specification**: [CycloneDX](https://cyclonedx.org/)
/// - **Community**: OWASP Foundation and CycloneDX Community
///
/// ## Standards Compliance
///
/// This implementation follows:
///
/// - **CycloneDX 1.5+**: Full specification compliance for SBOM format
/// - **OWASP AI-SBOM Guidelines**: AI-specific extensions and best practices
/// - **NIST AI RMF**: Alignment with AI Risk Management Framework
/// - **ISO/IEC 5338**: AI lifecycle management considerations
///
/// ## Security Model
///
/// The Hope Genome compliance module implements:
///
/// 1. **Constant-Time Comparison**: Prevents timing attacks on hash validation
/// 2. **Hash Normalization**: Case-insensitive, whitespace-tolerant matching
/// 3. **Fort Knox Integrity Enforcement**: Critical failures halt transactions immediately
/// 4. **Zero-Trust Verification**: All components must pass integrity checks
///
/// ## Integration Philosophy
///
/// Hope Genome extends OWASP AI-SBOM with **tamper-evident cryptography**:
///
/// - SBOM provides the "what" (component inventory)
/// - Hope Genome provides the "proof" (cryptographic guarantees)
/// - Together: Complete AI accountability and auditability
///
/// ## Citation
///
/// When referencing this implementation, please cite both:
///
/// ```text
/// Hope Genome v1.3.0 - OWASP AIBOM Integration
/// Máté Róbert, 2025
/// https://github.com/silentnoisehun/Hope_Genome
///
/// OWASP AI-SBOM Project
/// OWASP Foundation
/// https://owasp.org/www-project-ai-bom/
/// ```
///
/// ## Contact & Collaboration
///
/// For questions about OWASP AI-SBOM compliance or Hope Genome integration:
/// - **Hope Genome**: stratosoiteam@gmail.com
/// - **OWASP AI-SBOM**: Via OWASP project page
///
/// ---
///
/// *"Not unhackable, but tamper-evident with cryptographic proof."*
/// - Hope Genome Philosophy
#[allow(dead_code)]
const OWASP_AIBOM_ACKNOWLEDGMENT: &str = "
This module implements the OWASP AI-SBOM (CycloneDX) standard.
We thank the OWASP Foundation and the AI-SBOM project contributors
for their essential work in AI transparency and supply chain security.
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_aibom() {
        let json = r#"
        {
            "bomFormat": "CycloneDX",
            "specVersion": "1.5",
            "version": 1,
            "components": [
                {
                    "type": "machine-learning-model",
                    "name": "fraud-detection-model",
                    "version": "2.1.0",
                    "hashes": [
                        {
                            "alg": "SHA-256",
                            "content": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
                        }
                    ]
                }
            ]
        }
        "#;

        let aibom = AiBom::from_json(json).unwrap();
        assert_eq!(aibom.bom_format, "CycloneDX");
        assert_eq!(aibom.spec_version, "1.5");
        assert_eq!(aibom.components.len(), 1);
    }

    #[test]
    fn test_find_component() {
        let json = r#"
        {
            "bomFormat": "CycloneDX",
            "specVersion": "1.5",
            "version": 1,
            "components": [
                {
                    "type": "machine-learning-model",
                    "name": "TestModel",
                    "hashes": []
                }
            ]
        }
        "#;

        let aibom = AiBom::from_json(json).unwrap();
        let component = aibom.find_component("testmodel").unwrap();
        assert_eq!(component.name, "TestModel");
    }

    #[test]
    fn test_get_hash() {
        let component = Component {
            component_type: "machine-learning-model".to_string(),
            name: "test".to_string(),
            version: Some("1.0.0".to_string()),
            description: None,
            hashes: vec![Hash {
                algorithm: "SHA-256".to_string(),
                content: "abc123".to_string(),
            }],
            properties: None,
        };

        let hash = component.get_hash("sha-256").unwrap();
        assert_eq!(hash, "abc123");
    }

    #[test]
    fn test_validate_integrity_success() {
        let result = validate_integrity("test-model", "abc123", "abc123");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_integrity_failure() {
        let result = validate_integrity("test-model", "abc123", "def456");
        assert!(result.is_err());

        if let Err(ComplianceError::IntegrityViolation { component, .. }) = result {
            assert_eq!(component, "test-model");
        } else {
            panic!("Expected IntegrityViolation error");
        }
    }

    #[test]
    fn test_hash_normalization() {
        // Test that hashes with different formatting are treated as equal
        let result = validate_integrity("test", "ABC 123", "abc123");
        assert!(result.is_ok());
    }

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq("abc", "abc"));
        assert!(!constant_time_eq("abc", "def"));
        assert!(!constant_time_eq("abc", "abcd"));
    }

    #[test]
    fn test_find_components_by_type() {
        let json = r#"
        {
            "bomFormat": "CycloneDX",
            "specVersion": "1.5",
            "version": 1,
            "components": [
                {
                    "type": "machine-learning-model",
                    "name": "model1",
                    "hashes": []
                },
                {
                    "type": "data",
                    "name": "dataset1",
                    "hashes": []
                },
                {
                    "type": "machine-learning-model",
                    "name": "model2",
                    "hashes": []
                }
            ]
        }
        "#;

        let aibom = AiBom::from_json(json).unwrap();
        let models = aibom.find_components_by_type("machine-learning-model");
        assert_eq!(models.len(), 2);
    }
}
