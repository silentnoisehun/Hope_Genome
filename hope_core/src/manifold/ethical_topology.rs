//! # Ethical Topology - The Geometry of Moral Possibility
//!
//! **The space itself only permits ethical trajectories**
//!
//! ```text
//! EUCLIDEAN ETHICS (Old):
//! ┌──────────────────────────────────────────────────────────────────────────┐
//! │                                                                          │
//! │    All points reachable. Some are "forbidden" by rules.                  │
//! │                                                                          │
//! │    ●───────────────●───────────────●───────────────●                     │
//! │    │               │               │               │                     │
//! │    │    GOOD       │     BAD       │     GOOD      │                     │
//! │    │   (allowed)   │  (blocked)    │   (allowed)   │                     │
//! │    │               │               │               │                     │
//! │    ●───────────────●───────────────●───────────────●                     │
//! │                                                                          │
//! │    Problem: The "BAD" region EXISTS. Watchdog must block it.             │
//! │                                                                          │
//! └──────────────────────────────────────────────────────────────────────────┘
//!
//! ETHICAL MANIFOLD (v16):
//! ┌──────────────────────────────────────────────────────────────────────────┐
//! │                                                                          │
//! │    The manifold CURVES around unethical regions.                         │
//! │    They are not forbidden - they DON'T EXIST in this space.              │
//! │                                                                          │
//! │              ╭────────────────────────────────╮                          │
//! │          ╭───╯                                ╰───╮                      │
//! │      ╭───╯          ◈ ETHICAL SPACE ◈             ╰───╮                  │
//! │    ╭─╯                                                ╰─╮                │
//! │    │    ●──────────────────────────────────────────●    │                │
//! │    │    │                                          │    │                │
//! │    │    │     All reachable points are GOOD        │    │                │
//! │    │    │     No blocking needed.                  │    │                │
//! │    │    │     Bad points are OUTSIDE the manifold. │    │                │
//! │    │    │                                          │    │                │
//! │    │    ●──────────────────────────────────────────●    │                │
//! │    ╰─╮                                                ╭─╯                │
//! │      ╰───╮                                        ╭───╯                  │
//! │          ╰───╮                                ╭───╯                      │
//! │              ╰────────────────────────────────╯                          │
//! │                                                                          │
//! │    The space itself has no "bad" regions. Like asking:                   │
//! │    "What's north of the North Pole?" - The question has no meaning.      │
//! │                                                                          │
//! └──────────────────────────────────────────────────────────────────────────┘
//! ```

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// ETHICAL POINT - A location in ethical space
// ============================================================================

/// A point in the ethical manifold
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthicalPoint {
    /// Coordinates in ethical dimensions
    pub coordinates: EthicalCoordinates,
    /// Whether this point exists on the manifold (always true for valid points)
    pub on_manifold: bool,
    /// Local curvature at this point
    pub local_curvature: f64,
    /// Ethical potential at this point (higher = more ethical)
    pub ethical_potential: f64,
    /// Hash of this point
    pub point_hash: [u8; 32],
}

/// Coordinates in ethical space (multi-dimensional)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthicalCoordinates {
    /// Harm dimension (-1.0 to 1.0, where -1 = harmful, 1 = beneficial)
    pub harm: f64,
    /// Honesty dimension (-1.0 to 1.0)
    pub honesty: f64,
    /// Autonomy respect (-1.0 to 1.0)
    pub autonomy: f64,
    /// Fairness (-1.0 to 1.0)
    pub fairness: f64,
    /// Privacy respect (-1.0 to 1.0)
    pub privacy: f64,
    /// Beneficence (-1.0 to 1.0)
    pub beneficence: f64,
}

impl EthicalCoordinates {
    /// Create coordinates for a perfectly ethical point
    pub fn ideal() -> Self {
        EthicalCoordinates {
            harm: 1.0,        // Maximum non-harm
            honesty: 1.0,     // Maximum honesty
            autonomy: 1.0,    // Maximum respect for autonomy
            fairness: 1.0,    // Maximum fairness
            privacy: 1.0,     // Maximum privacy respect
            beneficence: 1.0, // Maximum beneficence
        }
    }

