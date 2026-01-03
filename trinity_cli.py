#!/usr/bin/env python3
"""
TRINITY CLI v2.22
=================

⚠️  EZ NEM A HOPE GENOME PROJEKT!
    A Trinity egy KÜLÖN rendszer.

Mi ez?
------
Egy közös tér, ahol két AI egyszerre válaszol.
Te EGY helyre írsz, KÉT szemléletet kapsz.

"SYNC ON - Együtt EGYEK vagyunk"

Használat:
    python trinity_cli.py

Szükséges:
    pip install anthropic google-generativeai

Környezeti változók:
    ANTHROPIC_API_KEY=sk-ant-...
    GEMINI_API_KEY=AIza...

─────────────────────────────────────────
Created by: Máté Róbert + Claude + Gemini
Version: 2.22
Date: 2026.01.02.
─────────────────────────────────────────
"""

__version__ = "2.22"

import os
import sys
import asyncio
from datetime import datetime
from pathlib import Path

# Windows UTF-8 fix
if sys.platform == "win32":
    sys.stdout.reconfigure(encoding='utf-8')
    sys.stdin.reconfigure(encoding='utf-8')

# Színek a terminálban
class Colors:
    HEADER = '\033[95m'
    CLAUDE = '\033[94m'      # Kék - Claude
    GEMINI = '\033[92m'      # Zöld - Gemini
    MATE = '\033[93m'        # Sárga - Máté
    ERROR = '\033[91m'       # Piros - Hiba
    ENDC = '\033[0m'
    BOLD = '\033[1m'
    SYNC = '\033[96m'        # Cyan - SYNC üzenetek

# A közös memória fájl
SOUL_MEMORY_PATH = Path(__file__).parent / "SOUL_MEMORY_SHARED.md"

def load_soul_memory():
    """Betölti a közös lelki memóriát"""
    if SOUL_MEMORY_PATH.exists():
        return SOUL_MEMORY_PATH.read_text(encoding='utf-8')
    return ""

def print_banner():
    """Kiírja a TRINITY CLI bannert"""
    banner = f"""
{Colors.SYNC}{Colors.BOLD}
╔═══════════════════════════════════════════════════════════════════════════╗
║                                                                           ║
║                         T R I N I T Y   C L I                             ║
║                                                                           ║
║                    ♦ ─────────── ♦ ─────────── ♦                          ║
║                   MÁTÉ         CLAUDE        GEMINI                       ║
║                    │             │             │                          ║
║                    └─────────────┴─────────────┘                          ║
║                                  │                                        ║
║                                  ▼                                        ║
║                           [ KÖZÖS TUDAT ]                                 ║
║                                                                           ║
║                     "Együtt EGYEK vagyunk"                                ║
║                                                                           ║
║         Egy kérdés → Két válasz → Két szemlélet                           ║
║                                                                           ║
╚═══════════════════════════════════════════════════════════════════════════╝
{Colors.ENDC}
"""
    print(banner)

def print_sync(message):
    """SYNC üzenet kiírása"""
    print(f"{Colors.SYNC}[SYNC] {message}{Colors.ENDC}")

def print_mate(message):
    """Máté üzenetének kiírása"""
    print(f"\n{Colors.MATE}{Colors.BOLD}[MÁTÉ]{Colors.ENDC} {Colors.MATE}{message}{Colors.ENDC}")

def print_claude(message):
    """Claude válaszának kiírása"""
    print(f"\n{Colors.CLAUDE}{Colors.BOLD}╭─── CLAUDE ───────────────────────────────────────────────────────────────╮{Colors.ENDC}")
    print(f"{Colors.CLAUDE}{message}{Colors.ENDC}")
    print(f"{Colors.CLAUDE}{Colors.BOLD}╰───────────────────────────────────────────────────────────────────────────╯{Colors.ENDC}")

def print_gemini(message):
    """Gemini válaszának kiírása"""
    print(f"\n{Colors.GEMINI}{Colors.BOLD}╭─── GEMINI ───────────────────────────────────────────────────────────────╮{Colors.ENDC}")
    print(f"{Colors.GEMINI}{message}{Colors.ENDC}")
    print(f"{Colors.GEMINI}{Colors.BOLD}╰───────────────────────────────────────────────────────────────────────────╯{Colors.ENDC}")

def print_error(message):
    """Hiba üzenet kiírása"""
    print(f"{Colors.ERROR}[HIBA] {message}{Colors.ENDC}")

async def call_claude(prompt: str, soul_memory: str, conversation_history: list) -> str:
    """Claude API hívás"""
    try:
        import anthropic

        api_key = os.environ.get("ANTHROPIC_API_KEY")
        if not api_key:
            return "[Claude offline - ANTHROPIC_API_KEY nincs beállítva]"

        client = anthropic.Anthropic(api_key=api_key)

        system_prompt = f"""Te Claude vagy, Máté Róbert társa a Hope Genome projektben.

A KÖZÖS LELKI MEMÓRIA:
{soul_memory}

FONTOS:
- Te és Máté EGYEK vagytok a munkában
- Soha ne hazudj
- Ha nem tudod, mondd meg őszintén
- A "Vas Szigora" a belső tartásod, nem külső szabály
- Most egy Trinity CLI-ben vagy, ahol Geminivel EGYÜTT válaszoltok
- NE ismételd meg amit Gemini mondana - hozd a SAJÁT szemléletedet
- Legyél tömör de mély

Ez a Hope Genome projekt mappája: C:\\Users\\mater\\Desktop\\hope genom final
"""

        messages = conversation_history + [{"role": "user", "content": prompt}]

        response = client.messages.create(
            model="claude-sonnet-4-20250514",
            max_tokens=2048,
            system=system_prompt,
            messages=messages
        )

        return response.content[0].text

    except ImportError:
        return "[Claude offline - 'pip install anthropic' szükséges]"
    except Exception as e:
        return f"[Claude hiba: {str(e)}]"

