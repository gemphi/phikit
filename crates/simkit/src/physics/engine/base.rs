//! Physics engine base trait and implementations
//!
//! Provides different physics engine implementations similar to AirSim:
//! - FastPhysicsEngine: Simple, efficient physics
//! - ExternalPhysicsEngine: Interface to external physics (Unreal, Unity, etc.)

use crate::physics::{KinematicsState, Wrench};
use glam::Vec3;

/// Physics engine trait - pluggable physics implementations
pub trait PhysicsEngine: Send + Sync {
    /// Initialize the physics engine
    fn initialize(&mut self);

    /// Shutdown the physics engine
    fn shutdown(&mut self);

    /// Update physics simulation
    fn update(&mut self, dt: f32);

    /// Get current simulation time
    fn simulation_time(&self) -> f64;

    /// Set gravity
    fn set_gravity(&mut self, gravity: Vec3);

    /// Get gravity
    fn gravity(&self) -> Vec3;

    /// Create a physics body
    fn create_body(&mut self, name: &str, mass: f32) -> usize;

    /// Remove a physics body
    fn remove_body(&mut self, handle: usize) -> bool;

    /// Get body state
    fn get_body_state(&self, handle: usize) -> Option<KinematicsState>;

    /// Set body state
    fn set_body_state(&mut self, handle: usize, state: KinematicsState);

    /// Apply force to body
    fn apply_force(&mut self, handle: usize, force: Vec3);

    /// Apply torque to body
    fn apply_torque(&mut self, handle: usize, torque: Vec3);

    /// Apply wrench to body
    fn apply_wrench(&mut self, handle: usize, wrench: Wrench);

    /// Set kinematic state (physics doesn't affect it)
    fn set_kinematic(&mut self, handle: usize, kinematic: bool);

    /// Check if body exists
    fn body_exists(&self, handle: usize) -> bool;

    /// Pause simulation
    fn pause(&mut self);

    /// Resume simulation
    fn resume(&mut self);

    /// Is paused
    fn is_paused(&self) -> bool;

    /// Reset simulation
    fn reset(&mut self);
}

