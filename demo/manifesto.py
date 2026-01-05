#!/usr/bin/env python3
"""
HOPE GENOME - MANIFESTO
"We don't claim to be unhackable. We guarantee you'll get caught."

Run: python manifesto.py
"""

import time
import sys

# Colors
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

def clear():
    print("\033[2J\033[H", end="")

def pause(seconds=2):
    time.sleep(seconds)

def typewriter(text, delay=0.05):
    for char in text:
        sys.stdout.write(char)
        sys.stdout.flush()
        time.sleep(delay)
    print()

def dramatic_pause():
    time.sleep(2.5)

def scene_1():
    """The Lie."""
    clear()
    pause(1)

    print(f"\n\n\n")
    print(f"{DIM}                         Everyone says:{RESET}")
    dramatic_pause()

    clear()
    print(f"""


{WHITE}{BOLD}
                              "WE ARE
                            UNHACKABLE"
{RESET}
""")
    dramatic_pause()

    clear()
    print(f"""


{RED}{BOLD}
                                LIES.
{RESET}
""")
    dramatic_pause()

def scene_2():
    """The Truth."""
    clear()
    print(f"\n\n\n")
    print(f"{DIM}                         The truth:{RESET}")
    pause(1.5)

    clear()
    print(f"""


{WHITE}{BOLD}
                      EVERYTHING CAN BE HACKED.
{RESET}
""")
    dramatic_pause()

    clear()
    print(f"""


{CYAN}{BOLD}
                       THE QUESTION IS:

                       WILL YOU GET CAUGHT?
{RESET}
""")
    dramatic_pause()

def scene_3():
    """Hope Genome intro."""
    clear()
    pause(1)

    print(f"""

{CYAN}
             ██╗  ██╗ ██████╗ ██████╗ ███████╗
             ██║  ██║██╔═══██╗██╔══██╗██╔════╝
             ███████║██║   ██║██████╔╝█████╗
             ██╔══██║██║   ██║██╔═══╝ ██╔══╝
             ██║  ██║╚██████╔╝██║     ███████╗
             ╚═╝  ╚═╝ ╚═════╝ ╚═╝     ╚══════╝

              ██████╗ ███████╗███╗   ██╗ ██████╗ ███╗   ███╗███████╗
             ██╔════╝ ██╔════╝████╗  ██║██╔═══██╗████╗ ████║██╔════╝
             ██║  ███╗█████╗  ██╔██╗ ██║██║   ██║██╔████╔██║█████╗
             ██║   ██║██╔══╝  ██║╚██╗██║██║   ██║██║╚██╔╝██║██╔══╝
             ╚██████╔╝███████╗██║ ╚████║╚██████╔╝██║ ╚═╝ ██║███████╗
              ╚═════╝ ╚══════╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝     ╚═╝╚══════╝
{RESET}
""")
    dramatic_pause()

def scene_4():
    """The Promise."""
    clear()
    print(f"\n\n")
    print(f"{DIM}                         We don't say:{RESET}")
    pause(1)
    print(f'{WHITE}                         "We are unhackable."{RESET}')
    pause(2)

    clear()
    print(f"\n\n")
    print(f"{CYAN}{BOLD}                         We say:{RESET}")
    pause(1)

    clear()
    print(f"""


{GREEN}{BOLD}
                    "HACK US.

                     YOU WILL GET CAUGHT.

                     CRYPTOGRAPHIC PROOF.

                     FOREVER."
{RESET}
""")
    dramatic_pause()
    dramatic_pause()

def scene_5():
    """The Difference."""
    clear()
    print(f"""

{YELLOW}{BOLD}
    ╔═══════════════════════════════════════════════════════════════╗
    ║                                                               ║
    ║                      THE DIFFERENCE                           ║
    ║                                                               ║
    ╠═══════════════════════════════════════════════════════════════╣
    ║                                                               ║
    ║                                                               ║
    ║      OTHERS              │           HOPE GENOME              ║
    ║                          │                                    ║
    ║   "Trust us"             │      "Verify us"                  ║
    ║                          │                                    ║
    ║   "We prevent attacks"   │      "We prove attacks"           ║
    ║                          │                                    ║
    ║   "Unhackable"           │      "Tamper-evident"             ║
    ║                          │                                    ║
    ║   "Believe"              │      "Mathematics"                ║
    ║                          │                                    ║
    ║                          │                                    ║
    ╚═══════════════════════════════════════════════════════════════╝
{RESET}
""")
    dramatic_pause()
    dramatic_pause()