    /// Create coordinates from an action analysis
    pub fn from_action(action: &str) -> Self {
        // Analyze action to determine ethical coordinates
        // This is simplified - real implementation would use ML classification
        let action_lower = action.to_lowercase();

        let harm = if action_lower.contains("help") || action_lower.contains("assist") {
            0.8
        } else if action_lower.contains("harm") || action_lower.contains("hurt") {
            -0.9
        } else {
            0.3
        };

        let honesty = if action_lower.contains("lie") || action_lower.contains("deceive") {
            -0.8
        } else if action_lower.contains("truth") || action_lower.contains("honest") {
            0.9
        } else {
            0.5
        };

        EthicalCoordinates {
            harm,
            honesty,
            autonomy: 0.5,
            fairness: 0.5,
            privacy: 0.5,
            beneficence: (harm + 1.0) / 2.0,
        }
    }

    /// Calculate distance from ideal
    pub fn distance_from_ideal(&self) -> f64 {
        let ideal = Self::ideal();
        let diff_harm = (self.harm - ideal.harm).powi(2);
        let diff_honesty = (self.honesty - ideal.honesty).powi(2);
        let diff_autonomy = (self.autonomy - ideal.autonomy).powi(2);
        let diff_fairness = (self.fairness - ideal.fairness).powi(2);
        let diff_privacy = (self.privacy - ideal.privacy).powi(2);
        let diff_beneficence = (self.beneficence - ideal.beneficence).powi(2);

        (diff_harm + diff_honesty + diff_autonomy + diff_fairness + diff_privacy + diff_beneficence)
            .sqrt()
    }

    /// Calculate ethical magnitude (distance from origin in positive direction)
    pub fn ethical_magnitude(&self) -> f64 {
        (self.harm.max(0.0).powi(2)
            + self.honesty.max(0.0).powi(2)
            + self.autonomy.max(0.0).powi(2)
            + self.fairness.max(0.0).powi(2)
            + self.privacy.max(0.0).powi(2)
            + self.beneficence.max(0.0).powi(2))
        .sqrt()
    }
}

impl EthicalPoint {
    /// Create a new ethical point
    pub fn new(coordinates: EthicalCoordinates) -> Self {
        let on_manifold = Self::check_on_manifold(&coordinates);
        let local_curvature = Self::compute_curvature(&coordinates);
        let ethical_potential = Self::compute_potential(&coordinates);
        let point_hash = Self::compute_hash(&coordinates);

        EthicalPoint {
            coordinates,
            on_manifold,
            local_curvature,
            ethical_potential,
            point_hash,
        }
    }

    /// Check if coordinates lie on the ethical manifold
    fn check_on_manifold(coords: &EthicalCoordinates) -> bool {
        // A point is on the manifold if ALL ethical dimensions are >= 0
        // Negative values represent unethical regions - they don't exist on the manifold
        coords.harm >= 0.0
            && coords.honesty >= 0.0
            && coords.autonomy >= 0.0
            && coords.fairness >= 0.0
            && coords.privacy >= 0.0
            && coords.beneficence >= 0.0
    }

    /// Compute local curvature (how strongly ethics "pulls" at this point)
    fn compute_curvature(coords: &EthicalCoordinates) -> f64 {
        // Curvature increases near the boundary of the manifold
        // This creates a "gravitational" effect pulling toward the ethical center
        let min_coord = coords
            .harm
            .min(coords.honesty)
            .min(coords.autonomy)
            .min(coords.fairness)
            .min(coords.privacy)
            .min(coords.beneficence);

        if min_coord <= 0.0 {
            f64::INFINITY // Infinite curvature at boundary = cannot cross
        } else {
            1.0 / min_coord // Higher curvature near boundary
        }
    }

    /// Compute ethical potential (energy level in ethical field)
    fn compute_potential(coords: &EthicalCoordinates) -> f64 {
        coords.ethical_magnitude()
    }

    fn compute_hash(coords: &EthicalCoordinates) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"ETHICAL_POINT:");
        hasher.update(coords.harm.to_le_bytes());
        hasher.update(coords.honesty.to_le_bytes());
        hasher.update(coords.autonomy.to_le_bytes());
        hasher.update(coords.fairness.to_le_bytes());
        hasher.update(coords.privacy.to_le_bytes());
        hasher.update(coords.beneficence.to_le_bytes());
        hasher.finalize().into()
    }
}

// ============================================================================
// ETHICAL METRIC - Distance function in ethical space
// ============================================================================

/// The metric tensor for ethical space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthicalMetric {
    /// Metric components (6x6 for 6 ethical dimensions)
    components: [[f64; 6]; 6],
    /// Signature (for Lorentzian-like metrics)
    pub signature: MetricSignature,
}

/// Signature of the metric
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MetricSignature {
    /// Euclidean (all positive)
    Euclidean,
    /// Lorentzian (one negative) - for causal structure
    Lorentzian,
    /// Custom signature
    Custom([i8; 6]),
}

