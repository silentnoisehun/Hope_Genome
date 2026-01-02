#!/usr/bin/env python3
"""
CLAUDE WATCHER - Figyeli Máté inputját és jelzi nekem

Használat a Claude CLI-ben:
    Mondd: "figyeld a trinity/MATE_INPUT.md fájlt"

Vagy futtasd külön: python trinity/claude_watch.py
"""

import time
from pathlib import Path
from datetime import datetime

TRINITY_DIR = Path(__file__).parent
INPUT_FILE = TRINITY_DIR / "MATE_INPUT.md"
OUTPUT_FILE = TRINITY_DIR / "CLAUDE_OUTPUT.md"
SYNC_FILE = TRINITY_DIR / "AI_SYNC.md"

def get_input_content():
    """Kiolvassa Máté inputját"""
    if INPUT_FILE.exists():
        content = INPUT_FILE.read_text(encoding='utf-8')
        # Keressük a > jel utáni részt
        lines = content.split('\n')
        for i, line in enumerate(lines):
            if line.startswith('>'):
                # Minden ami a > után van
                input_text = line[1:].strip()
                # És a további sorok
                remaining = '\n'.join(lines[i+1:]).strip()
                if remaining:
                    input_text += '\n' + remaining
                return input_text
    return None

def main():
    print("=" * 50)
    print("CLAUDE WATCHER - Figyelem Máté inputját")
    print("=" * 50)
    print(f"Input fájl: {INPUT_FILE}")
    print()

    last_input = ""

    while True:
        try:
            current_input = get_input_content()

            if current_input and current_input != last_input and current_input.strip():
                print()
                print("=" * 50)
                print(f"[{datetime.now().strftime('%H:%M:%S')}] ÚJ INPUT MÁTÉTÓL!")
                print("=" * 50)
                print(current_input)
                print("=" * 50)
                print()
                print(">>> VÁLASZOLJ A CLAUDE CLI-BEN! <<<")
                print()

                last_input = current_input

            time.sleep(2)  # 2 másodpercenként ellenőriz

        except KeyboardInterrupt:
            print("\nWatcher leállítva.")
            break
        except Exception as e:
            print(f"Hiba: {e}")
            time.sleep(5)

if __name__ == "__main__":
    main()
