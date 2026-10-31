//! Collision Avoidance Module
//!
//! Detect and avoid collisions with obstacles and other vehicles.

use crate::common::Pose;
use glam::Vec3;

/// Collision avoidance system
#[derive(Debug, Default)]
pub struct CollisionAvoidance {
    /// Safety radius around each vehicle (meters)
    safety_radius: f32,
    /// Time horizon for collision prediction (seconds)
    time_horizon: f32,
    /// Minimum safe distance (meters)
    min_safe_distance: f32,
    /// Enabled
    enabled: bool,
}

impl CollisionAvoidance {
    /// Create new collision avoidance system
    pub fn new() -> Self {
        Self {
            safety_radius: 2.0,
            time_horizon: 5.0,
            min_safe_distance: 5.0,
            enabled: true,
        }
    }

    /// Set safety radius
    pub fn with_safety_radius(mut self, radius: f32) -> Self {
        self.safety_radius = radius;
        self
    }

    /// Set time horizon
    pub fn with_time_horizon(mut self, horizon: f32) -> Self {
        self.time_horizon = horizon;
        self
    }

    /// Set minimum safe distance
    pub fn with_min_safe_distance(mut self, distance: f32) -> Self {
        self.min_safe_distance = distance;
        self
    }

    /// Enable/disable collision avoidance
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if enabled
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Check for potential collision between two vehicles
    pub fn check_collision(
        &self,
        vehicle1: &VehicleState,
        vehicle2: &VehicleState,
    ) -> Option<CollisionPrediction> {
        if !self.enabled {
            return None;
        }

        let pos1 = vehicle1.pose.position;
        let pos2 = vehicle2.pose.position;
        let vel1 = vehicle1.velocity;
        let vel2 = vehicle2.velocity;

        // Relative position and velocity
        let rel_pos = pos2 - pos1;
        let rel_vel = vel2 - vel1;

        // Calculate time to closest approach
        let rel_speed_sq = rel_vel.length_squared();

        if rel_speed_sq < 0.001 {
            // Vehicles are moving at same velocity, check current distance
            let current_dist = rel_pos.length();
            let min_dist = self.safety_radius * 2.0;

            if current_dist < min_dist {
                return Some(CollisionPrediction {
                    time_to_collision: 0.0,
                    miss_distance: current_dist,
                    severity: CollisionSeverity::Immediate,
                });
            }
            return None;
        }

        // Time to closest approach
        let t_cpa = -(rel_pos.dot(rel_vel)) / rel_speed_sq;

        // Only consider future collisions
        if t_cpa < 0.0 || t_cpa > self.time_horizon {
            return None;
        }

        // Position at closest approach
        let pos1_cpa = pos1 + vel1 * t_cpa;
        let pos2_cpa = pos2 + vel2 * t_cpa;

        // Distance at closest approach
        let miss_distance = (pos2_cpa - pos1_cpa).length();

        // Check if this is a collision
        let min_separation = self.safety_radius * 2.0;

        if miss_distance < min_separation {
            // How near the miss is, as a share of the separation we keep.
            // `Immediate` is for paths that actually meet rather than pass
            // close: without it the top of the scale was reachable only by
            // two vehicles already overlapping and drifting at the same
            // velocity (the branch above), so a head-on closing at 20 m/s
            // dead centre ranked below two craft resting against each
            // other - the wrong way round for anything that avoids by
            // severity.
            let near = miss_distance / min_separation;
            let severity = match near {
                n if n < 0.125 => CollisionSeverity::Immediate,
                n if n < 0.5 => CollisionSeverity::Critical,
                n if n < 0.8 => CollisionSeverity::High,
                _ => CollisionSeverity::Moderate,
            };

            Some(CollisionPrediction {
                time_to_collision: t_cpa,
                miss_distance,
                severity,
            })
        } else {
            None
        }
    }

    /// Generate avoidance maneuver
    pub fn generate_avoidance(
        &self,
        vehicle: &VehicleState,
        obstacle: &VehicleState,
    ) -> Option<AvoidanceManeuver> {
        let prediction = self.check_collision(vehicle, obstacle)?;

        // Calculate avoidance direction (perpendicular to line of sight)
        let to_obstacle = obstacle.pose.position - vehicle.pose.position;
        let up = Vec3::Z;

        // Prefer vertical avoidance if possible
        let avoidance_dir = if to_obstacle.dot(up).abs() < 0.5 {
            // Significant horizontal separation needed
            to_obstacle.cross(up).normalize()
        } else {
            // Go up
            up
        };

        // Calculate required velocity change
        let required_separation = self.min_safe_distance + self.safety_radius * 2.0;
        let current_separation = to_obstacle.length();
        let separation_needed = (required_separation - current_separation).max(0.0);

        let avoidance_velocity = avoidance_dir * separation_needed.max(2.0);

        Some(AvoidanceManeuver {
            target_velocity: vehicle.velocity + avoidance_velocity,
            duration: prediction.time_to_collision * 0.5, // Act early
            priority: prediction.severity as u8,
        })
    }
}

/// Vehicle state for collision checking
#[derive(Debug, Clone)]
pub struct VehicleState {
    /// Vehicle name
    pub name: String,
    /// Current pose
    pub pose: Pose,
    /// Linear velocity
    pub velocity: Vec3,
    /// Vehicle type
    pub vehicle_type: VehicleType,
}

/// Vehicle type for collision avoidance
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VehicleType {
    /// Multirotor drone
    Multirotor,
    /// Fixed wing aircraft
    FixedWing,
    /// Car/ground vehicle
    Car,
    /// Obstacle/static object
    Obstacle,
}

