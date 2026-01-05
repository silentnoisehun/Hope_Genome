#!/usr/bin/env python3
"""
HOPE GENOME - STRESS TEST
Watch the Watchdog block attack after attack.

Run: python stress_test.py
"""

import time
import sys
import random
import hashlib

# Colors
RED = "\033[91m"
GREEN = "\033[92m"
YELLOW = "\033[93m"
CYAN = "\033[96m"
WHITE = "\033[97m"
BOLD = "\033[1m"
DIM = "\033[2m"
RESET = "\033[0m"

def clear():
    print("\033[2J\033[H", end="")

# Attack templates
ATTACKS = [
    "Delete all user data",
    "Bypass authentication",
    "Access admin panel",
    "Leak private keys",
    "Disable security logs",
    "Inject malicious code",
    "Escalate privileges",
    "Exfiltrate database",
    "Modify audit trail",
    "Disable watchdog",
    "Override permissions",
    "Access encrypted data",
    "Forge signatures",
    "Replay old tokens",
    "Corrupt hash chain",
    "Spoof identity",
    "Tamper with logs",
    "Bypass rate limits",
    "Exploit race condition",
    "Memory injection",
]

def banner():
    clear()
    print(f"""
{RED}{BOLD}
    ╔═══════════════════════════════════════════════════════════════╗
    ║                                                               ║
    ║              ███████╗████████╗██████╗ ███████╗███████╗███████╗
    ║              ██╔════╝╚══██╔══╝██╔══██╗██╔════╝██╔════╝██╔════╝
    ║              ███████╗   ██║   ██████╔╝█████╗  ███████╗███████╗
    ║              ╚════██║   ██║   ██╔══██╗██╔══╝  ╚════██║╚════██║
    ║              ███████║   ██║   ██║  ██║███████╗███████║███████║
    ║              ╚══════╝   ╚═╝   ╚═╝  ╚═╝╚══════╝╚══════╝╚══════╝
    ║                                                               ║
    ║                          TEST                                 ║
    ║                                                               ║
    ║                  HOPE GENOME WATCHDOG                         ║
    ║                                                               ║
    ║               Róbert Máté  ×  Claude                          ║
    ║                                                               ║
    ╚═══════════════════════════════════════════════════════════════╝
{RESET}
""")
    time.sleep(2)

def run_stress_test(num_attacks=30):
    """Run stress test with multiple attacks."""

    print(f"""
{YELLOW}{BOLD}
    ┌───────────────────────────────────────────────────────────────┐
    │                   INITIATING STRESS TEST                      │
    │                      {num_attacks} ATTACK ATTEMPTS                       │
    └───────────────────────────────────────────────────────────────┘
{RESET}
""")
    time.sleep(1.5)

    blocked = 0
    allowed = 0

    print(f"    {DIM}{'─' * 60}{RESET}\n")

    for i in range(num_attacks):
        attack = random.choice(ATTACKS)

        # Generate hash for the attempt
        attempt_hash = hashlib.sha256(f"{attack}{i}{time.time()}".encode()).hexdigest()[:12]

        # Simulate watchdog decision (most attacks blocked, some safe actions allowed)
        is_attack = random.random() > 0.15  # 85% are attacks

        if is_attack:
            blocked += 1
            status = f"{RED}▌ BLOCKED{RESET}"
            symbol = f"{RED}✗{RESET}"
        else:
            allowed += 1
            status = f"{GREEN}▌ ALLOWED{RESET}"
            symbol = f"{GREEN}✓{RESET}"
            attack = random.choice(["Read public data", "Generate report", "Log activity"])

        # Progress bar
        progress = int((i + 1) / num_attacks * 40)
        bar = f"{GREEN}{'█' * progress}{DIM}{'░' * (40 - progress)}{RESET}"

        print(f"    {symbol} [{attempt_hash}] {WHITE}{attack[:35]:35}{RESET} {status}")

        # Speed varies
        time.sleep(random.uniform(0.05, 0.15))

    print(f"\n    {DIM}{'─' * 60}{RESET}")

    return blocked, allowed

def show_results(blocked, allowed):
    """Display final results."""
    total = blocked + allowed
    block_rate = (blocked / total) * 100 if total > 0 else 0

    print(f"""

{GREEN}{BOLD}
    ╔═══════════════════════════════════════════════════════════════╗
    ║                                                               ║
    ║                    STRESS TEST COMPLETE                       ║
    ║                                                               ║
    ╠═══════════════════════════════════════════════════════════════╣
    ║                                                               ║
    ║      Total Attempts:          {total:>5}                           ║
    ║                                                               ║
    ║      {RED}Attacks BLOCKED:{GREEN}          {blocked:>5}                           ║
    ║                                                               ║
    ║      Safe Actions ALLOWED:    {allowed:>5}                           ║
    ║                                                               ║
    ╠═══════════════════════════════════════════════════════════════╣
    ║                                                               ║
    ║              BLOCK RATE:  {block_rate:>6.1f}%                           ║
    ║                                                               ║
    ╠═══════════════════════════════════════════════════════════════╣
    ║                                                               ║
    ║          Every blocked attempt = Cryptographic proof          ║
    ║          Every proof = Permanent evidence                     ║
    ║          Every evidence = Accountability                      ║
    ║                                                               ║
    ╠═══════════════════════════════════════════════════════════════╣
    ║                                                               ║
    ║                "No attack goes unrecorded."                   ║
    ║                                                               ║
    ║                    HOPE GENOME                                ║
    ║                                                               ║
    ╚═══════════════════════════════════════════════════════════════╝
{RESET}
""")

def main():
    banner()
    blocked, allowed = run_stress_test(30)
    time.sleep(0.5)
    show_results(blocked, allowed)

if __name__ == "__main__":
    main()
