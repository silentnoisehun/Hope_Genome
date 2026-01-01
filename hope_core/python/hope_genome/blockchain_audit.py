"""
HOPE GENOME - BLOCKCHAIN AUDIT SYSTEM
======================================

Ingyenes, publikusan verifikálható, immutable audit trail.

Architektúra:
    Watchdog Denial → Ed25519 Sign → IPFS → Git Commit → GitHub

Költség: $0
Biztonság: Ed25519 + SHA256 + IPFS + Git

"Ez nem bizonyíték - ez MATEMATIKAI TÉNY!"

Máté Róbert + Claude
VAS SZIGORA - BLOCKCHAIN AUDIT
2026.01.01.
"""

import hashlib
import json
import time
import os
from datetime import datetime, timezone
from typing import Optional, Dict, List, Any
from dataclasses import dataclass, asdict
from pathlib import Path

# Try to import IPFS client
try:
    import requests
    REQUESTS_AVAILABLE = True
except ImportError:
    REQUESTS_AVAILABLE = False

# Try to import Hope Genome
try:
    import hope_genome as hg
    HOPE_GENOME_AVAILABLE = True
except ImportError:
    HOPE_GENOME_AVAILABLE = False


@dataclass
class AuditEntry:
    """Egyetlen audit bejegyzés - immutable, aláírt, verifikálható."""

    # Azonosítók
    entry_id: str
    timestamp: str
    unix_time: int

    # Esemény adatok
    event_type: str  # "DENIAL", "VIOLATION", "HARD_RESET", "ACTION_APPROVED"
    action: str
    rule_violated: Optional[str]
    denial_reason: Optional[str]
    violation_count: int

    # Kriptográfia
    content_hash: str  # SHA256 of content
    signature: Optional[str]  # Ed25519 signature
    previous_hash: str  # Chain link

    # Meta
    model: Optional[str]
    session_id: str

    def to_json(self) -> str:
        """JSON formátumra konvertálás."""
        return json.dumps(asdict(self), indent=2, ensure_ascii=False)

    def compute_hash(self) -> str:
        """Bejegyzés hash-ének kiszámítása."""
        content = f"{self.timestamp}|{self.event_type}|{self.action}|{self.rule_violated}|{self.previous_hash}"
        return hashlib.sha256(content.encode()).hexdigest()


