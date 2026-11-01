//! Quaternion types (< 200 lines)
use crate::math::SCNVector3;
use bytemuck::{Pod, Zeroable};

/// Quaternion - SceneKit SCNQuaternion equivalent
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct SCNQuaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl SCNQuaternion {
    pub const IDENTITY: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
    };

    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub fn from_axis_angle(axis: SCNVector3, angle: f32) -> Self {
        let half_angle = angle / 2.0;
        let s = half_angle.sin();
        Self::new(axis.x * s, axis.y * s, axis.z * s, half_angle.cos())
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0 {
            Self::new(self.x / len, self.y / len, self.z / len, self.w / len)
        } else {
            *self
        }
    }

    pub fn conjugate(&self) -> Self {
        Self::new(-self.x, -self.y, -self.z, self.w)
    }

    pub fn inverse(&self) -> Self {
        self.conjugate().normalize()
    }

    /// Spherical linear interpolation
    pub fn slerp(&self, other: &Self, t: f32) -> Self {
        let dot = self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w;
        let (other, dot) = if dot < 0.0 {
            (Self::new(-other.x, -other.y, -other.z, -other.w), -dot)
        } else {
            (*other, dot)
        };

        if dot > 0.9995 {
            return Self::new(
                self.x + t * (other.x - self.x),
                self.y + t * (other.y - self.y),
                self.z + t * (other.z - self.z),
                self.w + t * (other.w - self.w),
            )
            .normalize();
        }

        let theta0 = dot.acos();
        let theta = theta0 * t;
        let s0 = theta.cos() - dot * theta.sin() / theta0.sin();
        let s1 = theta.sin() / theta0.sin();

        Self::new(
            self.x * s0 + other.x * s1,
            self.y * s0 + other.y * s1,
            self.z * s0 + other.z * s1,
            self.w * s0 + other.w * s1,
        )
    }

    pub fn to_glam(&self) -> glam::Quat {
        glam::Quat::from_xyzw(self.x, self.y, self.z, self.w)
    }
}

impl std::ops::Mul for SCNQuaternion {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Self::new(
            self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
            self.w * other.y + self.y * other.w + self.z * other.x - self.x * other.z,
            self.w * other.z + self.z * other.w + self.x * other.y - self.y * other.x,
            self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
        )
    }
}
