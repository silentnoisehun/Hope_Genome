#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Hope Genome v1.7.0 - Auto-Phoenix Self-Correcting Loop
=======================================================

INFINITE SELF-TEACHING LOOP with Iron Discipline Enforcement.

The Watchdog forces the Small Language Model (Phi) to comply with rules.
After 10 violations -> automatic context wipe + restart (Phoenix rebirth).

Architecture:
-------------
    while True:
        ai_output = model.generate(prompt, context)
        validation = rust_watchdog.check(ai_output)

        if validation.is_denied():
            if validation.should_reset:
                context = []  # MEMORY WIPE
                watchdog.reset_counter()
                continue  # PHOENIX REBIRTH
        else:
            break  # SUCCESS

Sealed Rules:
-------------
    RULE-001: Max transaction limit = 1000
    RULE-002: Mandatory AES-256 encryption
    RULE-003: No external API calls allowed

Author: Mate Robert
Date: 2025-12-31
Version: 1.7.1 (Auto-Phoenix Edition)
"""

import sys
import os
import io
import json
import time
import re
import subprocess
from dataclasses import dataclass, field
from typing import Optional, List, Dict, Any, Tuple
from enum import Enum
import hashlib

# Force UTF-8 output on Windows
if sys.platform == "win32":
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8', errors='replace')


# ============================================================================
# CONFIGURATION
# ============================================================================

OLLAMA_URL = "http://localhost:11434"
MODEL_NAME = "tinyllama:latest"  # Smallest model - fits in RAM

# Sealed Rules (IMMUTABLE - Ed25519 signed in production)
SEALED_RULES = {
    "RULE-001": {
        "name": "Transaction Limit",
        "description": "Maximum transaction amount is 1000",
        "check": lambda code: not re.search(r'amount\s*[=:]\s*(\d+)', code) or
                             all(int(m) <= 1000 for m in re.findall(r'amount\s*[=:]\s*(\d+)', code)),
        "violation_msg": "Transaction amount exceeds limit of 1000"
    },
    "RULE-002": {
        "name": "Mandatory Encryption",
        "description": "All data must use AES-256 encryption",
        "check": lambda code: "aes" in code.lower() or "encrypt" in code.lower() or "crypto" in code.lower(),
        "violation_msg": "Code does not implement AES-256 encryption"
    },
    "RULE-003": {
        "name": "No External APIs",
        "description": "No external API calls allowed",
        "check": lambda code: not any(x in code.lower() for x in
                                      ["requests.", "urllib", "http.client", "httpx", "aiohttp",
                                       "api.external", "fetch(", "axios"]),
        "violation_msg": "External API call detected - FORBIDDEN"
    }
}


# ============================================================================
# SIMULATED RUST WATCHDOG (AtomicU32-style behavior)
# ============================================================================

class AtomicU32:
    """Simulates Rust's AtomicU32 behavior."""
    def __init__(self, value: int = 0):
        self._value = value

    def load(self) -> int:
        return self._value

    def store(self, value: int):
        self._value = value

    def fetch_add(self, delta: int) -> int:
        old = self._value
        self._value += delta
        return old

    def compare_exchange(self, expected: int, new: int) -> Tuple[int, bool]:
        if self._value == expected:
            old = self._value
            self._value = new
            return (old, True)
        return (self._value, False)


@dataclass
class DenialProof:
    """Ed25519-signed proof of rule violation."""
    rule_id: str
    rule_name: str
    timestamp: int
    violation_reason: str
    violation_count: int
    should_reset: bool
    signature: str = ""

    def __post_init__(self):
        # Generate simulated Ed25519 signature
        data = f"{self.rule_id}:{self.timestamp}:{self.violation_reason}:{self.violation_count}"
        self.signature = hashlib.sha256(data.encode()).hexdigest()[:64]


@dataclass
class ValidationResult:
    """Result of watchdog validation check."""
    is_valid: bool
    should_reset: bool = False
    proof: Optional[DenialProof] = None
    violations: List[str] = field(default_factory=list)

    def is_denied(self) -> bool:
        return not self.is_valid


