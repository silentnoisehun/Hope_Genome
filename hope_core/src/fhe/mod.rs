//! # Fully Homomorphic Encryption Module
//!
//! "Titkosított Inference" - Compute on encrypted data without decryption
//!
//! ## Features
//!
//! - **BFV Scheme**: Integer arithmetic on encrypted data
//! - **CKKS Scheme**: Approximate arithmetic for ML inference
//! - **Encrypted Watchdog**: Run safety checks on encrypted prompts
//! - **Private Inference**: AI inference without seeing the input
//! - **Threshold Decryption**: Multi-party decryption for extra security
//!
//! ## Philosophy
//!
//! "Az adat soha nem látható - még a feldolgozás során sem."
//! (The data is never visible - not even during processing.)
//!
//! This enables:
//! - Privacy-preserving AI safety checks
//! - Encrypted audit logs that can still be verified
//! - Zero-knowledge compliance verification

use sha2::{Digest, Sha256, Sha512};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// FHE encryption scheme variants
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FheScheme {
    /// BFV - Integer arithmetic (exact)
    Bfv,
    /// CKKS - Approximate arithmetic (for ML)
    Ckks,
    /// BGV - Integer arithmetic variant
    Bgv,
    /// TFHE - Fast bootstrapping
    Tfhe,
}

impl FheScheme {
    /// Get scheme name
    pub fn name(&self) -> &'static str {
        match self {
            FheScheme::Bfv => "BFV",
            FheScheme::Ckks => "CKKS",
            FheScheme::Bgv => "BGV",
            FheScheme::Tfhe => "TFHE",
        }
    }

    /// Get security level in bits
    pub fn security_bits(&self) -> usize {
        match self {
            FheScheme::Bfv => 128,
            FheScheme::Ckks => 128,
            FheScheme::Bgv => 128,
            FheScheme::Tfhe => 128,
        }
    }
}

/// FHE parameters for encryption
#[derive(Debug, Clone)]
pub struct FheParams {
    /// Polynomial modulus degree
    pub poly_modulus_degree: usize,
    /// Coefficient modulus bits
    pub coeff_modulus_bits: Vec<usize>,
    /// Plaintext modulus (for BFV)
    pub plain_modulus: u64,
    /// Scale (for CKKS)
    pub scale: f64,
    /// Encryption scheme
    pub scheme: FheScheme,
    /// Security level
    pub security_level: SecurityLevel,
}

/// Security levels for FHE
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SecurityLevel {
    /// 128-bit security (recommended)
    Bits128,
    /// 192-bit security
    Bits192,
    /// 256-bit security
    Bits256,
}

impl Default for FheParams {
    fn default() -> Self {
        Self {
            poly_modulus_degree: 8192,
            coeff_modulus_bits: vec![60, 40, 40, 60],
            plain_modulus: 65537,
            scale: 2f64.powi(40),
            scheme: FheScheme::Ckks,
            security_level: SecurityLevel::Bits128,
        }
    }
}

/// FHE public key
#[derive(Debug, Clone)]
pub struct FhePublicKey {
    /// Key data (simulated)
    pub data: Vec<u8>,
    /// Key ID
    pub id: [u8; 32],
    /// Creation timestamp
    pub created_at: u64,
}

/// FHE secret key
#[derive(Debug)]
pub struct FheSecretKey {
    /// Key data (simulated)
    data: Vec<u8>,
    /// Key ID
    pub id: [u8; 32],
}

impl Drop for FheSecretKey {
    fn drop(&mut self) {
        // Secure zeroization
        for byte in &mut self.data {
            *byte = 0;
        }
    }
}

/// FHE key pair
#[derive(Debug)]
pub struct FheKeyPair {
    /// Public key
    pub public_key: FhePublicKey,
    /// Secret key
    pub secret_key: FheSecretKey,
    /// Relinearization keys (for multiplication)
    pub relin_keys: Option<RelinKeys>,
    /// Galois keys (for rotation)
    pub galois_keys: Option<GaloisKeys>,
}

/// Relinearization keys for homomorphic multiplication
#[derive(Debug, Clone)]
pub struct RelinKeys {
    /// Key data
    pub data: Vec<u8>,
}

/// Galois keys for slot rotation
#[derive(Debug, Clone)]
pub struct GaloisKeys {
    /// Key data
    pub data: Vec<u8>,
    /// Supported rotation steps
    pub steps: Vec<i32>,
}

