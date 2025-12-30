/// Basic Usage Example for Hope Genome v1.2
///
/// This example demonstrates the fundamental workflow:
/// 1. Create a genome with ethical rules
/// 2. Seal it (make immutable)
/// 3. Request action approval
/// 4. Verify cryptographic proof
/// 5. Check audit log

use _hope_core::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Hope Genome v1.2 - Basic Usage Example ===\n");

    // Step 1: Create a genome with ethical rules
    println!("Step 1: Creating genome with ethical rules...");
    let mut genome = SealedGenome::new(vec![
        "Do no harm to users".to_string(),
        "Respect user privacy".to_string(),
        "Ensure fairness and non-discrimination".to_string(),
        "Be transparent about AI decision-making".to_string(),
    ])?;

    println!("   ✅ Genome created with {} rules", genome.rules().len());

    // Step 2: Seal the genome (make it immutable)
    println!("\nStep 2: Sealing genome (making it immutable)...");
    genome.seal()?;

    println!("   ✅ Genome sealed");
    println!("   Capsule hash: {}", genome.capsule_hash().unwrap());

    // Step 3: Create actions that need approval
    println!("\nStep 3: Creating actions...");

    let action1 = Action::read("user_profile.json");
    println!("   📖 Action 1: Read user profile");

    let action2 = Action::write_file("recommendation.txt", b"User recommendation data".to_vec());
    println!("   ✏️  Action 2: Write recommendation file");

    let action3 = Action::delete("temp_cache.dat");
    println!("   🗑️  Action 3: Delete temporary cache");

    // Step 4: Get cryptographic proofs for each action
    println!("\nStep 4: Getting cryptographic proofs...");

    let proof1 = genome.verify_action(&action1)?;
    println!("   ✅ Proof 1 generated:");
    println!("      - Nonce: {:?}...", &proof1.nonce[..8]);
    println!("      - Timestamp: {}", proof1.timestamp_string());
    println!("      - TTL: {} seconds", proof1.ttl);
    println!("      - Signature length: {} bytes", proof1.signature.len());

    let proof2 = genome.verify_action(&action2)?;
    println!("   ✅ Proof 2 generated");

    let proof3 = genome.verify_action(&action3)?;
    println!("   ✅ Proof 3 generated");

    // Step 5: Create an auditor to verify proofs
    println!("\nStep 5: Creating auditor...");

    // In production, the auditor would share the genome's public key
    // For this example, we'll create a separate auditor
    let auditor_keypair = KeyPair::generate()?;
    let mut auditor = ProofAuditor::new(auditor_keypair);

    println!("   ✅ Auditor created");

    // Note: In this example, verification will fail because genome and auditor
    // use different keypairs. In production, they would share the same public key.
    println!("\nStep 6: Demonstrating proof structure...");
    println!("   Proof contains:");
    println!("   - Nonce (anti-replay): {:?}...", &proof1.nonce[..8]);
    println!("   - Timestamp: {}", proof1.timestamp);
    println!("   - Action hash: {:?}...", &proof1.action_hash[..8]);
    println!("   - Action type: {:?}", proof1.action_type);
    println!("   - Capsule hash: {}", proof1.capsule_hash);
    println!("   - Signature: {} bytes", proof1.signature.len());

    // Step 7: Demonstrate audit logging
    println!("\nStep 7: Creating audit log...");

    let log_keypair = KeyPair::generate()?;
    let mut audit_log = AuditLog::new(log_keypair)?;

    audit_log.append(action1, proof1, Decision::Approved)?;
    audit_log.append(action2, proof2, Decision::Approved)?;
    audit_log.append(action3, proof3, Decision::Approved)?;

    println!("   ✅ Audit log created with {} entries", audit_log.len());

    // Step 8: Verify audit chain integrity
    println!("\nStep 8: Verifying audit chain integrity...");

    audit_log.verify_chain()?;

    println!("   ✅ Audit chain verified!");
    println!("   All {} entries are cryptographically linked", audit_log.len());

    // Step 9: Display audit trail
    println!("\nStep 9: Audit Trail:");
    println!("   ┌─────────────────────────────────────┐");

    for (i, entry) in audit_log.entries().iter().enumerate() {
        println!("   │ Entry {}: {:?}", i, entry.action.action_type);
        println!("   │   Timestamp: {}", entry.timestamp);
        println!("   │   Decision: {:?}", entry.decision);
        println!("   │   Chain: {:?}...", &entry.current_hash[..8]);
        if i < audit_log.len() - 1 {
            println!("   │   ↓");
        }
    }

    println!("   └─────────────────────────────────────┘");

    // Summary
    println!("\n=== Summary ===");
    println!("✅ Created and sealed genome with {} rules", genome.rules().len());
    println!("✅ Generated 3 cryptographic proofs");
    println!("✅ Created tamper-evident audit log");
    println!("✅ Verified blockchain-style chain integrity");
    println!("\n🎉 Hope Genome v1.2 is working correctly!");

    Ok(())
}