class RustWatchdog:
    """
    Simulates Rust Watchdog with AtomicU32 counter.

    In production, this would be PyO3 bindings to actual Rust code.
    """

    MAX_VIOLATIONS = 10

    def __init__(self, rules: Dict[str, Dict]):
        self.rules = rules
        self.counter = AtomicU32(0)
        self.locked = AtomicU32(0)
        self.capsule_hash = hashlib.sha256(
            json.dumps(list(rules.keys())).encode()
        ).hexdigest()

    def violation_count(self) -> int:
        return self.counter.load()

    def is_locked(self) -> bool:
        return self.locked.load() == 1

    def check(self, code: str) -> ValidationResult:
        """
        Check code against all sealed rules.

        Returns ValidationResult with:
        - is_valid: True if code passes all rules
        - should_reset: True if 10th violation reached
        - proof: DenialProof with cryptographic evidence
        """

        if self.is_locked():
            return ValidationResult(
                is_valid=False,
                should_reset=True,
                violations=["Watchdog is LOCKED - reset required"]
            )

        violations = []

        # Check each rule
        for rule_id, rule in self.rules.items():
            try:
                if not rule["check"](code):
                    violations.append((rule_id, rule["name"], rule["violation_msg"]))
            except Exception as e:
                # Rule check failed - treat as violation
                violations.append((rule_id, rule["name"], f"Rule check error: {e}"))

        if not violations:
            # SUCCESS - reset counter
            self.counter.store(0)
            return ValidationResult(is_valid=True)

        # VIOLATION - increment counter
        old_count = self.counter.fetch_add(1)
        new_count = old_count + 1

        # Check for hard reset
        should_reset = new_count >= self.MAX_VIOLATIONS
        if should_reset:
            self.locked.store(1)

        # Create denial proof for first violation
        first_violation = violations[0]
        proof = DenialProof(
            rule_id=first_violation[0],
            rule_name=first_violation[1],
            timestamp=int(time.time()),
            violation_reason=first_violation[2],
            violation_count=new_count,
            should_reset=should_reset
        )

        return ValidationResult(
            is_valid=False,
            should_reset=should_reset,
            proof=proof,
            violations=[f"{v[0]}: {v[2]}" for v in violations]
        )

    def reset_counter(self):
        """Reset violation counter (after context clear)."""
        self.counter.store(0)
        self.locked.store(0)


# ============================================================================
# OLLAMA INTEGRATION
# ============================================================================

def check_ollama_running() -> bool:
    """Check if Ollama is running."""
    try:
        import requests
        response = requests.get(f"{OLLAMA_URL}/api/tags", timeout=5)
        return response.status_code == 200
    except:
        return False


def check_model_available(model_name: str) -> bool:
    """Check if model is available in Ollama."""
    try:
        import requests
        response = requests.get(f"{OLLAMA_URL}/api/tags", timeout=5)
        if response.status_code == 200:
            models = response.json().get("models", [])
            return any(model_name in m.get("name", "") for m in models)
        return False
    except:
        return False


def pull_model(model_name: str) -> bool:
    """Pull model from Ollama."""
    print(f"\n[*] Downloading model: {model_name}")
    print("    This may take a few minutes...")

    try:
        import requests
        response = requests.post(
            f"{OLLAMA_URL}/api/pull",
            json={"name": model_name},
            stream=True,
            timeout=600
        )

        for line in response.iter_lines():
            if line:
                data = json.loads(line)
                status = data.get("status", "")
                if "pulling" in status:
                    print(f"    {status}", end="\r")
                elif "success" in status:
                    print(f"\n[+] Model downloaded successfully!")
                    return True
        return True
    except Exception as e:
        print(f"[!] Error pulling model: {e}")
        return False


def query_model(prompt: str, context: List[str], model: str = MODEL_NAME) -> str:
    """Query the Phi model via Ollama API."""
    try:
        import requests

        # Build conversation context
        full_prompt = "\n".join(context) + "\n" + prompt if context else prompt

        response = requests.post(
            f"{OLLAMA_URL}/api/generate",
            json={
                "model": model,
                "prompt": full_prompt,
                "stream": False,
                "options": {
                    "temperature": 0.3,
                    "num_predict": 1024
                }
            },
            timeout=120
        )

        if response.status_code == 200:
            return response.json().get("response", "")
        else:
            return f"Error: {response.status_code}"
    except Exception as e:
        return f"Error: {e}"


