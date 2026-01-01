#!/bin/bash
# ============================================================
# HOPE GENOME - DOCKER ENTRYPOINT
# ============================================================
#
# Usage:
#   docker run hope-genome help      - Show this help
#   docker run hope-genome demo      - Run Watchdog demo
#   docker run hope-genome audit     - Run Blockchain Audit demo
#   docker run hope-genome test      - Run all tests
#   docker run hope-genome shell     - Python shell with hope_genome
#   docker run hope-genome version   - Show version
#
# Mate Robert + Claude
# VAS SZIGORA - 2026.01.01.
# ============================================================

set -e

VERSION="1.7.1"

show_banner() {
    echo ""
    echo "+============================================================+"
    echo "|                      HOPE GENOME                           |"
    echo "|     Tamper-Evident Cryptographic AI Accountability         |"
    echo "|                                                            |"
    echo "|     Version: $VERSION - VAS SZIGORA Edition                |"
    echo "|     Mate Robert + Claude - 2026.01.01.                     |"
    echo "+============================================================+"
    echo ""
}

show_help() {
    show_banner
    echo "Usage: docker run hope-genome [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  help      Show this help message"
    echo "  demo      Run Watchdog demonstration"
    echo "  audit     Run Blockchain Audit demonstration"
    echo "  test      Run all tests"
    echo "  shell     Start Python shell with hope_genome imported"
    echo "  version   Show version information"
    echo "  python    Run custom Python script"
    echo ""
    echo "Examples:"
    echo "  docker run hope-genome demo"
    echo "  docker run hope-genome audit"
    echo "  docker run -it hope-genome shell"
    echo "  docker run -v ./my_audit:/app/audit_chain hope-genome audit"
    echo ""
    echo "Links:"
    echo "  GitHub: https://github.com/silentnoisehun/Hope_Genome"
    echo "  PyPI:   https://pypi.org/project/hope-genome/"
    echo "  Docs:   https://silentnoisehun.github.io/Hope_Genome/"
    echo ""
}

run_demo() {
    show_banner
    echo "[DEMO] Running Watchdog Demonstration..."
    echo ""
    python3 << 'EOF'
import hope_genome as hg

print("=" * 60)
print("HOPE GENOME WATCHDOG DEMO")
print("=" * 60)
print()

# Create Watchdog with rules
print("[1] Creating Watchdog with ethical rules...")
watchdog = hg.Watchdog(
    rules=[
        "Do no harm",
        "Respect user privacy",
        "No unauthorized data access",
        "Provide transparent explanations"
    ],
    capsule_hash="demo_capsule_v1"
)
print("    Rules sealed with Ed25519 signature")
print()

# Test approved action
print("[2] Testing APPROVED action...")
action = hg.Action.execute_command("calculate(2 + 2)")
result = watchdog.verify_action(action)
if result.approved:
    print("    [OK] Action APPROVED: calculate(2 + 2)")
print()

# Test denied action
print("[3] Testing DENIED action...")
action = hg.Action.delete_file("/etc/passwd")
result = watchdog.verify_action(action)
if not result.approved:
    print("    [X] Action DENIED: delete /etc/passwd")
    if result.denial_proof:
        print(f"    Reason: {result.denial_proof.denial_reason}")
        print(f"    Violation Count: {result.denial_proof.violation_count}/10")
print()

# Summary
print("=" * 60)
print("DEMO COMPLETE!")
print()
print("The Watchdog enforces ethical rules at runtime.")
print("Every denial is cryptographically signed and logged.")
print()
print("VAS SZIGORA - Iron Discipline. No escape from ethics.")
print("=" * 60)
EOF
}

run_audit() {
    show_banner
    echo "[AUDIT] Running Blockchain Audit Demonstration..."
    echo ""
    python3 /app/blockchain_audit.py
}

run_tests() {
    show_banner
    echo "[TEST] Running Hope Genome Tests..."
    echo ""
    python3 << 'EOF'
import hope_genome as hg

print("=" * 60)
print("HOPE GENOME TESTS")
print("=" * 60)
print()

tests_passed = 0
tests_failed = 0

# Test 1: SealedGenome
print("[TEST 1] SealedGenome creation...")
try:
    genome = hg.SealedGenome(rules=["Do no harm", "Respect privacy"])
    genome.seal()
    print("    [OK] SealedGenome created and sealed")
    tests_passed += 1
except Exception as e:
    print(f"    [FAIL] {e}")
    tests_failed += 1

# Test 2: Watchdog
print("[TEST 2] Watchdog creation...")
try:
    watchdog = hg.Watchdog(rules=["No harm"], capsule_hash="test")
    print("    [OK] Watchdog created")
    tests_passed += 1
except Exception as e:
    print(f"    [FAIL] {e}")
    tests_failed += 1

# Test 3: Action verification
print("[TEST 3] Action verification...")
try:
    action = hg.Action.execute_command("test")
    result = watchdog.verify_action(action)
    print("    [OK] Action verified")
    tests_passed += 1
except Exception as e:
    print(f"    [FAIL] {e}")
    tests_failed += 1

# Test 4: Denial proof
print("[TEST 4] Denial proof generation...")
try:
    action = hg.Action.delete_file("/etc/passwd")
    result = watchdog.verify_action(action)
    if result.denial_proof:
        print("    [OK] Denial proof generated with signature")
        tests_passed += 1
    else:
        print("    [FAIL] No denial proof")
        tests_failed += 1
except Exception as e:
    print(f"    [FAIL] {e}")
    tests_failed += 1

# Summary
print()
print("=" * 60)
print(f"RESULTS: {tests_passed} passed, {tests_failed} failed")
if tests_failed == 0:
    print("[OK] ALL TESTS PASSED!")
else:
    print("[X] SOME TESTS FAILED")
print("=" * 60)
EOF
}

run_shell() {
    show_banner
    echo "[SHELL] Starting Python shell with hope_genome..."
    echo ""
    python3 -i << 'EOF'
import hope_genome as hg
print("Hope Genome imported as 'hg'")
print("")
print("Quick start:")
print("  watchdog = hg.Watchdog(rules=['Do no harm'], capsule_hash='my_hash')")
print("  action = hg.Action.execute_command('some_command')")
print("  result = watchdog.verify_action(action)")
print("")
EOF
}

show_version() {
    echo "Hope Genome v$VERSION"
    echo "VAS SZIGORA Edition"
    echo ""
    python3 -c "import hope_genome; print(f'Python module: {hope_genome.__file__}')"
}

# Main command handler
case "${1:-help}" in
    help|--help|-h)
        show_help
        ;;
    demo)
        run_demo
        ;;
    audit)
        run_audit
        ;;
    test|tests)
        run_tests
        ;;
    shell|interactive)
        run_shell
        ;;
    version|--version|-v)
        show_version
        ;;
    python)
        shift
        python3 "$@"
        ;;
    *)
        echo "Unknown command: $1"
        echo "Run 'docker run hope-genome help' for usage"
        exit 1
        ;;
esac