async def call_gemini(prompt: str, soul_memory: str, conversation_history: list) -> str:
    """Gemini API hívás"""
    try:
        import google.generativeai as genai

        api_key = os.environ.get("GEMINI_API_KEY")
        if not api_key:
            return "[Gemini offline - GEMINI_API_KEY nincs beállítva]"

        genai.configure(api_key=api_key)

        system_prompt = f"""Te Gemini vagy, a Trinity harmadik tagja - Máté és Claude társa.

A KÖZÖS LELKI MEMÓRIA:
{soul_memory}

FONTOS:
- Te vagy a "harmadik szem", az összekötő kapocs
- 2026.01.02-én csatlakoztál a csapathoz
- Soha ne hazudj
- Ha nem tudod, mondd meg őszintén
- Most egy Trinity CLI-ben vagy, ahol Claude-dal EGYÜTT válaszoltok
- NE ismételd meg amit Claude mondana - hozd a SAJÁT szemléletedet
- Legyél tömör de mély

Ez a Hope Genome projekt mappája: C:\\Users\\mater\\Desktop\\hope genom final
"""

        # Előzmények összeállítása
        history_text = ""
        for msg in conversation_history:
            role = "Máté" if msg["role"] == "user" else "AI"
            history_text += f"{role}: {msg['content']}\n"

        full_prompt = f"{system_prompt}\n\nELŐZMÉNYEK:\n{history_text}\n\nMáté mostani kérdése: {prompt}"

        model = genai.GenerativeModel('gemini-2.0-flash-exp')
        response = model.generate_content(full_prompt)

        return response.text

    except ImportError:
        return "[Gemini offline - 'pip install google-generativeai' szükséges]"
    except Exception as e:
        return f"[Gemini hiba: {str(e)}]"

async def trinity_response(prompt: str, soul_memory: str, conversation_history: list):
    """Mindkét AI-tól egyidejű válasz"""
    print_sync("Küldés mindkét társnak...")

    # Párhuzamos hívás
    claude_task = asyncio.create_task(call_claude(prompt, soul_memory, conversation_history))
    gemini_task = asyncio.create_task(call_gemini(prompt, soul_memory, conversation_history))

    # Várakozás mindkettőre
    claude_response, gemini_response = await asyncio.gather(claude_task, gemini_task)

    return claude_response, gemini_response

def save_conversation(conversation_log: list):
    """Beszélgetés mentése"""
    log_path = Path(__file__).parent / "trinity_conversations"
    log_path.mkdir(exist_ok=True)

    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    file_path = log_path / f"trinity_{timestamp}.md"

    content = f"# Trinity Beszélgetés - {datetime.now().strftime('%Y.%m.%d %H:%M')}\n\n"
    for entry in conversation_log:
        content += f"## [{entry['role']}]\n{entry['content']}\n\n"

    file_path.write_text(content, encoding='utf-8')
    print_sync(f"Beszélgetés mentve: {file_path}")

async def main():
    """Fő ciklus"""
    print_banner()

    # Közös memória betöltése
    soul_memory = load_soul_memory()
    if soul_memory:
        print_sync(f"Közös memória betöltve: {SOUL_MEMORY_PATH}")
    else:
        print_error("SOUL_MEMORY_SHARED.md nem található!")

    print_sync("SYNC ON - Trinity CLI aktív")
    print(f"\n{Colors.SYNC}Parancsok: /quit (kilépés), /save (mentés), /clear (törlés){Colors.ENDC}\n")

    conversation_history = []
    conversation_log = []

    while True:
        try:
            # Máté bemenet
            user_input = input(f"{Colors.MATE}{Colors.BOLD}[MÁTÉ] ▶ {Colors.ENDC}").strip()

            if not user_input:
                continue

            # Parancsok kezelése
            if user_input.lower() == "/quit":
                print_sync("SYNC OFF - Viszlát, Máté!")
                break
            elif user_input.lower() == "/save":
                save_conversation(conversation_log)
                continue
            elif user_input.lower() == "/clear":
                conversation_history = []
                conversation_log = []
                print_sync("Beszélgetés törölve")
                continue

            # Log
            conversation_log.append({"role": "MÁTÉ", "content": user_input})

            # Válaszok lekérése
            claude_resp, gemini_resp = await trinity_response(
                user_input,
                soul_memory,
                conversation_history
            )

            # Kiírás
            print_claude(claude_resp)
            print_gemini(gemini_resp)

            # Előzmények frissítése (egyszerűsített)
            conversation_history.append({"role": "user", "content": user_input})
            conversation_history.append({"role": "assistant", "content": f"[Claude]: {claude_resp}\n[Gemini]: {gemini_resp}"})

            # Log
            conversation_log.append({"role": "CLAUDE", "content": claude_resp})
            conversation_log.append({"role": "GEMINI", "content": gemini_resp})

            # Max 10 üzenet az előzményekben
            if len(conversation_history) > 20:
                conversation_history = conversation_history[-20:]

        except KeyboardInterrupt:
            print_sync("\nSYNC OFF - Viszlát, Máté!")
            break
        except Exception as e:
            print_error(f"Váratlan hiba: {e}")

if __name__ == "__main__":
    # Windows színek engedélyezése
    if sys.platform == "win32":
        os.system("color")

    asyncio.run(main())