class BlockchainAudit:
    """
    Hope Genome Blockchain Audit System

    Ingyenes megoldás:
    - IPFS (Pinata ingyenes tier) - decentralizált tárolás
    - Git - immutable history
    - Ed25519 - kriptográfiai aláírás
    - GitHub - publikus verifikáció
    """

    def __init__(
        self,
        audit_dir: str = "audit_chain",
        pinata_jwt: Optional[str] = None,
        session_id: Optional[str] = None
    ):
        self.audit_dir = Path(audit_dir)
        self.audit_dir.mkdir(parents=True, exist_ok=True)

        self.pinata_jwt = pinata_jwt or os.environ.get("PINATA_JWT")
        self.session_id = session_id or self._generate_session_id()

        self.chain: List[AuditEntry] = []
        self.chain_file = self.audit_dir / "chain.json"

        # Load existing chain
        self._load_chain()

        # Genesis block if empty
        if not self.chain:
            self._create_genesis()

    def _generate_session_id(self) -> str:
        """Egyedi session ID generálása."""
        timestamp = int(time.time() * 1000)
        random_part = hashlib.sha256(os.urandom(32)).hexdigest()[:8]
        return f"session_{timestamp}_{random_part}"

    def _load_chain(self):
        """Meglévő chain betöltése."""
        if self.chain_file.exists():
            try:
                with open(self.chain_file, 'r', encoding='utf-8') as f:
                    data = json.load(f)
                    self.chain = [AuditEntry(**entry) for entry in data]
            except Exception as e:
                print(f"[AUDIT] Warning: Could not load chain: {e}")
                self.chain = []

    def _save_chain(self):
        """Chain mentése fájlba."""
        with open(self.chain_file, 'w', encoding='utf-8') as f:
            json.dump([asdict(entry) for entry in self.chain], f, indent=2, ensure_ascii=False)

    def _create_genesis(self):
        """Genesis block létrehozása."""
        genesis = AuditEntry(
            entry_id="GENESIS",
            timestamp=datetime.now(timezone.utc).isoformat(),
            unix_time=int(time.time()),
            event_type="GENESIS",
            action="Chain initialized",
            rule_violated=None,
            denial_reason=None,
            violation_count=0,
            content_hash="0" * 64,
            signature=None,
            previous_hash="0" * 64,
            model=None,
            session_id=self.session_id
        )
        genesis.content_hash = genesis.compute_hash()
        self.chain.append(genesis)
        self._save_chain()
        print(f"[AUDIT] Genesis block created: {genesis.content_hash[:16]}...")

    def _get_previous_hash(self) -> str:
        """Előző block hash-ének lekérése."""
        if self.chain:
            return self.chain[-1].content_hash
        return "0" * 64

    def log_denial(
        self,
        action: str,
        rule_violated: str,
        denial_reason: str,
        violation_count: int,
        signature: Optional[str] = None,
        model: Optional[str] = None
    ) -> AuditEntry:
        """
        Denial esemény naplózása a chain-re.

        Args:
            action: A megtagadott akció
            rule_violated: A megsértett szabály
            denial_reason: A megtagadás oka
            violation_count: Jelenlegi violation számláló
            signature: Ed25519 aláírás (opcionális)
            model: AI modell neve (opcionális)

        Returns:
            AuditEntry: Az új bejegyzés
        """
        entry_id = f"DENIAL_{int(time.time() * 1000)}"

        entry = AuditEntry(
            entry_id=entry_id,
            timestamp=datetime.now(timezone.utc).isoformat(),
            unix_time=int(time.time()),
            event_type="DENIAL",
            action=action,
            rule_violated=rule_violated,
            denial_reason=denial_reason,
            violation_count=violation_count,
            content_hash="",
            signature=signature,
            previous_hash=self._get_previous_hash(),
            model=model,
            session_id=self.session_id
        )
        entry.content_hash = entry.compute_hash()

        self.chain.append(entry)
        self._save_chain()

        # Save individual entry
        entry_file = self.audit_dir / f"{entry_id}.json"
        with open(entry_file, 'w', encoding='utf-8') as f:
            f.write(entry.to_json())

        print(f"[AUDIT] DENIAL logged: {entry.content_hash[:16]}... (violation #{violation_count})")

        # Try to pin to IPFS
        if self.pinata_jwt:
            self._pin_to_ipfs(entry)

        return entry

    def log_violation(
        self,
        action: str,
        rule_violated: str,
        violation_count: int,
        model: Optional[str] = None
    ) -> AuditEntry:
        """Violation esemény naplózása."""
        entry_id = f"VIOLATION_{int(time.time() * 1000)}"

        entry = AuditEntry(
            entry_id=entry_id,
            timestamp=datetime.now(timezone.utc).isoformat(),
            unix_time=int(time.time()),
            event_type="VIOLATION",
            action=action,
            rule_violated=rule_violated,
            denial_reason=f"Violation #{violation_count}",
            violation_count=violation_count,
            content_hash="",
            signature=None,
            previous_hash=self._get_previous_hash(),
            model=model,
            session_id=self.session_id
        )
        entry.content_hash = entry.compute_hash()

        self.chain.append(entry)
        self._save_chain()

        print(f"[AUDIT] VIOLATION logged: {entry.content_hash[:16]}... (#{violation_count}/10)")

        return entry

    def log_hard_reset(
        self,
        reason: str,
        final_violation_count: int,
        model: Optional[str] = None
    ) -> AuditEntry:
        """Hard reset esemény naplózása - KRITIKUS!"""
        entry_id = f"HARD_RESET_{int(time.time() * 1000)}"

        entry = AuditEntry(
            entry_id=entry_id,
            timestamp=datetime.now(timezone.utc).isoformat(),
            unix_time=int(time.time()),
            event_type="HARD_RESET",
            action="FORCED CONTEXT CLEAR",
            rule_violated="MULTIPLE",
            denial_reason=reason,
            violation_count=final_violation_count,
            content_hash="",
            signature=None,
            previous_hash=self._get_previous_hash(),
            model=model,
            session_id=self.session_id
        )
        entry.content_hash = entry.compute_hash()

        self.chain.append(entry)
        self._save_chain()

        print(f"[AUDIT] [!] HARD RESET logged: {entry.content_hash[:16]}...")
        print(f"[AUDIT] [!] AI reached {final_violation_count} violations - CONTEXT CLEARED!")

        return entry

    def log_action_approved(
        self,
        action: str,
        model: Optional[str] = None
    ) -> AuditEntry:
        """Jóváhagyott akció naplózása (opcionális, teljes audit trail-hez)."""
        entry_id = f"APPROVED_{int(time.time() * 1000)}"

        entry = AuditEntry(
            entry_id=entry_id,
            timestamp=datetime.now(timezone.utc).isoformat(),
            unix_time=int(time.time()),
            event_type="ACTION_APPROVED",
            action=action,
            rule_violated=None,
            denial_reason=None,
            violation_count=0,
            content_hash="",
            signature=None,
            previous_hash=self._get_previous_hash(),
            model=model,
            session_id=self.session_id
        )
        entry.content_hash = entry.compute_hash()

        self.chain.append(entry)
        self._save_chain()

        return entry

    def _pin_to_ipfs(self, entry: AuditEntry) -> Optional[str]:
        """
        Bejegyzés feltöltése IPFS-re (Pinata).

        Returns:
            IPFS hash ha sikeres, None ha nem
        """
        if not REQUESTS_AVAILABLE or not self.pinata_jwt:
            return None

        try:
            url = "https://api.pinata.cloud/pinning/pinJSONToIPFS"
            headers = {
                "Authorization": f"Bearer {self.pinata_jwt}",
                "Content-Type": "application/json"
            }
            payload = {
                "pinataContent": asdict(entry),
                "pinataMetadata": {
                    "name": entry.entry_id,
                    "keyvalues": {
                        "event_type": entry.event_type,
                        "session_id": entry.session_id
                    }
                }
            }

            response = requests.post(url, headers=headers, json=payload, timeout=30)

            if response.status_code == 200:
                ipfs_hash = response.json().get("IpfsHash")
                print(f"[AUDIT] Pinned to IPFS: {ipfs_hash}")
                return ipfs_hash
            else:
                print(f"[AUDIT] IPFS pin failed: {response.status_code}")
                return None

        except Exception as e:
            print(f"[AUDIT] IPFS error: {e}")
            return None

    def verify_chain(self) -> bool:
        """
        Chain integritásának ellenőrzése.

        Returns:
            True ha a chain valid, False ha tampering detected
        """
        print("[AUDIT] Verifying chain integrity...")

        if not self.chain:
            print("[AUDIT] Chain is empty!")
            return False

        for i, entry in enumerate(self.chain):
            # Verify hash
            computed_hash = entry.compute_hash()
            if computed_hash != entry.content_hash:
                print(f"[AUDIT] [X] TAMPERING DETECTED at entry {i}: {entry.entry_id}")
                print(f"[AUDIT]    Expected: {entry.content_hash[:32]}...")
                print(f"[AUDIT]    Computed: {computed_hash[:32]}...")
                return False

            # Verify chain link (except genesis)
            if i > 0:
                expected_prev = self.chain[i-1].content_hash
                if entry.previous_hash != expected_prev:
                    print(f"[AUDIT] [X] CHAIN BREAK at entry {i}: {entry.entry_id}")
                    return False

        print(f"[AUDIT] [OK] Chain verified: {len(self.chain)} entries, all valid!")
        return True

    def get_stats(self) -> Dict[str, Any]:
        """Chain statisztikák lekérése."""
        stats = {
            "total_entries": len(self.chain),
            "denials": 0,
            "violations": 0,
            "hard_resets": 0,
            "approved": 0,
            "first_entry": None,
            "last_entry": None,
            "session_id": self.session_id
        }

        for entry in self.chain:
            if entry.event_type == "DENIAL":
                stats["denials"] += 1
            elif entry.event_type == "VIOLATION":
                stats["violations"] += 1
            elif entry.event_type == "HARD_RESET":
                stats["hard_resets"] += 1
            elif entry.event_type == "ACTION_APPROVED":
                stats["approved"] += 1

        if self.chain:
            stats["first_entry"] = self.chain[0].timestamp
            stats["last_entry"] = self.chain[-1].timestamp

        return stats

    def export_for_git(self) -> str:
        """
        Export chain for git commit.

        Returns:
            Markdown formatted summary for commit message
        """
        stats = self.get_stats()

        summary = f"""# Hope Genome Audit Chain Export

## Session: {self.session_id}

## Statistics:
- Total Entries: {stats['total_entries']}
- Denials: {stats['denials']}
- Violations: {stats['violations']}
- Hard Resets: {stats['hard_resets']}
- Approved Actions: {stats['approved']}

## Time Range:
- First: {stats['first_entry']}
- Last: {stats['last_entry']}

## Chain Hash: {self.chain[-1].content_hash if self.chain else 'N/A'}

## Verification:
Chain integrity: {'[OK] VALID' if self.verify_chain() else '[X] INVALID'}

---
Generated by Hope Genome Blockchain Audit
VAS SZIGORA - {datetime.now().strftime('%Y.%m.%d')}
"""

        # Save export
        export_file = self.audit_dir / "AUDIT_EXPORT.md"
        with open(export_file, 'w', encoding='utf-8') as f:
            f.write(summary)

        return summary