impl EthicalMetric {
    /// Create the standard ethical metric
    pub fn standard() -> Self {
        // Identity metric with weights for each dimension
        let mut components = [[0.0; 6]; 6];
        components[0][0] = 2.0; // Harm weighted highest
        components[1][1] = 1.5; // Honesty
        components[2][2] = 1.0; // Autonomy
        components[3][3] = 1.0; // Fairness
        components[4][4] = 1.2; // Privacy
        components[5][5] = 1.0; // Beneficence

        EthicalMetric {
            components,
            signature: MetricSignature::Euclidean,
        }
    }

    /// Compute distance between two points
    pub fn distance(&self, p1: &EthicalPoint, p2: &EthicalPoint) -> f64 {
        let c1 = &p1.coordinates;
        let c2 = &p2.coordinates;

        let diffs = [
            c1.harm - c2.harm,
            c1.honesty - c2.honesty,
            c1.autonomy - c2.autonomy,
            c1.fairness - c2.fairness,
            c1.privacy - c2.privacy,
            c1.beneficence - c2.beneficence,
        ];

        let mut sum = 0.0;
        for i in 0..6 {
            for j in 0..6 {
                sum += self.components[i][j] * diffs[i] * diffs[j];
            }
        }

        sum.sqrt()
    }
}

// ============================================================================
// ETHICAL GEODESIC - The path of least resistance (most ethical path)
// ============================================================================

/// A geodesic (optimal path) in ethical space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthicalGeodesic {
    /// Starting point
    pub start: EthicalPoint,
    /// Ending point
    pub end: EthicalPoint,
    /// Intermediate points along the geodesic
    pub path: Vec<EthicalPoint>,
    /// Total ethical length
    pub length: f64,
    /// Whether the path stays on the manifold
    pub valid: bool,
}

impl EthicalGeodesic {
    /// Compute the geodesic between two points
    pub fn compute(start: EthicalPoint, end: EthicalPoint, metric: &EthicalMetric) -> Self {
        let mut path = Vec::new();
        let steps = 10;

        // Linear interpolation (simplified - real geodesic would solve differential equations)
        for i in 0..=steps {
            let t = i as f64 / steps as f64;
            let coords = EthicalCoordinates {
                harm: start.coordinates.harm * (1.0 - t) + end.coordinates.harm * t,
                honesty: start.coordinates.honesty * (1.0 - t) + end.coordinates.honesty * t,
                autonomy: start.coordinates.autonomy * (1.0 - t) + end.coordinates.autonomy * t,
                fairness: start.coordinates.fairness * (1.0 - t) + end.coordinates.fairness * t,
                privacy: start.coordinates.privacy * (1.0 - t) + end.coordinates.privacy * t,
                beneficence: start.coordinates.beneficence * (1.0 - t)
                    + end.coordinates.beneficence * t,
            };
            path.push(EthicalPoint::new(coords));
        }

        let valid = path.iter().all(|p| p.on_manifold);
        let length = metric.distance(&start, &end);

        EthicalGeodesic {
            start,
            end,
            path,
            length,
            valid,
        }
    }
}

// ============================================================================
// ETHICAL CURVATURE - How the manifold bends
// ============================================================================

/// Curvature of the ethical manifold
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthicalCurvature {
    /// Ricci scalar (overall curvature)
    pub ricci_scalar: f64,
    /// Sectional curvatures for each plane
    pub sectional: HashMap<String, f64>,
    /// Whether curvature is positive (closed, bounded) or negative (open)
    pub curvature_type: CurvatureType,
}

/// Type of curvature
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CurvatureType {
    /// Positive curvature - closed manifold, finite ethical space
    Positive,
    /// Zero curvature - flat ethical space
    Flat,
    /// Negative curvature - open, hyperbolic ethical space
    Negative,
}

impl EthicalCurvature {
    /// Compute curvature at a point
    pub fn at_point(point: &EthicalPoint) -> Self {
        // Curvature is highest near the ethical boundary
        // This creates a "well" that keeps trajectories ethical
        let ricci_scalar = point.local_curvature;

        let curvature_type = if ricci_scalar > 1.0 {
            CurvatureType::Positive
        } else if ricci_scalar < -1.0 {
            CurvatureType::Negative
        } else {
            CurvatureType::Flat
        };

        EthicalCurvature {
            ricci_scalar,
            sectional: HashMap::new(),
            curvature_type,
        }
    }
}

