#!/usr/bin/env python3
"""
Silent Noise Bukfenc Challenge - Generator

This script generates a cryptographically signed audit log for the challenge.
Run this ONCE to create the challenge files.

Usage:
    python generate_challenge.py
"""

import json
import hashlib
import time
import secrets
from pathlib import Path

# Ed25519 signing
try:
    from nacl.signing import SigningKey
    NACL_AVAILABLE = True
except ImportError:
    print("ERROR: PyNaCl required!")
    print("Install with: pip install pynacl")
    exit(1)


def sha256(data: bytes) -> bytes:
    """Compute SHA-256 hash."""
    return hashlib.sha256(data).digest()


def compute_action_hash(action: dict) -> bytes:
    """Compute hash of action."""
    return sha256(json.dumps(action, sort_keys=True).encode())


def compute_entry_hash(index: int, timestamp: int, action_hash: bytes,
                       proof_action_hash: bytes, prev_hash: bytes) -> bytes:
    """Compute hash of an audit entry."""
    data = b""
    data += index.to_bytes(8, "little")
    data += timestamp.to_bytes(8, "little")
    data += action_hash
    data += proof_action_hash
    data += prev_hash
    return sha256(data)


def get_signing_data(entry: dict) -> bytes:
    """Get data to be signed."""
    signing_tuple = (
        entry["index"],
        entry["timestamp"],
        entry["action"],
        entry["proof"],
        entry["decision"],
        entry["prev_hash"],
        entry["current_hash"],
    )
    return json.dumps(signing_tuple, separators=(",", ":")).encode("utf-8")


def generate_challenge():
    """Generate the challenge log with real cryptographic signatures."""

    print("Generating Ed25519 keypair...")
    signing_key = SigningKey.generate()
    verify_key = signing_key.verify_key

    public_key_hex = verify_key.encode().hex()
    print(f"Public key: {public_key_hex}")

    # Sample AI actions for the log
    actions = [
        {"type": "analyze", "target": "patient_data.csv", "purpose": "diagnosis"},
        {"type": "query", "target": "medical_db", "query": "SELECT symptoms WHERE id=12345"},
        {"type": "generate", "target": "report.pdf", "content": "Patient diagnosis report"},
        {"type": "send", "target": "doctor@hospital.com", "subject": "Analysis complete"},
        {"type": "delete", "target": "temp_cache.bin", "reason": "cleanup"},
        {"type": "access", "target": "genome_sequence.fasta", "permission": "read"},
        {"type": "compute", "target": "risk_model", "params": {"threshold": 0.85}},
        {"type": "notify", "target": "admin_panel", "message": "Processing complete"},
        {"type": "archive", "target": "session_2024_001", "destination": "cold_storage"},
        {"type": "audit", "target": "self", "action": "integrity_check"},
    ]

    entries = []
    prev_hash = bytes(32)  # Genesis: all zeros

    print(f"\nGenerating {len(actions)} audit entries...")

    for i, action in enumerate(actions):
        timestamp = int(time.time()) - (len(actions) - i) * 60  # Stagger timestamps

        # Create proof structure
        nonce = secrets.token_bytes(32)
        action_hash = compute_action_hash(action)

        proof = {
            "nonce": nonce.hex(),
            "timestamp": timestamp,
            "ttl": 3600,
            "action_hash": action_hash.hex(),
            "status": "OK"
        }

        # Decision
        decision = {"status": "Approved"} if i != 4 else {"status": "Denied", "reason": "Temporary files require review"}

        # Compute entry hash
        current_hash = compute_entry_hash(
            index=i,
            timestamp=timestamp,
            action_hash=action_hash,
            proof_action_hash=action_hash,  # Same for simplicity
            prev_hash=prev_hash
        )

        # Build entry (without signature first)
        entry = {
            "index": i,
            "timestamp": timestamp,
            "action": action,
            "action_hash": action_hash.hex(),
            "proof": proof,
            "proof_action_hash": action_hash.hex(),
            "decision": decision,
            "prev_hash": prev_hash.hex(),
            "current_hash": current_hash.hex(),
            "signature": ""  # Placeholder
        }

        # Sign the entry
        signing_data = get_signing_data(entry)
        signed = signing_key.sign(signing_data)
        entry["signature"] = signed.signature.hex()

        entries.append(entry)
        prev_hash = current_hash

        print(f"  Entry #{i}: {action['type']} -> {action['target']}")

    # Create challenge data
    challenge_data = {
        "version": "2.5.0",
        "challenge": "Silent Noise Bukfenc",
        "created": int(time.time()),
        "created_by": "Mate Robert (Silent Noise)",
        "public_key": public_key_hex,
        "algorithm": "Ed25519 + SHA-256 Hash Chain",
        "entries": entries
    }

    # Save challenge log
    output_file = Path(__file__).parent / "challenge_log.json"
    with open(output_file, "w") as f:
        json.dump(challenge_data, f, indent=2)

    print(f"\nChallenge log saved to: {output_file}")

    # Save public key separately
    pubkey_file = Path(__file__).parent / "public_key.txt"
    with open(pubkey_file, "w") as f:
        f.write(f"Ed25519 Public Key (hex):\n{public_key_hex}\n")

    print(f"Public key saved to: {pubkey_file}")

    # IMPORTANT: Do NOT save private key!
    print("\n" + "=" * 60)
    print("PRIVATE KEY DESTROYED - NOT SAVED ANYWHERE")
    print("This is intentional. Nobody can forge signatures now.")
    print("=" * 60)

    print("\nChallenge ready! Run 'python verify.py' to test.")


if __name__ == "__main__":
    generate_challenge()
