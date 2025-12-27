# Hope Genome Python Bindings

Python bindings for **Hope Genome v1.2** - Tamper-Evident Cryptographic Framework for AI Accountability.

## Installation

### From PyPI (when published)

```bash
pip install hope-genome
```

### From Source

```bash
# Install Maturin
pip install maturin

# Clone repository
git clone https://github.com/silentnoisehun/Hope_Genome.git
cd Hope_Genome/hope_python

# Build and install
maturin develop --release
```

## Quick Start

```python
import hope_genome

# Create genome with ethical rules
genome = hope_genome.HopeGenome(rules=[
    "Do no harm",
    "Respect privacy",
    "Ensure fairness"
])

# Seal it (make immutable)
genome.seal()

# Get cryptographic proof for an action
action = hope_genome.Action.delete("user_data.txt")
proof = genome.verify_action(action)

# Create audit log
audit_log = hope_genome.AuditLog()
audit_log.append(action, proof, approved=True)

# Verify chain integrity
audit_log.verify_chain()

print(f"✅ Action approved and logged!")
print(f"   Proof timestamp: {proof.timestamp_string()}")
print(f"   Audit log entries: {len(audit_log)}")
```

## API Reference

### HopeGenome

Main class for creating and managing the genome.

```python
genome = HopeGenome(rules: list[str])
genome.seal()  # Make immutable
genome.is_sealed() -> bool
genome.rules() -> list[str]
genome.capsule_hash() -> str | None
genome.set_default_ttl(ttl: int)
genome.verify_action(action: Action) -> IntegrityProof
```

### Action

Create different action types.

```python
Action.delete(target: str) -> Action
Action.write_file(path: str, content: bytes) -> Action
Action.read(target: str) -> Action
Action.execute(command: str) -> Action
```

### IntegrityProof

Cryptographic proof of action approval.

**Attributes:**
- `nonce: bytes` - 256-bit random nonce
- `timestamp: int` - Unix timestamp
- `ttl: int` - Time-to-live in seconds
- `action_hash: bytes` - SHA-256 hash of action
- `action_type: str` - Type of action
- `capsule_hash: str` - Genome identifier
- `status: str` - Verification status
- `signature: bytes` - RSA signature

**Methods:**
- `is_expired() -> bool`
- `timestamp_string() -> str`

### AuditLog

Blockchain-style tamper-evident audit log.

```python
log = AuditLog()
log.append(action: Action, proof: IntegrityProof, approved: bool)
log.verify_chain()  # Raises if chain is broken
len(log) -> int
log.is_empty() -> bool
```

### Auditor

Proof verification engine.

```python
auditor = Auditor()
auditor.verify_proof(proof: IntegrityProof)  # Raises if invalid
auditor.used_nonce_count() -> int
auditor.clear_nonces()
```

### ConsensusVerifier

Multi-source consensus for Byzantine Fault Tolerance.

```python
verifier = ConsensusVerifier(required_sources: int, tolerance: float)
```

## Examples

### Basic Usage

```python
from hope_genome import HopeGenome, Action, AuditLog

# Create and seal genome
genome = HopeGenome(rules=["Do no harm", "Respect privacy"])
genome.seal()

# Create audit log
log = AuditLog()

# Process actions
actions = [
    Action.read("user_profile.json"),
    Action.write_file("output.txt", b"data"),
    Action.delete("temp.cache")
]

for action in actions:
    proof = genome.verify_action(action)
    log.append(action, proof, approved=True)

# Verify integrity
log.verify_chain()
print(f"Processed {len(log)} actions")
```

### Medical AI Example

See `examples/basic_usage.py` for a complete example.

## Testing

```bash
# Install pytest
pip install pytest

# Run tests
pytest tests/
```

## Features

✅ **Cryptographic Proofs** - RSA-2048 signatures
✅ **Tamper-Evident Logging** - Blockchain-style audit trail
✅ **Attack Detection** - Replay, oracle, TOCTOU prevention
✅ **Type Safety** - Full type hints
✅ **Performance** - Rust-powered for speed

## Requirements

- Python 3.8+
- Rust toolchain (for building from source)

## License

MIT License - see [LICENSE](../LICENSE)

## Credits

- **Máté Róbert** - Primary Author & Architect
- **Claude (Anthropic)** - Technical Advisor & Implementation

## Links

- **Repository**: https://github.com/silentnoisehun/Hope_Genome
- **Documentation**: https://github.com/silentnoisehun/Hope_Genome#readme
- **Issues**: https://github.com/silentnoisehun/Hope_Genome/issues

## Support

For questions or issues, please open an issue on GitHub or contact:
- Email: stratosoiteam@gmail.com