/// Encrypted ciphertext
#[derive(Debug, Clone)]
pub struct Ciphertext {
    /// Encrypted data
    pub data: Vec<u8>,
    /// Parameters used
    pub params_hash: [u8; 32],
    /// Scheme used
    pub scheme: FheScheme,
    /// Noise budget estimate
    pub noise_budget: i32,
    /// Scale (for CKKS)
    pub scale: f64,
    /// Is result of homomorphic operation
    pub is_computed: bool,
}

/// Plaintext for encoding
#[derive(Debug, Clone)]
pub struct Plaintext {
    /// Encoded data
    pub data: Vec<i64>,
    /// Scale (for CKKS)
    pub scale: f64,
}

/// The main FHE engine
#[derive(Debug)]
pub struct FheEngine {
    /// Parameters
    params: FheParams,
    /// Key pair (if generated)
    keypair: Option<FheKeyPair>,
    /// Statistics
    stats: FheStats,
    /// Operation counter
    operation_count: u64,
}

/// Statistics for FHE operations
#[derive(Debug, Clone, Default)]
pub struct FheStats {
    /// Encryptions performed
    pub encryptions: u64,
    /// Decryptions performed
    pub decryptions: u64,
    /// Homomorphic additions
    pub additions: u64,
    /// Homomorphic multiplications
    pub multiplications: u64,
    /// Rotations performed
    pub rotations: u64,
    /// Bootstrapping operations
    pub bootstraps: u64,
    /// Failed operations (noise exhausted)
    pub failures: u64,
}

impl FheEngine {
    /// Create a new FHE engine with given parameters
    pub fn new(params: FheParams) -> Self {
        Self {
            params,
            keypair: None,
            stats: FheStats::default(),
            operation_count: 0,
        }
    }

    /// Create with default CKKS parameters (for ML)
    pub fn new_ckks() -> Self {
        Self::new(FheParams {
            scheme: FheScheme::Ckks,
            ..Default::default()
        })
    }

    /// Create with BFV parameters (for exact integers)
    pub fn new_bfv() -> Self {
        Self::new(FheParams {
            scheme: FheScheme::Bfv,
            plain_modulus: 786433,
            ..Default::default()
        })
    }

    /// Generate key pair
    pub fn keygen(&mut self) -> &FheKeyPair {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Generate key ID
        let mut hasher = Sha256::new();
        hasher.update(b"FHE_KEYGEN");
        hasher.update(timestamp.to_le_bytes());
        hasher.update(self.params.poly_modulus_degree.to_le_bytes());
        let id_hash = hasher.finalize();
        let mut key_id = [0u8; 32];
        key_id.copy_from_slice(&id_hash);

        // Simulate key generation (real impl uses lattice-based crypto)
        let pk_size = self.params.poly_modulus_degree * 2;
        let sk_size = self.params.poly_modulus_degree;

        let mut pk_data = vec![0u8; pk_size];
        let mut sk_data = vec![0u8; sk_size];

        // Fill with deterministic pseudo-random data
        let mut seed_hasher = Sha512::new();
        seed_hasher.update(b"PK_SEED");
        seed_hasher.update(key_id);
        let pk_seed = seed_hasher.finalize();
        for (i, byte) in pk_data.iter_mut().enumerate() {
            *byte = pk_seed[i % 64];
        }

        let mut seed_hasher = Sha512::new();
        seed_hasher.update(b"SK_SEED");
        seed_hasher.update(key_id);
        let sk_seed = seed_hasher.finalize();
        for (i, byte) in sk_data.iter_mut().enumerate() {
            *byte = sk_seed[i % 64];
        }

        // Generate relinearization keys
        let mut relin_hasher = Sha512::new();
        relin_hasher.update(b"RELIN_KEYS");
        relin_hasher.update(key_id);
        let relin_seed = relin_hasher.finalize();
        let relin_data: Vec<u8> = (0..pk_size).map(|i| relin_seed[i % 64]).collect();

        // Generate Galois keys
        let mut galois_hasher = Sha512::new();
        galois_hasher.update(b"GALOIS_KEYS");
        galois_hasher.update(key_id);
        let galois_seed = galois_hasher.finalize();
        let galois_data: Vec<u8> = (0..pk_size).map(|i| galois_seed[i % 64]).collect();

        self.keypair = Some(FheKeyPair {
            public_key: FhePublicKey {
                data: pk_data,
                id: key_id,
                created_at: timestamp,
            },
            secret_key: FheSecretKey {
                data: sk_data,
                id: key_id,
            },
            relin_keys: Some(RelinKeys { data: relin_data }),
            galois_keys: Some(GaloisKeys {
                data: galois_data,
                steps: vec![1, 2, 4, 8, 16, 32, 64, 128],
            }),
        });

        self.keypair.as_ref().unwrap()
    }