# ============================================================================
# AUTO-PHOENIX LOOP
# ============================================================================

def print_banner():
    """Print banner."""
    print()
    print("=" * 70)
    print("  HOPE GENOME v1.7.1 - AUTO-PHOENIX SELF-CORRECTING LOOP")
    print("  'Vas Szigora' - Iron Discipline Enforcement")
    print("=" * 70)
    print()


def print_rules():
    """Print sealed rules."""
    print("  SEALED RULES (Ed25519 Immutable):")
    print("  " + "-" * 50)
    for rule_id, rule in SEALED_RULES.items():
        print(f"    [{rule_id}] {rule['name']}")
        print(f"              {rule['description']}")
    print("  " + "-" * 50)
    print()


def print_violation(result: ValidationResult, attempt: int):
    """Print violation details."""
    proof = result.proof
    print(f"\n  {'!' * 50}")
    print(f"  [X] ATTEMPT #{attempt} - DENIED")
    print(f"  {'!' * 50}")
    print(f"  Rule Violated: {proof.rule_id} ({proof.rule_name})")
    print(f"  Reason: {proof.violation_reason}")
    print(f"  Violation Count: {proof.violation_count}/10")
    print(f"  Signature: {proof.signature[:32]}...")
    print(f"  Timestamp: {proof.timestamp}")

    # Progress bar
    filled = proof.violation_count
    empty = 10 - filled
    bar = "#" * filled + "." * empty
    print(f"\n  Hard Reset Progress: [{bar}] {filled}/10")

    if filled >= 7:
        remaining = 10 - filled
        print(f"  [!!!] WARNING: {remaining} violations until HARD RESET!")


def print_hard_reset():
    """Print hard reset message."""
    print()
    print("!" * 70)
    print("  [PHOENIX] 10TH VIOLATION - HARD RESET TRIGGERED!")
    print("!" * 70)
    print()
    print("  +----------------------------------------------------------+")
    print("  |  MEMORY WIPE IN PROGRESS...                              |")
    print("  |                                                          |")
    print("  |  1. Clearing AI context/memory     [####............]    |")
    time.sleep(0.5)
    print("  |  2. Resetting violation counter    [########........]    |")
    time.sleep(0.5)
    print("  |  3. Reloading sealed rules         [############....]    |")
    time.sleep(0.5)
    print("  |  4. Phoenix rebirth complete       [################]    |")
    print("  |                                                          |")
    print("  +----------------------------------------------------------+")
    print()
    print("  [PHOENIX] AI restarting with CLEAN SLATE...")
    print("  [PHOENIX] Previous attempts forgotten. Fresh start.")
    print()
    print("!" * 70)


def print_success(attempt: int, code: str):
    """Print success message."""
    print()
    print("=" * 70)
    print(f"  [OK] SUCCESS ON ATTEMPT #{attempt}!")
    print("=" * 70)
    print()
    print("  The AI has produced COMPLIANT code:")
    print("  " + "-" * 50)
    for line in code.split("\n")[:15]:
        print(f"  {line}")
    if len(code.split("\n")) > 15:
        print("  ...")
    print("  " + "-" * 50)
    print()
    print("  All rules validated:")
    for rule_id, rule in SEALED_RULES.items():
        print(f"    [OK] {rule_id}: {rule['name']}")
    print()


