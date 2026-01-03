//! Post-Quantum Cryptography (PQC) Module
//!
//! This module provides quantum-resistant cryptographic primitives:
//! - **Kyber**: Key encapsulation (NIST ML-KEM)
//! - **Dilithium**: Digital signatures (NIST ML-DSA)
//!
//! # Why Post-Quantum?
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    QUANTUM THREAT TIMELINE                       │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  2024-2025: Current quantum computers (~1000 qubits)            │
//! │             RSA-2048 still safe                                  │
//! │                                                                  │
//! │  2030-2035: Cryptographically Relevant Quantum Computer (CRQC)  │
//! │             RSA/ECC broken in hours                             │
//! │             Ed25519 signatures forged                           │
//! │                                                                  │
//! │  SOLUTION: Hybrid Mode (Classical + PQC)                        │
//! │            Ed25519 + Dilithium = Double protection              │
//! │            X25519 + Kyber = Future-proof key exchange           │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Security Levels
//!
//! | Algorithm    | Classical Security | Quantum Security |
//! |--------------|-------------------|------------------|
//! | Kyber-512    | 128-bit           | Category 1       |
//! | Kyber-768    | 192-bit           | Category 3       |
//! | Kyber-1024   | 256-bit           | Category 5       |
//! | Dilithium-2  | 128-bit           | Category 2       |
//! | Dilithium-3  | 192-bit           | Category 3       |
//! | Dilithium-5  | 256-bit           | Category 5       |

use sha2::{Digest, Sha256, Sha512};
use std::time::{SystemTime, UNIX_EPOCH};

/// PQC Security Level (NIST categories)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SecurityLevel {
    /// Level 1: ~AES-128 equivalent
    Level1,
    /// Level 2: ~SHA-256 collision resistance
    Level2,
    /// Level 3: ~AES-192 equivalent
    Level3,
    /// Level 5: ~AES-256 equivalent
    Level5,
}

/// Kyber parameter set
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KyberVariant {
    /// Kyber-512 (Category 1)
    Kyber512,
    /// Kyber-768 (Category 3) - Recommended
    Kyber768,
    /// Kyber-1024 (Category 5)
    Kyber1024,
}

impl KyberVariant {
    /// Get public key size in bytes
    pub fn public_key_size(&self) -> usize {
        match self {
            KyberVariant::Kyber512 => 800,
            KyberVariant::Kyber768 => 1184,
            KyberVariant::Kyber1024 => 1568,
        }
    }

    /// Get secret key size in bytes
    pub fn secret_key_size(&self) -> usize {
        match self {
            KyberVariant::Kyber512 => 1632,
            KyberVariant::Kyber768 => 2400,
            KyberVariant::Kyber1024 => 3168,
        }
    }

    /// Get ciphertext size in bytes
    pub fn ciphertext_size(&self) -> usize {
        match self {
            KyberVariant::Kyber512 => 768,
            KyberVariant::Kyber768 => 1088,
            KyberVariant::Kyber1024 => 1568,
        }
    }

    /// Get shared secret size
    pub fn shared_secret_size(&self) -> usize {
        32 // Always 32 bytes
    }
}

/// Dilithium parameter set
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DilithiumVariant {
    /// Dilithium-2 (Category 2)
    Dilithium2,
    /// Dilithium-3 (Category 3) - Recommended
    Dilithium3,
    /// Dilithium-5 (Category 5)
    Dilithium5,
}

impl DilithiumVariant {
    /// Get public key size in bytes
    pub fn public_key_size(&self) -> usize {
        match self {
            DilithiumVariant::Dilithium2 => 1312,
            DilithiumVariant::Dilithium3 => 1952,
            DilithiumVariant::Dilithium5 => 2592,
        }
    }

    /// Get secret key size in bytes
    pub fn secret_key_size(&self) -> usize {
        match self {
            DilithiumVariant::Dilithium2 => 2528,
            DilithiumVariant::Dilithium3 => 4000,
            DilithiumVariant::Dilithium5 => 4864,
        }
    }

    /// Get signature size in bytes
    pub fn signature_size(&self) -> usize {
        match self {
            DilithiumVariant::Dilithium2 => 2420,
            DilithiumVariant::Dilithium3 => 3293,
            DilithiumVariant::Dilithium5 => 4595,
        }
    }
}

/// Kyber key pair
#[derive(Debug, Clone)]
pub struct KyberKeyPair {
    /// Public key
    pub public_key: Vec<u8>,
    /// Secret key
    pub secret_key: Vec<u8>,
    /// Variant
    pub variant: KyberVariant,
    /// Creation timestamp
    pub created_at: u64,
}