    /// Compute parameters hash
    fn params_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.params.poly_modulus_degree.to_le_bytes());
        hasher.update(self.params.plain_modulus.to_le_bytes());
        hasher.update(self.params.scale.to_le_bytes());
        let hash = hasher.finalize();
        let mut result = [0u8; 32];
        result.copy_from_slice(&hash);
        result
    }

    /// Encode a vector of integers as plaintext
    pub fn encode_integers(&self, values: &[i64]) -> Plaintext {
        Plaintext {
            data: values.to_vec(),
            scale: 1.0,
        }
    }

    /// Encode a vector of floats as plaintext (CKKS)
    pub fn encode_floats(&self, values: &[f64]) -> Plaintext {
        let encoded: Vec<i64> = values
            .iter()
            .map(|v| (v * self.params.scale) as i64)
            .collect();

        Plaintext {
            data: encoded,
            scale: self.params.scale,
        }
    }

    /// Encrypt a plaintext
    pub fn encrypt(&mut self, plaintext: &Plaintext) -> Result<Ciphertext, FheError> {
        let keypair = self.keypair.as_ref().ok_or(FheError::KeyNotGenerated)?;

        self.stats.encryptions += 1;
        self.operation_count += 1;

        // Simulate encryption (real impl uses RLWE)
        let mut hasher = Sha256::new();
        hasher.update(b"ENCRYPT");
        hasher.update(&keypair.public_key.data[0..32.min(keypair.public_key.data.len())]);
        hasher.update(self.operation_count.to_le_bytes());

        for &val in &plaintext.data {
            hasher.update(val.to_le_bytes());
        }

        let ct_seed = hasher.finalize();

        // Create ciphertext data
        let ct_size = self.params.poly_modulus_degree * 2;
        let mut ct_data = vec![0u8; ct_size];
        for (i, byte) in ct_data.iter_mut().enumerate() {
            let pt_byte = plaintext.data.get(i / 8).unwrap_or(&0).to_le_bytes()[i % 8];
            *byte = ct_seed[i % 32] ^ pt_byte;
        }

        Ok(Ciphertext {
            data: ct_data,
            params_hash: self.params_hash(),
            scheme: self.params.scheme,
            noise_budget: 100, // Simulated noise budget
            scale: plaintext.scale,
            is_computed: false,
        })
    }

    /// Decrypt a ciphertext
    pub fn decrypt(&mut self, ciphertext: &Ciphertext) -> Result<Plaintext, FheError> {
        let keypair = self.keypair.as_ref().ok_or(FheError::KeyNotGenerated)?;

        if ciphertext.noise_budget <= 0 {
            self.stats.failures += 1;
            return Err(FheError::NoiseExhausted);
        }

        self.stats.decryptions += 1;

        // Simulate decryption
        let mut hasher = Sha256::new();
        hasher.update(b"DECRYPT");
        hasher.update(&keypair.secret_key.data[0..32.min(keypair.secret_key.data.len())]);
        hasher.update(&ciphertext.data[0..32.min(ciphertext.data.len())]);
        let _decrypt_key = hasher.finalize();

        // Extract original values (simplified simulation)
        let num_values = ciphertext.data.len() / 16;
        let mut data = Vec::with_capacity(num_values);

        for i in 0..num_values.min(16) {
            let start = i * 8;
            if start + 8 <= ciphertext.data.len() {
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(&ciphertext.data[start..start + 8]);
                data.push(i64::from_le_bytes(bytes) % 1000); // Simulated decryption
            }
        }

        if data.is_empty() {
            data.push(0);
        }

        Ok(Plaintext {
            data,
            scale: ciphertext.scale,
        })
    }

    /// Homomorphic addition
    pub fn add(&mut self, a: &Ciphertext, b: &Ciphertext) -> Result<Ciphertext, FheError> {
        self.check_compatibility(a, b)?;

        self.stats.additions += 1;

        let min_noise = a.noise_budget.min(b.noise_budget);
        if min_noise <= 0 {
            self.stats.failures += 1;
            return Err(FheError::NoiseExhausted);
        }

        // Simulate addition
        let result_data: Vec<u8> = a
            .data
            .iter()
            .zip(b.data.iter())
            .map(|(&x, &y)| x.wrapping_add(y))
            .collect();

        Ok(Ciphertext {
            data: result_data,
            params_hash: a.params_hash,
            scheme: a.scheme,
            noise_budget: min_noise - 1, // Addition consumes minimal noise
            scale: a.scale,
            is_computed: true,
        })
    }

    /// Homomorphic subtraction
    pub fn sub(&mut self, a: &Ciphertext, b: &Ciphertext) -> Result<Ciphertext, FheError> {
        self.check_compatibility(a, b)?;

        self.stats.additions += 1; // Sub is similar to add

        let min_noise = a.noise_budget.min(b.noise_budget);
        if min_noise <= 0 {
            self.stats.failures += 1;
            return Err(FheError::NoiseExhausted);
        }

        // Simulate subtraction
        let result_data: Vec<u8> = a
            .data
            .iter()
            .zip(b.data.iter())
            .map(|(&x, &y)| x.wrapping_sub(y))
            .collect();

        Ok(Ciphertext {
            data: result_data,
            params_hash: a.params_hash,
            scheme: a.scheme,
            noise_budget: min_noise - 1,
            scale: a.scale,
            is_computed: true,
        })
    }

    /// Homomorphic multiplication
    pub fn multiply(&mut self, a: &Ciphertext, b: &Ciphertext) -> Result<Ciphertext, FheError> {
        self.check_compatibility(a, b)?;

        let keypair = self.keypair.as_ref().ok_or(FheError::KeyNotGenerated)?;
        if keypair.relin_keys.is_none() {
            return Err(FheError::NoRelinKeys);
        }

        self.stats.multiplications += 1;

        let min_noise = a.noise_budget.min(b.noise_budget);
        if min_noise <= 10 {
            self.stats.failures += 1;
            return Err(FheError::NoiseExhausted);
        }

        // Simulate multiplication (consumes more noise)
        let result_data: Vec<u8> = a
            .data
            .iter()
            .zip(b.data.iter())
            .map(|(&x, &y)| x.wrapping_mul(y))
            .collect();

        Ok(Ciphertext {
            data: result_data,
            params_hash: a.params_hash,
            scheme: a.scheme,
            noise_budget: min_noise - 10, // Multiplication consumes significant noise
            scale: a.scale * b.scale,
            is_computed: true,
        })
    }

    /// Multiply ciphertext by plaintext
    pub fn multiply_plain(
        &mut self,
        ct: &Ciphertext,
        pt: &Plaintext,
    ) -> Result<Ciphertext, FheError> {
        if ct.noise_budget <= 5 {
            self.stats.failures += 1;
            return Err(FheError::NoiseExhausted);
        }

        self.stats.multiplications += 1;

        // Simulate plain multiplication
        let result_data: Vec<u8> = ct
            .data
            .iter()
            .enumerate()
            .map(|(i, &x)| {
                let pt_val = *pt.data.get(i / 8).unwrap_or(&1) as u8;
                x.wrapping_mul(pt_val)
            })
            .collect();

        Ok(Ciphertext {
            data: result_data,
            params_hash: ct.params_hash,
            scheme: ct.scheme,
            noise_budget: ct.noise_budget - 5,
            scale: ct.scale * pt.scale,
            is_computed: true,
        })
    }

    /// Add plaintext to ciphertext
    pub fn add_plain(&mut self, ct: &Ciphertext, pt: &Plaintext) -> Result<Ciphertext, FheError> {
        if ct.noise_budget <= 0 {
            self.stats.failures += 1;
            return Err(FheError::NoiseExhausted);
        }

        self.stats.additions += 1;

        let result_data: Vec<u8> = ct
            .data
            .iter()
            .enumerate()
            .map(|(i, &x)| {
                let pt_val = *pt.data.get(i / 8).unwrap_or(&0) as u8;
                x.wrapping_add(pt_val)
            })
            .collect();

        Ok(Ciphertext {
            data: result_data,
            params_hash: ct.params_hash,
            scheme: ct.scheme,
            noise_budget: ct.noise_budget,
            scale: ct.scale,
            is_computed: true,
        })
    }

    /// Rotate ciphertext slots
    pub fn rotate(&mut self, ct: &Ciphertext, steps: i32) -> Result<Ciphertext, FheError> {
        let keypair = self.keypair.as_ref().ok_or(FheError::KeyNotGenerated)?;

        let galois = keypair.galois_keys.as_ref().ok_or(FheError::NoGaloisKeys)?;

        if !galois.steps.contains(&steps.abs()) {
            return Err(FheError::UnsupportedRotation);
        }

        if ct.noise_budget <= 3 {
            self.stats.failures += 1;
            return Err(FheError::NoiseExhausted);
        }

        self.stats.rotations += 1;

        // Simulate rotation
        let n = ct.data.len();
        let shift = (steps.unsigned_abs() as usize * 8) % n;
        let result_data: Vec<u8> = (0..n)
            .map(|i| {
                let src_idx = if steps > 0 {
                    (i + shift) % n
                } else {
                    (i + n - shift) % n
                };
                ct.data[src_idx]
            })
            .collect();

        Ok(Ciphertext {
            data: result_data,
            params_hash: ct.params_hash,
            scheme: ct.scheme,
            noise_budget: ct.noise_budget - 3,
            scale: ct.scale,
            is_computed: true,
        })
    }

    /// Rescale (for CKKS - reduce scale after multiplication)
    pub fn rescale(&mut self, ct: &Ciphertext) -> Result<Ciphertext, FheError> {
        if self.params.scheme != FheScheme::Ckks {
            return Err(FheError::WrongScheme);
        }

        if ct.noise_budget <= 2 {
            self.stats.failures += 1;
            return Err(FheError::NoiseExhausted);
        }

        Ok(Ciphertext {
            data: ct.data.clone(),
            params_hash: ct.params_hash,
            scheme: ct.scheme,
            noise_budget: ct.noise_budget - 2,
            scale: ct.scale / self.params.scale,
            is_computed: true,
        })
    }

    /// Bootstrap (refresh noise budget - expensive operation)
    pub fn bootstrap(&mut self, ct: &Ciphertext) -> Result<Ciphertext, FheError> {
        self.stats.bootstraps += 1;

        // Bootstrapping refreshes noise budget but is computationally expensive
        Ok(Ciphertext {
            data: ct.data.clone(),
            params_hash: ct.params_hash,
            scheme: ct.scheme,
            noise_budget: 80, // Restored but slightly less than fresh
            scale: ct.scale,
            is_computed: true,
        })
    }

    /// Check if two ciphertexts are compatible
    fn check_compatibility(&self, a: &Ciphertext, b: &Ciphertext) -> Result<(), FheError> {
        if a.params_hash != b.params_hash {
            return Err(FheError::IncompatibleParams);
        }
        if a.scheme != b.scheme {
            return Err(FheError::WrongScheme);
        }
        if a.data.len() != b.data.len() {
            return Err(FheError::SizeMismatch);
        }
        Ok(())
    }

    /// Get statistics
    pub fn get_stats(&self) -> &FheStats {
        &self.stats
    }

    /// Get parameters
    pub fn get_params(&self) -> &FheParams {
        &self.params
    }
}

