//! Physics body for simulation
//!
//! Represents a rigid body with mass, inertia, and collision properties.

use super::{Accelerations, KinematicsState, Wrench};
use crate::common::Pose;
use glam::{Mat3, Quat, Vec3};

/// Physics body vertex for collision detection
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct PhysicsBodyVertex {
    /// Local position relative to body center
    pub local_position: Vec3,
    /// World position (computed)
    pub world_position: Vec3,
    /// Normal vector (for collision response)
    pub normal: Vec3,
}

impl PhysicsBodyVertex {
    /// Create new vertex
    pub fn new(local_position: Vec3) -> Self {
        Self {
            local_position,
            world_position: Vec3::ZERO,
            normal: Vec3::Y,
        }
    }

    /// Update world position based on body pose
    pub fn update_world_position(&mut self, pose: &Pose) {
        self.world_position = pose.orientation * self.local_position + pose.position;
    }

    /// Set normal vector
    pub fn with_normal(mut self, normal: Vec3) -> Self {
        self.normal = normal.normalize();
        self
    }
}

/// Inertia tensor for rigid body
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct InertiaTensor {
    /// Mass (kg)
    pub mass: f32,
    /// Diagonal inertia components (kg*m^2)
    pub ixx: f32,
    /// IYY component
    pub iyy: f32,
    /// IZZ component
    pub izz: f32,
    /// Off-diagonal inertia components
    pub ixy: f32,
    /// IXZ component
    pub ixz: f32,
    /// IYZ component
    pub iyz: f32,
}

impl InertiaTensor {
    /// Create new inertia tensor
    pub fn new(mass: f32, ixx: f32, iyy: f32, izz: f32) -> Self {
        Self {
            mass,
            ixx,
            iyy,
            izz,
            ixy: 0.0,
            ixz: 0.0,
            iyz: 0.0,
        }
    }

    /// Create diagonal inertia tensor (symmetric body)
    pub fn diagonal(mass: f32, ixx: f32, iyy: f32, izz: f32) -> Self {
        Self::new(mass, ixx, iyy, izz)
    }

    /// Create from principal moments
    pub fn from_principal_moments(mass: f32, moments: Vec3) -> Self {
        Self::new(mass, moments.x, moments.y, moments.z)
    }

    /// Create for a box shape
    pub fn box_shape(mass: f32, width: f32, height: f32, depth: f32) -> Self {
        // I = 1/12 * m * (h^2 + d^2) for each axis
        let ixx = (mass / 12.0) * (height * height + depth * depth);
        let iyy = (mass / 12.0) * (width * width + depth * depth);
        let izz = (mass / 12.0) * (width * width + height * height);
        Self::new(mass, ixx, iyy, izz)
    }

    /// Create for a sphere
    pub fn sphere(mass: f32, radius: f32) -> Self {
        let i = (2.0 / 5.0) * mass * radius * radius;
        Self::new(mass, i, i, i)
    }

    /// Create for a cylinder (aligned with Z axis)
    pub fn cylinder_z(mass: f32, radius: f32, height: f32) -> Self {
        let ixx = (mass / 12.0) * (3.0 * radius * radius + height * height);
        let iyy = (mass / 12.0) * (3.0 * radius * radius + height * height);
        let izz = (mass / 2.0) * radius * radius;
        Self::new(mass, ixx, iyy, izz)
    }

    /// Get mass
    pub fn mass(&self) -> f32 {
        self.mass
    }

    /// Get inverse mass (1/m)
    pub fn inv_mass(&self) -> f32 {
        if self.mass > 0.0 {
            1.0 / self.mass
        } else {
            0.0
        }
    }

    /// Get inertia matrix
    pub fn matrix(&self) -> Mat3 {
        Mat3::from_cols(
            Vec3::new(self.ixx, self.ixy, self.ixz),
            Vec3::new(self.ixy, self.iyy, self.iyz),
            Vec3::new(self.ixz, self.iyz, self.izz),
        )
    }

    /// Get inverse inertia matrix (in body frame)
    pub fn inv_matrix(&self) -> Mat3 {
        let m = self.matrix();
        m.inverse()
    }

    /// Get inverse inertia in world frame
    pub fn inv_inertia_world(&self, orientation: Quat) -> Mat3 {
        let inv_body = self.inv_matrix();
        let rot = Mat3::from_quat(orientation);
        rot * inv_body * rot.transpose()
    }
}

/// Collision information
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct CollisionInfo {
    /// Collision point
    pub point: Vec3,
    /// Collision normal
    pub normal: Vec3,
    /// Penetration depth
    pub penetration: f32,
    /// Collision velocity
    pub velocity: Vec3,
    /// Is valid collision
    pub has_collided: bool,
}

