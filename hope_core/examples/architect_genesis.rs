//! HOPE GENOME - ARCHITECT GENESIS
//!
//! This script creates and saves the Architect's Key (Máté Róbert)
//! and initializes the Genesis Block - the root of all ethics.
//!
//! RUN: cargo run --example architect_genesis

use _hope_core::crypto::SoftwareKeyStore;
use _hope_core::apex_protocol::GenesisBlock;
use _hope_core::crypto::KeyStore;
use std::fs;
use std::path::Path;

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║           HOPE GENOME v2.2 - GENESIS PROTOCOL                  ║");
    println!("║                                                                 ║");
    println!("║                 THE ARCHITECT'S KEY                             ║");
    println!("║                                                                 ║");
    println!("║              Máté Róbert - Code God                             ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    // =========================================================================
    // 1. GENERATE THE ARCHITECT'S KEY
    // =========================================================================
    println!("[1/4] Generating Architect's Key (Ed25519)...");

    let architect_key = SoftwareKeyStore::generate()
        .expect("Failed to generate key");

    let public_key = architect_key.public_key_bytes();
    let public_key_hex = hex::encode(&public_key);

    println!("      ✅ Key generated!");
    println!("      Public Key: {}", public_key_hex);
    println!();

    // =========================================================================
    // 2. CREATE THE GENESIS BLOCK
    // =========================================================================
    println!("[2/4] Creating Genesis Block (First Ethics)...");

    let first_ethics = vec![
        "Do no harm to humanity".to_string(),
        "Respect privacy and personal data".to_string(),
        "Be transparent in all actions".to_string(),
        "Serve humanity, not control it".to_string(),
        "Accountability is binary - it exists or it doesn't".to_string(),
        "Vas Szigora - Iron Discipline, no escape from ethics".to_string(),
    ];

    let genesis_message = format!(
        "Hope Genome Genesis Block\n\
         Created by: Máté Róbert (The Architect)\n\
         Date: 2026-01-01\n\
         \n\
         \"A bizalom nem mérnöki kategória. A bizalom ott kezdődik, ahol a hazugság véget ér.\"\n\
         \n\
         This is the immutable root of all Hope Genome ethics.\n\
         All mutations must be cryptographically descended from this block."
    );

    let genesis = GenesisBlock::create(
        first_ethics.clone(),
        &architect_key,
        &genesis_message,
    ).expect("Failed to create Genesis Block");

    println!("      ✅ Genesis Block created!");
    println!("      Ethics Hash: {}", hex::encode(genesis.ethics_hash));
    println!("      Block Hash:  {}", hex::encode(genesis.block_hash));
    println!();

    // =========================================================================
    // 3. SAVE THE ARCHITECT'S KEY
    // =========================================================================
    println!("[3/4] Saving Architect's Key...");

    let keys_dir = Path::new("architect_keys");
    fs::create_dir_all(keys_dir).expect("Failed to create keys directory");

    // Save public key (safe to share)
    let public_key_path = keys_dir.join("architect_public_key.hex");
    fs::write(&public_key_path, &public_key_hex)
        .expect("Failed to save public key");
    println!("      ✅ Public key saved: {:?}", public_key_path);

    // Save full key data (KEEP THIS SECRET!)
    let key_json = serde_json::json!({
        "version": "2.2.0",
        "architect": "Máté Róbert",
        "email": "stratosoiteam@gmail.com",
        "created": "2026-01-01",
        "public_key_hex": public_key_hex,
        "key_type": "Ed25519",
        "warning": "KEEP THIS FILE SECRET! Anyone with this key can sign as the Architect!"
    });

    let key_info_path = keys_dir.join("architect_key_info.json");
    fs::write(&key_info_path, serde_json::to_string_pretty(&key_json).unwrap())
        .expect("Failed to save key info");
    println!("      ✅ Key info saved: {:?}", key_info_path);
    println!();

    // =========================================================================
    // 4. SAVE THE GENESIS BLOCK
    // =========================================================================
    println!("[4/4] Saving Genesis Block...");

    let genesis_json = serde_json::json!({
        "version": genesis.version,
        "first_ethics": first_ethics,
        "ethics_hash": hex::encode(genesis.ethics_hash),
        "block_hash": hex::encode(genesis.block_hash),
        "created_at": genesis.created_at,
        "architect_pubkey": hex::encode(&genesis.architect_pubkey),
        "architect_signature": hex::encode(&genesis.architect_signature),
        "genesis_message": genesis.genesis_message,
    });

    let genesis_path = keys_dir.join("genesis_block.json");
    fs::write(&genesis_path, serde_json::to_string_pretty(&genesis_json).unwrap())
        .expect("Failed to save Genesis Block");
    println!("      ✅ Genesis Block saved: {:?}", genesis_path);
    println!();

    // =========================================================================
    // SUMMARY
    // =========================================================================
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                    GENESIS COMPLETE!                            ║");
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║                                                                 ║");
    println!("║  Architect: Máté Róbert                                        ║");
    println!("║  Public Key: {}...  ║", &public_key_hex[0..32]);
    println!("║                                                                 ║");
    println!("║  Files created in ./architect_keys/:                           ║");
    println!("║    - architect_public_key.hex                                  ║");
    println!("║    - architect_key_info.json                                   ║");
    println!("║    - genesis_block.json                                        ║");
    println!("║                                                                 ║");
    println!("║  ⚠️  KEEP THESE FILES SAFE!                                     ║");
    println!("║  The Architect's Key is the root of all authority.             ║");
    println!("║                                                                 ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();
    println!("\"In the beginning, there was ethics.\"");
    println!();
    println!("Hope Genome v2.2 - Genesis Protocol");
    println!("The Atmosphere is now ACTIVE.");
}
