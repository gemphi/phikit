//! Common types and utilities for rust-sim-kit

pub mod clock;
pub mod structs;
pub mod types;

pub use clock::*;
pub use structs::*;
pub use types::*;

use glam::{Quat, Vec2, Vec3};

/// Time point in nanoseconds (matching AirSim's TTimePoint)
pub type TimePoint = u64;

/// Real-valued type alias
pub type Real = f64;

/// Vector3 using Real (f64)
pub type Vector3r = glam::DVec3;

/// Vector2 using Real (f64)
pub type Vector2r = glam::DVec2;

/// Quaternion using Real (f64)
pub type Quaternionr = glam::DQuat;

/// 3D Pose with position and orientation
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct Pose {
    /// Position in 3D space
    pub position: Vec3,
    /// Orientation as quaternion
    pub orientation: Quat,
}

impl Pose {
    /// Create a new pose
    pub fn new(position: Vec3, orientation: Quat) -> Self {
        Self {
            position,
            orientation,
        }
    }

    /// Identity pose at origin with no rotation
    pub fn identity() -> Self {
        Self {
            position: Vec3::ZERO,
            orientation: Quat::IDENTITY,
        }
    }
}

/// Velocity twist (linear and angular)
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct Twist {
    /// Linear velocity (m/s)
    pub linear: Vec3,
    /// Angular velocity (rad/s)
    pub angular: Vec3,
}

impl Twist {
    /// Zero twist
    pub fn zero() -> Self {
        Self {
            linear: Vec3::ZERO,
            angular: Vec3::ZERO,
        }
    }
}

/// Force and torque wrench
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct Wrench {
    /// Force vector
    pub force: Vec3,
    /// Torque vector
    pub torque: Vec3,
}

impl Wrench {
    /// Zero wrench
    pub fn zero() -> Self {
        Self {
            force: Vec3::ZERO,
            torque: Vec3::ZERO,
        }
    }
}

impl std::ops::Add for Wrench {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            force: self.force + other.force,
            torque: self.torque + other.torque,
        }
    }
}

impl std::ops::AddAssign for Wrench {
    fn add_assign(&mut self, other: Self) {
        self.force += other.force;
        self.torque += other.torque;
    }
}

/// Geographic point (latitude, longitude, altitude)
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct GeoPoint {
    /// Latitude in degrees
    pub latitude: f64,
    /// Longitude in degrees
    pub longitude: f64,
    /// Altitude in meters
    pub altitude: f32,
}

impl GeoPoint {
    /// Create a new geo point
    pub fn new(latitude: f64, longitude: f64, altitude: f32) -> Self {
        Self {
            latitude,
            longitude,
            altitude,
        }
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        format!("{}, {}, {}", self.latitude, self.longitude, self.altitude)
    }
}

/// 2D bounding box
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct Box2D {
    /// Minimum corner
    pub min: Vec2,
    /// Maximum corner
    pub max: Vec2,
}

impl Box2D {
    /// Create a new 2D box
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }
}

/// 3D bounding box
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct Box3D {
    /// Minimum corner
    pub min: Vec3,
    /// Maximum corner
    pub max: Vec3,
}

impl Box3D {
    /// Create a new 3D box
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }
}

/// Collision information
#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct CollisionInfo {
    /// Whether a collision occurred
    pub has_collided: bool,
    /// Collision normal vector
    pub normal: Vec3,
    /// Impact point
    pub impact_point: Vec3,
    /// Position at collision
    pub position: Vec3,
    /// Penetration depth
    pub penetration_depth: f32,
    /// Timestamp
    pub time_stamp: TimePoint,
    /// Collision count
    pub collision_count: u32,
    /// Object name
    pub object_name: String,
    /// Object ID
    pub object_id: i32,
}

// Image types (ImageType, ImageRequest, ImageResponse) and CameraInfo are defined in structs.rs

/// Detection info for object detection
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DetectionInfo {
    /// Object name
    pub name: String,
    /// Geographic point
    pub geo_point: GeoPoint,
    /// 2D bounding box
    pub box2d: Box2D,
    /// 3D bounding box
    pub box3d: Box3D,
    /// Relative pose
    pub relative_pose: Pose,
}

impl Default for DetectionInfo {
    fn default() -> Self {
        Self {
            name: String::new(),
            geo_point: GeoPoint::default(),
            box2d: Box2D::default(),
            box3d: Box3D::default(),
            relative_pose: Pose::default(),
        }
    }
}

/// Remote control data
#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct RcData {
    /// Timestamp
    pub timestamp: TimePoint,
    /// Pitch (-1 to 1)
    pub pitch: f32,
    /// Roll (-1 to 1)
    pub roll: f32,
    /// Throttle (-1 to 1)
    pub throttle: f32,
    /// Yaw (-1 to 1)
    pub yaw: f32,
    /// Left Z axis
    pub left_z: f32,
    /// Right Z axis
    pub right_z: f32,
    /// Switch states
    pub switches: u16,
    /// Vendor ID
    pub vendor_id: String,
    /// Whether RC is initialized (connected)
    pub is_initialized: bool,
    /// Whether data is valid
    pub is_valid: bool,
}

impl RcData {
    /// Get switch state at index
    pub fn get_switch(&self, index: u16) -> u32 {
        if self.switches & (1 << index) != 0 {
            1
        } else {
            0
        }
    }
}

/// Lidar data point cloud
#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct LidarData {
    /// Timestamp
    pub time_stamp: TimePoint,
    /// Point cloud data (xyz xyz xyz ...)
    pub point_cloud: Vec<f64>,
    /// Lidar pose
    pub pose: Pose,
    /// Segmentation IDs for each point
    pub segmentation: Vec<i32>,
}

/// Distance sensor data
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct DistanceSensorData {
    /// Timestamp
    pub time_stamp: TimePoint,
    /// Measured distance in meters
    pub distance: f64,
    /// Minimum measurable distance
    pub min_distance: f64,
    /// Maximum measurable distance
    pub max_distance: f64,
    /// Sensor pose relative to vehicle
    pub relative_pose: Pose,
}

/// Mesh vertex buffer response
#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct MeshPositionVertexBuffersResponse {
    /// Mesh position
    pub position: Vec3,
    /// Mesh orientation
    pub orientation: Quat,
    /// Vertex positions (xyz xyz ...)
    pub vertices: Vec<f32>,
    /// Triangle indices
    pub indices: Vec<u32>,
    /// Mesh name
    pub name: String,
}