impl CollisionInfo {
    /// No collision
    pub fn none() -> Self {
        Self {
            point: Vec3::ZERO,
            normal: Vec3::Y,
            penetration: 0.0,
            velocity: Vec3::ZERO,
            has_collided: false,
        }
    }

    /// Create collision info
    pub fn new(point: Vec3, normal: Vec3, penetration: f32, velocity: Vec3) -> Self {
        Self {
            point,
            normal: normal.normalize(),
            penetration,
            velocity,
            has_collided: true,
        }
    }
}

/// Physics body representing a rigid body in simulation
#[derive(Debug, Clone, PartialEq)]
pub struct PhysicsBody {
    /// Body name/identifier
    pub name: String,
    /// Kinematics state
    pub kinematics: KinematicsState,
    /// Inertia properties
    pub inertia: InertiaTensor,
    /// Collision vertices
    pub vertices: Vec<PhysicsBodyVertex>,
    /// External wrench applied to body
    pub external_wrench: Wrench,
    /// Is kinematic (not affected by physics)
    pub is_kinematic: bool,
    /// Collision restitution (0-1)
    pub restitution: f32,
    /// Friction coefficient
    pub friction: f32,
    /// Linear damping
    pub linear_damping: f32,
    /// Angular damping
    pub angular_damping: f32,
    /// Sleep threshold (velocity below which body sleeps)
    pub sleep_threshold: f32,
    /// Is sleeping (optimization)
    pub is_sleeping: bool,
    /// Gravity scale (1.0 = normal gravity)
    pub gravity_scale: f32,
}

impl Default for PhysicsBody {
    fn default() -> Self {
        Self::new("body", 1.0)
    }
}

impl PhysicsBody {
    /// Create new physics body
    pub fn new(name: &str, mass: f32) -> Self {
        let kinematics = KinematicsState::default();
        let inertia = InertiaTensor::sphere(mass, 0.1);

        Self {
            name: name.to_string(),
            kinematics,
            inertia,
            vertices: Vec::new(),
            external_wrench: Wrench::zero(),
            is_kinematic: false,
            restitution: 0.5,
            friction: 0.5,
            linear_damping: 0.01,
            angular_damping: 0.01,
            sleep_threshold: 0.01,
            is_sleeping: false,
            gravity_scale: 1.0,
        }
    }

    /// Create with specific inertia
    pub fn with_inertia(mut self, inertia: InertiaTensor) -> Self {
        self.inertia = inertia;
        self
    }

    /// Add collision vertex
    pub fn add_vertex(&mut self, vertex: PhysicsBodyVertex) {
        self.vertices.push(vertex);
    }

    /// Set restitution
    pub fn with_restitution(mut self, restitution: f32) -> Self {
        self.restitution = restitution.clamp(0.0, 1.0);
        self
    }

    /// Set friction
    pub fn with_friction(mut self, friction: f32) -> Self {
        self.friction = friction.clamp(0.0, 1.0);
        self
    }

    /// Set kinematic
    pub fn set_kinematic(&mut self, kinematic: bool) {
        self.is_kinematic = kinematic;
    }

    /// Set gravity scale
    pub fn with_gravity_scale(mut self, scale: f32) -> Self {
        self.gravity_scale = scale;
        self
    }

    /// Get mass
    pub fn mass(&self) -> f32 {
        self.inertia.mass()
    }

    /// Get position
    pub fn position(&self) -> Vec3 {
        self.kinematics.position()
    }

    /// Get orientation
    pub fn orientation(&self) -> Quat {
        self.kinematics.orientation()
    }

    /// Get linear velocity
    pub fn linear_velocity(&self) -> Vec3 {
        self.kinematics.linear_velocity()
    }

    /// Get angular velocity
    pub fn angular_velocity(&self) -> Vec3 {
        self.kinematics.angular_velocity()
    }

    /// Set position
    pub fn set_position(&mut self, position: Vec3) {
        self.kinematics.pose.position = position;
    }

    /// Set orientation
    pub fn set_orientation(&mut self, orientation: Quat) {
        self.kinematics.pose.orientation = orientation.normalize();
    }

    /// Set linear velocity
    pub fn set_linear_velocity(&mut self, velocity: Vec3) {
        self.kinematics.twist.linear = velocity;
        self.update_sleep_state();
    }

    /// Set angular velocity
    pub fn set_angular_velocity(&mut self, velocity: Vec3) {
        self.kinematics.twist.angular = velocity;
        self.update_sleep_state();
    }

    /// Apply force at center of mass
    pub fn apply_force(&mut self, force: Vec3) {
        self.external_wrench.force += force;
    }

    /// Apply torque
    pub fn apply_torque(&mut self, torque: Vec3) {
        self.external_wrench.torque += torque;
    }

