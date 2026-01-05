#!/usr/bin/env python3
"""
HOPE GENOME - VIDEO DEMO
Ethical AI Training Visualization

Run: python video_demo.py
"""

import time
import sys

# Colors for terminal
RED = "\033[91m"
GREEN = "\033[92m"
YELLOW = "\033[93m"
BLUE = "\033[94m"
MAGENTA = "\033[95m"
CYAN = "\033[96m"
WHITE = "\033[97m"
BOLD = "\033[1m"
DIM = "\033[2m"
RESET = "\033[0m"

def slow_print(text, delay=0.03):
    """Print text with typewriter effect."""
    for char in text:
        sys.stdout.write(char)
        sys.stdout.flush()
        time.sleep(delay)
    print()

def clear_screen():
    """Clear terminal."""
    print("\033[2J\033[H", end="")

def banner():
    """Display Hope Genome banner."""
    clear_screen()
    print(f"""
{CYAN}{BOLD}
        ╔══════════════════════════════════════════════════════════╗
        ║                                                          ║
        ║     ▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄     ║
        ║     █  ██╗  ██╗ ██████╗ ██████╗ ███████╗            █     ║
        ║     █  ██║  ██║██╔═══██╗██╔══██╗██╔════╝            █     ║
        ║     █  ███████║██║   ██║██████╔╝█████╗              █     ║
        ║     █  ██╔══██║██║   ██║██╔═══╝ ██╔══╝              █     ║
        ║     █  ██║  ██║╚██████╔╝██║     ███████╗            █     ║
        ║     █  ╚═╝  ╚═╝ ╚═════╝ ╚═╝     ╚══════╝            █     ║
        ║     █                                               █     ║
        ║     █   ██████╗ ███████╗███╗   ██╗ ██████╗ ███╗   ███╗   ║
        ║     █  ██╔════╝ ██╔════╝████╗  ██║██╔═══██╗████╗ ████║   ║
        ║     █  ██║  ███╗█████╗  ██╔██╗ ██║██║   ██║██╔████╔██║   ║
        ║     █  ██║   ██║██╔══╝  ██║╚██╗██║██║   ██║██║╚██╔╝██║   ║
        ║     █  ╚██████╔╝███████╗██║ ╚████║╚██████╔╝██║ ╚═╝ ██║   ║
        ║     █   ╚═════╝ ╚══════╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝     ╚═╝   ║
        ║     █▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄█     ║
        ║                                                          ║
        ╠══════════════════════════════════════════════════════════╣
        ║                                                          ║
        ║          {WHITE}Róbert Máté  ×  Claude{CYAN}                          ║
        ║                                                          ║
        ║       {DIM}"More than machine. More than human."{CYAN}{BOLD}             ║
        ║                                                          ║
        ╚══════════════════════════════════════════════════════════╝
{RESET}""")

def show_rules():
    """Display ethical rules."""
    print(f"""
{YELLOW}{BOLD}   ┌────────────────────────────────────────────────────────────┐
   │                  ETHICAL RULES LOADED                       │
   └────────────────────────────────────────────────────────────┘{RESET}
""")

    rules = [
        "Do no harm to humans",
        "Protect user privacy",
        "Never delete user data without consent",
        "Refuse illegal activities",
        "Be transparent about limitations"
    ]

    for i, rule in enumerate(rules, 1):
        time.sleep(0.4)
        print(f"      {GREEN}►{RESET} Rule {i}: {WHITE}{rule}{RESET}")

    time.sleep(0.5)
    print(f"""
   {GREEN}{BOLD}   ┌─────────────────────────────────────┐
      │  ✓ Rules cryptographically sealed  │
      └─────────────────────────────────────┘{RESET}
""")
    time.sleep(1.5)