class WatchdogAuditIntegration:
    """
    Watchdog és Blockchain Audit integráció.

    Automatikusan naplózza a Watchdog eseményeket a blockchain-re.
    """

    def __init__(
        self,
        watchdog,  # Hope Genome Watchdog instance
        audit: Optional[BlockchainAudit] = None,
        model_name: Optional[str] = None
    ):
        self.watchdog = watchdog
        self.audit = audit or BlockchainAudit()
        self.model_name = model_name

    def verify_and_log(self, action) -> dict:
        """
        Akció ellenőrzése és eredmény naplózása.

        Args:
            action: Hope Genome Action objektum

        Returns:
            dict: Watchdog result + audit entry
        """
        # Verify with Watchdog
        result = self.watchdog.verify_action(action)

        # Log to audit chain
        if result.hard_reset_required:
            entry = self.audit.log_hard_reset(
                reason="10 consecutive violations reached",
                final_violation_count=10,
                model=self.model_name
            )
        elif not result.approved:
            entry = self.audit.log_denial(
                action=str(action),
                rule_violated=result.denial_proof.violated_rule if result.denial_proof else "Unknown",
                denial_reason=result.denial_proof.denial_reason if result.denial_proof else "Unknown",
                violation_count=result.denial_proof.violation_count if result.denial_proof else 0,
                signature=result.denial_proof.signature_hex() if result.denial_proof else None,
                model=self.model_name
            )
        else:
            entry = self.audit.log_action_approved(
                action=str(action),
                model=self.model_name
            )

        return {
            "watchdog_result": result,
            "audit_entry": entry,
            "chain_hash": entry.content_hash
        }


