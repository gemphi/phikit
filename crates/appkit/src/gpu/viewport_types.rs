//! 3D viewport core data types

use std::path::PathBuf;

/// 3D Camera with perspective projection
#[derive(Debug, Clone)]
pub struct Camera3D {
    pub position: [f32; 3],
    pub target: [f32; 3],
    pub up: [f32; 3],
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera3D {
    pub fn new() -> Self {
        Self {
            position: [5.0, 5.0, 5.0],
            target: [0.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0],
            fov: 60.0f32.to_radians(),
            near: 0.1,
            far: 1000.0,
        }
    }

    /// Orbit around target (left mouse drag)
    pub fn orbit(&mut self, delta_x: f32, delta_y: f32) {
        let dx = self.position[0] - self.target[0];
        let dy = self.position[1] - self.target[1];
        let dz = self.position[2] - self.target[2];

        let distance = (dx * dx + dy * dy + dz * dz).sqrt();
        let mut yaw = dz.atan2(dx);
        let mut pitch = (dy / distance).asin();

        yaw += delta_x * 0.01;
        pitch += delta_y * 0.01;
        pitch = pitch.clamp(-1.5, 1.5);

        self.position[0] = self.target[0] + distance * yaw.cos() * pitch.cos();
        self.position[1] = self.target[1] + distance * pitch.sin();
        self.position[2] = self.target[2] + distance * yaw.sin() * pitch.cos();
    }

    /// Pan camera (right mouse drag)
    pub fn pan(&mut self, delta_x: f32, delta_y: f32) {
        let forward = [
            self.target[0] - self.position[0],
            self.target[1] - self.position[1],
            self.target[2] - self.position[2],
        ];
        let forward_len =
            (forward[0] * forward[0] + forward[1] * forward[1] + forward[2] * forward[2]).sqrt();
        let forward = [
            forward[0] / forward_len,
            forward[1] / forward_len,
            forward[2] / forward_len,
        ];

        let right = [
            forward[2] * self.up[1] - forward[1] * self.up[2],
            forward[0] * self.up[2] - forward[2] * self.up[0],
            forward[1] * self.up[0] - forward[0] * self.up[1],
        ];
        let right_len = (right[0] * right[0] + right[1] * right[1] + right[2] * right[2]).sqrt();
        let right = [
            right[0] / right_len,
            right[1] / right_len,
            right[2] / right_len,
        ];

        let pan_x = right[0] * delta_x + self.up[0] * delta_y;
        let pan_y = right[1] * delta_x + self.up[1] * delta_y;
        let pan_z = right[2] * delta_x + self.up[2] * delta_y;

        self.position[0] += pan_x;
        self.position[1] += pan_y;
        self.position[2] += pan_z;
        self.target[0] += pan_x;
        self.target[1] += pan_y;
        self.target[2] += pan_z;
    }

    /// Zoom (scroll wheel)
    pub fn zoom(&mut self, delta: f32) {
        let dx = self.position[0] - self.target[0];
        let dy = self.position[1] - self.target[1];
        let dz = self.position[2] - self.target[2];

        let distance = (dx * dx + dy * dy + dz * dz).sqrt();
        let new_distance = (distance * (1.0 - delta * 0.1)).clamp(0.5, 500.0);
        let scale = new_distance / distance;

        self.position[0] = self.target[0] + dx * scale;
        self.position[1] = self.target[1] + dy * scale;
        self.position[2] = self.target[2] + dz * scale;
    }

    /// Frame camera on bounding box
    pub fn frame_bounds(&mut self, min: [f32; 3], max: [f32; 3]) {
        let center = [
            (min[0] + max[0]) * 0.5,
            (min[1] + max[1]) * 0.5,
            (min[2] + max[2]) * 0.5,
        ];
        let size = [max[0] - min[0], max[1] - min[1], max[2] - min[2]];
        let max_dim = size[0].max(size[1]).max(size[2]);
        let distance = max_dim * 1.5;

        self.target = center;
        self.position = [
            center[0] + distance * 0.707,
            center[1] + distance * 0.707,
            center[2] + distance * 0.707,
        ];
    }

    /// Get view matrix (look-at)
    pub fn view_matrix(&self) -> [[f32; 4]; 4] {
        let f = [
            self.target[0] - self.position[0],
            self.target[1] - self.position[1],
            self.target[2] - self.position[2],
        ];
        let f_len = (f[0] * f[0] + f[1] * f[1] + f[2] * f[2]).sqrt();
        let f = [f[0] / f_len, f[1] / f_len, f[2] / f_len];

        let s = [
            f[1] * self.up[2] - f[2] * self.up[1],
            f[2] * self.up[0] - f[0] * self.up[2],
            f[0] * self.up[1] - f[1] * self.up[0],
        ];
        let s_len = (s[0] * s[0] + s[1] * s[1] + s[2] * s[2]).sqrt();
        let s = [s[0] / s_len, s[1] / s_len, s[2] / s_len];

        let u = [
            s[1] * f[2] - s[2] * f[1],
            s[2] * f[0] - s[0] * f[2],
            s[0] * f[1] - s[1] * f[0],
        ];

        [
            [s[0], u[0], -f[0], 0.0],
            [s[1], u[1], -f[1], 0.0],
            [s[2], u[2], -f[2], 0.0],
            [
                -s[0] * self.position[0] - s[1] * self.position[1] - s[2] * self.position[2],
                -u[0] * self.position[0] - u[1] * self.position[1] - u[2] * self.position[2],
                f[0] * self.position[0] + f[1] * self.position[1] + f[2] * self.position[2],
                1.0,
            ],
        ]
    }

    /// Get perspective projection matrix
    pub fn projection_matrix(&self, aspect: f32) -> [[f32; 4]; 4] {
        let f = 1.0 / (self.fov * 0.5).tan();
        let nf = 1.0 / (self.near - self.far);

        [
            [f / aspect, 0.0, 0.0, 0.0],
            [0.0, f, 0.0, 0.0],
            [0.0, 0.0, (self.far + self.near) * nf, -1.0],
            [0.0, 0.0, 2.0 * self.far * self.near * nf, 0.0],
        ]
    }
}

impl Default for Camera3D {
    fn default() -> Self {
        Self::new()
    }
}

/// Mesh instance in the scene
#[derive(Debug, Clone)]
pub struct MeshInstance {
    pub path: PathBuf,
    pub name: String,
    pub transform: [[f32; 4]; 4],
    pub bounds: ([f32; 3], [f32; 3]),
}

/// 3D Scene data
#[derive(Debug, Clone)]
pub struct Scene3D {
    pub meshes: Vec<MeshInstance>,
    pub selected: Option<usize>,
}

impl Scene3D {
    pub fn new() -> Self {
        Self {
            meshes: Vec::new(),
            selected: None,
        }
    }

    pub fn add_mesh(
        &mut self,
        path: PathBuf,
        name: String,
        transform: [[f32; 4]; 4],
        bounds: ([f32; 3], [f32; 3]),
    ) {
        self.meshes.push(MeshInstance {
            path,
            name,
            transform,
            bounds,
        });
    }

    pub fn total_bounds(&self) -> Option<([f32; 3], [f32; 3])> {
        if self.meshes.is_empty() {
            return None;
        }

        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];

        for mesh in &self.meshes {
            for i in 0..3 {
                min[i] = min[i].min(mesh.bounds.0[i]);
                max[i] = max[i].max(mesh.bounds.1[i]);
            }
        }

        Some((min, max))
    }
}