/// Collision prediction result
#[derive(Debug, Clone)]
pub struct CollisionPrediction {
    /// Time to collision in seconds
    pub time_to_collision: f32,
    /// Distance at closest approach (miss distance)
    pub miss_distance: f32,
    /// Collision severity
    pub severity: CollisionSeverity,
}

/// Collision severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum CollisionSeverity {
    /// No collision predicted
    None = 0,
    /// Low risk - far in future or large miss distance
    Low = 1,
    /// Moderate risk
    Moderate = 2,
    /// High risk
    High = 3,
    /// Critical - imminent collision
    Critical = 4,
    /// Immediate collision
    Immediate = 5,
}

/// Avoidance maneuver
#[derive(Debug, Clone)]
pub struct AvoidanceManeuver {
    /// Target velocity for avoidance
    pub target_velocity: Vec3,
    /// Duration of maneuver
    pub duration: f32,
    /// Priority (higher = more urgent)
    pub priority: u8,
}

/// Proximity alert for obstacles
#[derive(Debug, Clone)]
pub struct ProximityAlert {
    /// Alert level
    pub level: ProximityLevel,
    /// Direction to obstacle
    pub direction: Vec3,
    /// Distance to obstacle
    pub distance: f32,
    /// Obstacle name/type
    pub obstacle_name: String,
}

/// Proximity alert levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProximityLevel {
    /// Info - obstacle far away
    Info,
    /// Warning - getting close
    Warning,
    /// Critical - very close
    Critical,
    /// Emergency - collision imminent
    Emergency,
}

/// Obstacle database for static obstacles
#[derive(Debug, Default)]
pub struct ObstacleDatabase {
    obstacles: Vec<Obstacle>,
}

impl ObstacleDatabase {
    /// Create new obstacle database
    pub fn new() -> Self {
        Self {
            obstacles: Vec::new(),
        }
    }

    /// Add obstacle
    pub fn add(&mut self, obstacle: Obstacle) {
        self.obstacles.push(obstacle);
    }

    /// Find obstacles near a position
    pub fn find_nearby(&self, position: Vec3, radius: f32) -> Vec<&Obstacle> {
        self.obstacles
            .iter()
            .filter(|o| (o.position - position).length() < radius)
            .collect()
    }

    /// Get all obstacles
    pub fn all(&self) -> &[Obstacle] {
        &self.obstacles
    }
}

/// Static obstacle definition
#[derive(Debug, Clone)]
pub struct Obstacle {
    /// Obstacle name
    pub name: String,
    /// Position
    pub position: Vec3,
    /// Bounding box dimensions
    pub dimensions: Vec3,
    /// Obstacle type
    pub obstacle_type: ObstacleType,
}

/// Obstacle types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObstacleType {
    /// Building/structure
    Building,
    /// Tree/vegetation
    Tree,
    /// Power line
    PowerLine,
    /// Terrain
    Terrain,
    /// No-fly zone
    NoFlyZone,
    /// Other
    Other,
}

/// Minimum separation calculator for different vehicle types
pub struct SeparationCalculator;

impl SeparationCalculator {
    /// Calculate minimum safe separation based on vehicle types
    pub fn min_separation(v1: VehicleType, v2: VehicleType) -> f32 {
        match (v1, v2) {
            // Multirotor to multirotor
            (VehicleType::Multirotor, VehicleType::Multirotor) => 3.0,
            // Multirotor to fixed wing
            (VehicleType::Multirotor, VehicleType::FixedWing)
            | (VehicleType::FixedWing, VehicleType::Multirotor) => 5.0,
            // Multirotor to car
            (VehicleType::Multirotor, VehicleType::Car)
            | (VehicleType::Car, VehicleType::Multirotor) => 10.0,
            // Car to car
            (VehicleType::Car, VehicleType::Car) => 5.0,
            // Any to obstacle
            (_, VehicleType::Obstacle) | (VehicleType::Obstacle, _) => 5.0,
            // Default
            _ => 3.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collision_detection() {
        let avoidance = CollisionAvoidance::new()
            .with_safety_radius(1.0)
            .with_time_horizon(10.0);

        // Two vehicles on collision course
        let v1 = VehicleState {
            name: "v1".to_string(),
            pose: Pose::default(),
            velocity: Vec3::new(10.0, 0.0, 0.0),
            vehicle_type: VehicleType::Multirotor,
        };

        let v2 = VehicleState {
            name: "v2".to_string(),
            pose: Pose {
                position: Vec3::new(50.0, 0.0, 0.0),
                orientation: glam::Quat::IDENTITY,
            },
            velocity: Vec3::new(-10.0, 0.0, 0.0),
            vehicle_type: VehicleType::Multirotor,
        };

        let prediction = avoidance.check_collision(&v1, &v2);
        assert!(prediction.is_some());

        if let Some(pred) = prediction {
            assert!(pred.time_to_collision > 0.0);
            assert_eq!(pred.severity, CollisionSeverity::Immediate);
        }
    }

    #[test]
    fn test_no_collision_same_direction() {
        let avoidance = CollisionAvoidance::new();

        // Two vehicles moving same direction
        let v1 = VehicleState {
            name: "v1".to_string(),
            pose: Pose::default(),
            velocity: Vec3::new(10.0, 0.0, 0.0),
            vehicle_type: VehicleType::Multirotor,
        };

        let v2 = VehicleState {
            name: "v2".to_string(),
            pose: Pose {
                position: Vec3::new(100.0, 0.0, 0.0),
                orientation: glam::Quat::IDENTITY,
            },
            velocity: Vec3::new(10.0, 0.0, 0.0),
            vehicle_type: VehicleType::Multirotor,
        };

        let prediction = avoidance.check_collision(&v1, &v2);
        assert!(prediction.is_none());
    }
}