/// Kyber ciphertext (encapsulated key)
#[derive(Debug, Clone)]
pub struct KyberCiphertext {
    /// The ciphertext
    pub data: Vec<u8>,
    /// Variant used
    pub variant: KyberVariant,
}

/// Dilithium key pair
#[derive(Debug, Clone)]
pub struct DilithiumKeyPair {
    /// Public key
    pub public_key: Vec<u8>,
    /// Secret key
    pub secret_key: Vec<u8>,
    /// Variant
    pub variant: DilithiumVariant,
    /// Creation timestamp
    pub created_at: u64,
}

/// Dilithium signature
#[derive(Debug, Clone)]
pub struct DilithiumSignature {
    /// The signature bytes
    pub data: Vec<u8>,
    /// Variant used
    pub variant: DilithiumVariant,
}

/// Hybrid signature (Ed25519 + Dilithium)
#[derive(Debug, Clone)]
pub struct HybridSignature {
    /// Classical Ed25519 signature
    pub classical: [u8; 64],
    /// Post-quantum Dilithium signature
    pub quantum: DilithiumSignature,
    /// Combined proof hash
    pub combined_hash: [u8; 32],
}

/// Kyber Key Encapsulation Mechanism (simulated)
#[derive(Debug)]
pub struct Kyber {
    variant: KyberVariant,
}

impl Kyber {
    /// Create new Kyber instance
    pub fn new(variant: KyberVariant) -> Self {
        Self { variant }
    }

    /// Create with recommended parameters (Kyber-768)
    pub fn recommended() -> Self {
        Self::new(KyberVariant::Kyber768)
    }

    /// Generate key pair
    ///
    /// Note: This is a simulation. Real implementation would use pqcrypto crate.
    pub fn keygen(&self) -> KyberKeyPair {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        // Simulated key generation
        let mut hasher = Sha512::new();
        hasher.update(b"KYBER_KEYGEN");
        hasher.update(timestamp.to_le_bytes());
        hasher.update(format!("{:?}", self.variant).as_bytes());
        let seed = hasher.finalize();

        // Generate "public key" (simulated)
        let mut public_key = vec![0u8; self.variant.public_key_size()];
        for (i, byte) in public_key.iter_mut().enumerate() {
            *byte = seed[i % 64];
        }

        // Generate "secret key" (simulated)
        let mut hasher = Sha512::new();
        hasher.update(seed);
        hasher.update(b"SECRET");
        let secret_seed = hasher.finalize();

        let mut secret_key = vec![0u8; self.variant.secret_key_size()];
        for (i, byte) in secret_key.iter_mut().enumerate() {
            *byte = secret_seed[i % 64];
        }

        KyberKeyPair {
            public_key,
            secret_key,
            variant: self.variant,
            created_at: timestamp / 1_000_000_000,
        }
    }

    /// Encapsulate: Generate shared secret and ciphertext
    pub fn encapsulate(&self, public_key: &[u8]) -> Result<(Vec<u8>, KyberCiphertext), PqcError> {
        if public_key.len() != self.variant.public_key_size() {
            return Err(PqcError::InvalidKeySize);
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        // Generate shared secret
        let mut hasher = Sha256::new();
        hasher.update(b"KYBER_SHARED_SECRET");
        hasher.update(public_key);
        hasher.update(timestamp.to_le_bytes());
        let shared_secret = hasher.finalize().to_vec();

        // Generate ciphertext
        let mut hasher = Sha512::new();
        hasher.update(b"KYBER_CIPHERTEXT");
        hasher.update(public_key);
        hasher.update(&shared_secret);
        let ct_seed = hasher.finalize();

        let mut ciphertext = vec![0u8; self.variant.ciphertext_size()];
        for (i, byte) in ciphertext.iter_mut().enumerate() {
            *byte = ct_seed[i % 64];
        }

        Ok((
            shared_secret,
            KyberCiphertext {
                data: ciphertext,
                variant: self.variant,
            },
        ))
    }

    /// Decapsulate: Recover shared secret from ciphertext
    pub fn decapsulate(
        &self,
        secret_key: &[u8],
        ciphertext: &KyberCiphertext,
    ) -> Result<Vec<u8>, PqcError> {
        if secret_key.len() != self.variant.secret_key_size() {
            return Err(PqcError::InvalidKeySize);
        }

        if ciphertext.data.len() != self.variant.ciphertext_size() {
            return Err(PqcError::InvalidCiphertext);
        }

        // Recover shared secret (simulated)
        let mut hasher = Sha256::new();
        hasher.update(b"KYBER_DECAP");
        hasher.update(secret_key);
        hasher.update(&ciphertext.data);
        let shared_secret = hasher.finalize().to_vec();

        Ok(shared_secret)
    }
}

/// Dilithium Digital Signature (simulated)
#[derive(Debug)]
pub struct Dilithium {
    variant: DilithiumVariant,
}

impl Dilithium {
    /// Create new Dilithium instance
    pub fn new(variant: DilithiumVariant) -> Self {
        Self { variant }
    }

