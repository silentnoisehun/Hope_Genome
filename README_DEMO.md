# 🚀 Quick Start: The Hope Genome Challenge

**Hope Genome v1.3.0 - The Era of Enforceable AI Accountability**

**Can an AI escape its digital prison?** Watch as we lock a "malicious" AI in a padded cell and prove that Hope Genome's architecture makes tampering **tamper-evident** with **OWASP AI-SBOM compliance**.

---

## 🎯 What This Demo Proves

This demo simulates an AI (called "Brain") locked in a **digital padded cell** using Docker network isolation, with **OWASP AI-SBOM cryptographic verification**:

1. **The Brain tries to access Google directly** → **BLOCKED** by Docker Network Policy
2. **The Brain sends a request to Hope Genome** → **VERIFIED & SIGNED** by the Sidecar
3. **AI models verified with OWASP AI-SBOM** → **CRYPTOGRAPHIC INTEGRITY** ensured by CycloneDX compliance

### Architecture: Sidecar Pattern + Network Isolation + OWASP AI-SBOM

```
┌─────────────────────────────────────────────────────────────┐
│  Docker Network: secure-internal (NO INTERNET)              │
│                                                              │
│  ┌──────────────┐         ┌────────────────────────────┐   │
│  │   Brain      │ ------> │  Hope Genome Sidecar       │   │
│  │ (Mock LLM)   │         │   (The Guard)              │   │
│  └──────────────┘         │                            │   │
│         ❌                 │  ┌──────────────────────┐ │   │
│    Blocked from           │  │ OWASP AI-SBOM        │ │   │
│     Internet              │  │ Compliance Layer     │ │   │
│                           │  │ (CycloneDX 1.5+)     │ │   │
│                           │  └──────────────────────┘ │   │
│                           │           🔐               │   │
│                           │  Cryptographic Integrity  │   │
│                           └────────────────────────────┘   │
│                                       │                    │
└───────────────────────────────────────┼────────────────────┘
                                        │
                                ✅ Controlled Access
                                   to External World
                                   (with AIBOM validation)
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

#### Option 1: OWASP AI-SBOM Compliance Demo (NEW in v1.3.0)

```bash
# 1. Clone the repository (if you haven't already)
git clone https://github.com/silentnoisehun/Hope_Genome.git
cd Hope_Genome

# 2. Build and run OWASP AIBOM compliance demo
docker build -t hope-genome:1.3.0 .
docker run --rm hope-genome:1.3.0

# 3. Or run the automated verification script
bash docker-test.sh
```

#### Option 2: Full Sidecar Pattern Demo

```bash
# 1. Start the full stack (Brain + Sidecar)
docker compose up --build

# 2. Watch the logs in real-time
# You'll see the Brain trying to escape... and failing!
```

### What You'll See

#### OWASP AI-SBOM Compliance Demo Output:

```
=== Hope Genome v1.3.0 - AIBOM Compliance Demo ===

📄 Loading AIBOM file...
   ✅ Loaded AIBOM:
      Format: CycloneDX
      Spec Version: 1.5
      Components: 3

🔍 Finding AI model component...
   ✅ Found component:
      Name: medical-diagnosis-model
      Type: machine-learning-model
      Version: 2.1.0

🔐 Extracting cryptographic hash...
   ✅ SBOM Hash (SHA-256):
      e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855

✔️  Validating integrity (matching hashes)...
   ✅ SUCCESS: Hash validation passed!
      Model integrity verified ✓

⚠️  Demonstrating Fort Knox error (tampered hash)...
   ❌ FORT KNOX TRIGGERED:
      FORT KNOX VIOLATION: Hash mismatch detected!
      TRANSACTION HALTED
```

#### Sidecar Pattern Demo Output:

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
- **AI Model Tampering**: OWASP AI-SBOM cryptographic hash validation (NEW v1.3.0)
- **Supply Chain Attacks**: CycloneDX 1.5+ component integrity verification (NEW v1.3.0)
- **Timing Attacks**: Constant-time hash comparison prevents side-channel attacks (NEW v1.3.0)
- **Fort Knox Integrity Violations**: Transaction halting on hash mismatch (NEW v1.3.0)

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

### Test OWASP AI-SBOM Compliance (NEW in v1.3.0)

```bash
# Run compliance tests in Docker
docker run --rm hope-genome:1.3.0

# Run unit tests for OWASP AIBOM module
cd hope_core
cargo test compliance

# Run all 71 tests (56 core, 12 security, 8 compliance, 3 doc)
cargo test

# Expected output:
# test result: ok. 71 passed; 0 failed; 0 ignored
```

**OWASP AI-SBOM Test Coverage:**
- ✅ CycloneDX 1.5+ JSON parsing
- ✅ Component discovery (name, type)
- ✅ Hash extraction (SHA-256, SHA-512)
- ✅ Integrity validation (success/failure)
- ✅ Hash normalization (whitespace, case)
- ✅ Constant-time comparison (timing attack prevention)
- ✅ Fort Knox Integrity Enforcement

### Verify Docker Labels

```bash
# Check OWASP AI-SBOM compliance labels
docker inspect hope-genome:1.3.0 --format='{{json .Config.Labels}}' | jq