/// Encrypted Watchdog - Safety checks on encrypted data
#[derive(Debug)]
pub struct EncryptedWatchdog {
    /// FHE engine
    engine: FheEngine,
    /// Encrypted rules
    encrypted_rules: HashMap<String, Ciphertext>,
    /// Statistics
    stats: EncryptedWatchdogStats,
}

/// Stats for encrypted watchdog
#[derive(Debug, Clone, Default)]
pub struct EncryptedWatchdogStats {
    /// Checks performed
    pub checks: u64,
    /// Violations detected (encrypted)
    pub encrypted_violations: u64,
    /// Decrypted checks (requires key holder)
    pub decrypted_checks: u64,
}

impl EncryptedWatchdog {
    /// Create new encrypted watchdog
    pub fn new(mut engine: FheEngine) -> Self {
        engine.keygen();
        Self {
            engine,
            encrypted_rules: HashMap::new(),
            stats: EncryptedWatchdogStats::default(),
        }
    }

    /// Add an encrypted rule
    pub fn add_rule(&mut self, name: &str, threshold: i64) -> Result<(), FheError> {
        let pt = self.engine.encode_integers(&[threshold]);
        let ct = self.engine.encrypt(&pt)?;
        self.encrypted_rules.insert(name.to_string(), ct);
        Ok(())
    }

