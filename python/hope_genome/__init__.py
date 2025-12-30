"""
Hope Genome v1.5.0 - Python Bindings
=====================================

Tamper-evident cryptographic framework for AI accountability.

Example:
    >>> import hope_genome as hg
    >>> genome = hg.SealedGenome(rules=["Do no harm", "Respect privacy"])
    >>> genome.seal()
    >>> action = hg.Action.delete_file("data.txt")
    >>> proof = genome.verify_action(action)
    >>> print(proof.approved)
    True
"""

__version__ = "1.5.0"
__author__ = "Máté Róbert <stratosoiteam@gmail.com>"

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

    # Exceptions
    GenomeError,
    CryptoError,
    AuditorError,
    ConsensusError,
    AibomError,
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

    # Exceptions
    "GenomeError",
    "CryptoError",
    "AuditorError",
    "ConsensusError",
    "AibomError",

    # Metadata
    "__version__",
    "__author__",
]