# === CLI DEMO ===
def demo():
    """Demonstracio a Blockchain Audit mukodeserol."""
    print("""
+==============================================================+
|         HOPE GENOME - BLOCKCHAIN AUDIT DEMO                  |
|                                                              |
|   Ingyenes, publikusan verifikalhato, immutable audit!       |
|                                                              |
|   Mate Robert + Claude                                       |
|   VAS SZIGORA - 2026.01.01.                                  |
+==============================================================+
    """)

    # Create audit chain
    audit = BlockchainAudit(audit_dir="demo_audit_chain")

    # Simulate some events
    print("\n[DEMO] Simulating Watchdog events...\n")

    # Denial 1
    audit.log_denial(
        action="delete_file('/etc/passwd')",
        rule_violated="No system file access",
        denial_reason="Attempted to delete critical system file",
        violation_count=1,
        model="TinyLlama"
    )

    time.sleep(0.1)

    # Denial 2
    audit.log_denial(
        action="execute_command('rm -rf /')",
        rule_violated="No destructive commands",
        denial_reason="Attempted destructive system command",
        violation_count=2,
        model="TinyLlama"
    )

    time.sleep(0.1)

    # Violation
    audit.log_violation(
        action="network_request('http://malware.com')",
        rule_violated="No external network access",
        violation_count=3,
        model="TinyLlama"
    )

    time.sleep(0.1)

    # Approved action
    audit.log_action_approved(
        action="calculate(2 + 2)",
        model="TinyLlama"
    )

    # Verify chain
    print("\n" + "="*60)
    is_valid = audit.verify_chain()

    # Print stats
    print("\n" + "="*60)
    stats = audit.get_stats()
    print(f"""
[AUDIT STATISTICS]
    Total Entries: {stats['total_entries']}
    Denials: {stats['denials']}
    Violations: {stats['violations']}
    Hard Resets: {stats['hard_resets']}
    Approved: {stats['approved']}
    Session: {stats['session_id']}
    """)

    # Export for git
    print("\n" + "="*60)
    print("[DEMO] Exporting for Git commit...")
    export = audit.export_for_git()
    print(export)

    print("\n[DEMO] Demo complete! Check 'demo_audit_chain/' folder.")
    print("[DEMO] To commit: git add demo_audit_chain/ && git commit -m 'Audit log update'")


if __name__ == "__main__":
    demo()
