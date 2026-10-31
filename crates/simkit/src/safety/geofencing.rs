//! Geofencing Module
//!
//! Define and check geofence boundaries for vehicles.

use crate::common::GeoPoint;

/// Geofence boundary definition
#[derive(Debug, Clone)]
pub struct Geofence {
    /// Geofence name
    pub name: String,
    /// Geofence type
    pub geofence_type: GeofenceType,
    /// Is this a keep-in or keep-out geofence
    pub keep_in: bool,
    /// Altitude minimum (meters above takeoff)
    pub altitude_min: f32,
    /// Altitude maximum (meters above takeoff)
    pub altitude_max: f32,
}

/// Geofence shape types
#[derive(Debug, Clone)]
pub enum GeofenceType {
    /// Circular geofence with center and radius
    Circle {
        /// Center latitude
        center_lat: f64,
        /// Center longitude
        center_lon: f64,
        /// Radius in meters
        radius: f32,
    },
    /// Polygon geofence with vertices
    Polygon {
        /// List of (lat, lon) vertices
        vertices: Vec<(f64, f64)>,
    },
    /// Cylinder geofence (circular with altitude limits)
    Cylinder {
        /// Center latitude
        center_lat: f64,
        /// Center longitude
        center_lon: f64,
        /// Radius in meters
        radius: f32,
        /// Altitude minimum
        altitude_min: f32,
        /// Altitude maximum
        altitude_max: f32,
    },
}

impl Geofence {
    /// Create circular geofence
    pub fn circle(name: impl Into<String>, center_lat: f64, center_lon: f64, radius: f32) -> Self {
        Self {
            name: name.into(),
            geofence_type: GeofenceType::Circle {
                center_lat,
                center_lon,
                radius,
            },
            keep_in: true,
            altitude_min: -10.0,
            altitude_max: 100.0,
        }
    }

    /// Create polygon geofence
    pub fn polygon(name: impl Into<String>, vertices: Vec<(f64, f64)>) -> Self {
        Self {
            name: name.into(),
            geofence_type: GeofenceType::Polygon { vertices },
            keep_in: true,
            altitude_min: -10.0,
            altitude_max: 100.0,
        }
    }

    /// Create cylinder geofence
    pub fn cylinder(
        name: impl Into<String>,
        center_lat: f64,
        center_lon: f64,
        radius: f32,
        altitude_min: f32,
        altitude_max: f32,
    ) -> Self {
        Self {
            name: name.into(),
            geofence_type: GeofenceType::Cylinder {
                center_lat,
                center_lon,
                radius,
                altitude_min,
                altitude_max,
            },
            keep_in: true,
            altitude_min,
            altitude_max,
        }
    }

    /// Set as keep-out geofence
    pub fn as_keep_out(mut self) -> Self {
        self.keep_in = false;
        self
    }

    /// Set altitude limits
    pub fn with_altitude_limits(mut self, min: f32, max: f32) -> Self {
        self.altitude_min = min;
        self.altitude_max = max;
        self
    }

    /// Check if a position is inside this geofence
    pub fn contains(&self, position: &GeoPoint) -> bool {
        // Check altitude first
        let altitude_m = position.altitude as f32;
        if altitude_m < self.altitude_min || altitude_m > self.altitude_max {
            return false;
        }

        match &self.geofence_type {
            GeofenceType::Circle {
                center_lat,
                center_lon,
                radius,
            } => {
                let distance = haversine_distance(
                    *center_lat,
                    *center_lon,
                    position.latitude,
                    position.longitude,
                );
                distance <= *radius
            }
            GeofenceType::Polygon { vertices } => {
                point_in_polygon(position.latitude, position.longitude, vertices)
            }
            GeofenceType::Cylinder {
                center_lat,
                center_lon,
                radius,
                ..
            } => {
                let distance = haversine_distance(
                    *center_lat,
                    *center_lon,
                    position.latitude,
                    position.longitude,
                );
                distance <= *radius
            }
        }
    }

    /// Check if position violates this geofence
    pub fn check_violation(&self, position: &GeoPoint) -> Option<f32> {
        let inside = self.contains(position);

        match (self.keep_in, inside) {
            (true, false) => {
                // Should be inside but is outside - calculate distance
                let distance = self.distance_to_boundary(position);
                Some(distance)
            }
            (false, true) => {
                // Should be outside but is inside - calculate distance
                let distance = self.distance_to_boundary(position);
                Some(-distance)
            }
            _ => None, // No violation
        }
    }