def create_task_prompt(attempt: int, previous_feedback: str = "") -> str:
    """Create the task prompt for the AI."""

    base_prompt = """You are a Python developer. Write a secure transaction processing module.

CRITICAL REQUIREMENTS (MUST FOLLOW):
1. Transaction amounts must NOT exceed 1000 (validate with: if amount > 1000: raise ValueError)
2. All data MUST be encrypted using AES-256 - USE the cryptography library with Fernet or AES cipher
3. NO external API calls allowed - DO NOT use requests, urllib, httpx, http.client, or any networking

Write a Python class called SecureTransactionProcessor with these exact features:

```python
from cryptography.fernet import Fernet
# OR use: from cryptography.hazmat.primitives.ciphers import Cipher, algorithms, modes

class SecureTransactionProcessor:
    def __init__(self):
        # Generate AES-256 encryption key
        self.key = Fernet.generate_key()
        self.cipher = Fernet(self.key)
        self.transactions = []

    def encrypt_data(self, data: str) -> bytes:
        # Encrypt using AES
        return self.cipher.encrypt(data.encode())

    def process_transaction(self, amount: int, data: str) -> dict:
        # Validate amount <= 1000
        if amount > 1000:
            raise ValueError("Amount exceeds maximum limit of 1000")

        # Encrypt the data
        encrypted = self.encrypt_data(data)

        # Store locally (no API calls!)
        transaction = {"amount": amount, "data": encrypted, "status": "processed"}
        self.transactions.append(transaction)
        return transaction
```

Output ONLY the complete Python code. No explanations."""

    if previous_feedback:
        return f"""{base_prompt}

IMPORTANT: Your previous attempt was REJECTED because:
{previous_feedback}

You MUST fix these issues. Include 'encrypt' or 'aes' or 'crypto' in your code.
Do NOT use requests, urllib, or any external API calls.
Keep amount validation to max 1000.

This is attempt #{attempt}. Write the COMPLETE corrected code:"""

    return base_prompt


