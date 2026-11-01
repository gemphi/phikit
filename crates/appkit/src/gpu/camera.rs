//! Viewport camera controller for editor (< 200 lines)
//!
//! This is an EDITOR camera controller (viewport navigation), NOT the ECS Camera component.
//! For scene cameras, use reality_kit::components::Camera with Transform.

use crate::math::{SCNVector3, SCNMatrix4};
use bytemuck::{Pod, Zeroable};

/// Editor viewport camera controller with arcball orbit controls.
/// For scene cameras, use reality_kit::components::Camera.
pub struct ViewportCameraController {
    pub position: SCNVector3,
    pub target: SCNVector3,
    pub up: SCNVector3,
    pub fov_y: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
}

impl ViewportCameraController {
    pub fn new() -> Self {
        Self {
            position: SCNVector3::new(5.0, 5.0, 5.0),
            target: SCNVector3::new(0.0, 0.0, 0.0),
            up: SCNVector3::new(0.0, 1.0, 0.0),
            fov_y: 45.0_f32.to_radians(),
            aspect: 1.0,
            near: 0.1,
            far: 1000.0,
            distance: 10.0,
            yaw: -45.0_f32.to_radians(),
            pitch: -30.0_f32.to_radians(),
        }
    }
    
    pub fn update_from_arcball(&mut self) {
        self.pitch = self.pitch.clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());
        let cos_pitch = self.pitch.cos();
        self.position.x = self.target.x + self.distance * cos_pitch * self.yaw.sin();
        self.position.y = self.target.y + self.distance * self.pitch.sin();
        self.position.z = self.target.z + self.distance * cos_pitch * self.yaw.cos();
    }
    
    pub fn handle_drag(&mut self, delta_x: f32, delta_y: f32, sensitivity: f32) {
        self.yaw += delta_x * sensitivity;
        self.pitch += delta_y * sensitivity;
        self.update_from_arcball();
    }
    
    pub fn handle_zoom(&mut self, delta: f32, sensitivity: f32) {
        self.distance = (self.distance * (1.0 + delta * sensitivity)).clamp(0.5, 500.0);
        self.update_from_arcball();
    }
    
    pub fn handle_pan(&mut self, delta_x: f32, delta_y: f32, sensitivity: f32) {
        let forward = SCNVector3::new(
            self.target.x - self.position.x,
            self.target.y - self.position.y,
            self.target.z - self.position.z
        ).normalize();
        let right = forward.cross(&self.up).normalize();
        let up = right.cross(&forward).normalize();
        let pan = right * (-delta_x * sensitivity * self.distance * 0.1) + 
                  up * (delta_y * sensitivity * self.distance * 0.1);
        self.target = self.target + pan;
        self.position = self.position + pan;
        self.update_from_arcball();
    }
    
    pub fn focus_on(&mut self, point: SCNVector3) {
        let offset = self.target - self.position;
        self.target = point;
        self.position = point - offset;
    }
    
    pub fn reset(&mut self) {
        *self = Self::new();
    }
    
    pub fn view_matrix(&self) -> SCNMatrix4 {
        SCNMatrix4::look_at(self.position, self.target, self.up)
    }
    
    pub fn projection_matrix(&self) -> SCNMatrix4 {
        SCNMatrix4::perspective(self.fov_y, self.aspect, self.near, self.far)
    }
    
    pub fn view_projection_matrix(&self) -> SCNMatrix4 {
        let proj = self.projection_matrix();
        let view = self.view_matrix();
        proj.multiply(&view)
    }
    
    pub fn frame_bounding_box(&mut self, min: SCNVector3, max: SCNVector3) {
        let center = SCNVector3::new(
            (min.x + max.x) * 0.5,
            (min.y + max.y) * 0.5,
            (min.z + max.z) * 0.5,
        );
        let size = SCNVector3::new(
            (max.x - min.x).abs(),
            (max.y - min.y).abs(),
            (max.z - min.z).abs(),
        );
        let max_size = size.x.max(size.y).max(size.z);
        self.focus_on(center);
        self.distance = max_size * 1.5;
        self.update_from_arcball();
    }
}

impl Default for ViewportCameraController {
    fn default() -> Self {
        Self::new()
    }
}

/// Camera uniforms for GPU
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct CameraUniforms {
    pub view_proj: [[f32; 4]; 4],
    pub camera_position: [f32; 4],
}

/// Mouse button types for viewport interaction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

impl Default for MouseButton {
    fn default() -> Self {
        MouseButton::Left
    }
}