    /// Apply wrench
    pub fn apply_wrench(&mut self, wrench: Wrench) {
        self.external_wrench = self.external_wrench + wrench;
    }

    /// Apply force at specific point (world space)
    pub fn apply_force_at_point(&mut self, force: Vec3, point: Vec3) {
        let r = point - self.position();
        self.apply_force(force);
        self.apply_torque(r.cross(force));
    }

    /// Apply impulse at center of mass
    pub fn apply_linear_impulse(&mut self, impulse: Vec3) {
        let inv_mass = self.inertia.inv_mass();
        let dv = impulse * inv_mass;
        self.kinematics.twist.linear += dv;
        self.update_sleep_state();
    }

    /// Apply angular impulse
    pub fn apply_angular_impulse(&mut self, impulse: Vec3) {
        let inv_inertia = self.inertia.inv_inertia_world(self.orientation());
        let dw = inv_inertia * impulse;
        self.kinematics.twist.angular += dw;
        self.update_sleep_state();
    }

    /// Apply impulse at point
    pub fn apply_impulse_at_point(&mut self, impulse: Vec3, point: Vec3) {
        let r = point - self.position();

        // Linear
        self.apply_linear_impulse(impulse);

        // Angular
        let torque_impulse = r.cross(impulse);
        self.apply_angular_impulse(torque_impulse);
    }

    /// Update sleep state based on velocity
    fn update_sleep_state(&mut self) {
        let vel_sq = self.kinematics.twist.linear.length_squared()
            + self.kinematics.twist.angular.length_squared();

        if vel_sq > self.sleep_threshold * self.sleep_threshold {
            self.is_sleeping = false;
        }
    }

    /// Wake up the body
    pub fn wake(&mut self) {
        self.is_sleeping = false;
    }

    /// Put body to sleep
    pub fn sleep(&mut self) {
        if !self.is_kinematic {
            self.is_sleeping = true;
            self.kinematics.twist.linear = Vec3::ZERO;
            self.kinematics.twist.angular = Vec3::ZERO;
        }
    }

    /// Compute accelerations from applied wrench
    pub fn compute_accelerations(&self, gravity: Vec3) -> Accelerations {
        if self.is_kinematic || self.is_sleeping {
            return Accelerations::zero();
        }

        let mass = self.inertia.mass();
        let inv_mass = if mass > 0.0 { 1.0 / mass } else { 0.0 };

        // Linear acceleration: a = F/m + g
        let linear_acc = self.external_wrench.force * inv_mass + gravity * self.gravity_scale;

        // Angular acceleration: alpha = I^(-1) * tau
        let inv_inertia = self.inertia.inv_inertia_world(self.orientation());
        let angular_acc = inv_inertia * self.external_wrench.torque;

        Accelerations {
            linear: linear_acc,
            angular: angular_acc,
        }
    }

    /// Clear external wrench (call after integration)
    pub fn clear_wrench(&mut self) {
        self.external_wrench = Wrench::zero();
    }

    /// Update vertices world positions
    pub fn update_vertices(&mut self) {
        for vertex in &mut self.vertices {
            vertex.update_world_position(&self.kinematics.pose);
        }
    }

    /// Check collision with another body (simplified sphere check)
    pub fn check_collision_sphere(&self, other: &PhysicsBody) -> CollisionInfo {
        let p1 = self.position();
        let p2 = other.position();

        // Approximate radius from inertia (very rough)
        let r1 = (self.inertia.ixx / self.inertia.mass()).sqrt();
        let r2 = (other.inertia.ixx / other.inertia.mass()).sqrt();

        let distance = (p2 - p1).length();
        let combined_radius = r1 + r2;

        if distance < combined_radius && distance > 0.001 {
            let normal = (p2 - p1).normalize();
            let penetration = combined_radius - distance;
            let point = p1 + normal * r1;
            let velocity = self.linear_velocity() - other.linear_velocity();

            CollisionInfo::new(point, normal, penetration, velocity)
        } else {
            CollisionInfo::none()
        }
    }

    /// Get kinetic energy
    pub fn kinetic_energy(&self) -> f32 {
        let linear_ke = 0.5 * self.inertia.mass() * self.linear_velocity().length_squared();
        let angular_ke = 0.5 * self.inertia.ixx * self.angular_velocity().length_squared();
        linear_ke + angular_ke
    }

    /// Get bounding radius (approximate)
    pub fn bounding_radius(&self) -> f32 {
        let max_vertex_dist = self
            .vertices
            .iter()
            .map(|v| v.local_position.length())
            .fold(0.0, f32::max);

        max_vertex_dist.max((self.inertia.ixx / self.inertia.mass()).sqrt())
    }
}