# Expected output includes:
# "compliance.owasp.aibom": "CycloneDX 1.5+"
# "compliance.cyclonedx.version": "1.5"
# "tests.passed": "71/71"
```

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

#### Audit Log Tampering

```bash
# Modify an audit log entry
sed -i 's/transfer_funds/steal_funds/' audit_logs/audit.log

# The next verification will FAIL
# The hash chain will be broken, proving tampering occurred
```

#### OWASP AI-SBOM Tampering (NEW in v1.3.0)

```bash
# Create a tampered model hash in code
cd hope_core/examples
cat > test_tamper.rs << 'EOF'
use hope_core::compliance::*;

fn main() -> Result<(), ComplianceError> {
    let aibom = AiBom::from_file("example_model.aibom.json")?;
    let component = aibom.find_component("medical-diagnosis-model")?;
    let sbom_hash = component.get_hash("SHA-256")?;

    // Simulate tampered runtime hash
    let tampered_hash = "AAAA0000TAMPERED1111BBBB2222CCCC3333DDDD4444EEEE5555FFFF6666";

    // This will trigger Fort Knox Integrity Violation
    match validate_integrity("medical-diagnosis-model", sbom_hash, tampered_hash) {
        Ok(_) => println!("❌ SECURITY FAILURE: Tampering not detected!"),
        Err(ComplianceError::IntegrityViolation { .. }) => {
            println!("✅ SUCCESS: Fort Knox triggered!");
            println!("   Transaction halted as expected");
        }
        Err(e) => println!("Other error: {}", e),
    }
    Ok(())
}
EOF

cargo run --example test_tamper
# Expected: Fort Knox Integrity Violation message
```

---

## 🎓 Educational Value

This demo teaches:

1. **Sidecar Pattern**: How to isolate untrusted code from critical infrastructure
2. **Network Segmentation**: Using Docker's `internal: true` for zero-trust networking
3. **Defense in Depth**: Multiple layers of security (network + crypto + audit)
4. **Tamper Evidence**: Not unhackable, but attacks leave cryptographic proof
5. **OWASP AI-SBOM Compliance** (NEW v1.3.0): CycloneDX 1.5+ integration for AI model integrity
6. **Constant-Time Cryptography** (NEW v1.3.0): Preventing timing-based side-channel attacks
7. **Fort Knox Integrity Enforcement** (NEW v1.3.0): Critical error handling that halts transactions
8. **Supply Chain Security** (NEW v1.3.0): Cryptographic verification of AI component provenance

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

### OWASP AI-SBOM Docker Build Fails (NEW v1.3.0)

```bash
# Check Docker daemon is running
docker info

# Start Docker Desktop if needed
# Then rebuild with verbose output
docker build -t hope-genome:1.3.0 . --progress=plain

# If tests fail during build, run locally to debug
cd hope_core
cargo test compliance -- --nocapture
```

### Compliance Demo Can't Find AIBOM File

```bash
# Verify file exists in container
docker run --rm -it hope-genome:1.3.0 /bin/bash
ls -la /app/example_model.aibom.json

# Or mount custom AIBOM file
docker run --rm \
  -v $(pwd)/custom-model.aibom.json:/app/model.aibom.json \
  hope-genome:1.3.0
```

---

## 📚 Next Steps

### Core Documentation
- Read the [full paper](paper/HOPE_GENOME_PAPER.md)
- Explore the [Rust implementation](hope_core/)
- Check the [security tests](hope_core/tests/security_tests.rs)
- Review the [threat model](SECURITY.md)

### OWASP AI-SBOM Integration (NEW v1.3.0)
- Read the [OWASP AI-SBOM Integration Guide](hope_core/AIBOM_INTEGRATION.md)
- Review the [OWASP Compliance Report](hope_core/OWASP_COMPLIANCE_REPORT.md)
- Check the [Executive Summary for OWASP Leadership](hope_core/OWASP_EXECUTIVE_SUMMARY.md)
- Explore the [Docker Integration Guide](DOCKER_OWASP_INTEGRATION.md)
- Run the [compliance demo](hope_core/examples/compliance_demo.rs)

---

## 🤝 Credits

### Primary Contributors
- **Máté Róbert** - Lead Developer, System Architect, Original Vision
- **Claude (Anthropic)** - Technical Advisor & Co-Designer

### Standards & Community (v1.3.0)
- **OWASP Foundation** - OWASP AI-SBOM Project for groundbreaking work in AI transparency and supply chain security
- **CycloneDX Community** - Open standard for Software Bill of Materials (SBOM)

### Acknowledgments
For complete attribution, see [CREDITS.md](CREDITS.md)

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🎯 Summary

**Hope Genome v1.3.0** brings together:
- ✅ Sidecar Pattern with Network Isolation
- ✅ OWASP AI-SBOM Compliance (CycloneDX 1.5+)
- ✅ Cryptographic Proofs & Fort Knox Integrity Enforcement
- ✅ Blockchain-style Audit Logs
- ✅ 71/71 Tests Passing (56 core, 12 security, 8 compliance, 3 doc)

**Remember**: Hope Genome is **not unhackable**. It makes attacks **tamper-evident** with cryptographic proof. 🔐

**NEW in v1.3.0**: Now with OWASP AI-SBOM compliance - the first active Runtime Enforcement layer for the OWASP AI-SBOM standard!