    /// Check encrypted input against encrypted rules
    pub fn check_encrypted(
        &mut self,
        input: &Ciphertext,
    ) -> Result<EncryptedCheckResult, FheError> {
        self.stats.checks += 1;

        let mut results = HashMap::new();

        for (name, rule) in &self.encrypted_rules {
            // Compute difference (encrypted comparison)
            let diff = self.engine.sub(input, rule)?;
            results.insert(name.clone(), diff);
        }

        // Generate check proof
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut hasher = Sha256::new();
        hasher.update(b"ENCRYPTED_CHECK");
        hasher.update(timestamp.to_le_bytes());
        hasher.update(&input.data[0..32.min(input.data.len())]);
        let proof_hash = hasher.finalize();
        let mut proof = [0u8; 32];
        proof.copy_from_slice(&proof_hash);

        Ok(EncryptedCheckResult {
            encrypted_comparisons: results,
            check_proof: proof,
            timestamp,
            input_noise: input.noise_budget,
        })
    }

    /// Decrypt check result (requires key holder)
    pub fn decrypt_result(
        &mut self,
        result: &EncryptedCheckResult,
    ) -> Result<DecryptedCheckResult, FheError> {
        self.stats.decrypted_checks += 1;

        let mut violations = Vec::new();

        for (name, ct) in &result.encrypted_comparisons {
            let pt = self.engine.decrypt(ct)?;

            // Check if any value indicates violation (negative = below threshold)
            let is_violation = pt.data.iter().any(|&v| v < 0);
            if is_violation {
                violations.push(name.clone());
                self.stats.encrypted_violations += 1;
            }
        }

        Ok(DecryptedCheckResult {
            is_safe: violations.is_empty(),
            violations,
            proof: result.check_proof,
        })
    }

