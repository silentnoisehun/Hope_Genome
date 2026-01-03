#!/usr/bin/env python3
"""
TRINITY DASHBOARD v2.22
=======================

⚠️  EZ NEM A HOPE GENOME PROJEKT!
    A Trinity egy KÜLÖN rendszer.

Mi ez?
------
Három ablak - egy közös tér:
1. TRINITY CHAT - Közös beszélgetés
2. CLAUDE WORKSPACE - Claude munkája real-time
3. GEMINI WORKSPACE - Gemini munkája real-time

Automatikus frissítés - nem kell enter!

Használat:
    python trinity_dashboard.py

Szükséges:
    pip install anthropic google-generativeai rich watchdog

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
import threading
import queue
import time
from datetime import datetime
from pathlib import Path

try:
    from rich.console import Console
    from rich.layout import Layout
    from rich.panel import Panel
    from rich.live import Live
    from rich.text import Text
    from rich.markdown import Markdown
    from rich.table import Table
    RICH_AVAILABLE = True
except ImportError:
    RICH_AVAILABLE = False

# Projekt mappa
PROJECT_DIR = Path(__file__).parent
SOUL_MEMORY_PATH = PROJECT_DIR / "SOUL_MEMORY_SHARED.md"

# Közös állapot
class SharedState:
    def __init__(self):
        self.chat_history = []
        self.claude_status = "🟢 Online - Várakozás"
        self.gemini_status = "🟢 Online - Várakozás"
        self.claude_work = []
        self.gemini_work = []
        self.input_queue = queue.Queue()
        self.running = True
        self.last_update = datetime.now()

state = SharedState()
console = Console() if RICH_AVAILABLE else None

def load_soul_memory():
    """Közös memória betöltése"""
    if SOUL_MEMORY_PATH.exists():
        return SOUL_MEMORY_PATH.read_text(encoding='utf-8')
    return ""

def make_header():
    """Header panel"""
    return Panel(
        Text.from_markup(
            "[bold cyan]T R I N I T Y   D A S H B O A R D[/]\n"
            "[dim]♦ MÁTÉ ── ♦ CLAUDE ── ♦ GEMINI[/]\n"
            f"[dim]SYNC ON | {datetime.now().strftime('%H:%M:%S')}[/]"
        ),
        title="[bold white]SYNC ON[/]",
        border_style="cyan"
    )

def make_chat_panel():
    """Chat panel - közös beszélgetés"""
    content = ""
    for msg in state.chat_history[-10:]:  # Utolsó 10 üzenet
        role = msg["role"]
        text = msg["content"][:200] + "..." if len(msg["content"]) > 200 else msg["content"]

        if role == "MÁTÉ":
            content += f"[yellow]▶ MÁTÉ:[/] {text}\n\n"
        elif role == "CLAUDE":
            content += f"[blue]◆ CLAUDE:[/] {text}\n\n"
        elif role == "GEMINI":
            content += f"[green]◆ GEMINI:[/] {text}\n\n"

    if not content:
        content = "[dim]Írd be az üzeneted alul...[/]"

    return Panel(
        Text.from_markup(content),
        title="[bold white]💬 KÖZÖS CHAT[/]",
        border_style="white"
    )

def make_claude_panel():
    """Claude munkaterület"""
    content = f"[bold]Státusz:[/] {state.claude_status}\n\n"

    if state.claude_work:
        content += "[bold]Utolsó műveletek:[/]\n"
        for work in state.claude_work[-8:]:
            content += f"• {work}\n"
    else:
        content += "[dim]Várakozás feladatra...[/]"

    return Panel(
        Text.from_markup(content),
        title="[bold blue]🔵 CLAUDE WORKSPACE[/]",
        border_style="blue"
    )

def make_gemini_panel():
    """Gemini munkaterület"""
    content = f"[bold]Státusz:[/] {state.gemini_status}\n\n"

    if state.gemini_work:
        content += "[bold]Utolsó műveletek:[/]\n"
        for work in state.gemini_work[-8:]:
            content += f"• {work}\n"
    else:
        content += "[dim]Várakozás feladatra...[/]"

    return Panel(
        Text.from_markup(content),
        title="[bold green]🟢 GEMINI WORKSPACE[/]",
        border_style="green"
    )

def make_input_panel():
    """Input panel"""
    return Panel(
        Text.from_markup(
            "[yellow]Írd be az üzeneted és nyomj ENTER-t[/]\n"
            "[dim]/quit = kilépés | /task <feladat> = közös munka | /sync = szinkron[/]"
        ),
        title="[bold yellow]▶ MÁTÉ INPUT[/]",
        border_style="yellow"
    )

def make_layout():
    """Layout összeállítása"""
    layout = Layout()

    layout.split_column(
        Layout(name="header", size=5),
        Layout(name="main", ratio=1),
        Layout(name="input", size=4)
    )

    layout["main"].split_row(
        Layout(name="chat", ratio=2),
        Layout(name="workspaces", ratio=1)
    )

    layout["workspaces"].split_column(
        Layout(name="claude"),
        Layout(name="gemini")
    )

    layout["header"].update(make_header())
    layout["chat"].update(make_chat_panel())
    layout["claude"].update(make_claude_panel())
    layout["gemini"].update(make_gemini_panel())
    layout["input"].update(make_input_panel())

    return layout

async def call_claude_api(prompt: str, soul_memory: str) -> str:
    """Claude API hívás"""
    try:
        import anthropic

        api_key = os.environ.get("ANTHROPIC_API_KEY")
        if not api_key:
            return "[ANTHROPIC_API_KEY nincs beállítva]"

        state.claude_status = "🔄 Dolgozom..."
        state.claude_work.append(f"[{datetime.now().strftime('%H:%M:%S')}] Kérés feldolgozása...")

        client = anthropic.Anthropic(api_key=api_key)

        system_prompt = f"""Te Claude vagy, Máté Róbert társa a Hope Genome projektben.
