//! Bounding volume types (< 200 lines)
use crate::math::SCNVector3;

/// Bounding box
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SCNBoundingBox {
    pub min: SCNVector3,
    pub max: SCNVector3,
}

impl SCNBoundingBox {
    pub fn new(min: SCNVector3, max: SCNVector3) -> Self {
        Self { min, max }
    }

    pub fn center(&self) -> SCNVector3 {
        SCNVector3::new(
            (self.min.x + self.max.x) / 2.0,
            (self.min.y + self.max.y) / 2.0,
            (self.min.z + self.max.z) / 2.0,
        )
    }

    pub fn size(&self) -> SCNVector3 {
        SCNVector3::new(
            self.max.x - self.min.x,
            self.max.y - self.min.y,
            self.max.z - self.min.z,
        )
    }

    pub fn is_empty(&self) -> bool {
        self.min.x > self.max.x || self.min.y > self.max.y || self.min.z > self.max.z
    }

    pub fn union(&self, other: &Self) -> Self {
        Self::new(
            SCNVector3::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z),
            ),
            SCNVector3::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
                self.max.z.max(other.max.z),
            ),
        )
    }
}

/// Bounding sphere
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SCNBoundingSphere {
    pub center: SCNVector3,
    pub radius: f32,
}

impl SCNBoundingSphere {
    pub fn new(center: SCNVector3, radius: f32) -> Self {
        Self { center, radius }
    }

    pub fn is_empty(&self) -> bool {
        self.radius < 0.0
    }

    pub fn intersects(&self, other: &Self) -> bool {
        let dist = (self.center - other.center).length();
        dist < (self.radius + other.radius)
    }
}

/// Bounding volume trait
pub trait SCNBoundingVolume {
    fn bounding_box(&self) -> SCNBoundingBox;
    fn bounding_sphere(&self) -> SCNBoundingSphere;
    fn set_bounding_box(&mut self, box_: SCNBoundingBox);
}

/// Action timing modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SCNActionTimingMode {
    Linear,
    EaseIn,
    EaseOut,
    EaseInEaseOut,
}