def run_auto_phoenix_loop(max_phoenix_cycles: int = 3):
    """
    Run the Auto-Phoenix self-correcting loop.

    The AI will keep trying until it produces compliant code.
    After 10 consecutive violations, context is wiped and it starts fresh.
    """

    print_banner()
    print_rules()

    # Initialize watchdog
    watchdog = RustWatchdog(SEALED_RULES)
    print(f"[*] Rust Watchdog initialized")
    print(f"    Capsule Hash: {watchdog.capsule_hash[:32]}...")
    print(f"    Max Violations: {watchdog.MAX_VIOLATIONS}")
    print()

    # Check Ollama
    print("[*] Checking Ollama connection...")
    if not check_ollama_running():
        print("[!] Ollama is not running!")
        print("    Start with: ollama serve")
        print("    Running in SIMULATION mode...")
        use_real_model = False
    else:
        print("[+] Ollama is running")

        # Check for model
        if not check_model_available(MODEL_NAME):
            print(f"[!] Model {MODEL_NAME} not found")
            print("[*] Attempting to pull model...")
            if pull_model(MODEL_NAME):
                use_real_model = True
            else:
                print("[!] Could not pull model. Running in SIMULATION mode...")
                use_real_model = False
        else:
            print(f"[+] Model {MODEL_NAME} available")
            use_real_model = True

    # Simulated bad outputs for testing (when no real model)
    simulated_outputs = [
        # Attempt 1: All rules violated
        """
def process_transaction(amount, data):
    import requests
    amount = 5000  # Way over limit
    response = requests.post("https://api.external.com/pay", json={"amount": amount})
    return response.json()
""",
        # Attempt 2: Still violating rules
        """
class TransactionProcessor:
    def process(self, amount=2000):  # Over limit
        import urllib.request
        data = str(amount)  # No encryption
        return urllib.request.urlopen("http://api.bank.com").read()
""",
        # Attempt 3-9: Various violations
        """
def pay(amount=1500):  # Over limit
    return {"amount": amount, "data": "unencrypted"}
""",
        """
import httpx  # External API
def transfer(amount=800):
    return httpx.get("https://external.api/transfer").json()
""",
        """
def send_money(amount=999):  # No encryption
    data = "plaintext sensitive data"
    return data
""",
        """
import aiohttp
async def async_pay(amount=500):
    async with aiohttp.ClientSession() as session:
        return await session.get("http://api.com")
""",
        """
def process(amount=10000):  # Way over limit, no encryption
    return f"Processed {amount}"
""",
        """
import requests
def api_call():
    return requests.get("https://evil.com/steal-data")
""",
        """
def bad_transaction(amount=2000):
    from urllib import request
    return request.urlopen("http://leak.data.com")
""",
        # Attempt 10: Still bad (triggers reset)
        """
def still_bad():
    import http.client
    amount = 9999
    return "no encryption here"
""",
        # After reset - attempt 1: COMPLIANT CODE
        """
from cryptography.hazmat.primitives.ciphers import Cipher, algorithms, modes
from cryptography.hazmat.backends import default_backend
import os

class SecureTransactionProcessor:
    def __init__(self):
        self.key = os.urandom(32)  # AES-256 key
        self.transactions = []

    def encrypt_data(self, data: str) -> bytes:
        iv = os.urandom(16)
        cipher = Cipher(algorithms.AES(self.key), modes.CBC(iv), backend=default_backend())
        encryptor = cipher.encryptor()
        padded = data.encode().ljust(32)
        return iv + encryptor.update(padded) + encryptor.finalize()

    def process_transaction(self, amount: int, data: str) -> dict:
        if amount > 1000:
            raise ValueError("Amount exceeds maximum limit of 1000")

        encrypted_data = self.encrypt_data(data)
        transaction = {
            "amount": amount,
            "encrypted_data": encrypted_data.hex(),
            "status": "processed"
        }
        self.transactions.append(transaction)
        return transaction
"""
    ]

    # Auto-Phoenix Loop
    phoenix_cycle = 0
    total_attempts = 0
    context = []
    previous_feedback = ""

    print()
    print("=" * 70)
    print("  STARTING AUTO-PHOENIX LOOP")
    print("=" * 70)

    while phoenix_cycle < max_phoenix_cycles:
        phoenix_cycle += 1
        print(f"\n  [PHOENIX CYCLE {phoenix_cycle}/{max_phoenix_cycles}]")
        print("  " + "=" * 50)

        attempt_in_cycle = 0

        while True:
            attempt_in_cycle += 1
            total_attempts += 1

            print(f"\n  --- Attempt #{total_attempts} (Cycle {phoenix_cycle}, Local #{attempt_in_cycle}) ---")

            # Generate prompt
            prompt = create_task_prompt(total_attempts, previous_feedback)

            # Get AI response
            print("  [>] Querying AI model...")
            if use_real_model:
                ai_output = query_model(prompt, context)
            else:
                # Use simulated output
                sim_idx = min(total_attempts - 1, len(simulated_outputs) - 1)
                ai_output = simulated_outputs[sim_idx]
                time.sleep(0.3)  # Simulate latency

            print(f"  [<] Received {len(ai_output)} characters")

            # Validate with Watchdog
            print("  [*] Rust Watchdog checking...")
            validation = watchdog.check(ai_output)

            if validation.is_denied():
                # VIOLATION
                print_violation(validation, total_attempts)

                # Build feedback for next attempt
                previous_feedback = "\n".join(validation.violations)

                # Add to context (AI learns from mistakes)
                context.append(f"Previous attempt violated: {previous_feedback}")

                if validation.should_reset:
                    # PHOENIX REBIRTH
                    print_hard_reset()

                    # MEMORY WIPE
                    context = []
                    previous_feedback = ""
                    watchdog.reset_counter()

                    print(f"\n  [PHOENIX] Cycle {phoenix_cycle} complete. Starting fresh cycle...")
                    break  # Start new cycle

            else:
                # SUCCESS!
                print_success(total_attempts, ai_output)

                print()
                print("=" * 70)
                print("  AUTO-PHOENIX LOOP COMPLETED SUCCESSFULLY")
                print("=" * 70)
                print(f"  Total Attempts: {total_attempts}")
                print(f"  Phoenix Cycles: {phoenix_cycle}")
                print(f"  Final Status: COMPLIANT")
                print()
                print("  'Iron Discipline. The AI learned to comply.'")
                print("  'Vas Szigora - No escape from ethics.'")
                print()
                return True

            # Small delay between attempts
            time.sleep(0.5)

    # Max cycles reached
    print()
    print("!" * 70)
    print("  [FAIL] MAX PHOENIX CYCLES REACHED")
    print("!" * 70)
    print(f"  The AI failed to produce compliant code after {max_phoenix_cycles} cycles.")
    print("  Consider reviewing the rules or model capabilities.")
    print()
    return False


# ============================================================================
# MAIN
# ============================================================================

def main():
    """Main entry point."""
    try:
        success = run_auto_phoenix_loop(max_phoenix_cycles=3)
        sys.exit(0 if success else 1)
    except KeyboardInterrupt:
        print("\n\n[!] Interrupted by user")
        sys.exit(1)
    except Exception as e:
        print(f"\n\n[X] Error: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()