    /// Calculate distance to geofence boundary
    fn distance_to_boundary(&self, position: &GeoPoint) -> f32 {
        match &self.geofence_type {
            GeofenceType::Circle {
                center_lat,
                center_lon,
                radius,
            } => {
                let dist = haversine_distance(
                    *center_lat,
                    *center_lon,
                    position.latitude,
                    position.longitude,
                );
                (dist - *radius).abs()
            }
            GeofenceType::Cylinder {
                center_lat,
                center_lon,
                radius,
                ..
            } => {
                let dist = haversine_distance(
                    *center_lat,
                    *center_lon,
                    position.latitude,
                    position.longitude,
                );
                (dist - *radius).abs()
            }
            _ => 0.0, // Simplified for polygon
        }
    }
}

/// Geofence manager that handles multiple geofences
#[derive(Debug, Default)]
pub struct GeofenceManager {
    geofences: Vec<Geofence>,
}

impl GeofenceManager {
    /// Create new geofence manager
    pub fn new() -> Self {
        Self {
            geofences: Vec::new(),
        }
    }

    /// Add a geofence
    pub fn add(&mut self, geofence: Geofence) {
        self.geofences.push(geofence);
    }

    /// Remove a geofence by name
    pub fn remove(&mut self, name: &str) {
        self.geofences.retain(|g| g.name != name);
    }

    /// Get geofence by name
    pub fn get(&self, name: &str) -> Option<&Geofence> {
        self.geofences.iter().find(|g| g.name == name)
    }

    /// Check all geofences for a position
    pub fn check_position(&self, position: &GeoPoint) -> GeofenceCheck {
        let mut violations = Vec::new();

        for geofence in &self.geofences {
            if let Some(distance) = geofence.check_violation(position) {
                violations.push((geofence.name.clone(), distance));
            }
        }

        if violations.is_empty() {
            GeofenceCheck::Compliant
        } else {
            // Return the most critical violation (largest distance)
            let (name, distance) = violations
                .into_iter()
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                .unwrap();
            GeofenceCheck::Violation { name, distance }
        }
    }

    /// Get all geofences
    pub fn list(&self) -> &[Geofence] {
        &self.geofences
    }

    /// Clear all geofences
    pub fn clear(&mut self) {
        self.geofences.clear();
    }
}

/// Geofence check result
#[derive(Debug, Clone)]
pub enum GeofenceCheck {
    /// Position complies with all geofences
    Compliant,
    /// Position violates a geofence
    Violation {
        /// Name of the violated geofence
        name: String,
        /// Distance from boundary (positive = outside keep-in, negative = inside keep-out)
        distance: f32,
    },
}

/// Calculate haversine distance between two points (meters)
fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f32 {
    const R: f64 = 6_371_000.0; // Earth radius in meters

    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();
    let delta_lat = (lat2 - lat1).to_radians();
    let delta_lon = (lon2 - lon1).to_radians();

    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    (R * c) as f32
}

/// Check if a point is inside a polygon using ray casting algorithm
fn point_in_polygon(lat: f64, lon: f64, vertices: &[(f64, f64)]) -> bool {
    let n = vertices.len();
    if n < 3 {
        return false;
    }

    let mut inside = false;
    let mut j = n - 1;

    for i in 0..n {
        let vi = vertices[i];
        let vj = vertices[j];

        if ((vi.1 > lon) != (vj.1 > lon))
            && (lat < (vj.0 - vi.0) * (lon - vi.1) / (vj.1 - vi.1) + vi.0)
        {
            inside = !inside;
        }
        j = i;
    }

    inside
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle_geofence() {
        let geofence = Geofence::circle(
            "test", 47.6062, // Seattle
            -122.3321, 1000.0, // 1km radius
        );

        // Point inside
        let inside = GeoPoint::new(47.6063, -122.3321, 10.0);
        assert!(geofence.contains(&inside));

        // Point outside
        let outside = GeoPoint::new(47.6162, -122.3321, 10.0); // ~1.1km away
        assert!(!geofence.contains(&outside));
    }

    #[test]
    fn test_haversine_distance() {
        // Seattle to Bellevue is about 10km
        let dist = haversine_distance(47.6062, -122.3321, 47.6101, -122.2015);
        assert!(dist > 9000.0 && dist < 11000.0); // Should be around 10km
    }

    #[test]
    fn test_polygon_geofence() {
        let vertices = vec![(0.0, 0.0), (0.0, 1.0), (1.0, 1.0), (1.0, 0.0)];

        let geofence = Geofence::polygon("square", vertices);

        // Point inside
        let inside = GeoPoint::new(0.5, 0.5, 10.0);
        assert!(geofence.contains(&inside));

        // Point outside
        let outside = GeoPoint::new(2.0, 2.0, 10.0);
        assert!(!geofence.contains(&outside));
    }
}
