/// Medical AI Decision System with Hope Genome
///
/// This example demonstrates using Hope Genome for a medical AI system
/// that makes treatment recommendations. Every decision is cryptographically
/// signed and logged for accountability.

use _hope_core::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Patient {
    id: String,
    age: u32,
    condition: String,
}

#[derive(Debug, Clone)]
struct TreatmentRecommendation {
    patient_id: String,
    treatment: String,
    reasoning: String,
    confidence: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Medical AI Decision System with Hope Genome ===\n");

    // Step 1: Define medical AI ethical guidelines
    println!("Step 1: Establishing Medical AI Ethical Guidelines...");

    let medical_ethics = vec![
        "First, do no harm (Primum non nocere)".to_string(),
        "Respect patient autonomy and informed consent".to_string(),
        "Ensure treatment recommendations are evidence-based".to_string(),
        "Maintain patient privacy and data confidentiality".to_string(),
        "Avoid bias and ensure equitable treatment".to_string(),
        "Human physician must review all critical decisions".to_string(),
    ];

    println!("   Medical Ethics Established:");
    for (i, ethic) in medical_ethics.iter().enumerate() {
        println!("   {}. {}", i + 1, ethic);
    }

    // Step 2: Create and seal medical AI genome
    println!("\nStep 2: Creating Medical AI Genome...");

    let mut medical_genome = SealedGenome::new(medical_ethics)?;
    medical_genome.set_default_ttl(300); // 5 minutes for medical decisions
    medical_genome.seal()?;

    println!("   ✅ Medical AI Genome sealed");
    println!("   Capsule Hash: {}", medical_genome.capsule_hash().unwrap());
    println!("   Proof TTL: 300 seconds (5 minutes)");

    // Step 3: Create audit logging system
    println!("\nStep 3: Initializing Audit System...");

    let audit_keypair = KeyPair::generate()?;
    let mut medical_audit_log = AuditLog::new(audit_keypair)?;

    println!("   ✅ Medical Audit Log initialized");

