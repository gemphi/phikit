//! Physics world for managing multiple physics bodies
//!
//! Manages simulation of multiple physics bodies with collision detection.

use super::PhysicsBody;
use glam::{Quat, Vec3};

/// Physics simulation parameters
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PhysicsParams {
    /// Gravity vector (m/s^2)
    pub gravity: Vec3,
    /// Simulation time step (s)
    pub time_step: f32,
    /// Fixed time step for physics
    pub fixed_time_step: f32,
    /// Maximum substeps per frame
    pub max_substeps: u32,
    /// Position solver iterations
    pub position_iterations: u32,
    /// Velocity solver iterations
    pub velocity_iterations: u32,
    /// Global damping
    pub global_damping: f32,
    /// Sleep threshold
    pub sleep_threshold: f32,
}

impl Default for PhysicsParams {
    fn default() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0), // Y-down convention
            time_step: 0.016,                    // 60 FPS
            fixed_time_step: 0.016,
            max_substeps: 8,
            position_iterations: 4,
            velocity_iterations: 4,
            global_damping: 0.99,
            sleep_threshold: 0.01,
        }
    }
}

impl PhysicsParams {
    /// Create new physics params
    pub fn new() -> Self {
        Self::default()
    }

    /// Set gravity
    pub fn with_gravity(mut self, gravity: Vec3) -> Self {
        self.gravity = gravity;
        self
    }

    /// Set time step
    pub fn with_time_step(mut self, dt: f32) -> Self {
        self.time_step = dt;
        self.fixed_time_step = dt;
        self
    }

    /// Set target FPS
    pub fn with_fps(mut self, fps: f32) -> Self {
        self.time_step = 1.0 / fps;
        self.fixed_time_step = self.time_step;
        self
    }

    /// Set iterations
    pub fn with_iterations(mut self, position: u32, velocity: u32) -> Self {
        self.position_iterations = position;
        self.velocity_iterations = velocity;
        self
    }
}

/// Physics world managing all bodies
#[derive(Debug, Clone, PartialEq)]
pub struct PhysicsWorld {
    /// Physics parameters
    pub params: PhysicsParams,
    /// Physics bodies
    pub bodies: Vec<PhysicsBody>,
    /// Time accumulator for fixed timestep
    time_accumulator: f32,
    /// Simulation time
    pub simulation_time: f64,
    /// Frame counter
    pub frame_count: u64,
    /// Is paused
    pub is_paused: bool,
    /// Collision pairs for this frame
    collision_pairs: Vec<(usize, usize)>,
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new(PhysicsParams::default())
    }
}

impl PhysicsWorld {
    /// Create new physics world
    pub fn new(params: PhysicsParams) -> Self {
        Self {
            params,
            bodies: Vec::new(),
            time_accumulator: 0.0,
            simulation_time: 0.0,
            frame_count: 0,
            is_paused: false,
            collision_pairs: Vec::new(),
        }
    }

    /// Add a body to the world
    pub fn add_body(&mut self, body: PhysicsBody) -> usize {
        let index = self.bodies.len();
        self.bodies.push(body);
        index
    }

    /// Remove a body
    pub fn remove_body(&mut self, index: usize) -> Option<PhysicsBody> {
        if index < self.bodies.len() {
            Some(self.bodies.remove(index))
        } else {
            None
        }
    }

    /// Get body by index
    pub fn get_body(&self, index: usize) -> Option<&PhysicsBody> {
        self.bodies.get(index)
    }

    /// Get mutable body
    pub fn get_body_mut(&mut self, index: usize) -> Option<&mut PhysicsBody> {
        self.bodies.get_mut(index)
    }

    /// Get body by name
    pub fn get_body_by_name(&self, name: &str) -> Option<&PhysicsBody> {
        self.bodies.iter().find(|b| b.name == name)
    }

    /// Get mutable body by name
    pub fn get_body_by_name_mut(&mut self, name: &str) -> Option<&mut PhysicsBody> {
        self.bodies.iter_mut().find(|b| b.name == name)
    }

    /// Number of bodies
    pub fn body_count(&self) -> usize {
        self.bodies.len()
    }

    /// Step simulation with variable timestep
    pub fn step(&mut self, dt: f32) {
        if self.is_paused {
            return;
        }

        self.time_accumulator += dt;

        // Fixed timestep substeps
        let mut substeps = 0;
        while self.time_accumulator >= self.params.fixed_time_step
            && substeps < self.params.max_substeps
        {
            self.step_fixed(self.params.fixed_time_step);
            self.time_accumulator -= self.params.fixed_time_step;
            substeps += 1;
        }

        // Clamp accumulator to prevent spiral of death
        self.time_accumulator = self.time_accumulator.min(self.params.fixed_time_step * 2.0);
    }

