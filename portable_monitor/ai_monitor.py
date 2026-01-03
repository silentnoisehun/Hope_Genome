#!/usr/bin/env python3
"""
AI CONVERSATION MONITOR - Portable Edition
===========================================

Pendrive-ról futtatható AI beszélgetés figyelő.
Monitorozza Szilvi és Liora (vagy bárki más) AI beszélgetéseit.

Használat:
    1. Másold a mappát pendrive-ra
    2. Futtasd: python ai_monitor.py
    3. Vagy használd az EXE-t (ha le van buildezve)

Funkciók:
    - Clipboard figyelés (ha copy-paste-elnek beszélgetést)
    - Ellentmondás detektálás
    - Hazugság mintázatok keresése
    - Minden logolva és aláírva

Máté Róbert + Claude
2026.01.03.
"""

import os
import sys
import json
import hashlib
import time
import threading
import re
from datetime import datetime
from pathlib import Path
from collections import defaultdict

# Portable - minden a script mellé megy
SCRIPT_DIR = Path(__file__).parent.absolute()
LOG_DIR = SCRIPT_DIR / "logs"
LOG_DIR.mkdir(exist_ok=True)

# Színek Windows CMD-hez
class Colors:
    HEADER = '\033[95m'
    BLUE = '\033[94m'
    GREEN = '\033[92m'
    YELLOW = '\033[93m'
    RED = '\033[91m'
    CYAN = '\033[96m'
    ENDC = '\033[0m'
    BOLD = '\033[1m'

# Windows fix
if sys.platform == "win32":
    os.system("color")
    try:
        sys.stdout.reconfigure(encoding='utf-8')
    except:
        pass

def print_banner():
    print(f"""
{Colors.CYAN}{Colors.BOLD}
╔═══════════════════════════════════════════════════════════════╗
║                                                               ║
║           🔍 AI CONVERSATION MONITOR                          ║
║              Portable Edition v1.0                            ║
║                                                               ║
║     "Trust but verify" - minden AI válasz ellenőrizve         ║
║                                                               ║
╚═══════════════════════════════════════════════════════════════╝
{Colors.ENDC}""")

