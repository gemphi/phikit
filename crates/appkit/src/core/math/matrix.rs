//! Matrix types (< 200 lines)
use crate::math::SCNVector3;
use bytemuck::{Pod, Zeroable};

/// 4x4 Matrix - SceneKit SCNMatrix4 equivalent
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct SCNMatrix4 {
    pub m: [[f32; 4]; 4],
}

impl SCNMatrix4 {
    pub const IDENTITY: Self = Self {
        m: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };

    pub fn identity() -> Self {
        Self::IDENTITY
    }

    pub fn translation(x: f32, y: f32, z: f32) -> Self {
        let mut m = Self::IDENTITY;
        m.m[3][0] = x;
        m.m[3][1] = y;
        m.m[3][2] = z;
        m
    }

    pub fn scale(x: f32, y: f32, z: f32) -> Self {
        let mut m = Self::IDENTITY;
        m.m[0][0] = x;
        m.m[1][1] = y;
        m.m[2][2] = z;
        m
    }

    pub fn look_at(eye: SCNVector3, target: SCNVector3, up: SCNVector3) -> Self {
        let view = glam::Mat4::look_at_rh(
            glam::Vec3::new(eye.x, eye.y, eye.z),
            glam::Vec3::new(target.x, target.y, target.z),
            glam::Vec3::new(up.x, up.y, up.z),
        );
        Self {
            m: view.to_cols_array_2d(),
        }
    }

    pub fn perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> Self {
        // Right-handed perspective matrix that maps Z to [0, 1] for WGPU/Metal/Vulkan
        let f = 1.0 / (fov_y / 2.0).tan();
        let nf = 1.0 / (near - far);
        Self {
            m: [
                [f / aspect, 0.0, 0.0, 0.0],
                [0.0, f, 0.0, 0.0],
                [0.0, 0.0, far * nf, -1.0],
                [0.0, 0.0, far * near * nf, 0.0],
            ],
        }
    }

    pub fn to_glam(&self) -> glam::Mat4 {
        glam::Mat4::from_cols_array_2d(&self.m)
    }

    /// Alias for look_at (SceneKit naming)
    pub fn make_look_at(eye: SCNVector3, target: SCNVector3, up: SCNVector3) -> Self {
        Self::look_at(eye, target, up)
    }

    /// Alias for perspective (SceneKit naming)
    pub fn make_perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> Self {
        Self::perspective(fov_y, aspect, near, far)
    }

    /// Multiply with another matrix
    pub fn multiply(&self, other: &Self) -> Self {
        Self {
            m: (self.to_glam() * other.to_glam()).to_cols_array_2d(),
        }
    }
}

impl std::ops::Mul for SCNMatrix4 {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        self.multiply(&other)
    }
}