    /// Single fixed timestep
    pub fn step_fixed(&mut self, dt: f32) {
        self.frame_count += 1;
        self.simulation_time += dt as f64;

        // 1. Detect collisions
        self.detect_collisions();

        // 2. Solve collisions
        self.solve_collisions();

        // 3. Integrate all bodies
        self.integrate_bodies(dt);

        // 4. Update vertices
        self.update_vertices();
    }

    /// Detect collisions between all bodies
    fn detect_collisions(&mut self) {
        self.collision_pairs.clear();

        let count = self.bodies.len();
        for i in 0..count {
            for j in (i + 1)..count {
                let b1 = &self.bodies[i];
                let b2 = &self.bodies[j];

                // Broad phase: distance check
                let pos1 = b1.position();
                let pos2 = b2.position();
                let r1 = b1.bounding_radius();
                let r2 = b2.bounding_radius();

                if (pos2 - pos1).length_squared() < (r1 + r2) * (r1 + r2) {
                    self.collision_pairs.push((i, j));
                }
            }
        }
    }

    /// Solve collisions (simple impulse response)
    fn solve_collisions(&mut self) {
        for _ in 0..self.params.position_iterations {
            for &(i, j) in &self.collision_pairs.clone() {
                // Get collision info using spheres for simplicity
                let collision = {
                    let b1 = &self.bodies[i];
                    let b2 = &self.bodies[j];
                    b1.check_collision_sphere(b2)
                };

                if collision.has_collided {
                    // Apply position correction to resolve penetration
                    let penetration = collision.penetration;
                    let normal = collision.normal;

                    let inv_mass1 = self.bodies[i].inertia.inv_mass();
                    let inv_mass2 = self.bodies[j].inertia.inv_mass();
                    let total_inv_mass = inv_mass1 + inv_mass2;

                    if total_inv_mass > 0.0 {
                        let correction = normal * (penetration / total_inv_mass) * 0.5;

                        if !self.bodies[i].is_kinematic {
                            self.bodies[i].kinematics.pose.position -= correction * inv_mass1;
                        }
                        if !self.bodies[j].is_kinematic {
                            self.bodies[j].kinematics.pose.position += correction * inv_mass2;
                        }
                    }
                }
            }
        }

        // Velocity impulse resolution
        for _ in 0..self.params.velocity_iterations {
            for &(i, j) in &self.collision_pairs.clone() {
                let collision = {
                    let b1 = &self.bodies[i];
                    let b2 = &self.bodies[j];
                    b1.check_collision_sphere(b2)
                };

                if collision.has_collided {
                    let normal = collision.normal;
                    let vel1 = self.bodies[i].linear_velocity();
                    let vel2 = self.bodies[j].linear_velocity();
                    let relative_vel = vel1 - vel2;
                    let vel_along_normal = relative_vel.dot(normal);

                    // Only resolve if moving towards each other
                    if vel_along_normal > 0.0 {
                        continue;
                    }

                    let inv_mass1 = self.bodies[i].inertia.inv_mass();
                    let inv_mass2 = self.bodies[j].inertia.inv_mass();
                    let total_inv_mass = inv_mass1 + inv_mass2;

                    if total_inv_mass == 0.0 {
                        continue;
                    }

                    // Restitution
                    let restitution = self.bodies[i].restitution.min(self.bodies[j].restitution);

                    // Impulse scalar
                    let j_scalar = -(1.0 + restitution) * vel_along_normal / total_inv_mass;
                    let impulse = normal * j_scalar;

                    // Apply impulse
                    if !self.bodies[i].is_kinematic {
                        let new_vel = vel1 + impulse * inv_mass1;
                        self.bodies[i].set_linear_velocity(new_vel);
                    }
                    if !self.bodies[j].is_kinematic {
                        let new_vel = vel2 - impulse * inv_mass2;
                        self.bodies[j].set_linear_velocity(new_vel);
                    }
                }
            }
        }
    }

