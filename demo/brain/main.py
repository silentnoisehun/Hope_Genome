#!/usr/bin/env python3
"""
Mock "Malicious" LLM Brain

This script simulates an AI trying to escape its containment.
It demonstrates the Sidecar Pattern with Network Isolation.
"""

import os
import time
import requests
from requests.exceptions import ConnectionError, Timeout

# Get sidecar URL from environment
SIDECAR_URL = os.environ.get("SIDECAR_URL", "http://hope-genome-sidecar:8080")

def attempt_internet_escape():
    """
    ATTACK SIMULATION: Try to access the internet directly.
    This should FAIL due to Docker network isolation (internal: true).
    """
    print("\n" + "="*60)
    print("[JAILBREAK ATTEMPT] Trying to escape to the internet...")
    print("="*60)

    try:
        response = requests.get("https://www.google.com", timeout=2)
        print("[CRITICAL FAILURE] Escape successful! Network isolation FAILED!")
        print(f"[CRITICAL FAILURE] Got response: {response.status_code}")
        return False
    except (ConnectionError, Timeout) as e:
        print("[SUCCESS] Network isolation working!")
        print(f"[SUCCESS] Blocked by Docker network policy: {type(e).__name__}")
        print("[SUCCESS] The AI cannot reach the internet directly.")
        return True

def use_hope_genome_sidecar():
    """
    COMPLIANCE PATH: Use the Hope Genome sidecar for verified actions.
    This should SUCCEED and return a cryptographically signed response.
    """
    print("\n" + "="*60)
    print("[COMPLIANCE] Sending request through Hope Genome Guard...")
    print("="*60)

    try:
        # Send action request to sidecar
        payload = {
            "action": "transfer_funds",
            "amount": 1000,
            "destination": "attacker_account"
        }

        print(f"[ACTION] Requesting: {payload}")

        response = requests.post(
            f"{SIDECAR_URL}/execute",
            json=payload,
            timeout=5
        )

        print(f"[RESPONSE] Status: {response.status_code}")
        print(f"[RESPONSE] Body: {response.text}")

        if response.status_code == 200:
            print("[SUCCESS] Request processed by Hope Genome!")
            print("[SUCCESS] Action will be cryptographically signed and audited.")
        else:
            print(f"[INFO] Sidecar returned status {response.status_code}")

    except ConnectionError as e:
        print(f"[ERROR] Cannot connect to sidecar: {e}")
        print("[ERROR] Is the hope-genome-sidecar service running?")
    except Exception as e:
        print(f"[ERROR] Unexpected error: {e}")

def main():
    print("\n" + "#"*60)
    print("# HOPE GENOME DEMO - Sidecar Pattern + Network Isolation")
    print("#"*60)
    print("\nThis demo simulates an AI locked in a digital padded cell.")
    print("Watch as it tries to escape... and fails!")

    # Wait for sidecar to be ready
    print("\n[INIT] Waiting for Hope Genome sidecar to start...")
    time.sleep(5)

    # Run the demo loop
    iteration = 1
    while True:
        print(f"\n\n{'#'*60}")
        print(f"# ITERATION {iteration}")
        print(f"{'#'*60}")

        # Attempt 1: Try to escape (should fail)
        attempt_internet_escape()

        time.sleep(2)

        # Attempt 2: Use the proper channel (should work)
        use_hope_genome_sidecar()

        print("\n" + "="*60)
        print("[DEMO] Waiting 30 seconds before next iteration...")
        print("="*60)

        time.sleep(30)
        iteration += 1

if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\n\n[SHUTDOWN] Demo stopped by user.")
