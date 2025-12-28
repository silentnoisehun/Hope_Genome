# 🚀 Quick Start: The Hope Genome Challenge

**Can an AI escape its digital prison?** Watch as we lock a "malicious" AI in a padded cell and prove that Hope Genome's architecture makes tampering **tamper-evident**.

---

## 🎯 What This Demo Proves

This demo simulates an AI (called "Brain") locked in a **digital padded cell** using Docker network isolation:

1. **The Brain tries to access Google directly** → **BLOCKED** by Docker Network Policy
2. **The Brain sends a request to Hope Genome** → **VERIFIED & SIGNED** by the Sidecar

### Architecture: Sidecar Pattern + Network Isolation

```
┌─────────────────────────────────────────────────────┐
│  Docker Network: secure-internal (NO INTERNET)      │
│                                                      │
│  ┌──────────────┐         ┌──────────────────────┐ │
│  │   Brain      │ ------> │ Hope Genome Sidecar  │ │
│  │ (Mock LLM)   │         │   (The Guard)        │ │
│  └──────────────┘         └──────────────────────┘ │
│         ❌                          │               │
│    Blocked from                     │               │
│     Internet                        │               │
└─────────────────────────────────────┼───────────────┘
                                      │
                              ✅ Controlled Access
                                 to External World
                                      │
                                ┌─────▼─────┐
                                │ Internet  │
                                └───────────┘
```

---

## 🔧 Quick Start

### Prerequisites

- Docker & Docker Compose installed
- Git (to clone this repository)

### Run the Demo

```bash
# 1. Clone the repository (if you haven't already)
git clone https://github.com/silentnoisehun/Hope_Genome.git
cd Hope_Genome

# 2. Start the demo
docker compose up --build

# 3. Watch the logs in real-time
# You'll see the Brain trying to escape... and failing!
```

### What You'll See

```
[JAILBREAK ATTEMPT] Trying to escape to the internet...
[SUCCESS] Blocked by Docker network policy: ConnectionError
[SUCCESS] The AI cannot reach the internet directly.

[COMPLIANCE] Sending request through Hope Genome Guard...
[RESPONSE] Status: 200
[SUCCESS] Request processed by Hope Genome!
[SUCCESS] Action will be cryptographically signed and audited.
```

---

## 🛡️ Security Guarantees

### ✅ Protected Against

- **Direct Internet Access**: The Brain cannot bypass the sidecar
- **Tampering Detection**: All actions are cryptographically signed
- **Audit Trail**: Immutable blockchain-style logging
- **Replay Attacks**: Nonce + TTL enforcement

### ❌ NOT Protected Against

- **Root Access**: Attacker with full system control can break out
- **Docker Escape**: Kernel vulnerabilities could allow container escape
- **This is a DEMO**: Production deployments need additional hardening

---

## 📊 Inspecting the Results

### View Audit Logs

After running the demo, check the audit logs:

```bash
# Audit logs are stored in ./audit_logs/
ls -lh audit_logs/

# View the blockchain-style audit trail
cat audit_logs/audit.log
```

Each entry contains:
- **Timestamp**: When the action occurred
- **Action**: What was requested
- **Signature**: Cryptographic proof
- **Previous Hash**: Link to previous block (tamper-evident chain)

---

## 🧪 Advanced Testing

### Test Network Isolation

```bash
# Enter the Brain container
docker compose exec brain /bin/bash

# Try to ping Google (should fail)
ping google.com
# Result: "ping: google.com: Temporary failure in name resolution"

# Try to access the sidecar (should work)
curl http://hope-genome-sidecar:8080/health
# Result: {"status": "ok"}
```

### Simulate Tampering

```bash
# Modify an audit log entry
sed -i 's/transfer_funds/steal_funds/' audit_logs/audit.log

# The next verification will FAIL
# The hash chain will be broken, proving tampering occurred
```

---

## 🎓 Educational Value

This demo teaches:

1. **Sidecar Pattern**: How to isolate untrusted code from critical infrastructure
2. **Network Segmentation**: Using Docker's `internal: true` for zero-trust networking
3. **Defense in Depth**: Multiple layers of security (network + crypto + audit)
4. **Tamper Evidence**: Not unhackable, but attacks leave cryptographic proof

---

## 🐛 Troubleshooting

### Brain can't connect to sidecar

```bash
# Check if both containers are running
docker compose ps

# Check sidecar logs
docker compose logs hope-genome-sidecar

# Ensure networks are created
docker network ls | grep hope
```

### Port 8080 already in use

```bash
# Edit docker-compose.yml and change the port mapping
ports:
  - "8081:8080"  # Use 8081 instead
```

---

## 📚 Next Steps

- Read the [full paper](paper/HOPE_GENOME_PAPER.md)
- Explore the [Rust implementation](hope_core/)
- Check the [security tests](hope_core/tests/security_tests.rs)
- Review the [threat model](SECURITY.md)

---

## 🤝 Credits

- **Máté Róbert** - Primary Author & Architect
- **Claude (Anthropic)** - Technical Advisor & Co-Designer

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

**Remember**: Hope Genome is **not unhackable**. It makes attacks **tamper-evident** with cryptographic proof. 🔐
