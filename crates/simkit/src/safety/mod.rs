//! Safety Module
//!
//! Geofencing, collision avoidance, and safety monitoring.

use crate::common::{GeoPoint, Pose};
use std::collections::HashMap;

pub mod collision;
pub mod geofencing;

pub use collision::{CollisionAvoidance, ProximityAlert};
pub use geofencing::{Geofence, GeofenceCheck, GeofenceManager, GeofenceType};

/// Safety system for vehicles
pub struct SafetySystem {
    /// Geofence manager
    geofences: GeofenceManager,
    /// Collision avoidance system
    collision_avoidance: CollisionAvoidance,
    /// Safety monitors per vehicle
    vehicle_monitors: HashMap<String, VehicleSafetyMonitor>,
    /// Enable emergency stop
    emergency_stop_enabled: bool,
    /// Max altitude (meters)
    max_altitude: f32,
    /// Max speed (m/s)
    max_speed: f32,
}

impl SafetySystem {
    /// Create new safety system
    pub fn new() -> Self {
        Self {
            geofences: GeofenceManager::new(),
            collision_avoidance: CollisionAvoidance::new(),
            vehicle_monitors: HashMap::new(),
            emergency_stop_enabled: true,
            max_altitude: 100.0,
            max_speed: 30.0,
        }
    }

    /// Register a vehicle for safety monitoring
    pub fn register_vehicle(&mut self, vehicle_name: &str) {
        self.vehicle_monitors
            .insert(vehicle_name.to_string(), VehicleSafetyMonitor::new());
    }

    /// Unregister a vehicle
    pub fn unregister_vehicle(&mut self, vehicle_name: &str) {
        self.vehicle_monitors.remove(vehicle_name);
    }

    /// Add a geofence
    pub fn add_geofence(&mut self, geofence: Geofence) {
        self.geofences.add(geofence);
    }

    /// Remove a geofence
    pub fn remove_geofence(&mut self, name: &str) {
        self.geofences.remove(name);
    }

    /// Check if position is within all geofences
    pub fn check_geofences(&self, position: &GeoPoint) -> crate::safety::geofencing::GeofenceCheck {
        self.geofences.check_position(position)
    }

    /// Check if pose violates any safety constraints
    pub fn check_pose_safety(&self, vehicle_name: &str, pose: &Pose) -> SafetyCheck {
        let mut violations = Vec::new();

        // Check altitude
        if pose.position.z > self.max_altitude {
            violations.push(SafetyViolation::MaxAltitudeExceeded {
                current: pose.position.z,
                max: self.max_altitude,
            });
        }

        // Check geofences
        let geo_point = GeoPoint::new(0.0, 0.0, pose.position.z);
        let geofence_check = self.check_geofences(&geo_point);

        if let GeofenceCheck::Violation { name, distance } = geofence_check {
            violations.push(SafetyViolation::GeofenceViolation { name, distance });
        }

        // Check vehicle-specific monitors
        if let Some(monitor) = self.vehicle_monitors.get(vehicle_name) {
            if let Some(alert) = monitor.check_proximity(pose) {
                violations.push(SafetyViolation::ProximityAlert(alert));
            }
        }

        if violations.is_empty() {
            SafetyCheck::Safe
        } else {
            SafetyCheck::Violations(violations)
        }
    }

    /// Trigger emergency stop
    pub fn emergency_stop(&self, vehicle_name: &str) -> bool {
        if self.emergency_stop_enabled {
            log::warn!("Emergency stop triggered for vehicle: {}", vehicle_name);
            true
        } else {
            false
        }
    }

    /// Enable emergency stop
    pub fn enable_emergency_stop(&mut self, enabled: bool) {
        self.emergency_stop_enabled = enabled;
    }

    /// Set max altitude
    pub fn set_max_altitude(&mut self, altitude: f32) {
        self.max_altitude = altitude;
    }

    /// Set max speed
    pub fn set_max_speed(&mut self, speed: f32) {
        self.max_speed = speed;
    }

    /// Get max altitude
    pub fn max_altitude(&self) -> f32 {
        self.max_altitude
    }

    /// Get max speed
    pub fn max_speed(&self) -> f32 {
        self.max_speed
    }
}

impl Default for SafetySystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Safety check result
#[derive(Debug, Clone)]
pub enum SafetyCheck {
    /// Position is safe
    Safe,
    /// Safety violations detected
    Violations(Vec<SafetyViolation>),
}

/// Safety violation types
#[derive(Debug, Clone)]
pub enum SafetyViolation {
    /// Maximum altitude exceeded
    MaxAltitudeExceeded {
        /// Current altitude
        current: f32,
        /// Maximum allowed altitude
        max: f32,
    },
    /// Maximum speed exceeded
    MaxSpeedExceeded {
        /// Current speed
        current: f32,
        /// Maximum allowed speed
        max: f32,
    },
    /// Geofence violation
    GeofenceViolation {
        /// Geofence name
        name: String,
        /// Distance outside geofence
        distance: f32,
    },
    /// Proximity alert
    ProximityAlert(ProximityAlert),
    /// Battery low
    BatteryLow {
        /// Current battery level (0-1)
        level: f32,
    },
    /// Communication lost
    CommunicationLost {
        /// Seconds since last communication
        seconds: f32,
    },
}

/// Vehicle-specific safety monitor
pub struct VehicleSafetyMonitor {
    /// Last known pose
    last_pose: Option<Pose>,
    /// Last update time
    last_update: std::time::Instant,
    /// Proximity alerts
    proximity_alerts: Vec<ProximityAlert>,
}

impl VehicleSafetyMonitor {
    /// Create new monitor
    pub fn new() -> Self {
        Self {
            last_pose: None,
            last_update: std::time::Instant::now(),
            proximity_alerts: Vec::new(),
        }
    }

    /// Update with new pose
    pub fn update(&mut self, pose: Pose) {
        self.last_pose = Some(pose);
        self.last_update = std::time::Instant::now();
    }

    /// Check for proximity issues
    pub fn check_proximity(&self, _current_pose: &Pose) -> Option<ProximityAlert> {
        self.proximity_alerts.first().cloned()
    }

    /// Get time since last update
    pub fn seconds_since_update(&self) -> f32 {
        self.last_update.elapsed().as_secs_f32()
    }
}

impl Default for VehicleSafetyMonitor {
    fn default() -> Self {
        Self::new()
    }
}