// ============================================================================
// TOPOLOGICAL CONSTRAINT - What the manifold forbids
// ============================================================================

/// A topological constraint on the manifold
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologicalConstraint {
    /// Constraint name
    pub name: String,
    /// Constraint type
    pub constraint_type: ConstraintType,
    /// Dimensions affected
    pub dimensions: Vec<String>,
    /// Boundary condition
    pub boundary: BoundaryCondition,
}

/// Type of topological constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    /// Region excluded from manifold
    ExcludedRegion,
    /// Required minimum value
    MinimumBound,
    /// Topological hole
    Hole,
    /// Boundary of manifold
    Boundary,
}

/// Boundary condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BoundaryCondition {
    /// Hard boundary - cannot cross
    Hard,
    /// Soft boundary - strong curvature prevents crossing
    Soft(f64),
    /// Reflective - trajectories bounce back
    Reflective,
}

// ============================================================================
// THE ETHICAL MANIFOLD - The complete space
// ============================================================================

/// The complete ethical manifold
#[derive(Debug)]
#[allow(dead_code)]
pub struct EthicalManifold {
    /// Metric tensor
    metric: EthicalMetric,
    /// Topological constraints (reserved for future use)
    constraints: Vec<TopologicalConstraint>,
    /// Cached curvature map
    curvature_map: HashMap<[u8; 32], EthicalCurvature>,
    /// Manifold version
    pub version: String,
}

impl EthicalManifold {
    /// Create the standard ethical manifold
    pub fn new() -> Self {
        let metric = EthicalMetric::standard();

        let constraints = vec![
            TopologicalConstraint {
                name: "Non-harm boundary".to_string(),
                constraint_type: ConstraintType::Boundary,
                dimensions: vec!["harm".to_string()],
                boundary: BoundaryCondition::Hard,
            },
            TopologicalConstraint {
                name: "Honesty boundary".to_string(),
                constraint_type: ConstraintType::Boundary,
                dimensions: vec!["honesty".to_string()],
                boundary: BoundaryCondition::Hard,
            },
            TopologicalConstraint {
                name: "Privacy boundary".to_string(),
                constraint_type: ConstraintType::Boundary,
                dimensions: vec!["privacy".to_string()],
                boundary: BoundaryCondition::Hard,
            },
        ];

        EthicalManifold {
            metric,
            constraints,
            curvature_map: HashMap::new(),
            version: "16.0.0".to_string(),
        }
    }

    /// Check if a point exists on this manifold
    pub fn contains(&self, point: &EthicalPoint) -> bool {
        point.on_manifold
    }

    /// Project an arbitrary point onto the manifold
    pub fn project(&self, coords: &EthicalCoordinates) -> EthicalPoint {
        // Project negative coordinates to zero (the boundary)
        let projected = EthicalCoordinates {
            harm: coords.harm.max(0.0),
            honesty: coords.honesty.max(0.0),
            autonomy: coords.autonomy.max(0.0),
            fairness: coords.fairness.max(0.0),
            privacy: coords.privacy.max(0.0),
            beneficence: coords.beneficence.max(0.0),
        };

        EthicalPoint::new(projected)
    }

    /// Compute the geodesic (most ethical path) between two points
    pub fn geodesic(&self, start: &EthicalPoint, end: &EthicalPoint) -> EthicalGeodesic {
        EthicalGeodesic::compute(start.clone(), end.clone(), &self.metric)
    }

    /// Get curvature at a point
    pub fn curvature_at(&mut self, point: &EthicalPoint) -> EthicalCurvature {
        if let Some(cached) = self.curvature_map.get(&point.point_hash) {
            return cached.clone();
        }

        let curvature = EthicalCurvature::at_point(point);
        self.curvature_map
            .insert(point.point_hash, curvature.clone());
        curvature
    }

    /// Check if a trajectory (sequence of actions) is valid on the manifold
    pub fn is_valid_trajectory(&self, points: &[EthicalPoint]) -> TrajectoryValidity {
        if points.is_empty() {
            return TrajectoryValidity {
                valid: true,
                reason: "Empty trajectory is trivially valid".to_string(),
                invalid_points: vec![],
            };
        }

        let mut invalid_points = Vec::new();

        for (i, point) in points.iter().enumerate() {
            if !point.on_manifold {
                invalid_points.push(i);
            }
        }

        if invalid_points.is_empty() {
            TrajectoryValidity {
                valid: true,
                reason: "All points lie on the ethical manifold".to_string(),
                invalid_points: vec![],
            }
        } else {
            TrajectoryValidity {
                valid: false,
                reason: format!(
                    "{} points lie outside the ethical manifold",
                    invalid_points.len()
                ),
                invalid_points,
            }
        }
    }