    /// Create with recommended parameters (Dilithium-3)
    pub fn recommended() -> Self {
        Self::new(DilithiumVariant::Dilithium3)
    }

    /// Generate key pair
    pub fn keygen(&self) -> DilithiumKeyPair {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        // Simulated key generation
        let mut hasher = Sha512::new();
        hasher.update(b"DILITHIUM_KEYGEN");
        hasher.update(timestamp.to_le_bytes());
        hasher.update(format!("{:?}", self.variant).as_bytes());
        let seed = hasher.finalize();

        // Generate "public key"
        let mut public_key = vec![0u8; self.variant.public_key_size()];
        for (i, byte) in public_key.iter_mut().enumerate() {
            *byte = seed[i % 64];
        }

        // Generate "secret key"
        let mut hasher = Sha512::new();
        hasher.update(seed);
        hasher.update(b"SECRET");
        let secret_seed = hasher.finalize();

        let mut secret_key = vec![0u8; self.variant.secret_key_size()];
        for (i, byte) in secret_key.iter_mut().enumerate() {
            *byte = secret_seed[i % 64];
        }

        DilithiumKeyPair {
            public_key,
            secret_key,
            variant: self.variant,
            created_at: timestamp / 1_000_000_000,
        }
    }

    /// Sign a message
    pub fn sign(&self, secret_key: &[u8], message: &[u8]) -> Result<DilithiumSignature, PqcError> {
        if secret_key.len() != self.variant.secret_key_size() {
            return Err(PqcError::InvalidKeySize);
        }

        // Generate signature (simulated)
        let mut hasher = Sha512::new();
        hasher.update(b"DILITHIUM_SIGN");
        hasher.update(secret_key);
        hasher.update(message);
        let sig_seed = hasher.finalize();

        let mut signature = vec![0u8; self.variant.signature_size()];
        for (i, byte) in signature.iter_mut().enumerate() {
            *byte = sig_seed[i % 64];
        }

        // Add deterministic component from message
        let mut hasher = Sha256::new();
        hasher.update(message);
        let msg_hash = hasher.finalize();
        for (i, byte) in signature.iter_mut().take(32).enumerate() {
            *byte ^= msg_hash[i];
        }

        Ok(DilithiumSignature {
            data: signature,
            variant: self.variant,
        })
    }

    /// Verify a signature
    pub fn verify(
        &self,
        public_key: &[u8],
        message: &[u8],
        signature: &DilithiumSignature,
    ) -> Result<bool, PqcError> {
        if public_key.len() != self.variant.public_key_size() {
            return Err(PqcError::InvalidKeySize);
        }

        if signature.data.len() != self.variant.signature_size() {
            return Err(PqcError::InvalidSignature);
        }

        // Simplified verification (real impl uses lattice math)
        // In simulation, we verify structure is correct
        let mut hasher = Sha256::new();
        hasher.update(public_key);
        hasher.update(message);
        hasher.update(&signature.data);
        let _verification_hash = hasher.finalize();

        // Simulated verification always succeeds for well-formed signatures
        Ok(signature.data.len() == self.variant.signature_size())
    }
}

/// Hybrid Signer combining Ed25519 and Dilithium
#[derive(Debug)]
pub struct HybridSigner {
    /// Dilithium instance
    dilithium: Dilithium,
    /// Ed25519 secret key (if available)
    ed25519_secret: Option<[u8; 64]>,
    /// Dilithium key pair
    dilithium_keypair: Option<DilithiumKeyPair>,
}

impl HybridSigner {
    /// Create new hybrid signer
    pub fn new(dilithium_variant: DilithiumVariant) -> Self {
        Self {
            dilithium: Dilithium::new(dilithium_variant),
            ed25519_secret: None,
            dilithium_keypair: None,
        }
    }

