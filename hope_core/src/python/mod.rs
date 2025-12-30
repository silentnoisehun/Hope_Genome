//! Python bindings for Hope Genome
//!
//! This module provides complete Python bindings via PyO3, exposing all Hope Genome
//! functionality to Python with native performance.

use pyo3::prelude::*;

mod action;
mod auditlog;
mod auditor;
mod consensus;
mod errors;
mod genome;
mod keystore;
mod noncestore;
mod proof;
// mod aibom;  // TODO v1.5.1: Complete AIBOM wrapper

pub use action::*;
pub use auditlog::*;
pub use auditor::*;
pub use consensus::*;
pub use errors::*;
pub use genome::*;
pub use keystore::*;
pub use noncestore::*;
pub use proof::*;
// pub use aibom::*;

/// Hope Genome Python module
///
/// Provides tamper-evident cryptographic framework for AI accountability.
///
/// # Example
/// ```python
/// import hope_genome as hg
///
/// # Create and seal genome
/// genome = hg.SealedGenome(rules=["Do no harm", "Respect privacy"])
/// genome.seal()
///
/// # Verify action
/// action = hg.Action.delete_file("user_data.txt")
/// proof = genome.verify_action(action)
///
/// # Audit proof
/// auditor = hg.ProofAuditor()
/// auditor.verify_proof(proof)
/// ```
#[pymodule]
fn _hope_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Core classes
    m.add_class::<PySealedGenome>()?;
    m.add_class::<PyAction>()?;
    m.add_class::<PyProof>()?;
    m.add_class::<PyProofAuditor>()?;
    m.add_class::<PyConsensusEngine>()?;

    // KeyStore classes
    m.add_class::<PySoftwareKeyStore>()?;

    // NonceStore classes
    m.add_class::<PyMemoryNonceStore>()?;

    // Audit
    m.add_class::<PyAuditLogger>()?;
    m.add_class::<PyAuditEntry>()?;
    // TODO v1.5.1: AIBOM classes
    // m.add_class::<PyAibomVerifier>()?;
    // m.add_class::<PyAibomComponent>()?;

    // Exceptions
    m.add("GenomeError", m.py().get_type::<PyGenomeError>())?;
    m.add("CryptoError", m.py().get_type::<PyCryptoError>())?;
    m.add("AuditorError", m.py().get_type::<PyAuditorError>())?;
    m.add("ConsensusError", m.py().get_type::<PyConsensusError>())?;
    m.add("AibomError", m.py().get_type::<PyAibomError>())?;

    // Module metadata
    m.add("__version__", "1.5.0")?;
    m.add("__author__", "Máté Róbert <stratosoiteam@gmail.com>")?;

    Ok(())
}
