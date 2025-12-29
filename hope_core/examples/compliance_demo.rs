//! # Hope Genome AIBOM Compliance Demo
//!
//! This example demonstrates how to use the OWASP AIBOM (CycloneDX) compliance
//! module for AI model integrity verification.
//!
//! Run with: cargo run --example compliance_demo

use hope_core::compliance::*;
use hope_core::crypto::hash_bytes;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("=== Hope Genome v1.3.0 - AIBOM Compliance Demo ===\n");

    // 1. Load AIBOM file
    println!("📄 Loading AIBOM file...");
    let aibom_path = "examples/example_model.aibom.json";
    let aibom = AiBom::from_file(aibom_path)?;

    println!("   ✅ Loaded AIBOM:");
    println!("      Format: {}", aibom.bom_format);
    println!("      Spec Version: {}", aibom.spec_version);
    println!("      Components: {}\n", aibom.components.len());

    // 2. Find specific AI model component
    println!("🔍 Finding AI model component...");
    let model_name = "medical-diagnosis-model";
    let component = aibom.find_component(model_name)?;

    println!("   ✅ Found component:");
    println!("      Name: {}", component.name);
    println!("      Type: {}", component.component_type);
    println!("      Version: {}", component.version.as_ref().unwrap_or(&"N/A".to_string()));
    println!("      Hashes: {}\n", component.hashes.len());

    // 3. Extract SHA-256 hash from SBOM
    println!("🔐 Extracting cryptographic hash...");
    let sbom_hash = component.get_hash("SHA-256")?;
    println!("   ✅ SBOM Hash (SHA-256):");
    println!("      {}\n", sbom_hash);

    // 4. Simulate runtime hash computation
    println!("⚙️  Simulating runtime hash computation...");
    // In a real scenario, this would be the hash of the actual model file
    let model_data = b""; // Empty string has the same hash as in our example
    let runtime_hash_bytes = hash_bytes(model_data);
    let runtime_hash = hex::encode(runtime_hash_bytes);

    println!("   ✅ Runtime Hash (SHA-256):");
    println!("      {}\n", runtime_hash);

    // 5. Validate integrity - SUCCESS CASE
    println!("✔️  Validating integrity (matching hashes)...");
    match validate_integrity(model_name, sbom_hash, &runtime_hash) {
        Ok(()) => {
            println!("   ✅ SUCCESS: Hash validation passed!");
            println!("      Model integrity verified ✓\n");
        }
        Err(e) => {
            println!("   ❌ FAILED: {}\n", e);
        }
    }

    // 6. Demonstrate Fort Knox error - FAILURE CASE
    println!("⚠️  Demonstrating Fort Knox error (tampered hash)...");
    let tampered_hash = "TAMPERED_HASH_VALUE_INVALID";

    match validate_integrity(model_name, sbom_hash, tampered_hash) {
        Ok(()) => {
            println!("   ⚠️  WARNING: Validation passed (unexpected)\n");
        }
        Err(e) => {
            println!("   ❌ FORT KNOX TRIGGERED:");
            println!("      {}\n", e);
        }
    }

    // 7. Find all ML models in AIBOM
    println!("📊 Finding all machine learning models...");
    let ml_models = aibom.find_components_by_type("machine-learning-model");
    println!("   ✅ Found {} ML models:", ml_models.len());
    for model in ml_models {
        println!("      - {} (v{})",
            model.name,
            model.version.as_ref().unwrap_or(&"N/A".to_string())
        );
    }
    println!();

    // 8. Demonstrate complete workflow with convenience function
    println!("🚀 Complete workflow with convenience function...");
    match validate_component_integrity(
        aibom_path,
        model_name,
        "SHA-256",
        &runtime_hash,
    ) {
        Ok(()) => {
            println!("   ✅ Complete validation workflow successful!\n");
        }
        Err(e) => {
            println!("   ❌ Validation failed: {}\n", e);
        }
    }

    // 9. Display component properties (if available)
    if let Some(props) = &component.properties {
        println!("📋 Model properties:");
        for prop in props {
            println!("   {} = {}", prop.name, prop.value);
        }
        println!();
    }

    // 10. Check for specific hash algorithm
    println!("🔍 Checking available hash algorithms...");
    for algo in &["SHA-256", "SHA-512", "MD5"] {
        let has_algo = component.has_hash(algo);
        let status = if has_algo { "✅" } else { "❌" };
        println!("   {} {}", status, algo);
    }
    println!();

    println!("=== Demo Complete ===");
    println!("\n💡 Key Takeaways:");
    println!("   • AIBOM provides cryptographic proof of AI model integrity");
    println!("   • Fort Knox errors halt transactions on hash mismatch");
    println!("   • Constant-time comparison prevents timing attacks");
    println!("   • CycloneDX standard ensures interoperability");
    println!("   • Hope Genome v1.3.0 is production-ready for AI compliance\n");

    Ok(())
}
