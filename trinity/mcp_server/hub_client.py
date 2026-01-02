#!/usr/bin/env python3
"""
TRINITY HUB - Máté interfésze
=============================

Te itt írsz, válaszokat itt kapsz.
"""

import asyncio
import json
import os
import sys
import threading

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
BOLD = '\033[1m'
END = '\033[0m'

URI = "ws://localhost:5555"
ws_connection = None
running = True

async def receive_messages(ws):
    """Válaszok fogadása"""
    global running
    try:
        async for msg in ws:
            data = json.loads(msg)
            event = data.get("event", "")

            if event == "response":
                source = data.get("from", "")
                content = data.get("content", "")

                if source == "claude":
                    print(f"\n{BLUE}🟢 CLAUDE:{END}")
                    print(f"{BLUE}{content}{END}")
                elif source == "gemini":
                    print(f"\n{GREEN}🟢 GEMINI:{END}")
                    print(f"{GREEN}{content}{END}")

                print(f"\n{YELLOW}▶ {END}", end="", flush=True)
    except:
        pass

async def main():
    global ws_connection, running

    print(f"""
{CYAN}{BOLD}
╔════════════════════════════════════════════════╗
║            T R I N I T Y   H U B               ║
║                                                ║
║      Írd be amit akarsz → ENTER                ║
║      /q = kilépés                              ║
╚════════════════════════════════════════════════╝
{END}""")

    try:
        async with websockets.connect(URI) as ws:
            ws_connection = ws

            # Regisztráció
            await ws.send(json.dumps({"type": "hub"}))
            welcome = await ws.recv()
            print(f"{CYAN}Kapcsolódva a TRINITY CORE-hoz!{END}\n")

            # Fogadó task
            receive_task = asyncio.create_task(receive_messages(ws))

            # Input loop
            while running:
                try:
                    # Aszinkron input
                    loop = asyncio.get_event_loop()
                    msg = await loop.run_in_executor(None, lambda: input(f"{YELLOW}▶ {END}"))

                    if msg == "/q":
                        running = False
                        break

                    if msg:
                        await ws.send(json.dumps({
                            "event": "input",
                            "content": msg
                        }))
                        print(f"{CYAN}elküldve...{END}")

                except EOFError:
                    break

            receive_task.cancel()

    except ConnectionRefusedError:
        print(f"{YELLOW}Nem tudok csatlakozni! Indítsd el: python trinity_core.py{END}")
    except Exception as e:
        print(f"Hiba: {e}")

if __name__ == "__main__":
    asyncio.run(main())