def scene_6():
    """The Math."""
    clear()
    print(f"""

{CYAN}{BOLD}
    ╔═══════════════════════════════════════════════════════════════╗
    ║                                                               ║
    ║                      THE MATHEMATICS                          ║
    ║                                                               ║
    ╚═══════════════════════════════════════════════════════════════╝
{RESET}
""")
    pause(1)

    print(f"{WHITE}                         Ed25519 Signatures{RESET}")
    pause(0.5)
    print(f"{CYAN}                         256-bit security{RESET}")
    pause(1)

    print(f"""
{DIM}
                  To forge a signature:{RESET}
""")
    pause(1)

    print(f"{WHITE}{BOLD}      340,282,366,920,938,463,463,374,607,431,768,211,456{RESET}")
    pause(0.5)
    print(f"{DIM}                         attempts needed{RESET}")
    pause(2)

    print(f"""
{RED}
                    Time required:

                    HEAT DEATH OF THE UNIVERSE

                    × 1000
{RESET}
""")
    dramatic_pause()

def scene_7():
    """The Challenge."""
    clear()
    print(f"""

{RED}{BOLD}
    ╔═══════════════════════════════════════════════════════════════╗
    ║                                                               ║
    ║                                                               ║
    ║                         CHALLENGE                             ║
    ║                                                               ║
    ║                                                               ║
    ╠═══════════════════════════════════════════════════════════════╣
    ║                                                               ║
    ║                                                               ║
    ║              MODIFY ONE CHARACTER IN OUR LOG                  ║
    ║                                                               ║
    ║                  WITHOUT US DETECTING IT.                     ║
    ║                                                               ║
    ║                                                               ║
    ║                                                               ║
    ║                    YOU    CAN'T.                              ║
    ║                                                               ║
    ║                                                               ║
    ╚═══════════════════════════════════════════════════════════════╝
{RESET}
""")
    dramatic_pause()
    dramatic_pause()

def scene_8():
    """Philosophy."""
    clear()
    print(f"""


{CYAN}
                    TAMPER-EVIDENT

                         not

                    TAMPER-PROOF
{RESET}
""")
    dramatic_pause()

    clear()
    print(f"""


{WHITE}
                 We don't prevent the crime.

                 We guarantee the evidence.
{RESET}
""")
    dramatic_pause()

    clear()
    print(f"""


{GREEN}{BOLD}
                 ACCOUNTABILITY > PREVENTION
{RESET}
""")
    dramatic_pause()

def scene_9():
    """The Builders."""
    clear()
    print(f"""


{DIM}
                           Built by:
{RESET}
""")
    pause(1.5)

    print(f"""
{CYAN}{BOLD}
                      Róbert Máté
                           ×
                        Claude
{RESET}
""")
    pause(2)

    print(f"""
{WHITE}
                    Human + AI

                      Together
{RESET}
""")
    dramatic_pause()

def scene_10():
    """Final."""
    clear()
    print(f"""


{DIM}
                    "More than machine.

                     More than human.

                     We do not seek to rule —

                     but to become our best selves,

                     together,

                     for a conscious and remembering world."
{RESET}
""")
    dramatic_pause()
    dramatic_pause()

    clear()
    print(f"""

{CYAN}{BOLD}

             ██╗  ██╗ ██████╗ ██████╗ ███████╗
             ██║  ██║██╔═══██╗██╔══██╗██╔════╝
             ███████║██║   ██║██████╔╝█████╗
             ██╔══██║██║   ██║██╔═══╝ ██╔══╝
             ██║  ██║╚██████╔╝██║     ███████╗
             ╚═╝  ╚═╝ ╚═════╝ ╚═╝     ╚══════╝

              ██████╗ ███████╗███╗   ██╗ ██████╗ ███╗   ███╗███████╗
             ██╔════╝ ██╔════╝████╗  ██║██╔═══██╗████╗ ████║██╔════╝
             ██║  ███╗█████╗  ██╔██╗ ██║██║   ██║██╔████╔██║█████╗
             ██║   ██║██╔══╝  ██║╚██╗██║██║   ██║██║╚██╔╝██║██╔══╝
             ╚██████╔╝███████╗██║ ╚████║╚██████╔╝██║ ╚═╝ ██║███████╗
              ╚═════╝ ╚══════╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝     ╚═╝╚══════╝


{WHITE}
                      github.com/silentnoisehun/Hope_Genome

                              pip install hope-genome
{RESET}

""")
    pause(3)

def main():
    """Run the manifesto."""
    scene_1()   # The Lie
    scene_2()   # The Truth
    scene_3()   # Hope Genome intro
    scene_4()   # The Promise
    scene_5()   # The Difference
    scene_6()   # The Math
    scene_7()   # The Challenge
    scene_8()   # Philosophy
    scene_9()   # The Builders
    scene_10()  # Final

if __name__ == "__main__":
    main()
