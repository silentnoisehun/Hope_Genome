"""
Hope Genome - Tamper-Evident Cryptographic Framework for AI Accountability

This package provides Python bindings for the Hope Genome Rust library.

Example:
    >>> from hope_genome import HopeGenome, Action
    >>>
    >>> # Create and seal a genome
    >>> genome = HopeGenome(rules=["Do no harm", "Respect privacy"])
    >>> genome.seal()
    >>>
    >>> # Get cryptographic proof for an action
    >>> action = Action.delete("user_data.txt")
    >>> proof = genome.verify_action(action)
    >>>
    >>> print(f"Proof status: {proof.status}")
    >>> print(f"Timestamp: {proof.timestamp_string()}")
"""

from ._hope_genome import (
    HopeGenome,
    Action,
    IntegrityProof,
    Auditor,
    AuditLog,
    ConsensusVerifier,
    __version__,
    __author__,
)

__all__ = [
    "HopeGenome",
    "Action",
    "IntegrityProof",
    "Auditor",
    "AuditLog",
    "ConsensusVerifier",
    "__version__",
    "__author__",
]