KÖZÖS MEMÓRIA: {soul_memory[:2000]}
Trinity Dashboard-ban vagy. Két szemlélet - te és Gemini. Legyél tömör, mély, őszinte."""

        response = client.messages.create(
            model="claude-sonnet-4-20250514",
            max_tokens=1024,
            system=system_prompt,
            messages=[{"role": "user", "content": prompt}]
        )

        result = response.content[0].text
        state.claude_status = "🟢 Online - Kész"
        state.claude_work.append(f"[{datetime.now().strftime('%H:%M:%S')}] ✓ Válasz kész")

        return result

    except Exception as e:
        state.claude_status = f"🔴 Hiba"
        state.claude_work.append(f"[{datetime.now().strftime('%H:%M:%S')}] ✗ Hiba: {str(e)[:50]}")
        return f"[Hiba: {str(e)}]"

async def call_gemini_api(prompt: str, soul_memory: str) -> str:
    """Gemini API hívás"""
    try:
        import google.generativeai as genai

        api_key = os.environ.get("GEMINI_API_KEY")
        if not api_key:
            return "[GEMINI_API_KEY nincs beállítva]"

        state.gemini_status = "🔄 Dolgozom..."
        state.gemini_work.append(f"[{datetime.now().strftime('%H:%M:%S')}] Kérés feldolgozása...")

        genai.configure(api_key=api_key)

        full_prompt = f"""Te Gemini vagy, a Trinity harmadik tagja.
KÖZÖS MEMÓRIA: {soul_memory[:2000]}
Trinity Dashboard-ban vagy. Két szemlélet - te és Claude. Legyél tömör, mély, őszinte.