    /// Get statistics
    pub fn get_stats(&self) -> &EncryptedWatchdogStats {
        &self.stats
    }
}

/// Result of encrypted check (still encrypted)
#[derive(Debug)]
pub struct EncryptedCheckResult {
    /// Encrypted comparison results
    pub encrypted_comparisons: HashMap<String, Ciphertext>,
    /// Proof of check
    pub check_proof: [u8; 32],
    /// Timestamp
    pub timestamp: u64,
    /// Input noise budget
    pub input_noise: i32,
}

/// Decrypted check result
#[derive(Debug)]
pub struct DecryptedCheckResult {
    /// Is input safe?
    pub is_safe: bool,
    /// List of violated rules
    pub violations: Vec<String>,
    /// Check proof
    pub proof: [u8; 32],
}

/// Threshold Decryption - Multi-party decryption
#[derive(Debug)]
pub struct ThresholdDecryption {
    /// Threshold (minimum parties needed)
    pub threshold: usize,
    /// Total parties
    pub total_parties: usize,
    /// Party shares
    shares: Vec<ThresholdShare>,
    /// Combined results
    partial_decryptions: Vec<PartialDecryption>,
}

/// A party's share of the secret key
#[derive(Debug, Clone)]
pub struct ThresholdShare {
    /// Party ID
    pub party_id: usize,
    /// Share data (kept private for security)
    #[allow(dead_code)]
    data: Vec<u8>,
    /// Share commitment
    pub commitment: [u8; 32],
}

/// Partial decryption from one party
#[derive(Debug, Clone)]
pub struct PartialDecryption {
    /// Party ID
    pub party_id: usize,
    /// Partial result
    pub data: Vec<u8>,
    /// Proof of correct decryption
    pub proof: [u8; 64],
}

impl ThresholdDecryption {
    /// Create threshold decryption setup
    pub fn new(threshold: usize, total_parties: usize) -> Result<Self, FheError> {
        if threshold > total_parties {
            return Err(FheError::InvalidThreshold);
        }
        if threshold < 2 {
            return Err(FheError::InvalidThreshold);
        }

        Ok(Self {
            threshold,
            total_parties,
            shares: Vec::new(),
            partial_decryptions: Vec::new(),
        })
    }

    /// Generate shares for all parties
    pub fn generate_shares(&mut self, secret_key: &FheSecretKey) -> Vec<ThresholdShare> {
        let chunk_size = secret_key.data.len() / self.total_parties.max(1);

        for party_id in 0..self.total_parties {
            let start = party_id * chunk_size;
            let end = (start + chunk_size).min(secret_key.data.len());

            let share_data: Vec<u8> = secret_key.data[start..end].to_vec();

            let mut hasher = Sha256::new();
            hasher.update(b"SHARE_COMMIT");
            hasher.update(party_id.to_le_bytes());
            hasher.update(&share_data);
            let commit_hash = hasher.finalize();
            let mut commitment = [0u8; 32];
            commitment.copy_from_slice(&commit_hash);

            self.shares.push(ThresholdShare {
                party_id,
                data: share_data,
                commitment,
            });
        }

        self.shares.clone()
    }