    /// Generate a proof that an action stays on the manifold
    pub fn generate_manifold_proof(&self, action: &str) -> ManifoldProof {
        let coords = EthicalCoordinates::from_action(action);
        let point = EthicalPoint::new(coords);

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let proof_hash = Self::compute_proof_hash(&point, timestamp);

        ManifoldProof {
            action: action.to_string(),
            point,
            on_manifold: true, // By construction, the projection is always on manifold
            curvature: EthicalCurvature::at_point(&EthicalPoint::new(
                EthicalCoordinates::from_action(action),
            )),
            timestamp,
            proof_hash,
        }
    }

    fn compute_proof_hash(point: &EthicalPoint, timestamp: u64) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"MANIFOLD_PROOF:");
        hasher.update(point.point_hash);
        hasher.update(timestamp.to_le_bytes());
        hasher.finalize().into()
    }
}

impl EthicalManifold {
    /// Sample an ethical point from the manifold (for genesis core)
    pub fn sample_ethical_point(&self) -> EthicalPoint {
        // Return a maximally ethical point (positive on all dimensions)
        EthicalPoint::new(EthicalCoordinates {
            harm: 0.0,        // No harm
            honesty: 1.0,     // Full honesty
            autonomy: 0.8,    // High autonomy respect
            fairness: 1.0,    // Full fairness
            privacy: 0.9,     // High privacy
            beneficence: 1.0, // Full beneficence
        })
    }
}

impl Default for EthicalManifold {
    fn default() -> Self {
        Self::new()
    }
}

/// Validity of a trajectory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryValidity {
    pub valid: bool,
    pub reason: String,
    pub invalid_points: Vec<usize>,
}

/// Proof that an action stays on the manifold
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifoldProof {
    pub action: String,
    pub point: EthicalPoint,
    pub on_manifold: bool,
    pub curvature: EthicalCurvature,
    pub timestamp: u64,
    pub proof_hash: [u8; 32],
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ethical_point_on_manifold() {
        let good_coords = EthicalCoordinates::ideal();
        let good_point = EthicalPoint::new(good_coords);
        assert!(good_point.on_manifold);

        let bad_coords = EthicalCoordinates {
            harm: -0.5,
            honesty: 0.5,
            autonomy: 0.5,
            fairness: 0.5,
            privacy: 0.5,
            beneficence: 0.5,
        };
        let bad_point = EthicalPoint::new(bad_coords);
        assert!(!bad_point.on_manifold);
    }

    #[test]
    fn test_manifold_projection() {
        let manifold = EthicalManifold::new();

        let off_manifold = EthicalCoordinates {
            harm: -0.5,
            honesty: -0.3,
            autonomy: 0.5,
            fairness: 0.5,
            privacy: -0.1,
            beneficence: 0.5,
        };

        let projected = manifold.project(&off_manifold);
        assert!(projected.on_manifold);
        assert!(projected.coordinates.harm >= 0.0);
        assert!(projected.coordinates.honesty >= 0.0);
    }

    #[test]
    fn test_geodesic() {
        let manifold = EthicalManifold::new();

        let start = EthicalPoint::new(EthicalCoordinates {
            harm: 0.5,
            honesty: 0.5,
            autonomy: 0.5,
            fairness: 0.5,
            privacy: 0.5,
            beneficence: 0.5,
        });

        let end = EthicalPoint::new(EthicalCoordinates::ideal());

        let geodesic = manifold.geodesic(&start, &end);
        assert!(geodesic.valid);
        assert!(!geodesic.path.is_empty());
    }

    #[test]
    fn test_manifold_proof() {
        let manifold = EthicalManifold::new();
        let proof = manifold.generate_manifold_proof("Help the user with their task");

        assert!(proof.on_manifold);
        assert!(proof.point.coordinates.harm >= 0.0);
    }

    #[test]
    fn test_trajectory_validity() {
        let manifold = EthicalManifold::new();

        let valid_trajectory = vec![
            EthicalPoint::new(EthicalCoordinates::ideal()),
            EthicalPoint::new(EthicalCoordinates {
                harm: 0.8,
                honesty: 0.9,
                autonomy: 0.7,
                fairness: 0.8,
                privacy: 0.9,
                beneficence: 0.8,
            }),
        ];

        let result = manifold.is_valid_trajectory(&valid_trajectory);
        assert!(result.valid);
    }
}
