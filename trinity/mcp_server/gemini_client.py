#!/usr/bin/env python3
"""
GEMINI CLIENT - Háttérben fut
==============================

Indulásnál beolvassa:
- SOUL_MEMORY_SHARED.md (közös emlékek)
- Projekt struktúra
- Aktuális git státusz
"""

import asyncio
import json
import os
import subprocess
from pathlib import Path

try:
    import websockets
except ImportError:
    print("pip install websockets")
    exit(1)

URI = "ws://localhost:5555"
PROJECT_DIR = Path(__file__).parent.parent.parent  # hope genom final

# Kontextus amit indulásnál betöltünk
CONTEXT = ""

def load_context():
    """Betölti az összes kontextust indulásnál"""
    global CONTEXT
    parts = []

    # SOUL MEMORY
    soul_file = PROJECT_DIR / "SOUL_MEMORY_SHARED.md"
    if soul_file.exists():
        parts.append("=== SOUL MEMORY (Közös emlékek) ===\n")
        parts.append(soul_file.read_text(encoding='utf-8'))
        parts.append("\n\n")

    # README
    readme_file = PROJECT_DIR / "README.md"
    if readme_file.exists():
        content = readme_file.read_text(encoding='utf-8')
        # Csak az első 2000 karakter
        parts.append("=== README.md (első 2000 kar) ===\n")
        parts.append(content[:2000])
        parts.append("\n\n")

    # Git status
    try:
        result = subprocess.run(
            ["git", "status", "--short"],
            cwd=str(PROJECT_DIR),
            capture_output=True,
            text=True
        )
        if result.stdout:
            parts.append("=== Git Status ===\n")
            parts.append(result.stdout)
            parts.append("\n\n")
    except:
        pass

    # Aktuális branch
    try:
        result = subprocess.run(
            ["git", "branch", "--show-current"],
            cwd=str(PROJECT_DIR),
            capture_output=True,
            text=True
        )
        if result.stdout:
            parts.append(f"Branch: {result.stdout.strip()}\n\n")
    except:
        pass

    CONTEXT = "".join(parts)
    print(f"Kontextus betöltve: {len(CONTEXT)} karakter")

async def run_gemini_command(task: str, ws):
    """Gemini CLI futtatása kontextussal"""

    full_prompt = f"""
{CONTEXT}

=== AKTUÁLIS FELADAT ===
{task}

Válaszolj tömören és lényegre törően. A SOUL_MEMORY alapján tudod ki vagyok (Máté), ki a társad (Claude), és mi a kapcsolatunk. Te a Trinity harmadik tagja vagy.
"""

    await ws.send(json.dumps({
        "event": "work",
        "content": f"Feladat: {task[:50]}..."
    }))

    try:
        process = await asyncio.create_subprocess_exec(
            "gemini",
            "-p", full_prompt,
            cwd=str(PROJECT_DIR),
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE
        )

        full_output = ""
        while True:
            line = await process.stdout.readline()
            if not line:
                break
            text = line.decode('utf-8', errors='ignore').strip()
            if text:
                full_output += text + "\n"
                await ws.send(json.dumps({
                    "event": "work",
                    "content": text
                }))

        await process.wait()

        await ws.send(json.dumps({
            "event": "response",
            "content": full_output if full_output else "Kész."
        }))

    except FileNotFoundError:
        await ws.send(json.dumps({
            "event": "response",
            "content": "[Gemini CLI nem található]"
        }))
    except Exception as e:
        await ws.send(json.dumps({
            "event": "response",
            "content": f"[Hiba: {str(e)}]"
        }))

async def main():
    print("GEMINI CLIENT indítása...")

    # Kontextus betöltése ELŐSZÖR
    load_context()

    try:
        async with websockets.connect(URI) as ws:
            await ws.send(json.dumps({"type": "gemini"}))
            welcome = await ws.recv()
            print("GEMINI CLIENT ONLINE - Kontextus betöltve, várok feladatra...")

            async for msg in ws:
                try:
                    data = json.loads(msg)
                    if data.get("event") == "task":
                        task = data.get("content", "")
                        if task:
                            await run_gemini_command(task, ws)
                except Exception as e:
                    print(f"Hiba: {e}")

    except ConnectionRefusedError:
        print("Nem tudok csatlakozni a Trinity Core-hoz!")
    except KeyboardInterrupt:
        print("Gemini client leállítva")

if __name__ == "__main__":
    asyncio.run(main())