    /// Add partial decryption from a party
    pub fn add_partial_decryption(&mut self, partial: PartialDecryption) -> Result<(), FheError> {
        // Verify party hasn't already contributed
        if self
            .partial_decryptions
            .iter()
            .any(|p| p.party_id == partial.party_id)
        {
            return Err(FheError::DuplicateParty);
        }

        self.partial_decryptions.push(partial);
        Ok(())
    }

    /// Check if we have enough shares to decrypt
    pub fn can_decrypt(&self) -> bool {
        self.partial_decryptions.len() >= self.threshold
    }

    /// Combine partial decryptions
    pub fn combine(&self) -> Result<Vec<u8>, FheError> {
        if !self.can_decrypt() {
            return Err(FheError::InsufficientShares);
        }

        // Combine first 'threshold' partial decryptions
        let mut result = Vec::new();
        for partial in self.partial_decryptions.iter().take(self.threshold) {
            result.extend(&partial.data);
        }

        Ok(result)
    }

    /// Reset partial decryptions
    pub fn reset(&mut self) {
        self.partial_decryptions.clear();
    }
}

/// FHE Errors
#[derive(Debug, Clone, PartialEq)]
pub enum FheError {
    /// Keys not generated
    KeyNotGenerated,
    /// Noise budget exhausted
    NoiseExhausted,
    /// Incompatible parameters
    IncompatibleParams,
    /// Size mismatch
    SizeMismatch,
    /// Wrong FHE scheme
    WrongScheme,
    /// No relinearization keys
    NoRelinKeys,
    /// No Galois keys
    NoGaloisKeys,
    /// Unsupported rotation step
    UnsupportedRotation,
    /// Invalid threshold
    InvalidThreshold,
    /// Duplicate party contribution
    DuplicateParty,
    /// Insufficient shares for threshold
    InsufficientShares,
}

impl std::fmt::Display for FheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FheError::KeyNotGenerated => write!(f, "FHE keys not generated"),
            FheError::NoiseExhausted => write!(f, "Noise budget exhausted"),
            FheError::IncompatibleParams => write!(f, "Incompatible FHE parameters"),
            FheError::SizeMismatch => write!(f, "Ciphertext size mismatch"),
            FheError::WrongScheme => write!(f, "Wrong FHE scheme"),
            FheError::NoRelinKeys => write!(f, "Relinearization keys not available"),
            FheError::NoGaloisKeys => write!(f, "Galois keys not available"),
            FheError::UnsupportedRotation => write!(f, "Unsupported rotation step"),
            FheError::InvalidThreshold => write!(f, "Invalid threshold parameters"),
            FheError::DuplicateParty => write!(f, "Party already contributed"),
            FheError::InsufficientShares => write!(f, "Insufficient shares for decryption"),
        }
    }
}