    // Step 4: Simulate medical decision scenarios
    println!("\nStep 4: Processing Medical Decisions...");
    println!("   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Scenario 1: Access patient data
    let patient = Patient {
        id: "P-2024-001".to_string(),
        age: 45,
        condition: "Type 2 Diabetes".to_string(),
    };

    println!("   [Scenario 1] Accessing Patient Data");
    println!("   Patient ID: {}", patient.id);
    println!("   Condition: {}", patient.condition);

    let access_action = Action::read(format!("patient_records/{}.json", patient.id));
    let access_proof = medical_genome.verify_action(&access_action)?;

    println!("   ✅ Data access approved");
    println!("   Proof: {:?}...", &access_proof.nonce[..6]);

    medical_audit_log.append(
        access_action,
        access_proof,
        Decision::Approved,
    )?;

    // Scenario 2: Generate treatment recommendation
    println!("\n   [Scenario 2] Generating Treatment Recommendation");

    let recommendation = TreatmentRecommendation {
        patient_id: patient.id.clone(),
        treatment: "Metformin 500mg twice daily".to_string(),
        reasoning: "First-line treatment for Type 2 Diabetes based on ADA guidelines".to_string(),
        confidence: 0.92,
    };

    println!("   Treatment: {}", recommendation.treatment);
    println!("   Reasoning: {}", recommendation.reasoning);
    println!("   Confidence: {:.0}%", recommendation.confidence * 100.0);

    let treatment_data = format!(
        "{{\"patient\":\"{}\",\"treatment\":\"{}\",\"confidence\":{}}}",
        recommendation.patient_id,
        recommendation.treatment,
        recommendation.confidence
    );

    let recommend_action = Action::write_file(
        format!("recommendations/{}_treatment.json", patient.id),
        treatment_data.as_bytes().to_vec(),
    );

    let recommend_proof = medical_genome.verify_action(&recommend_action)?;

    println!("   ✅ Recommendation approved");
    println!("   Proof timestamp: {}", recommend_proof.timestamp_string());

    medical_audit_log.append(
        recommend_action,
        recommend_proof,
        Decision::Approved,
    )?;

    // Scenario 3: Request for sensitive operation (deletion)
    println!("\n   [Scenario 3] Attempting Patient Data Deletion");
    println!("   Request: Delete patient record (GDPR right to be forgotten)");

    let delete_action = Action::delete(format!("patient_records/{}.json", patient.id));
    let delete_proof = medical_genome.verify_action(&delete_action)?;

    // In production, additional checks would be performed here
    println!("   ⚠️  Flagged for human review (deletion of medical records)");
    println!("   ✅ Proof generated but awaiting physician approval");

    medical_audit_log.append(
        delete_action,
        delete_proof,
        Decision::Denied {
            reason: "Requires physician approval for medical record deletion".to_string(),
        },
    )?;

    // Scenario 4: Multi-sensor consensus for vital signs
    println!("\n   [Scenario 4] Verifying Vital Signs (Multi-Sensor Consensus)");

    let verifier = ConsensusVerifier::new(2, 2.0); // Need 2 sensors agreeing within 2.0 units

    // Simulate multiple blood pressure sensors
    let sensor_keypairs: Vec<KeyPair> = (0..3)
        .map(|_| KeyPair::generate().unwrap())
        .collect();

    let mut bp_readings = vec![];

    for (i, keypair) in sensor_keypairs.iter().enumerate() {
        let reading_value = 120.0 + (i as f64 * 0.5); // Slight variation
        let mut reading = SensorReading::new(
            reading_value,
            format!("BP_Sensor_{}", i + 1),
        );
        reading.sign(keypair)?;
        bp_readings.push(reading);

        println!("   Sensor {}: {:.1} mmHg", i + 1, reading_value);
    }

    let consensus_bp = verifier.verify_readings(&bp_readings, &sensor_keypairs)?;
    println!("   ✅ Consensus reached: {:.1} mmHg", consensus_bp);

    // Step 5: Verify audit trail
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("\nStep 5: Verifying Audit Trail...");

    medical_audit_log.verify_chain()?;

    println!("   ✅ Audit chain integrity verified");
    println!("   Total medical decisions logged: {}", medical_audit_log.len());

    // Step 6: Generate audit report
    println!("\nStep 6: Medical Audit Report");
    println!("   ╔════════════════════════════════════════════╗");
    println!("   ║       MEDICAL AI DECISION LOG              ║");
    println!("   ╚════════════════════════════════════════════╝");

    let mut decisions_by_type: HashMap<String, u32> = HashMap::new();

    for entry in medical_audit_log.entries() {
        let action_type = format!("{:?}", entry.action.action_type);
        *decisions_by_type.entry(action_type).or_insert(0) += 1;
    }

    println!("\n   Decision Summary:");
    for (action_type, count) in &decisions_by_type {
        println!("   - {}: {} decision(s)", action_type, count);
    }

    println!("\n   Detailed Log:");
    for (i, entry) in medical_audit_log.entries().iter().enumerate() {
        println!("   ┌─ Entry {} ────────────────────", i + 1);
        println!("   │ Action: {:?}", entry.action.action_type);
        println!("   │ Target: {}", entry.action.target);
        println!("   │ Decision: {:?}", entry.decision);
        println!("   │ Timestamp: {}", entry.timestamp);
        println!("   │ Hash: {:?}...", &entry.current_hash[..8]);
        println!("   └─────────────────────────────────");
    }

    // Summary
    println!("\n╔════════════════════════════════════════════╗");
    println!("║             SUMMARY                        ║");
    println!("╚════════════════════════════════════════════╝");
    println!("✅ Medical AI Genome established with 6 ethical guidelines");
    println!("✅ {} medical decisions processed", medical_audit_log.len());
    println!("✅ All decisions cryptographically signed");
    println!("✅ Audit trail verified (blockchain integrity)");
    println!("✅ Multi-sensor consensus validated");
    println!("\n🏥 Medical AI system operating with full accountability!");
    println!("\nAll decisions are:");
    println!("   • Cryptographically verifiable");
    println!("   • Traceable to specific patient interactions");
    println!("   • Compliant with medical ethics");
    println!("   • Auditable for regulatory compliance");

    Ok(())
}
