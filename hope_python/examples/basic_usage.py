"""
Basic Usage Example - Hope Genome Python Bindings

This example demonstrates the fundamental workflow:
1. Create a genome with ethical rules
2. Seal it (make immutable)
3. Request action approval
4. Verify cryptographic proof
5. Check audit log
"""

import hope_genome


def main():
    print("=== Hope Genome v1.2 - Python Basic Usage Example ===\n")

    # Step 1: Create a genome with ethical rules
    print("Step 1: Creating genome with ethical rules...")
    genome = hope_genome.HopeGenome(rules=[
        "Do no harm to users",
        "Respect user privacy",
        "Ensure fairness and non-discrimination",
        "Be transparent about AI decision-making",
    ])
    print(f"   ✅ Genome created with {len(genome.rules())} rules")

    # Step 2: Seal the genome (make it immutable)
    print("\nStep 2: Sealing genome (making it immutable)...")
    genome.seal()
    print("   ✅ Genome sealed")
    print(f"   Capsule hash: {genome.capsule_hash()}")

    # Step 3: Create actions that need approval
    print("\nStep 3: Creating actions...")

    action1 = hope_genome.Action.read("user_profile.json")
    print("   📖 Action 1: Read user profile")

    action2 = hope_genome.Action.write_file("recommendation.txt", b"User recommendation data")
    print("   ✏️  Action 2: Write recommendation file")

    action3 = hope_genome.Action.delete("temp_cache.dat")
    print("   🗑️  Action 3: Delete temporary cache")

    # Step 4: Get cryptographic proofs for each action
    print("\nStep 4: Getting cryptographic proofs...")

    proof1 = genome.verify_action(action1)
    print("   ✅ Proof 1 generated:")
    print(f"      - Nonce: {proof1.nonce[:8].hex()}...")
    print(f"      - Timestamp: {proof1.timestamp_string()}")
    print(f"      - TTL: {proof1.ttl} seconds")
    print(f"      - Signature length: {len(proof1.signature)} bytes")

    proof2 = genome.verify_action(action2)
    print("   ✅ Proof 2 generated")

    proof3 = genome.verify_action(action3)
    print("   ✅ Proof 3 generated")

    # Step 5: Demonstrate audit logging
    print("\nStep 5: Creating audit log...")

    audit_log = hope_genome.AuditLog()

    audit_log.append(action1, proof1, approved=True)
    audit_log.append(action2, proof2, approved=True)
    audit_log.append(action3, proof3, approved=True)

    print(f"   ✅ Audit log created with {len(audit_log)} entries")

    # Step 6: Verify audit chain integrity
    print("\nStep 6: Verifying audit chain integrity...")

    audit_log.verify_chain()

    print("   ✅ Audit chain verified!")
    print(f"   All {len(audit_log)} entries are cryptographically linked")

    # Summary
    print("\n=== Summary ===")
    print(f"✅ Created and sealed genome with {len(genome.rules())} rules")
    print("✅ Generated 3 cryptographic proofs")
    print("✅ Created tamper-evident audit log")
    print("✅ Verified blockchain-style chain integrity")
    print("\n🎉 Hope Genome v1.2 Python bindings are working correctly!")


if __name__ == "__main__":
    main()
