use super::PhysicsEngine;
use crate::physics::{Environment, KinematicsState, PhysicsBody, PhysicsParams, PhysicsWorld, Wrench};
use glam::Vec3;
/// Fast physics engine - efficient lightweight physics
///
/// Similar to AirSim's FastPhysicsEngine, this provides a simple
/// but efficient physics simulation suitable for real-time use.
pub struct FastPhysicsEngine {
    /// Physics world
    world: PhysicsWorld,
    /// Environment state
    environment: Environment,
    /// Is running
    is_running: bool,
    /// Debug visualization enabled
    debug_visualization: bool,
    /// Step callback
    on_step: Option<Box<dyn Fn(&PhysicsWorld) + Send + Sync>>,
}

impl std::fmt::Debug for FastPhysicsEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FastPhysicsEngine")
            .field("world", &self.world)
            .field("environment", &self.environment)
            .field("is_running", &self.is_running)
            .field("debug_visualization", &self.debug_visualization)
            .field("on_step", &self.on_step.as_ref().map(|_| "callback"))
            .finish()
    }
}

impl Default for FastPhysicsEngine {
    fn default() -> Self {
        Self::new(PhysicsParams::default())
    }
}

impl FastPhysicsEngine {
    /// Create new fast physics engine
    pub fn new(params: PhysicsParams) -> Self {
        Self {
            world: PhysicsWorld::new(params),
            environment: Environment::default(),
            is_running: false,
            debug_visualization: false,
            on_step: None,
        }
    }

    /// Create with default parameters
    pub fn default_params() -> Self {
        Self::new(PhysicsParams::default())
    }

    /// Set debug visualization
    pub fn set_debug_visualization(&mut self, enabled: bool) {
        self.debug_visualization = enabled;
    }

    /// Set step callback
    pub fn on_step<F>(&mut self, callback: F)
    where
        F: Fn(&PhysicsWorld) + Send + Sync + 'static,
    {
        self.on_step = Some(Box::new(callback));
    }

    /// Get reference to physics world
    pub fn world(&self) -> &PhysicsWorld {
        &self.world
    }

    /// Get mutable reference to physics world
    pub fn world_mut(&mut self) -> &mut PhysicsWorld {
        &mut self.world
    }

    /// Get environment
    pub fn environment(&self) -> &Environment {
        &self.environment
    }

    /// Get mutable environment
    pub fn environment_mut(&mut self) -> &mut Environment {
        &mut self.environment
    }

    /// Get body by name
    pub fn get_body(&self, name: &str) -> Option<&PhysicsBody> {
        self.world.get_body_by_name(name)
    }

    /// Get mutable body by name
    pub fn get_body_mut(&mut self, name: &str) -> Option<&mut PhysicsBody> {
        self.world.get_body_by_name_mut(name)
    }

    /// Raycast in the world
    pub fn raycast(
        &self,
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
    ) -> Option<(usize, f32, Vec3)> {
        self.world.raycast(origin, direction, max_distance)
    }

    /// Set time step
    pub fn set_time_step(&mut self, dt: f32) {
        self.world.params.time_step = dt;
        self.world.params.fixed_time_step = dt;
    }

    /// Set FPS target
    pub fn set_target_fps(&mut self, fps: f32) {
        self.world.params = self.world.params.with_fps(fps);
    }
}

impl PhysicsEngine for FastPhysicsEngine {
    fn initialize(&mut self) {
        self.is_running = true;
    }

    fn shutdown(&mut self) {
        self.is_running = false;
        self.world.clear();
    }

    fn update(&mut self, dt: f32) {
        if !self.is_running {
            return;
        }

        // Update environment
        let wind = self.environment.state().wind;

        // Apply wind forces to bodies
        for body in &mut self.world.bodies {
            if !body.is_kinematic && !body.is_sleeping {
                // Simple drag force: F = 0.5 * rho * v^2 * Cd * A
                let rel_vel = body.linear_velocity() - wind;
                let drag_force = -rel_vel * 0.1 * rel_vel.length();
                body.apply_force(drag_force);
            }
        }

        // Step physics world
        self.world.step(dt);

        // Call step callback
        if let Some(ref callback) = self.on_step {
            callback(&self.world);
        }
    }

    fn simulation_time(&self) -> f64 {
        self.world.simulation_time
    }

    fn set_gravity(&mut self, gravity: Vec3) {
        self.world.params.gravity = gravity;
    }

    fn gravity(&self) -> Vec3 {
        self.world.params.gravity
    }

    fn create_body(&mut self, name: &str, mass: f32) -> usize {
        let body = PhysicsBody::new(name, mass);
        self.world.add_body(body)
    }

    fn remove_body(&mut self, handle: usize) -> bool {
        self.world.remove_body(handle).is_some()
    }

    fn get_body_state(&self, handle: usize) -> Option<KinematicsState> {
        self.world.get_body(handle).map(|b| b.kinematics)
    }

    fn set_body_state(&mut self, handle: usize, state: KinematicsState) {
        if let Some(body) = self.world.get_body_mut(handle) {
            body.kinematics = state;
        }
    }

    fn apply_force(&mut self, handle: usize, force: Vec3) {
        if let Some(body) = self.world.get_body_mut(handle) {
            body.apply_force(force);
        }
    }

    fn apply_torque(&mut self, handle: usize, torque: Vec3) {
        if let Some(body) = self.world.get_body_mut(handle) {
            body.apply_torque(torque);
        }
    }

    fn apply_wrench(&mut self, handle: usize, wrench: Wrench) {
        if let Some(body) = self.world.get_body_mut(handle) {
            body.apply_wrench(wrench);
        }
    }

    fn set_kinematic(&mut self, handle: usize, kinematic: bool) {
        if let Some(body) = self.world.get_body_mut(handle) {
            body.set_kinematic(kinematic);
        }
    }

    fn body_exists(&self, handle: usize) -> bool {
        handle < self.world.bodies.len()
    }

    fn pause(&mut self) {
        self.world.pause();
    }

    fn resume(&mut self) {
        self.world.resume();
    }

    fn is_paused(&self) -> bool {
        self.world.is_paused
    }

    fn reset(&mut self) {
        self.world.reset();
    }
}