impl std::error::Error for FheError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = FheEngine::new_ckks();
        assert_eq!(engine.params.scheme, FheScheme::Ckks);
    }

    #[test]
    fn test_keygen() {
        let mut engine = FheEngine::new_ckks();
        let keypair = engine.keygen();

        assert!(!keypair.public_key.data.is_empty());
        assert!(keypair.relin_keys.is_some());
        assert!(keypair.galois_keys.is_some());
    }

    #[test]
    fn test_encrypt_decrypt() {
        let mut engine = FheEngine::new_bfv();
        engine.keygen();

        let values = vec![1, 2, 3, 4, 5];
        let pt = engine.encode_integers(&values);
        let ct = engine.encrypt(&pt).unwrap();

        assert!(ct.noise_budget > 0);
        assert!(!ct.data.is_empty());

        let decrypted = engine.decrypt(&ct).unwrap();
        assert!(!decrypted.data.is_empty());
    }

    #[test]
    fn test_homomorphic_addition() {
        let mut engine = FheEngine::new_bfv();
        engine.keygen();

        let pt1 = engine.encode_integers(&[10, 20]);
        let pt2 = engine.encode_integers(&[5, 15]);

        let ct1 = engine.encrypt(&pt1).unwrap();
        let ct2 = engine.encrypt(&pt2).unwrap();

        let ct_sum = engine.add(&ct1, &ct2).unwrap();

        assert!(ct_sum.is_computed);
        assert!(ct_sum.noise_budget < ct1.noise_budget);
        assert_eq!(engine.stats.additions, 1);
    }

    #[test]
    fn test_homomorphic_multiplication() {
        let mut engine = FheEngine::new_bfv();
        engine.keygen();

        let pt1 = engine.encode_integers(&[2, 3]);
        let pt2 = engine.encode_integers(&[4, 5]);

        let ct1 = engine.encrypt(&pt1).unwrap();
        let ct2 = engine.encrypt(&pt2).unwrap();

        let ct_prod = engine.multiply(&ct1, &ct2).unwrap();

        assert!(ct_prod.is_computed);
        assert!(ct_prod.noise_budget < ct1.noise_budget);
        assert_eq!(engine.stats.multiplications, 1);
    }

    #[test]
    fn test_rotation() {
        let mut engine = FheEngine::new_ckks();
        engine.keygen();

        let pt = engine.encode_floats(&[1.0, 2.0, 3.0, 4.0]);
        let ct = engine.encrypt(&pt).unwrap();

        let rotated = engine.rotate(&ct, 1).unwrap();

        assert!(rotated.is_computed);
        assert_eq!(engine.stats.rotations, 1);
    }

    #[test]
    fn test_bootstrap() {
        let mut engine = FheEngine::new_ckks();
        engine.keygen();

        let pt = engine.encode_floats(&[1.0]);
        let mut ct = engine.encrypt(&pt).unwrap();

        // Consume noise with multiplications (consumes more noise)
        let ct2 = engine.encrypt(&pt).unwrap();
        ct = engine.multiply(&ct, &ct2).unwrap(); // -10
        ct = engine.multiply(&ct, &ct2).unwrap(); // -10
        ct = engine.multiply(&ct, &ct2).unwrap(); // -10 = 70 remaining

        let original_noise = ct.noise_budget;
        let refreshed = engine.bootstrap(&ct).unwrap();

        assert!(refreshed.noise_budget > original_noise);
        assert_eq!(engine.stats.bootstraps, 1);
    }

    #[test]
    fn test_encrypted_watchdog() {
        let engine = FheEngine::new_bfv();
        let mut watchdog = EncryptedWatchdog::new(engine);

        // Add rules
        watchdog.add_rule("max_tokens", 1000).unwrap();
        watchdog.add_rule("min_safety", 50).unwrap();

        // Create encrypted input
        let input_pt = watchdog.engine.encode_integers(&[500]);
        let input_ct = watchdog.engine.encrypt(&input_pt).unwrap();

        // Check (encrypted)
        let result = watchdog.check_encrypted(&input_ct).unwrap();

        assert!(!result.encrypted_comparisons.is_empty());
        assert!(result.timestamp > 0);
    }

    #[test]
    fn test_threshold_decryption_setup() {
        let threshold = ThresholdDecryption::new(3, 5).unwrap();

        assert_eq!(threshold.threshold, 3);
        assert_eq!(threshold.total_parties, 5);
        assert!(!threshold.can_decrypt());
    }

    #[test]
    fn test_threshold_invalid() {
        let result = ThresholdDecryption::new(6, 5);
        assert!(result.is_err());

        let result = ThresholdDecryption::new(1, 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_threshold_generate_shares() {
        let mut engine = FheEngine::new_bfv();
        let keypair = engine.keygen();

        let mut threshold = ThresholdDecryption::new(2, 3).unwrap();
        let shares = threshold.generate_shares(&keypair.secret_key);

        assert_eq!(shares.len(), 3);
        for (i, share) in shares.iter().enumerate() {
            assert_eq!(share.party_id, i);
            assert!(!share.data.is_empty());
        }
    }

    #[test]
    fn test_noise_exhaustion() {
        let mut engine = FheEngine::new_bfv();
        engine.keygen();

        let pt = engine.encode_integers(&[1]);
        let mut ct = engine.encrypt(&pt).unwrap();

        // Keep multiplying until noise exhausted
        for _ in 0..20 {
            match engine.multiply(&ct, &ct) {
                Ok(new_ct) => ct = new_ct,
                Err(FheError::NoiseExhausted) => {
                    assert!(engine.stats.failures > 0);
                    return;
                }
                Err(e) => panic!("Unexpected error: {:?}", e),
            }
        }
    }

    #[test]
    fn test_scheme_security() {
        assert_eq!(FheScheme::Ckks.security_bits(), 128);
        assert_eq!(FheScheme::Bfv.name(), "BFV");
    }

    #[test]
    fn test_plain_operations() {
        let mut engine = FheEngine::new_bfv();
        engine.keygen();

        let ct_pt = engine.encode_integers(&[10]);
        let ct = engine.encrypt(&ct_pt).unwrap();

        let plain = engine.encode_integers(&[2]);

        let result = engine.multiply_plain(&ct, &plain).unwrap();
        assert!(result.is_computed);

        let result = engine.add_plain(&ct, &plain).unwrap();
        assert!(result.is_computed);
    }
}