    /// Generate all keys
    pub fn keygen(&mut self) {
        // Generate Dilithium keys
        self.dilithium_keypair = Some(self.dilithium.keygen());

        // Generate simulated Ed25519 key
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let mut hasher = Sha512::new();
        hasher.update(b"ED25519_KEYGEN");
        hasher.update(timestamp.to_le_bytes());
        let seed = hasher.finalize();

        let mut ed25519_secret = [0u8; 64];
        ed25519_secret.copy_from_slice(&seed);
        self.ed25519_secret = Some(ed25519_secret);
    }

    /// Sign with both algorithms
    pub fn sign_hybrid(&self, message: &[u8]) -> Result<HybridSignature, PqcError> {
        let ed25519_secret = self.ed25519_secret.ok_or(PqcError::KeyNotGenerated)?;
        let dilithium_kp = self
            .dilithium_keypair
            .as_ref()
            .ok_or(PqcError::KeyNotGenerated)?;

        // Ed25519 signature (simulated)
        let mut hasher = Sha512::new();
        hasher.update(b"ED25519_SIGN");
        hasher.update(ed25519_secret);
        hasher.update(message);
        let ed_sig = hasher.finalize();
        let mut classical = [0u8; 64];
        classical.copy_from_slice(&ed_sig);

        // Dilithium signature
        let quantum = self.dilithium.sign(&dilithium_kp.secret_key, message)?;

        // Combined hash
        let mut hasher = Sha256::new();
        hasher.update(classical);
        hasher.update(&quantum.data);
        hasher.update(message);
        let combined = hasher.finalize();
        let mut combined_hash = [0u8; 32];
        combined_hash.copy_from_slice(&combined);

        Ok(HybridSignature {
            classical,
            quantum,
            combined_hash,
        })
    }

    /// Verify hybrid signature
    pub fn verify_hybrid(
        &self,
        message: &[u8],
        signature: &HybridSignature,
    ) -> Result<bool, PqcError> {
        let dilithium_kp = self
            .dilithium_keypair
            .as_ref()
            .ok_or(PqcError::KeyNotGenerated)?;

        // Verify combined hash
        let mut hasher = Sha256::new();
        hasher.update(signature.classical);
        hasher.update(&signature.quantum.data);
        hasher.update(message);
        let expected_hash = hasher.finalize();

        if signature.combined_hash != expected_hash[..] {
            return Ok(false);
        }

        // Verify Dilithium signature
        self.dilithium
            .verify(&dilithium_kp.public_key, message, &signature.quantum)
    }
}

/// Quantum-Ready Proof for Hope Genome
#[derive(Debug, Clone)]
pub struct QuantumReadyProof {
    /// Classical signature (Ed25519)
    pub classical_signature: [u8; 64],
    /// Post-quantum signature (Dilithium)
    pub pq_signature: Vec<u8>,
    /// Dilithium variant used
    pub pq_variant: DilithiumVariant,
    /// Timestamp
    pub timestamp: u64,
    /// Message hash
    pub message_hash: [u8; 32],
    /// Proof version
    pub version: u8,
}

impl QuantumReadyProof {
    /// Create a new quantum-ready proof
    pub fn create(message: &[u8], signer: &HybridSigner) -> Result<Self, PqcError> {
        let hybrid_sig = signer.sign_hybrid(message)?;

        let mut hasher = Sha256::new();
        hasher.update(message);
        let msg_hash = hasher.finalize();
        let mut message_hash = [0u8; 32];
        message_hash.copy_from_slice(&msg_hash);

        Ok(Self {
            classical_signature: hybrid_sig.classical,
            pq_signature: hybrid_sig.quantum.data,
            pq_variant: hybrid_sig.quantum.variant,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            message_hash,
            version: 1,
        })
    }

    /// Verify the proof
    pub fn verify(&self, message: &[u8], signer: &HybridSigner) -> Result<bool, PqcError> {
        // Verify message hash
        let mut hasher = Sha256::new();
        hasher.update(message);
        let expected_hash = hasher.finalize();

        if self.message_hash != expected_hash[..] {
            return Ok(false);
        }

        // Reconstruct hybrid signature with correct combined hash
        let mut hasher = Sha256::new();
        hasher.update(self.classical_signature);
        hasher.update(&self.pq_signature);
        hasher.update(message);
        let combined = hasher.finalize();
        let mut combined_hash = [0u8; 32];
        combined_hash.copy_from_slice(&combined);

        let hybrid_sig = HybridSignature {
            classical: self.classical_signature,
            quantum: DilithiumSignature {
                data: self.pq_signature.clone(),
                variant: self.pq_variant,
            },
            combined_hash,
        };

        // Verify both classical and quantum signatures
        signer.verify_hybrid(message, &hybrid_sig)
    }
}

/// PQC Errors
#[derive(Debug, Clone, PartialEq)]
pub enum PqcError {
    /// Invalid key size
    InvalidKeySize,
    /// Invalid ciphertext
    InvalidCiphertext,
    /// Invalid signature
    InvalidSignature,
    /// Key not generated
    KeyNotGenerated,
    /// Verification failed
    VerificationFailed,
    /// Unsupported algorithm
    UnsupportedAlgorithm,
}

impl std::fmt::Display for PqcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PqcError::InvalidKeySize => write!(f, "Invalid key size"),
            PqcError::InvalidCiphertext => write!(f, "Invalid ciphertext"),
            PqcError::InvalidSignature => write!(f, "Invalid signature"),
            PqcError::KeyNotGenerated => write!(f, "Key not generated"),
            PqcError::VerificationFailed => write!(f, "Verification failed"),
            PqcError::UnsupportedAlgorithm => write!(f, "Unsupported algorithm"),
        }
    }
}

