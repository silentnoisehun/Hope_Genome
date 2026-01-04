# Security Policy

**Hope Genome v2.5.0 - Enterprise Edition**

---

## 🎯 Our Security Philosophy

> **"Not unhackable, but tamper-evident with cryptographic proof."**

Hope Genome is designed to make attacks **detectable**, not impossible. We prioritize:
1. **Transparency** over obscurity
2. **Evidence** over prevention
3. **Auditability** over invincibility

---

## 🛡️ Supported Versions

We actively support the following versions with security updates:

| Version | Supported          | End of Life | Security Level |
|---------|--------------------|-------------|----------------|
| 1.4.x   | ✅ **Active**       | 2026-06-30  | **Hardened**   |
| 1.3.x   | ⚠️ **Limited**      | 2026-03-31  | Standard       |
| 1.2.x   | ❌ **Unsupported**  | 2025-12-31  | Legacy         |
| < 1.2   | ❌ **Unsupported**  | 2025-09-30  | Legacy         |

**Recommendation:** Upgrade to v1.4.0 or later for critical security fixes.

---

## 🔐 Security Features (v1.4.0)

### ✅ **Protected Against**

| Attack Vector | Mitigation | Implementation | Status |
|---------------|------------|----------------|--------|
| **Marvin Attack** | Ed25519 (no padding) | `ed25519-dalek` crate | ✅ **Immune** |
| **Timing Attack** | Constant-time ops | Ed25519 by design | ✅ **Immune** |
| **Replay Attack (Pre-Restart)** | Nonce tracking | `NonceStore` trait | ✅ **Protected** |
| **Replay Attack (Post-Restart)** | Persistent nonces | RocksDB/Redis | ✅ **Protected** |
| **Cryptographic Forgery** | Ed25519 signatures | 256-bit security | ✅ **Protected** |
| **Oracle Attack** | Action binding | `action_hash` field | ✅ **Protected** |
| **TOCTOU** | Rust ownership | Borrow checker | ✅ **Protected** |
| **Log Tampering** | Blockchain chain | SHA-256 linkage | ✅ **Protected** |

### ⚠️ **Known Limitations**

| Threat | Risk Level | Mitigation | Status |
|--------|------------|------------|--------|
| **Root Access** | **Critical** | HSM + secure boot | 🔜 v1.5.0 |
| **Side-Channel (Power/EM)** | High | Use HSM in production | 🔜 v1.5.0 |
| **Sensor Manipulation** | Medium | Consensus verifier | ✅ Mitigated |
| **Supply Chain Attack** | Medium | OWASP AI-SBOM | ✅ Mitigated |

---

## 🚨 Reporting a Vulnerability

### **DO NOT** Create a Public Issue

Security vulnerabilities should be reported **privately** to protect users.

### Reporting Process

1. **Email:** stratosoiteam@gmail.com
   - Subject: `[SECURITY] Hope Genome - [Brief Description]`
   - Include:
     - Affected version(s)
     - Proof-of-concept (if safe)
     - Severity assessment (Critical/High/Medium/Low)
     - Suggested fix (if any)

2. **PGP Encryption (Optional):**
   - Public key: [Link to PGP key or fingerprint]
   - Fingerprint: `[TODO: Add PGP fingerprint]`

3. **Response Timeline:**
   - **Initial Response:** Within 48 hours
   - **Triage:** Within 7 days
   - **Fix ETA:** Depends on severity
     - Critical: 7-14 days
     - High: 14-30 days
     - Medium: 30-60 days
     - Low: Best effort

4. **Disclosure:**
   - **Coordinated Disclosure:** We follow a 90-day disclosure window
   - **Credit:** We acknowledge reporters (unless requested otherwise)
   - **CVE Assignment:** For eligible vulnerabilities

### Severity Classification

| Level | Criteria | Examples |
|-------|----------|----------|
| **Critical** | Remote code execution, authentication bypass | Buffer overflow, signature bypass |
| **High** | Data leakage, privilege escalation | Replay attack, nonce collision |
| **Medium** | DoS, information disclosure | Memory leak, timing attack |
| **Low** | Configuration issues, minor bugs | Verbose error messages |

---

## 🔍 Security Audit History

### Gemini Red Team Audit (2025-12-30)

**Auditor:** Google Gemini Red Team
**Version Tested:** 1.3.0
**Report Date:** 2025-12-25
**Score:** 8.5/10

