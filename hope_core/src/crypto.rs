use rand::rngs::OsRng;
use rsa::{Pkcs1v15Sign, RsaPrivateKey, RsaPublicKey};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Failed to generate keypair: {0}")]
    KeyGeneration(String),

    #[error("Failed to sign data: {0}")]
    SigningFailed(String),

    #[error("Failed to verify signature: {0}")]
    VerificationFailed(String),

    #[error("Invalid signature")]
    InvalidSignature,
}

pub type Result<T> = std::result::Result<T, CryptoError>;

/// RSA keypair for signing and verification
#[derive(Debug, Clone)]
pub struct KeyPair {
    private_key: RsaPrivateKey,
    public_key: RsaPublicKey,
}

impl KeyPair {
    /// Generate a new 2048-bit RSA keypair
    pub fn generate() -> Result<Self> {
        let mut rng = OsRng;
        let bits = 2048;

        let private_key = RsaPrivateKey::new(&mut rng, bits)
            .map_err(|e| CryptoError::KeyGeneration(e.to_string()))?;

        let public_key = RsaPublicKey::from(&private_key);

        Ok(KeyPair {
            private_key,
            public_key,
        })
    }

    /// Sign data with private key
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Hash the data first
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hashed = hasher.finalize();

        // Create padding scheme
        let padding = Pkcs1v15Sign::new_unprefixed();

        // Sign the hash
        let signature = self
            .private_key
            .sign(padding, &hashed)
            .map_err(|e| CryptoError::SigningFailed(e.to_string()))?;

        Ok(signature)
    }

    /// Verify signature with public key
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> Result<()> {
        // Hash the data first
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hashed = hasher.finalize();

        // Create padding scheme
        let padding = Pkcs1v15Sign::new_unprefixed();

        // Verify the signature
        self.public_key
            .verify(padding, &hashed, signature)
            .map_err(|_| CryptoError::InvalidSignature)?;

        Ok(())
    }

    pub fn public_key(&self) -> &RsaPublicKey {
        &self.public_key
    }
}

/// Hash data using SHA-256
pub fn hash_bytes(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Generate a cryptographically secure random nonce
pub fn generate_nonce() -> [u8; 32] {
    let mut nonce = [0u8; 32];
    rand::RngCore::fill_bytes(&mut OsRng, &mut nonce);
    nonce
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsa::traits::PublicKeyParts;

    #[test]
    fn test_keypair_generation() {
        let keypair = KeyPair::generate().unwrap();
        assert!(keypair.public_key.size() >= 256); // 2048 bits = 256 bytes
    }

    #[test]
    fn test_sign_and_verify() {
        let keypair = KeyPair::generate().unwrap();
        let data = b"test message";

        let signature = keypair.sign(data).unwrap();
        assert!(keypair.verify(data, &signature).is_ok());
    }

    #[test]
    fn test_verify_fails_on_tampered_data() {
        let keypair = KeyPair::generate().unwrap();
        let data = b"original message";
        let signature = keypair.sign(data).unwrap();

        let tampered_data = b"tampered message";
        assert!(keypair.verify(tampered_data, &signature).is_err());
    }

    #[test]
    fn test_verify_fails_on_tampered_signature() {
        let keypair = KeyPair::generate().unwrap();
        let data = b"test message";
        let mut signature = keypair.sign(data).unwrap();

        // Tamper with signature
        signature[0] ^= 0xFF;

        assert!(keypair.verify(data, &signature).is_err());
    }

    #[test]
    fn test_hash_deterministic() {
        let data = b"test data";
        let hash1 = hash_bytes(data);
        let hash2 = hash_bytes(data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_nonce_uniqueness() {
        let nonce1 = generate_nonce();
        let nonce2 = generate_nonce();
        assert_ne!(nonce1, nonce2);
    }
}