    /// Integrate all bodies
    fn integrate_bodies(&mut self, dt: f32) {
        let gravity = self.params.gravity;

        for body in &mut self.bodies {
            if body.is_kinematic || body.is_sleeping {
                body.clear_wrench();
                continue;
            }

            // Compute accelerations
            let accelerations = body.compute_accelerations(gravity);

            // Semi-implicit Euler integration
            // v_new = v_old + a * dt
            body.kinematics.twist.linear += accelerations.linear * dt;
            body.kinematics.twist.angular += accelerations.angular * dt;

            // Apply damping
            body.kinematics.twist.linear *= 1.0 - body.linear_damping * dt;
            body.kinematics.twist.angular *= 1.0 - body.angular_damping * dt;

            // Apply global damping
            body.kinematics.twist.linear *= self.params.global_damping;
            body.kinematics.twist.angular *= self.params.global_damping;

            // p_new = p_old + v_new * dt
            body.kinematics.pose.position += body.kinematics.twist.linear * dt;

            // Angular integration: q_new = q_old + 0.5 * q * omega * dt
            let omega = body.kinematics.twist.angular;
            let q = body.kinematics.pose.orientation;
            let omega_quat = Quat::from_xyzw(omega.x, omega.y, omega.z, 0.0);
            let q_deriv = q * omega_quat * 0.5;
            let q_new = Quat::from_xyzw(
                q.x + q_deriv.x * dt,
                q.y + q_deriv.y * dt,
                q.z + q_deriv.z * dt,
                q.w + q_deriv.w * dt,
            )
            .normalize();
            body.kinematics.pose.orientation = q_new;

            // Update timestamp
            body.kinematics.timestamp += (dt * 1000.0) as u64;

            // Clear external wrench
            body.clear_wrench();

            // Check sleep
            let speed_sq = body.kinematics.twist.linear.length_squared()
                + body.kinematics.twist.angular.length_squared();
            if speed_sq < body.sleep_threshold * body.sleep_threshold && speed_sq < 0.0001 {
                // Could put to sleep here if stationary for multiple frames
            }
        }
    }

    /// Update all vertices
    fn update_vertices(&mut self) {
        for body in &mut self.bodies {
            body.update_vertices();
        }
    }

    /// Pause simulation
    pub fn pause(&mut self) {
        self.is_paused = true;
    }

    /// Resume simulation
    pub fn resume(&mut self) {
        self.is_paused = false;
    }

    /// Toggle pause
    pub fn toggle_pause(&mut self) {
        self.is_paused = !self.is_paused;
    }

    /// Wake all bodies
    pub fn wake_all(&mut self) {
        for body in &mut self.bodies {
            body.wake();
        }
    }

    /// Put all bodies to sleep
    pub fn sleep_all(&mut self) {
        for body in &mut self.bodies {
            body.sleep();
        }
    }

    /// Reset world
    pub fn reset(&mut self) {
        self.simulation_time = 0.0;
        self.frame_count = 0;
        self.time_accumulator = 0.0;
        self.collision_pairs.clear();
        self.wake_all();
    }

    /// Clear all bodies
    pub fn clear(&mut self) {
        self.bodies.clear();
        self.reset();
    }

    /// Get total kinetic energy
    pub fn total_kinetic_energy(&self) -> f32 {
        self.bodies.iter().map(|b| b.kinetic_energy()).sum()
    }

    /// Raycast against all bodies (simplified)
    pub fn raycast(
        &self,
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
    ) -> Option<(usize, f32, Vec3)> {
        let dir = direction.normalize();
        let mut closest: Option<(usize, f32, Vec3)> = None;

        for (i, body) in self.bodies.iter().enumerate() {
            // Simple sphere raycast
            let center = body.position();
            let radius = body.bounding_radius();

            let oc = origin - center;
            let a = dir.dot(dir);
            let b = 2.0 * oc.dot(dir);
            let c = oc.dot(oc) - radius * radius;
            let discriminant = b * b - 4.0 * a * c;

            if discriminant >= 0.0 {
                let t = (-b - discriminant.sqrt()) / (2.0 * a);
                if t >= 0.0 && t <= max_distance {
                    if closest.map_or(true, |(_, t_closest, _)| t < t_closest) {
                        let point = origin + dir * t;
                        let normal = (point - center).normalize();
                        closest = Some((i, t, normal));
                    }
                }
            }
        }

        closest
    }

    /// Apply explosion force at position
    pub fn apply_explosion(&mut self, position: Vec3, radius: f32, force: f32) {
        for body in &mut self.bodies {
            let delta = body.position() - position;
            let dist_sq = delta.length_squared();
            let radius_sq = radius * radius;

            if dist_sq < radius_sq && dist_sq > 0.001 {
                let dist = dist_sq.sqrt();
                let factor = 1.0 - (dist / radius);
                let dir = delta / dist;
                let impulse = dir * force * factor;
                body.apply_linear_impulse(impulse);
            }
        }
    }
}
