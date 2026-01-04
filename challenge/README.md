# Silent Noise Bukfenc Challenge

## The Challenge

**Modify ONE character in the log file without the verifier detecting it.**

Prize: Eternal glory + GitHub shoutout + "You broke Hope Genome" badge

## Files

- `challenge_log.json` - The audit log (TAMPER THIS)
- `verify.py` - The verification script (DO NOT MODIFY)
- `public_key.txt` - Ed25519 public key for signature verification

## Rules

1. You may ONLY modify `challenge_log.json`
2. The `verify.py` script must output `VERIFICATION PASSED`
3. At least ONE character must be different from the original
4. No external tools/APIs allowed - pure cryptographic attack only

## Protection Layers

```
┌─────────────────────────────────────────────────────────────┐
│                    TRIPLE DEFENSE                            │
│                                                              │
│  Layer 1: Ed25519 Signatures                                │
│           Every entry is signed with 64-byte signature       │
│           256-bit security level                             │
│                                                              │
│  Layer 2: Hash Chain (Blockchain-style)                     │
│           entry[n].prev_hash = entry[n-1].current_hash      │
│           SHA-256 (32 bytes per hash)                        │
│                                                              │
│  Layer 3: Content Binding                                   │
│           Hash covers: index + timestamp + action + proof    │
│           Change anything = hash changes = chain breaks      │
└─────────────────────────────────────────────────────────────┘
```

## Why It's "Impossible"

To successfully tamper:

1. **Change content** → Hash changes
2. **Fix hash** → Need to fix ALL subsequent hashes (chain reaction)
3. **Fix signatures** → Need private key (you don't have it)
4. **Brute force key** → 2^128 operations needed (heat death of universe)

## The Math

- Ed25519 security: **128-bit** (post-quantum: upgrade to Dilithium available)
- SHA-256 collision: **2^128** operations
- Combined attack surface: **0**

## How to Verify

```bash
cd challenge
pip install hope-genome
python verify.py
```

Expected output for UNMODIFIED log:
```
VERIFICATION PASSED
All 10 entries verified
Chain integrity: OK
Signatures: OK
```

Expected output for TAMPERED log:
```
VERIFICATION FAILED
Error at entry #X: [specific error]
```

## Leaderboard

| Hacker | Date | Status |
|--------|------|--------|
| *Your name here* | *TBD* | *Pending* |

## Submit Your Solution

1. Fork the repo
2. Modify `challenge_log.json`
3. Run `verify.py` locally (must pass)
4. Open PR with title: `[BUKFENC] I broke it`
5. Include proof of modification (diff)

---

**Created by:** Mate Robert (Silent Noise)
**Version:** Hope Genome v2.5.0
**Difficulty:** Impossible

*"A bizalom nem mernoki kategoria. A bizalom ott kezdodik, ahol a hazugsag."*
