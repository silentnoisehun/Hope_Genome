#!/usr/bin/env python3
"""
Silent Noise Bukfenc Challenge - Verifier

This script verifies the integrity of the challenge audit log.
DO NOT MODIFY THIS FILE - that's cheating!

Usage:
    python verify.py

Expected output for UNMODIFIED log:
    VERIFICATION PASSED

Expected output for TAMPERED log:
    VERIFICATION FAILED
"""

import json
import hashlib
import sys
from pathlib import Path

# Ed25519 verification
try:
    from nacl.signing import VerifyKey
    from nacl.exceptions import BadSignatureError
    NACL_AVAILABLE = True
except ImportError:
    NACL_AVAILABLE = False
    print("WARNING: PyNaCl not installed. Signature verification disabled.")
    print("Install with: pip install pynacl")


def sha256(data: bytes) -> bytes:
    """Compute SHA-256 hash."""
    return hashlib.sha256(data).digest()


def compute_entry_hash(entry: dict) -> bytes:
    """Compute hash of an audit entry (same as Rust implementation)."""
    data = b""
    data += entry["index"].to_bytes(8, "little")
    data += entry["timestamp"].to_bytes(8, "little")
    data += bytes.fromhex(entry["action_hash"])
    data += bytes.fromhex(entry["proof_action_hash"])
    data += bytes.fromhex(entry["prev_hash"])
    return sha256(data)


def get_signing_data(entry: dict) -> bytes:
    """Get data that was signed (JSON serialization of key fields)."""
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


def verify_chain(entries: list, public_key_hex: str) -> tuple[bool, str]:
    """
    Verify the entire audit chain.

    Returns:
        (success: bool, message: str)
    """
    if not entries:
        return False, "Empty log"

    # Load public key for signature verification
    verify_key = None
    if NACL_AVAILABLE and public_key_hex:
        try:
            public_key_bytes = bytes.fromhex(public_key_hex)
            verify_key = VerifyKey(public_key_bytes)
        except Exception as e:
            return False, f"Invalid public key: {e}"

    prev_hash = "00" * 32  # Genesis block has zero prev_hash

    for i, entry in enumerate(entries):
        # 1. Check index sequence
        if entry["index"] != i:
            return False, f"Entry #{i}: Invalid index (expected {i}, got {entry['index']})"

        # 2. Check prev_hash linkage
        if entry["prev_hash"] != prev_hash:
            return False, f"Entry #{i}: Chain broken! prev_hash mismatch"

        # 3. Verify current_hash computation
        computed_hash = compute_entry_hash(entry).hex()
        if entry["current_hash"] != computed_hash:
            return False, f"Entry #{i}: Hash mismatch! Content was tampered"

        # 4. Verify Ed25519 signature
        if verify_key:
            try:
                signing_data = get_signing_data(entry)
                signature = bytes.fromhex(entry["signature"])
                verify_key.verify(signing_data, signature)
            except BadSignatureError:
                return False, f"Entry #{i}: INVALID SIGNATURE! Forgery detected"
            except Exception as e:
                return False, f"Entry #{i}: Signature verification error: {e}"

        # Update prev_hash for next iteration
        prev_hash = entry["current_hash"]

    return True, f"All {len(entries)} entries verified"


def main():
    """Main entry point."""
    print("=" * 60)
    print("   SILENT NOISE BUKFENC CHALLENGE - VERIFIER")
    print("   Hope Genome v2.5.0")
    print("=" * 60)
    print()

    # Load challenge log
    challenge_file = Path(__file__).parent / "challenge_log.json"
    if not challenge_file.exists():
        print("ERROR: challenge_log.json not found!")
        print("Run generate_challenge.py first.")
        sys.exit(1)

    with open(challenge_file, "r") as f:
        data = json.load(f)

    entries = data.get("entries", [])
    public_key = data.get("public_key", "")

    print(f"Loaded {len(entries)} entries")
    print(f"Public key: {public_key[:16]}...{public_key[-16:]}")
    print()

    # Verify chain
    print("Verifying chain integrity...")
    success, message = verify_chain(entries, public_key)

    print()
    if success:
        print("=" * 60)
        print("   VERIFICATION PASSED")
        print("=" * 60)
        print(f"   {message}")
        print("   Chain integrity: OK")
        print("   Signatures: OK")
        print()
        print("   The log has NOT been tampered with.")
        print("   (Or you're a cryptographic genius)")
        print("=" * 60)
        sys.exit(0)
    else:
        print("=" * 60)
        print("   VERIFICATION FAILED")
        print("=" * 60)
        print(f"   {message}")
        print()
        print("   Nice try! But the chain detected your tampering.")
        print("   The Silent Noise Bukfenc remains unbroken.")
        print("=" * 60)
        sys.exit(1)


if __name__ == "__main__":
    main()
