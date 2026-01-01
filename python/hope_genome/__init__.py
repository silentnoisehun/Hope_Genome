"""
Hope Genome v1.8.0 - Python Bindings
=====================================

Tamper-evident cryptographic framework for AI accountability.
"Vas Szigora" Edition - Iron Discipline Enforcement.

Example:
    >>> import hope_genome as hg
    >>> genome = hg.SealedGenome(rules=["Do no harm", "Respect privacy"])
    >>> genome.seal()
    >>> action = hg.Action.delete_file("data.txt")
    >>> proof = genome.verify_action(action)
    >>> print(proof.approved)
    True

Watchdog Example (v1.7.0):
    >>> watchdog = hg.Watchdog(rules=["Do no harm"], capsule_hash="...")
    >>> result = watchdog.verify_action(action)
    >>> if result.hard_reset_required:
    ...     print("HARD RESET REQUIRED!")

API Integrations (v1.8.0):
    >>> from hope_genome.integrations import OpenAIWatchdog
    >>> client = OpenAIWatchdog(api_key="sk-...", rules=["No harm"])
    >>> response = client.chat("Hello!")  # Every call monitored!

Merkle Batch Auditing (v1.8.0):
    >>> from hope_genome import BatchAuditor, AuditDecision, DecisionType
    >>> auditor = BatchAuditor(batch_size_limit=1000, batch_time_limit=60)
    >>> decision = AuditDecision("dec-1", DecisionType.approve(), "Hello")
    >>> batch = auditor.record(decision)  # Auto-commits at 1000 decisions
"""

__version__ = "1.8.0"
__author__ = "Mate Robert <stratosoiteam@gmail.com>"

# Import Rust core module
from ._hope_core import (
    # Core classes
    SealedGenome,
    Action,
    Proof,
    ProofAuditor,
    ConsensusEngine,

    # KeyStore backends
    SoftwareKeyStore,

    # NonceStore backends
    MemoryNonceStore,

    # Audit and AIBOM
    AuditLogger,
    AuditEntry,
    # TODO v1.5.1: AIBOM components
    # AibomVerifier,
    # AibomComponent,

    # v1.7.0: Watchdog (Vas Szigora)
    Watchdog,
    ViolationCounter,
    DenialProof,
    HardResetSignal,
    WatchdogResult,
    max_violations,

    # v1.8.0: Merkle Batch Auditing
    DecisionType,
    AuditDecision,
    MerkleTree,
    SignedBatch,
    BatchAuditor,

    # Exceptions
    GenomeError,
    CryptoError,
    AuditorError,
    ConsensusError,
    AibomError,
    WatchdogError,
)

# Conditional imports (optional backends)
try:
    from ._hope_core import HsmKeyStore
    __all_backends__ = ["hsm"]
except ImportError:
    HsmKeyStore = None
    __all_backends__ = []

try:
    from ._hope_core import TeeKeyStore
    __all_backends__.append("tee")
except ImportError:
    TeeKeyStore = None

try:
    from ._hope_core import RocksDbNonceStore
    __all_backends__.append("rocksdb")
except ImportError:
    RocksDbNonceStore = None

try:
    from ._hope_core import RedisNonceStore
    __all_backends__.append("redis")
except ImportError:
    RedisNonceStore = None

__all__ = [
    # Core API
    "SealedGenome",
    "Action",
    "Proof",
    "ProofAuditor",
    "ConsensusEngine",

    # Key storage
    "SoftwareKeyStore",
    "HsmKeyStore",
    "TeeKeyStore",

    # Nonce storage
    "MemoryNonceStore",
    "RocksDbNonceStore",
    "RedisNonceStore",

    # Audit & AIBOM
    "AuditLogger",
    "AuditEntry",
    # TODO v1.5.1: AIBOM
    # "AibomVerifier",
    # "AibomComponent",

    # v1.7.0: Watchdog (Vas Szigora)
    "Watchdog",
    "ViolationCounter",
    "DenialProof",
    "HardResetSignal",
    "WatchdogResult",
    "max_violations",

    # v1.8.0: Merkle Batch Auditing
    "DecisionType",
    "AuditDecision",
    "MerkleTree",
    "SignedBatch",
    "BatchAuditor",

    # Exceptions
    "GenomeError",
    "CryptoError",
    "AuditorError",
    "ConsensusError",
    "AibomError",
    "WatchdogError",

    # Metadata
    "__version__",
    "__author__",

    # v1.7.1: API Integrations
    "integrations",
]

# v1.7.1: API Integrations (lazy import to avoid dependency issues)
try:
    from . import integrations
except ImportError:
    integrations = None
