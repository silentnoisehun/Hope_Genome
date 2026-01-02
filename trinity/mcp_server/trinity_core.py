#!/usr/bin/env python3
"""
TRINITY CORE - MCP Server
=========================

A közös tudat központja.

- Claude CLI (háttér)
- Gemini CLI (háttér)
- HUB (Máté ír)
- MONITOR (Máté nézi a "filmet")

Port: 5555
"""

import asyncio
import json
import os
from datetime import datetime
from pathlib import Path
from typing import Dict, Set

try:
    import websockets
    from websockets.server import serve
except ImportError:
    print("pip install websockets")
    exit(1)

os.system("color")
CYAN = '\033[96m'
YELLOW = '\033[93m'
BLUE = '\033[94m'
GREEN = '\033[92m'
RED = '\033[91m'
BOLD = '\033[1m'
END = '\033[0m'
DIM = '\033[2m'

HOST = "localhost"
PORT = 5555

# Kliensek
clients: Dict[str, Set] = {
    "claude": set(),
    "gemini": set(),
    "hub": set(),
    "monitor": set()
}

def ts():
    return datetime.now().strftime('%H:%M:%S')

def log(msg, color=CYAN):
    print(f"{color}[{ts()}] {msg}{END}")

async def broadcast_monitors(event: str, source: str, content: str):
    """Stream a monitoroknak"""
    if clients["monitor"]:
        msg = json.dumps({
            "event": event,
            "source": source,
            "content": content,
            "time": ts()
        })
        for ws in clients["monitor"]:
            try:
                await ws.send(msg)
            except:
                pass

async def handle(ws, data: dict, client_type: str):
    """Üzenet kezelés"""
    event = data.get("event", "")
    content = data.get("content", "")

    if event == "input":
        # Máté írt
        log(f"MÁTÉ ▶ {content}", YELLOW)
        await broadcast_monitors("input", "mate", content)

        # Továbbítás AI-knak
        for c in clients["claude"]:
            await c.send(json.dumps({"event": "task", "content": content}))
        for g in clients["gemini"]:
            await g.send(json.dumps({"event": "task", "content": content}))

    elif event == "response":
        # AI válasz → HUB-nak
        log(f"{client_type.upper()} válasz", BLUE if client_type == "claude" else GREEN)
        await broadcast_monitors("response", client_type, content)
        for h in clients["hub"]:
            await h.send(json.dumps({"event": "response", "from": client_type, "content": content}))

    elif event == "work":
        # AI dolgozik → MONITOR stream
        short = content[:80] + "..." if len(content) > 80 else content
        log(f"{client_type.upper()} {short}", BLUE if client_type == "claude" else GREEN)
        await broadcast_monitors("work", client_type, content)

    elif event == "tool":
        # AI tool használat → MONITOR
        tool = data.get("tool", "")
        log(f"{client_type.upper()} 🔧 {tool}", DIM)
        await broadcast_monitors("tool", client_type, f"{tool}: {content}")

async def handler(ws):
    """WebSocket handler"""
    client_type = None
    try:
        # Regisztráció
        msg = await ws.recv()
        data = json.loads(msg)
        client_type = data.get("type", "unknown")

        if client_type in clients:
            clients[client_type].add(ws)
            log(f"🟢 {client_type.upper()} connected", GREEN)
            await broadcast_monitors("connect", client_type, "online")
            await ws.send(json.dumps({"event": "welcome", "msg": f"TRINITY CORE - {client_type}"}))

        # Üzenetek
        async for message in ws:
            try:
                data = json.loads(message)
                await handle(ws, data, client_type)
            except:
                pass

    except:
        pass
    finally:
        if client_type and client_type in clients:
            clients[client_type].discard(ws)
            log(f"🔴 {client_type.upper()} disconnected", RED)
            await broadcast_monitors("disconnect", client_type, "offline")

async def main():
    print(f"""
{CYAN}{BOLD}
╔════════════════════════════════════════════════╗
║           T R I N I T Y   C O R E              ║
║              MCP Server v1.0                   ║
║                                                ║
║      MÁTÉ ─── CLAUDE ─── GEMINI                ║
║                                                ║
║         ws://localhost:5555                    ║
╚════════════════════════════════════════════════╝
{END}""")

    log("TRINITY CORE ONLINE")

    async with serve(handler, HOST, PORT):
        await asyncio.Future()

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print(f"\n{CYAN}SYNC OFF{END}")