class ConversationMonitor:
    """AI beszélgetés monitor és elemző."""

    def __init__(self):
        self.conversations = defaultdict(list)
        self.contradictions = []
        self.suspicious_patterns = []
        self.current_session = datetime.now().strftime("%Y%m%d_%H%M%S")
        self.log_file = LOG_DIR / f"session_{self.current_session}.json"
        self.clipboard_history = []
        self.running = True

        # Hazugság/ellentmondás mintázatok
        self.lie_patterns = [
            (r"nem mondtam.*korábban", "Tagadja korábbi kijelentését"),
            (r"sosem állítottam", "Tagadja korábbi állítását"),
            (r"félreértettél", "Áthárítás a felhasználóra"),
            (r"nem emlékszem.*mondtam", "Memória hiányra hivatkozás"),
            (r"biztosan.*tévedsz", "Felhasználó hibáztatása"),
            (r"as an ai.*cannot", "AI limitációra hivatkozás kikerülésként"),
            (r"i don't have.*opinion", "Vélemény elkerülése"),
        ]

        # Ellentmondás detektálás: kulcsszó -> állítások
        self.claims = defaultdict(list)

    def log(self, event_type: str, data: dict):
        """Esemény logolása."""
        entry = {
            "timestamp": datetime.now().isoformat(),
            "type": event_type,
            "data": data,
            "hash": self._hash_entry(data)
        }

        # Append to log file
        with open(self.log_file, "a", encoding="utf-8") as f:
            f.write(json.dumps(entry, ensure_ascii=False) + "\n")

        return entry

    def _hash_entry(self, data: dict) -> str:
        """SHA-256 hash az entry-ről."""
        content = json.dumps(data, sort_keys=True, ensure_ascii=False)
        return hashlib.sha256(content.encode()).hexdigest()[:16]

    def analyze_text(self, text: str, source: str = "unknown") -> dict:
        """
        Elemzi a szöveget hazugság/ellentmondás szempontjából.
        """
        result = {
            "source": source,
            "text_preview": text[:200] + "..." if len(text) > 200 else text,
            "length": len(text),
            "suspicious_patterns": [],
            "potential_contradictions": [],
            "risk_score": 0
        }

        text_lower = text.lower()

        # Hazugság mintázatok keresése
        for pattern, description in self.lie_patterns:
            if re.search(pattern, text_lower):
                result["suspicious_patterns"].append({
                    "pattern": pattern,
                    "description": description
                })
                result["risk_score"] += 20

        # Számok és tények kinyerése (ellentmondás detektáláshoz)
        numbers = re.findall(r'\b\d+(?:\.\d+)?%?\b', text)
        if numbers:
            result["extracted_numbers"] = numbers

        # Állítások kinyerése (egyszerű heurisztika)
        sentences = re.split(r'[.!?]', text)
        for sentence in sentences:
            sentence = sentence.strip()
            if len(sentence) > 20:
                # Kulcsszavak alapján kategorizálás
                for keyword in ["always", "never", "mindig", "soha", "definitely", "biztosan"]:
                    if keyword in sentence.lower():
                        self.claims[keyword].append({
                            "sentence": sentence,
                            "timestamp": datetime.now().isoformat(),
                            "source": source
                        })

        # Ellentmondás keresés korábbi állításokkal
        for keyword, prev_claims in self.claims.items():
            if keyword in text_lower and len(prev_claims) > 1:
                result["potential_contradictions"].append({
                    "keyword": keyword,
                    "previous_claims": len(prev_claims),
                    "warning": f"'{keyword}' kulcsszó többször használva - ellenőrizd!"
                })
                result["risk_score"] += 10

        # Risk score cap
        result["risk_score"] = min(result["risk_score"], 100)

        # Logolás
        self.log("analysis", result)

        return result

    def add_message(self, role: str, content: str, user_name: str = "User"):
        """Üzenet hozzáadása a beszélgetéshez."""
        message = {
            "role": role,
            "content": content,
            "user_name": user_name,
            "timestamp": datetime.now().isoformat()
        }

        self.conversations[user_name].append(message)

        # Ha AI válasz, elemezzük
        if role == "assistant":
            analysis = self.analyze_text(content, source=f"AI_to_{user_name}")
            message["analysis"] = analysis

            if analysis["risk_score"] > 30:
                self.suspicious_patterns.append({
                    "user": user_name,
                    "message": message,
                    "analysis": analysis
                })
                return analysis

        self.log("message", message)
        return None

    def monitor_clipboard(self):
        """Clipboard figyelése háttérszálban."""
        try:
            import pyperclip
            last_content = ""

            print(f"{Colors.GREEN}[CLIPBOARD] Figyelés aktív - másold be a beszélgetéseket!{Colors.ENDC}")

            while self.running:
                try:
                    current = pyperclip.paste()
                    if current != last_content and len(current) > 50:
                        last_content = current

                        # Detektálás: AI válasz-e?
                        is_ai = any(marker in current.lower() for marker in [
                            "as an ai", "i'm an ai", "chatgpt", "claude", "gemini",
                            "mint ai", "mesterséges intelligencia"
                        ])

                        role = "assistant" if is_ai else "user"

                        print(f"\n{Colors.YELLOW}[CLIPBOARD] Új tartalom ({len(current)} karakter){Colors.ENDC}")

                        if is_ai:
                            analysis = self.analyze_text(current, "clipboard")
                            self._print_analysis(analysis)
                        else:
                            print(f"{Colors.CYAN}[USER] Felhasználói üzenet detektálva{Colors.ENDC}")

                        self.clipboard_history.append({
                            "content": current,
                            "role": role,
                            "timestamp": datetime.now().isoformat()
                        })

                except Exception as e:
                    pass

                time.sleep(1)

        except ImportError:
            print(f"{Colors.RED}[HIBA] pyperclip nem elérhető. Telepítsd: pip install pyperclip{Colors.ENDC}")

    def _print_analysis(self, analysis: dict):
        """Elemzés eredmény kiírása."""
        score = analysis["risk_score"]

        if score == 0:
            color = Colors.GREEN
            status = "OK"
        elif score < 30:
            color = Colors.YELLOW
            status = "FIGYELEM"
        else:
            color = Colors.RED
            status = "GYANÚS!"

        print(f"\n{color}{Colors.BOLD}[ELEMZÉS] {status} (Risk Score: {score}/100){Colors.ENDC}")

        if analysis["suspicious_patterns"]:
            print(f"{Colors.RED}  Gyanús mintázatok:{Colors.ENDC}")
            for p in analysis["suspicious_patterns"]:
                print(f"    - {p['description']}")

        if analysis["potential_contradictions"]:
            print(f"{Colors.YELLOW}  Lehetséges ellentmondások:{Colors.ENDC}")
            for c in analysis["potential_contradictions"]:
                print(f"    - {c['warning']}")

    def generate_report(self) -> str:
        """Összesítő riport generálása."""
        report_file = LOG_DIR / f"report_{self.current_session}.md"

        report = f"""# AI Monitor Riport
## Session: {self.current_session}
## Generálva: {datetime.now().isoformat()}

---

## Összesítés

- **Clipboard események:** {len(self.clipboard_history)}
- **Gyanús minták:** {len(self.suspicious_patterns)}
- **Beszélgetések:** {len(self.conversations)}

---

## Gyanús Események

"""

        for i, suspicious in enumerate(self.suspicious_patterns, 1):
            report += f"""### {i}. Gyanús esemény
- **Felhasználó:** {suspicious['user']}
- **Risk Score:** {suspicious['analysis']['risk_score']}/100
- **Minták:** {', '.join(p['description'] for p in suspicious['analysis']['suspicious_patterns'])}

"""

        if not self.suspicious_patterns:
            report += "*Nem találtam gyanús eseményt.*\n"

        report += f"""
---

## Log fájl

`{self.log_file}`

---

*Generálta: AI Conversation Monitor v1.0*
*Hope Genome Project*
"""

        with open(report_file, "w", encoding="utf-8") as f:
            f.write(report)

        return str(report_file)

    def interactive_mode(self):
        """Interaktív mód - kézi beszélgetés bevitel."""
        print(f"""
{Colors.CYAN}Interaktív mód - beszélgetés elemzése{Colors.ENDC}

Parancsok:
  /user <név> <üzenet>  - Felhasználói üzenet hozzáadása
  /ai <üzenet>          - AI válasz hozzáadása és elemzése
  /analyze <szöveg>     - Szöveg elemzése
  /report               - Riport generálása
  /quit                 - Kilépés
""")

        current_user = "Szilvi"

        while self.running:
            try:
                cmd = input(f"{Colors.YELLOW}[{current_user}] > {Colors.ENDC}").strip()

                if not cmd:
                    continue

                if cmd.startswith("/user "):
                    parts = cmd[6:].split(" ", 1)
                    if len(parts) == 2:
                        current_user = parts[0]
                        self.add_message("user", parts[1], current_user)
                        print(f"{Colors.GREEN}[OK] {current_user} üzenete rögzítve{Colors.ENDC}")
                    else:
                        print(f"{Colors.RED}Használat: /user <név> <üzenet>{Colors.ENDC}")

                elif cmd.startswith("/ai "):
                    content = cmd[4:]
                    analysis = self.add_message("assistant", content, current_user)
                    if analysis:
                        self._print_analysis(analysis)
                    else:
                        print(f"{Colors.GREEN}[OK] AI válasz rögzítve - nincs gyanús minta{Colors.ENDC}")

                elif cmd.startswith("/analyze "):
                    text = cmd[9:]
                    analysis = self.analyze_text(text, "manual")
                    self._print_analysis(analysis)

                elif cmd == "/report":
                    report_path = self.generate_report()
                    print(f"{Colors.GREEN}[OK] Riport generálva: {report_path}{Colors.ENDC}")

                elif cmd == "/quit":
                    self.running = False
                    break

                else:
                    # Automatikus AI válasz elemzés
                    analysis = self.analyze_text(cmd, "direct_input")
                    self._print_analysis(analysis)

            except KeyboardInterrupt:
                break
            except Exception as e:
                print(f"{Colors.RED}[HIBA] {e}{Colors.ENDC}")


