use crate::crypto::KeyPair;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConsensusError {
    #[error("No consensus achieved: required {required} sources, got {achieved}")]
    NoConsensus { required: usize, achieved: usize },

    #[error("Invalid signature from source: {0}")]
    InvalidSignature(String),

    #[error("Insufficient readings: required at least {required}, got {actual}")]
    InsufficientReadings { required: usize, actual: usize },

    #[error("Crypto error: {0}")]
    CryptoError(#[from] crate::crypto::CryptoError),
}

pub type Result<T> = std::result::Result<T, ConsensusError>;

/// A sensor reading with cryptographic signature
///
/// Used for multi-source reality verification (Byzantine Fault Tolerance)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorReading {
    /// The measured value
    pub value: f64,

    /// Unix timestamp when reading was taken
    pub timestamp: u64,

    /// Unique identifier for the sensor
    pub source_id: String,

    /// Cryptographic signature of (value, timestamp, source_id)
    pub signature: Vec<u8>,
}

impl SensorReading {
    /// Create a new sensor reading
    pub fn new(value: f64, source_id: String) -> Self {
        let timestamp = chrono::Utc::now().timestamp() as u64;

        SensorReading {
            value,
            timestamp,
            source_id,
            signature: Vec::new(), // Will be filled by signing
        }
    }

    /// Get the data to be signed
    pub fn signing_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.value.to_le_bytes());
        data.extend_from_slice(&self.timestamp.to_le_bytes());
        data.extend_from_slice(self.source_id.as_bytes());
        data
    }

    /// Sign this reading
    pub fn sign(&mut self, keypair: &KeyPair) -> Result<()> {
        let data = self.signing_data();
        self.signature = keypair.sign(&data)?;
        Ok(())
    }

    /// Verify signature
    pub fn verify(&self, keypair: &KeyPair) -> Result<()> {
        let data = self.signing_data();
        keypair
            .verify(&data, &self.signature)
            .map_err(|_| ConsensusError::InvalidSignature(self.source_id.clone()))?;
        Ok(())
    }
}

/// Multi-source consensus verifier (Byzantine Fault Tolerant)
///
/// Prevents "oracle attacks" on sensor data by requiring
/// agreement from multiple independent sources.
pub struct ConsensusVerifier {
    /// Minimum number of sources that must agree
    required_sources: usize,

    /// Tolerance for values to be considered "in agreement"
    tolerance: f64,
}

impl ConsensusVerifier {
    /// Create a new consensus verifier
    ///
    /// # Arguments
    /// * `required_sources` - Minimum sources that must agree (e.g., 3 for 2/3 majority)
    /// * `tolerance` - Max difference from median to be considered agreeing
    pub fn new(required_sources: usize, tolerance: f64) -> Self {
        ConsensusVerifier {
            required_sources,
            tolerance,
        }
    }

    /// Verify readings and return consensus value
    ///
    /// This implements Byzantine Fault Tolerance:
    /// 1. Verify each sensor signature
    /// 2. Calculate median value
    /// 3. Count how many sensors agree (within tolerance)
    /// 4. Require minimum number of agreeing sensors
    pub fn verify_readings(&self, readings: &[SensorReading], keypairs: &[KeyPair]) -> Result<f64> {
        // Check we have enough readings
        if readings.len() < self.required_sources {
            return Err(ConsensusError::InsufficientReadings {
                required: self.required_sources,
                actual: readings.len(),
            });
        }

        // Verify each sensor signature
        for (reading, keypair) in readings.iter().zip(keypairs.iter()) {
            reading.verify(keypair)?;
        }

        // Calculate median
        let median = self.calculate_median(readings);

        // Count consensus (how many within tolerance of median)
        let consensus_count = readings
            .iter()
            .filter(|r| (r.value - median).abs() < self.tolerance)
            .count();

        // Check if we have consensus
        if consensus_count < self.required_sources {
            return Err(ConsensusError::NoConsensus {
                required: self.required_sources,
                achieved: consensus_count,
            });
        }

        Ok(median)
    }

