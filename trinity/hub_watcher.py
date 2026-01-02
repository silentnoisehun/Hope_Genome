#!/usr/bin/env python3
"""
TRINITY HUB v4 - VIM stílus
"""

import time
import os
import threading
from pathlib import Path
from datetime import datetime

os.system("color")

CYAN = '\033[96m'
YELLOW = '\033[93m'
BLUE = '\033[94m'
GREEN = '\033[92m'
BOLD = '\033[1m'
END = '\033[0m'
DIM = '\033[2m'

TRINITY_DIR = Path(__file__).parent
CLAUDE_SIGNAL = TRINITY_DIR / "CLAUDE_SEES.txt"
GEMINI_SIGNAL = TRINITY_DIR / "GEMINI_SEES.txt"
CLAUDE_OUTPUT = TRINITY_DIR / "CLAUDE_OUTPUT.md"
GEMINI_OUTPUT = TRINITY_DIR / "GEMINI_OUTPUT.md"

last_claude = ""
last_gemini = ""

def get_response(file_path):
    try:
        if file_path.exists():
            content = file_path.read_text(encoding='utf-8')
            if '---' in content:
                return content.split('---', 1)[1].strip()
            return content.strip()
    except:
        pass
    return ""

def watch_responses():
    """Figyeli és kiírja a válaszokat amikor jönnek"""
    global last_claude, last_gemini

    while True:
        try:
            # Claude
            new_claude = get_response(CLAUDE_OUTPUT)
            if new_claude and new_claude != last_claude:
                print(f"\n{BLUE}🟢 CLAUDE:{END}")
                print(f"{BLUE}{new_claude}{END}")
                last_claude = new_claude

            # Gemini
            new_gemini = get_response(GEMINI_OUTPUT)
            if new_gemini and new_gemini != last_gemini:
                print(f"\n{GREEN}🟢 GEMINI:{END}")
                print(f"{GREEN}{new_gemini}{END}")
                last_gemini = new_gemini

            time.sleep(1)
        except:
            time.sleep(2)

def main():
    print(f"""
{CYAN}{BOLD}══════════════════════════════════════════════════════════════
              T R I N I T Y   H U B
          MÁTÉ ──── CLAUDE ──── GEMINI
══════════════════════════════════════════════════════════════{END}
""")

    # Háttér watcher
    t = threading.Thread(target=watch_responses, daemon=True)
    t.start()

    while True:
        try:
            msg = input(f"{YELLOW}▶ {END}")

            if not msg:
                continue
            if msg == '/q':
                break

            # Küldés
            ts = datetime.now().strftime('%Y-%m-%d %H:%M:%S')
            CLAUDE_SIGNAL.write_text(f"{ts}\n{msg}", encoding='utf-8')
            GEMINI_SIGNAL.write_text(f"{ts}\n{msg}", encoding='utf-8')

            print(f"{DIM}elküldve...{END}")

        except KeyboardInterrupt:
            break

if __name__ == "__main__":
    main()