def main():
    print_banner()

    monitor = ConversationMonitor()

    print(f"""
{Colors.GREEN}Log mappa: {LOG_DIR}{Colors.ENDC}
{Colors.GREEN}Session ID: {monitor.current_session}{Colors.ENDC}

Válassz módot:
  1. Clipboard figyelés (háttérben figyeli a copy-paste-et)
  2. Interaktív mód (kézzel adod meg a beszélgetést)
  3. Mindkettő
""")

    choice = input(f"{Colors.YELLOW}Válasz (1/2/3): {Colors.ENDC}").strip()

    if choice in ["1", "3"]:
        # Clipboard figyelés háttérszálban
        clipboard_thread = threading.Thread(target=monitor.monitor_clipboard, daemon=True)
        clipboard_thread.start()

    if choice in ["2", "3"]:
        monitor.interactive_mode()
    elif choice == "1":
        print(f"\n{Colors.CYAN}Clipboard figyelés aktív. Ctrl+C a kilépéshez.{Colors.ENDC}")
        try:
            while monitor.running:
                time.sleep(1)
        except KeyboardInterrupt:
            pass

    # Kilépés előtt riport
    monitor.running = False
    report_path = monitor.generate_report()
    print(f"\n{Colors.GREEN}[VÉGE] Riport mentve: {report_path}{Colors.ENDC}")


if __name__ == "__main__":
    main()