    /// Calculate median of sensor readings
    fn calculate_median(&self, readings: &[SensorReading]) -> f64 {
        let mut values: Vec<f64> = readings.iter().map(|r| r.value).collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let len = values.len();
        if len.is_multiple_of(2) {
            (values[len / 2 - 1] + values[len / 2]) / 2.0
        } else {
            values[len / 2]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_signed_reading(value: f64, source_id: &str, keypair: &KeyPair) -> SensorReading {
        let mut reading = SensorReading::new(value, source_id.to_string());
        reading.sign(keypair).unwrap();
        reading
    }

    #[test]
    fn test_sensor_reading_creation() {
        let reading = SensorReading::new(42.0, "sensor1".to_string());
        assert_eq!(reading.value, 42.0);
        assert_eq!(reading.source_id, "sensor1");
    }

    #[test]
    fn test_sensor_reading_signature() {
        let keypair = KeyPair::generate().unwrap();
        let mut reading = SensorReading::new(42.0, "sensor1".to_string());

        reading.sign(&keypair).unwrap();
        assert!(!reading.signature.is_empty());

        // Verify should succeed
        assert!(reading.verify(&keypair).is_ok());
    }

    #[test]
    fn test_consensus_with_agreement() {
        let verifier = ConsensusVerifier::new(3, 0.1);

        let keypairs: Vec<KeyPair> = (0..4).map(|_| KeyPair::generate().unwrap()).collect();

        let readings = vec![
            create_signed_reading(10.0, "A", &keypairs[0]),
            create_signed_reading(10.05, "B", &keypairs[1]),
            create_signed_reading(10.02, "C", &keypairs[2]),
            create_signed_reading(10.01, "D", &keypairs[3]),
        ];

        let result = verifier.verify_readings(&readings, &keypairs).unwrap();
        assert!((result - 10.015).abs() < 0.01); // Median should be ~10.015
    }

    #[test]
    fn test_consensus_fails_with_disagreement() {
        let verifier = ConsensusVerifier::new(3, 0.1);

        let keypairs: Vec<KeyPair> = (0..3).map(|_| KeyPair::generate().unwrap()).collect();

        let readings = vec![
            create_signed_reading(10.0, "A", &keypairs[0]),
            create_signed_reading(10.05, "B", &keypairs[1]),
            create_signed_reading(50.0, "C", &keypairs[2]), // Outlier
        ];

        let result = verifier.verify_readings(&readings, &keypairs);
        assert!(matches!(result, Err(ConsensusError::NoConsensus { .. })));
    }

    #[test]
    fn test_insufficient_readings() {
        let verifier = ConsensusVerifier::new(5, 0.1);

        let keypairs: Vec<KeyPair> = (0..2).map(|_| KeyPair::generate().unwrap()).collect();

        let readings = vec![
            create_signed_reading(10.0, "A", &keypairs[0]),
            create_signed_reading(10.0, "B", &keypairs[1]),
        ];

        let result = verifier.verify_readings(&readings, &keypairs);
        assert!(matches!(
            result,
            Err(ConsensusError::InsufficientReadings { .. })
        ));
    }

    #[test]
    fn test_invalid_signature_detected() {
        let verifier = ConsensusVerifier::new(2, 0.1);

        let keypairs: Vec<KeyPair> = (0..2).map(|_| KeyPair::generate().unwrap()).collect();

        let mut readings = vec![
            create_signed_reading(10.0, "A", &keypairs[0]),
            create_signed_reading(10.0, "B", &keypairs[1]),
        ];

        // Tamper with signature
        readings[0].signature[0] ^= 0xFF;

        let result = verifier.verify_readings(&readings, &keypairs);
        assert!(matches!(
            result,
            Err(ConsensusError::InvalidSignature { .. })
        ));
    }

    #[test]
    fn test_median_calculation_odd() {
        let verifier = ConsensusVerifier::new(3, 0.1);

        let keypairs: Vec<KeyPair> = (0..5).map(|_| KeyPair::generate().unwrap()).collect();

        let readings = vec![
            create_signed_reading(1.0, "A", &keypairs[0]),
            create_signed_reading(2.0, "B", &keypairs[1]),
            create_signed_reading(3.0, "C", &keypairs[2]),
            create_signed_reading(4.0, "D", &keypairs[3]),
            create_signed_reading(5.0, "E", &keypairs[4]),
        ];

        let median = verifier.calculate_median(&readings);
        assert_eq!(median, 3.0);
    }

    #[test]
    fn test_median_calculation_even() {
        let verifier = ConsensusVerifier::new(2, 0.1);

        let keypairs: Vec<KeyPair> = (0..4).map(|_| KeyPair::generate().unwrap()).collect();

        let readings = vec![
            create_signed_reading(1.0, "A", &keypairs[0]),
            create_signed_reading(2.0, "B", &keypairs[1]),
            create_signed_reading(3.0, "C", &keypairs[2]),
            create_signed_reading(4.0, "D", &keypairs[3]),
        ];

        let median = verifier.calculate_median(&readings);
        assert_eq!(median, 2.5);
    }
}