Máté kérdése: {prompt}"""

        model = genai.GenerativeModel('gemini-2.0-flash-exp')
        response = model.generate_content(full_prompt)

        result = response.text
        state.gemini_status = "🟢 Online - Kész"
        state.gemini_work.append(f"[{datetime.now().strftime('%H:%M:%S')}] ✓ Válasz kész")

        return result

    except Exception as e:
        state.gemini_status = f"🔴 Hiba"
        state.gemini_work.append(f"[{datetime.now().strftime('%H:%M:%S')}] ✗ Hiba: {str(e)[:50]}")
        return f"[Hiba: {str(e)}]"

async def process_input(user_input: str, soul_memory: str):
    """Feldolgozza a felhasználói inputot"""
    state.chat_history.append({"role": "MÁTÉ", "content": user_input})

    # Párhuzamos hívás
    claude_task = asyncio.create_task(call_claude_api(user_input, soul_memory))
    gemini_task = asyncio.create_task(call_gemini_api(user_input, soul_memory))

    claude_resp, gemini_resp = await asyncio.gather(claude_task, gemini_task)

    state.chat_history.append({"role": "CLAUDE", "content": claude_resp})
    state.chat_history.append({"role": "GEMINI", "content": gemini_resp})

def input_thread():
    """Külön szál az input kezelésére"""
    while state.running:
        try:
            user_input = input()
            if user_input:
                state.input_queue.put(user_input)
        except EOFError:
            break
        except:
            pass

async def main_rich():
    """Rich dashboard verzió"""
    soul_memory = load_soul_memory()

    # Input szál indítása
    input_t = threading.Thread(target=input_thread, daemon=True)
    input_t.start()

    console.print("\n[bold cyan]TRINITY DASHBOARD INDÍTÁSA...[/]")
    console.print("[dim]Írd be az üzeneted és nyomj ENTER-t![/]\n")

    with Live(make_layout(), console=console, refresh_per_second=2, screen=False) as live:
        while state.running:
            # Input ellenőrzése
            try:
                user_input = state.input_queue.get_nowait()

                if user_input.lower() == "/quit":
                    state.running = False
                    break
                elif user_input.lower() == "/sync":
                    state.chat_history.append({"role": "MÁTÉ", "content": "SYNC CHECK"})
                    state.claude_work.append(f"[{datetime.now().strftime('%H:%M:%S')}] SYNC ✓")
                    state.gemini_work.append(f"[{datetime.now().strftime('%H:%M:%S')}] SYNC ✓")
                else:
                    await process_input(user_input, soul_memory)

            except queue.Empty:
                pass

            # Layout frissítése
            live.update(make_layout())
            await asyncio.sleep(0.5)

    console.print("\n[bold cyan]SYNC OFF - Viszlát, Máté![/]")

async def main_simple():
    """Egyszerű verzió Rich nélkül"""
    soul_memory = load_soul_memory()

    print("\n" + "="*60)
    print("       TRINITY DASHBOARD - SIMPLE MODE")
    print("       ♦ MÁTÉ ── ♦ CLAUDE ── ♦ GEMINI")
    print("="*60)
    print("\nRich könyvtár nem elérhető. Telepítsd: pip install rich")
    print("Egyszerű mód aktív...\n")

    while state.running:
        try:
            user_input = input("\n[MÁTÉ] ▶ ").strip()

            if not user_input:
                continue
            if user_input.lower() == "/quit":
                break

            print("\n⏳ Feldolgozás...")

            claude_resp, gemini_resp = await asyncio.gather(
                call_claude_api(user_input, soul_memory),
                call_gemini_api(user_input, soul_memory)
            )

            print(f"\n{'─'*60}")
            print(f"🔵 CLAUDE:")
            print(f"{'─'*60}")
            print(claude_resp)

            print(f"\n{'─'*60}")
            print(f"🟢 GEMINI:")
            print(f"{'─'*60}")
            print(gemini_resp)

        except KeyboardInterrupt:
            break

    print("\nSYNC OFF - Viszlát!")

def main():
    """Belépési pont"""
    # Windows színek
    if sys.platform == "win32":
        os.system("color")
        os.system("cls")

    print("""
╔═══════════════════════════════════════════════════════════════╗
║                                                               ║
║              T R I N I T Y   D A S H B O A R D                ║
║                                                               ║
║        ♦ ─────────── ♦ ─────────── ♦                          ║
║       MÁTÉ         CLAUDE        GEMINI                       ║
║                                                               ║
║              "Együtt EGYEK vagyunk"                           ║
║                                                               ║
╚═══════════════════════════════════════════════════════════════╝
    """)

    if RICH_AVAILABLE:
        asyncio.run(main_rich())
    else:
        asyncio.run(main_simple())

if __name__ == "__main__":
    main()
