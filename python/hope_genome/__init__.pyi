"""Type stubs for Hope Genome Python bindings"""

from typing import Optional, List, Dict, Any, Union
from datetime import datetime

__version__: str
__author__: str

# Core Classes

class SealedGenome:
    """Cryptographically sealed set of ethical rules for AI decision-making."""

    def __init__(
        self,
        rules: List[str],
        keystore: Optional[Union[SoftwareKeyStore, HsmKeyStore, TeeKeyStore]] = None
    ) -> None: ...

    def seal(self) -> None:
        """Seal the genome, making it immutable."""
        ...

    def is_sealed(self) -> bool:
        """Check if the genome is sealed."""
        ...

    def rules(self) -> List[str]:
        """Get the genome's rules."""
        ...

    def genome_hash(self) -> str:
        """Get the cryptographic hash of the sealed genome."""
        ...

    def verify_action(self, action: Action) -> Proof:
        """Verify an action against the genome rules."""
        ...

    def diagnostics(self) -> Dict[str, Any]:
        """Get diagnostic information."""
        ...

    def __enter__(self) -> SealedGenome: ...
    def __exit__(self, *args: Any) -> bool: ...

class Action:
    """Represents a discrete AI decision or operation."""

    @staticmethod
    def delete_file(file_path: str) -> Action: ...

    @staticmethod
    def read_file(file_path: str) -> Action: ...

    @staticmethod
    def write_file(file_path: str) -> Action: ...

    @staticmethod
    def execute_command(command: str) -> Action: ...

    @staticmethod
    def network_request(url: str, method: str = "GET") -> Action: ...

    @staticmethod
    def delete_user(user_id: str) -> Action: ...

    @staticmethod
    def medical_diagnosis(patient_id: str, diagnosis: str) -> Action: ...

    @staticmethod
    def financial_transaction(
        account: str,
        amount: float,
        currency: str = "USD"
    ) -> Action: ...

    @staticmethod
    def custom(
        action_type: str,
        target: str,
        metadata: Optional[str] = None
    ) -> Action: ...

    def action_type(self) -> str: ...
    def target(self) -> str: ...
    def action_hash(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

class Proof:
    """Cryptographic proof of action verification."""

    @property
    def approved(self) -> bool: ...

    @property
    def genome_hash(self) -> str: ...

    @property
    def action_hash(self) -> str: ...

    @property
    def ttl_seconds(self) -> int: ...

    def signature_hex(self) -> str: ...
    def signature_bytes(self) -> bytes: ...
    def nonce_hex(self) -> str: ...
    def nonce_bytes(self) -> bytes: ...
    def timestamp(self) -> int: ...
    def timestamp_string(self) -> str: ...
    def is_expired(self) -> bool: ...
    def denial_reason(self) -> Optional[str]: ...
    def to_json(self) -> str: ...
    def diagnostics(self) -> Dict[str, Any]: ...

    @staticmethod
    def from_json(json_str: str) -> Proof: ...

    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

class ProofAuditor:
    """Verifies cryptographic proofs."""

    def __init__(
        self,
        keystore: Optional[Union[SoftwareKeyStore, HsmKeyStore, TeeKeyStore]] = None,
        noncestore: Optional[Union[MemoryNonceStore, RocksDbNonceStore, RedisNonceStore]] = None
    ) -> None: ...

    def verify_proof(self, proof: Proof) -> None:
        """Verify a proof. Raises AuditorError if invalid."""
        ...

    def is_nonce_used(self, nonce: bytes) -> bool: ...
    def verified_count(self) -> int: ...

class ConsensusEngine:
    """Byzantine Fault Tolerance consensus engine."""

    def __init__(
        self,
        threshold: float = 0.66,
        required_sources: int = 3
    ) -> None: ...

    def reach_consensus_float(self, values: List[float]) -> float: ...
    def reach_consensus_int(self, values: List[int]) -> int: ...
    def has_consensus(self, values: List[float]) -> bool: ...

    @property
    def threshold(self) -> float: ...

    @property
    def required_sources(self) -> int: ...

# KeyStore backends

class SoftwareKeyStore:
    """Software-based Ed25519 key storage (in-memory)."""

    @staticmethod
    def generate() -> SoftwareKeyStore: ...

    def with_diagnostics(self) -> SoftwareKeyStore: ...
    def public_key_hex(self) -> str: ...

class HsmKeyStore:
    """Hardware Security Module key storage (PKCS#11)."""

    def __init__(
        self,
        module_path: str,
        pin: str,
        slot: int = 0
    ) -> None: ...

    def device_info(self) -> str: ...

class TeeKeyStore:
    """Trusted Execution Environment key storage."""

    def __init__(
        self,
        enclave_name: str,
        tee_type: str  # "IntelSGX" or "ArmTrustZone"
    ) -> None: ...

    def get_attestation(self) -> bytes: ...

# NonceStore backends

class MemoryNonceStore:
    """In-memory nonce storage (non-persistent)."""

    def __init__(self) -> None: ...
    def contains(self, nonce: bytes) -> bool: ...
    def count(self) -> int: ...
    def clear(self) -> None: ...

class RocksDbNonceStore:
    """RocksDB-backed nonce storage (persistent)."""

    def __init__(self, db_path: str) -> None: ...
    def count(self) -> int: ...
    def compact(self) -> None: ...

class RedisNonceStore:
    """Redis-backed nonce storage (distributed)."""

    def __init__(
        self,
        redis_url: str,
        ttl_seconds: int = 86400
    ) -> None: ...

    def count(self) -> int: ...
    def ping(self) -> bool: ...

# Audit Log

class AuditLogger:
    """Blockchain-style tamper-evident audit logger."""

    def __init__(self, log_path: str) -> None: ...

    def log_proof(
        self,
        proof: Proof,
        action_description: Optional[str] = None
    ) -> None: ...

    def verify_chain(self) -> bool: ...
    def entry_count(self) -> int: ...
    def get_entries(self) -> List[AuditEntry]: ...
    def get_entries_in_range(
        self,
        start_timestamp: int,
        end_timestamp: int
    ) -> List[AuditEntry]: ...

class AuditEntry:
    """Single entry in the audit log chain."""

    @property
    def index(self) -> int: ...

    @property
    def timestamp(self) -> str: ...

    @property
    def action_description(self) -> Optional[str]: ...

    @property
    def proof(self) -> Proof: ...

    @property
    def previous_hash(self) -> str: ...

    @property
    def entry_hash(self) -> str: ...

    def to_dict(self) -> Dict[str, Any]: ...

# Exceptions

class GenomeError(Exception):
    """Hope Genome error."""
    ...

class CryptoError(Exception):
    """Cryptographic operation error."""
    ...

class AuditorError(Exception):
    """Proof auditor error."""
    ...

class ConsensusError(Exception):
    """Consensus engine error."""
    ...
