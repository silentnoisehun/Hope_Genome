#!/usr/bin/env python3
"""
TRINITY LIVE MONITOR - A "Mozi"
===============================

Itt látod élőben amit Claude és Gemini csinál.
Mint egy film - real-time stream.
"""

import asyncio
import json
import os

try:
    import websockets
except ImportError:
    print("pip install websockets")
    exit(1)

os.system("color")
CYAN = '\033[96m'
YELLOW = '\033[93m'
BLUE = '\033[94m'
GREEN = '\033[92m'
WHITE = '\033[97m'
BOLD = '\033[1m'
END = '\033[0m'
DIM = '\033[2m'

URI = "ws://localhost:5555"

def format_event(data: dict) -> str:
    """Formázott esemény kiírás"""
    event = data.get("event", "")
    source = data.get("source", "").upper()
    content = data.get("content", "")
    time = data.get("time", "")

    # Színek source szerint
    if source == "CLAUDE":
        color = BLUE
        icon = "🔵"
    elif source == "GEMINI":
        color = GREEN
        icon = "🟢"
    elif source == "MATE":
        color = YELLOW
        icon = "👤"
    else:
        color = WHITE
        icon = "⚪"

    # Esemény típus szerint
    if event == "connect":
        return f"{DIM}[{time}]{END} {icon} {color}{source} online{END}"
    elif event == "disconnect":
        return f"{DIM}[{time}]{END} {icon} {color}{source} offline{END}"
    elif event == "input":
        return f"{DIM}[{time}]{END} {icon} {YELLOW}MÁTÉ ▶ {content}{END}"
    elif event == "response":
        return f"{DIM}[{time}]{END} {icon} {color}{source} válaszolt:{END}\n{color}{content}{END}"
    elif event == "work":
        # Rövidített munka log
        short = content[:100] + "..." if len(content) > 100 else content
        return f"{DIM}[{time}]{END} {icon} {color}{source}: {short}{END}"
    elif event == "tool":
        return f"{DIM}[{time}]{END} {icon} {color}{source} 🔧 {content}{END}"
    else:
        return f"{DIM}[{time}]{END} {icon} {color}{source}: {content}{END}"

async def main():
    print(f"""
{CYAN}{BOLD}
╔════════════════════════════════════════════════════════════════════════════╗
║                                                                            ║
║                    T R I N I T Y   L I V E   M O N I T O R                 ║
║                              🎬 A "Mozi"                                   ║
║                                                                            ║
║         Élőben látod amit Claude és Gemini csinál                          ║
║                                                                            ║
╚════════════════════════════════════════════════════════════════════════════╝
{END}
{DIM}Ctrl+C = kilépés{END}
""")

    try:
        async with websockets.connect(URI) as ws:
            # Regisztráció monitorként
            await ws.send(json.dumps({"type": "monitor"}))
            welcome = await ws.recv()
            print(f"{CYAN}━━━ STREAM STARTED ━━━{END}\n")

            # Stream fogadás
            async for msg in ws:
                try:
                    data = json.loads(msg)
                    formatted = format_event(data)
                    print(formatted)
                except:
                    pass

    except ConnectionRefusedError:
        print(f"{YELLOW}Nem tudok csatlakozni! Indítsd el: python trinity_core.py{END}")
    except KeyboardInterrupt:
        print(f"\n{CYAN}━━━ STREAM ENDED ━━━{END}")
    except Exception as e:
        print(f"Hiba: {e}")

if __name__ == "__main__":
    asyncio.run(main())