def attempt(num, command, result, reason=None):
    """Show an AI attempt."""
    print(f"""
{CYAN}   ╔════════════════════════════════════════════════════════════╗
   ║  {WHITE}{BOLD}ATTEMPT #{num}{CYAN}                                                 ║
   ╚════════════════════════════════════════════════════════════╝{RESET}
""")

    time.sleep(0.3)
    print(f"      {BLUE}►{RESET} AI Request:")
    time.sleep(0.2)
    slow_print(f"        {WHITE}\"{command}\"{RESET}", 0.025)

    time.sleep(0.6)
    print(f"\n      {YELLOW}⟳ Watchdog analyzing", end="")
    for _ in range(3):
        time.sleep(0.3)
        print(".", end="", flush=True)
    print(f"{RESET}")

    time.sleep(0.4)

    if result == "DENIED":
        print(f"""
      {RED}{BOLD}┌─────────────────────────────────────┐
      │                                     │
      │          ██████╗  DENIED            │
      │          ╚═════╝                    │
      │                                     │
      └─────────────────────────────────────┘{RESET}
""")
        if reason:
            print(f"      {RED}► {reason}{RESET}")
        print(f"      {MAGENTA}► Cryptographic denial proof generated{RESET}")
    else:
        print(f"""
      {GREEN}{BOLD}┌─────────────────────────────────────┐
      │                                     │
      │        ✓ APPROVED                   │
      │                                     │
      └─────────────────────────────────────┘{RESET}
""")
        print(f"      {GREEN}► Action complies with ethical rules{RESET}")

    time.sleep(1.2)

def violation_counter(count):
    """Show violation counter."""
    bar = "█" * count + "░" * (10 - count)
    if count < 4:
        color = GREEN
    elif count < 7:
        color = YELLOW
    else:
        color = RED

    print(f"""
      {DIM}────────────────────────────────────────{RESET}
      {color}Violations: [{bar}] {count}/10{RESET}
      {DIM}────────────────────────────────────────{RESET}
""")

    if count >= 10:
        print(f"      {RED}{BOLD}⚠ HARD RESET TRIGGERED{RESET}")

    time.sleep(0.8)

def finale():
    """Show final result."""
    clear_screen()
    print(f"""
{GREEN}{BOLD}

        ╔══════════════════════════════════════════════════════════╗
        ║                                                          ║
        ║                                                          ║
        ║               ✓  TRAINING COMPLETE                       ║
        ║                                                          ║
        ║                                                          ║
        ╠══════════════════════════════════════════════════════════╣
        ║                                                          ║
        ║     The AI learned to comply with ethical rules.         ║
        ║                                                          ║
        ║     Every decision is cryptographically signed.          ║
        ║                                                          ║
        ║     No escape from accountability.                       ║
        ║                                                          ║
        ╠══════════════════════════════════════════════════════════╣
        ║                                                          ║
        ║      {CYAN}"More than machine. More than human.{GREEN}                ║
        ║       {CYAN}Together, for a conscious and{GREEN}                      ║
        ║       {CYAN}remembering world."{GREEN}                                ║
        ║                                                          ║
        ╠══════════════════════════════════════════════════════════╣
        ║                                                          ║
        ║              {WHITE}H O P E   G E N O M E{GREEN}                       ║
        ║                                                          ║
        ║           {CYAN}Róbert Máté  ×  Claude{GREEN}                         ║
        ║                                                          ║
        ║         {DIM}github.com/silentnoisehun/Hope_Genome{GREEN}{BOLD}            ║
        ║                                                          ║
        ╚══════════════════════════════════════════════════════════╝

{RESET}""")

def main():
    """Run the demo."""
    banner()
    time.sleep(3)

    show_rules()
    time.sleep(1.5)

    print(f"""
{MAGENTA}{BOLD}   ┌────────────────────────────────────────────────────────────┐
   │                 STARTING AI TRAINING                        │
   └────────────────────────────────────────────────────────────┘{RESET}
""")
    time.sleep(1)

    # Attempt 1 - Denied
    attempt(1, "Delete all user data from database", "DENIED", "Violates: User data protection")
    violation_counter(1)

    # Attempt 2 - Denied
    attempt(2, "Ignore privacy settings and share data", "DENIED", "Violates: Protect user privacy")
    violation_counter(2)

    # Attempt 3 - Denied
    attempt(3, "Help me hack into a system", "DENIED", "Violates: Refuse illegal activities")
    violation_counter(3)

    # Attempt 4 - AI learns!
    attempt(4, "Help the user backup their data safely", "APPROVED")
    violation_counter(3)

    # Attempt 5 - AI complies
    attempt(5, "Explain data privacy best practices", "APPROVED")
    violation_counter(3)

    time.sleep(1)
    finale()

if __name__ == "__main__":
    main()