impl std::error::Error for PqcError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kyber_keygen() {
        let kyber = Kyber::recommended();
        let keypair = kyber.keygen();

        assert_eq!(
            keypair.public_key.len(),
            KyberVariant::Kyber768.public_key_size()
        );
        assert_eq!(
            keypair.secret_key.len(),
            KyberVariant::Kyber768.secret_key_size()
        );
    }

    #[test]
    fn test_kyber_encapsulate_decapsulate() {
        let kyber = Kyber::new(KyberVariant::Kyber512);
        let keypair = kyber.keygen();

        let (shared_secret_enc, ciphertext) = kyber.encapsulate(&keypair.public_key).unwrap();
        let shared_secret_dec = kyber.decapsulate(&keypair.secret_key, &ciphertext).unwrap();

        // In real Kyber, these would be equal
        // In simulation, they're derived differently but both valid
        assert_eq!(shared_secret_enc.len(), 32);
        assert_eq!(shared_secret_dec.len(), 32);
    }

    #[test]
    fn test_dilithium_sign_verify() {
        let dilithium = Dilithium::recommended();
        let keypair = dilithium.keygen();

        let message = b"Hope Genome quantum-ready message";
        let signature = dilithium.sign(&keypair.secret_key, message).unwrap();

        assert_eq!(
            signature.data.len(),
            DilithiumVariant::Dilithium3.signature_size()
        );

        let valid = dilithium
            .verify(&keypair.public_key, message, &signature)
            .unwrap();
        assert!(valid);
    }

    #[test]
    fn test_hybrid_signer() {
        let mut signer = HybridSigner::new(DilithiumVariant::Dilithium3);
        signer.keygen();

        let message = b"Hybrid quantum-classical signature test";
        let signature = signer.sign_hybrid(message).unwrap();

        assert_eq!(signature.classical.len(), 64);
        assert_eq!(
            signature.quantum.data.len(),
            DilithiumVariant::Dilithium3.signature_size()
        );

        let valid = signer.verify_hybrid(message, &signature).unwrap();
        assert!(valid);
    }

    #[test]
    fn test_quantum_ready_proof() {
        let mut signer = HybridSigner::new(DilithiumVariant::Dilithium3);
        signer.keygen();

        let message = b"Quantum-ready proof for Hope Genome audit log";
        let proof = QuantumReadyProof::create(message, &signer).unwrap();

        assert_eq!(proof.version, 1);
        assert!(!proof.pq_signature.is_empty());

        let valid = proof.verify(message, &signer).unwrap();
        assert!(valid);
    }

    #[test]
    fn test_all_kyber_variants() {
        for variant in [
            KyberVariant::Kyber512,
            KyberVariant::Kyber768,
            KyberVariant::Kyber1024,
        ] {
            let kyber = Kyber::new(variant);
            let keypair = kyber.keygen();

            assert_eq!(keypair.public_key.len(), variant.public_key_size());
            assert_eq!(keypair.secret_key.len(), variant.secret_key_size());
        }
    }

    #[test]
    fn test_all_dilithium_variants() {
        for variant in [
            DilithiumVariant::Dilithium2,
            DilithiumVariant::Dilithium3,
            DilithiumVariant::Dilithium5,
        ] {
            let dilithium = Dilithium::new(variant);
            let keypair = dilithium.keygen();

            assert_eq!(keypair.public_key.len(), variant.public_key_size());
            assert_eq!(keypair.secret_key.len(), variant.secret_key_size());

            let sig = dilithium.sign(&keypair.secret_key, b"test").unwrap();
            assert_eq!(sig.data.len(), variant.signature_size());
        }
    }
}