**Critical Findings:**
1. ❌ **Marvin Attack** - RSA PKCS#1v15 vulnerable → **Fixed in v1.4.0** (Ed25519)
2. ❌ **Replay Post-Restart** - Nonces lost → **Fixed in v1.4.0** (RocksDB/Redis)
3. ⚠️ **No HSM Support** - Keys in memory → **Addressed in v1.4.0** (Architecture ready)

**Re-Audit Status:** Pending (v1.4.0)

---

## 🔧 Security Best Practices

### For Developers

1. **Use Latest Version:** Always use the latest stable release (v1.4.0+)
2. **Enable Persistent Nonces:**
   ```toml
   hope_core = { version = "1.4.0", features = ["rocksdb-nonce-store"] }
   ```
3. **Validate Proofs:** Always verify `IntegrityProof` before executing actions
4. **Rotate Keys:** Periodically rotate cryptographic keys (recommended: annually)
5. **Audit Logs:** Regularly review `AuditLog` for anomalies
6. **HSM in Production:** Use hardware security modules for production (v1.5.0+)

### For Operators

1. **Docker Hardening:**
   ```yaml
   # docker-compose.yml
   read_only: true
   cap_drop:
     - ALL
   security_opt:
     - no-new-privileges:true
   ```
2. **Network Isolation:** Run Hope Genome in isolated networks
3. **Resource Limits:** Set CPU/memory limits to prevent DoS
4. **Monitoring:** Monitor nonce store size and proof verification rates
5. **Backups:** Regularly backup nonce store (critical for replay protection)

### For Users

1. **Verify Signatures:** Use `ProofAuditor` to verify all proofs
2. **Check Capsule Hash:** Ensure genome hasn't been tampered with
3. **Report Anomalies:** Report suspicious behavior to stratosoiteam@gmail.com
4. **Update Dependencies:** Keep `hope_core` updated via `cargo update`

---

## 📜 Compliance & Standards

### Standards Followed

- **OWASP ASVS 4.0** - Application Security Verification Standard
- **OWASP AI-SBOM** - AI Software Bill of Materials (CycloneDX)
- **NIST Cybersecurity Framework** - Identify, Protect, Detect, Respond, Recover
- **ISO/IEC 27001** - Information Security Management (aligned)

### Cryptographic Standards

- **Ed25519** - FIPS 186-4 (digital signatures)
- **SHA-256** - FIPS 180-4 (hashing)
- **RFC 8032** - Edwards-Curve Digital Signature Algorithm

---

## 🔎 Security Testing

### Automated Testing

- **Unit Tests:** 79/79 passing (100% coverage on crypto, nonce store, auditor)
- **Integration Tests:** End-to-end workflows with proof verification
- **Fuzzing:** (TODO: AFL/libfuzzer for crypto module)
- **SAST:** Clippy lints with `-D warnings`

### Manual Testing

- **Code Review:** All PRs reviewed by maintainer
- **Threat Modeling:** STRIDE analysis for each major feature
- **Penetration Testing:** Annual red team engagement (Gemini)

---

## 📞 Contact

- **Security Email:** stratosoiteam@gmail.com
- **Maintainer:** Máté Róbert (@silentnoisehun)
- **GitHub:** [Hope_Genome](https://github.com/silentnoisehun/Hope_Genome)
- **Response Time:** 48 hours (weekdays), 72 hours (weekends)

---

## 🏆 Acknowledgments

We thank the following individuals and organizations for responsible disclosure:

| Reporter | Date | Vulnerability | Severity |
|----------|------|---------------|----------|
| Gemini Red Team | 2025-12-25 | Marvin Attack (RSA) | Critical |
| Gemini Red Team | 2025-12-25 | Replay Post-Restart | High |
| [Your Name Here] | YYYY-MM-DD | [Description] | [Severity] |

**Want to be listed?** Report a vulnerability responsibly!

---

## 📝 Changelog

| Date | Change | Version |
|------|--------|---------|
| 2025-12-30 | Ed25519 migration, persistent nonces | v1.4.0 |
| 2025-12-15 | OWASP AI-SBOM compliance | v1.3.0 |
| 2025-12-01 | Initial security policy | v1.2.0 |

---

**Last Updated:** 2025-12-30
**Version:** 1.4.0
**Status:** Active
