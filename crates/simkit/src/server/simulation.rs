//! Simulation Core
//!
//! Core simulation logic that manages the world state, vehicles, and physics.

use std::collections::HashMap;

use crate::common::{CollisionInfo, GeoPoint, Pose};
use crate::physics::{EnvironmentState, KinematicsState};
use crate::vehicles::{CarVehicle, MultirotorVehicle, Vehicle};

/// Main simulation structure
pub struct Simulation {
    /// Simulation vehicles
    vehicles: HashMap<String, Box<dyn Vehicle>>,
    /// Global environment state
    environment: crate::physics::Environment,
    /// Is simulation paused
    paused: bool,
    /// Is recording active
    recording: bool,
    /// API control enabled per vehicle
    api_control: HashMap<String, bool>,
    /// Vehicle armed state
    armed: HashMap<String, bool>,
    /// Simulation time
    sim_time: f64,
    /// Time step
    dt: f32,
    /// The link to a phiecs world, when one is driving the vehicles.
    ///
    /// With a link attached, every RPC call that *commands* a vehicle is
    /// translated by [`ecs::wire`](crate::ecs::wire) and put on this link
    /// — the same link a gamepad pushes onto — and every call that *asks*
    /// is answered from the report the world last published. Without one,
    /// the server answers from this struct's own vehicles, as it did
    /// before there was a world to ask.
    link: Option<crate::ecs::ControlLink>,
}

impl Simulation {
    /// Create new simulation
    pub fn new() -> Self {
        Self {
            vehicles: HashMap::new(),
            environment: crate::physics::Environment::default(),
            paused: false,
            recording: false,
            api_control: HashMap::new(),
            armed: HashMap::new(),
            sim_time: 0.0,
            dt: 0.016, // 60 FPS
            link: None,
        }
    }

    /// Wires this server to a phiecs world's [`ControlLink`], so a
    /// client's commands take the same path a pilot's stick does.
    ///
    /// [`ControlLink`]: crate::ecs::ControlLink
    pub fn attach(&mut self, link: crate::ecs::ControlLink) {
        self.link = Some(link);
    }

    /// The world's link, if one is attached.
    pub fn link(&self) -> Option<&crate::ecs::ControlLink> {
        self.link.as_ref()
    }

    /// The vehicle a client that named none must have meant: the only one
    /// there is, or nothing when there is a choice to be made.
    pub fn only_vehicle(&self) -> String {
        let named: Vec<String> = match self.link.as_ref() {
            Some(link) => link.reported(),
            None => self.vehicles.keys().cloned().collect(),
        };
        match named.len() {
            1 => named[0].clone(),
            _ => String::new(),
        }
    }

    /// What the world last said about `vehicle`, if a world is attached
    /// and has stepped.
    pub fn report(&self, vehicle: &str) -> Option<crate::ecs::Report> {
        let link = self.link.as_ref()?;
        let named = match vehicle.is_empty() {
            true => self.only_vehicle(),
            false => vehicle.to_string(),
        };
        link.report(&named)
    }

    /// Add a vehicle to the simulation
    pub fn add_vehicle(&mut self, name: &str, vehicle: Box<dyn Vehicle>) {
        self.vehicles.insert(name.to_string(), vehicle);
        self.api_control.insert(name.to_string(), false);
        self.armed.insert(name.to_string(), false);
    }

    /// Remove a vehicle
    pub fn remove_vehicle(&mut self, name: &str) {
        self.vehicles.remove(name);
        self.api_control.remove(name);
        self.armed.remove(name);
    }

    /// Get vehicle reference
    pub fn get_vehicle(&self, name: &str) -> Option<&Box<dyn Vehicle>> {
        self.vehicles.get(name)
    }

    /// Get vehicle mutable reference
    pub fn get_vehicle_mut(&mut self, name: &str) -> Option<&mut Box<dyn Vehicle>> {
        self.vehicles.get_mut(name)
    }

    /// List all vehicle names
    pub fn list_vehicles(&self) -> Vec<String> {
        self.vehicles.keys().cloned().collect()
    }

    /// Step the simulation forward
    pub fn step(&mut self) {
        if self.paused {
            return;
        }

        self.sim_time += self.dt as f64;

        // Update all vehicles
        for (_name, vehicle) in &mut self.vehicles {
            vehicle.update(self.dt, &self.environment);
        }
    }

    /// Pause/unpause simulation
    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    /// Check if paused
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Enable API control for vehicle
    pub fn enable_api_control(&mut self, vehicle_name: &str, enabled: bool) {
        if let Some(ctrl) = self.api_control.get_mut(vehicle_name) {
            *ctrl = enabled;
        }
    }

    /// Check if API control is enabled
    pub fn is_api_control_enabled(&self, vehicle_name: &str) -> bool {
        self.api_control.get(vehicle_name).copied().unwrap_or(false)
    }

    /// Arm/disarm vehicle
    pub fn arm_disarm(&mut self, vehicle_name: &str, arm: bool) -> bool {
        if let Some(armed) = self.armed.get_mut(vehicle_name) {
            *armed = arm;
            true
        } else {
            false
        }
    }

    /// Check if vehicle is armed
    pub fn is_armed(&self, vehicle_name: &str) -> bool {
        self.armed.get(vehicle_name).copied().unwrap_or(false)
    }

    /// Get home geo point
    pub fn get_home_geo_point(&self, _vehicle_name: &str) -> GeoPoint {
        // Default home point
        GeoPoint::new(47.6062, -122.3321, 0.0)
    }

    /// Get vehicle pose
    pub fn get_vehicle_pose(&self, vehicle_name: &str) -> Option<Pose> {
        self.vehicles.get(vehicle_name).map(|v| *v.pose())
    }

    /// Set vehicle pose
    pub fn set_vehicle_pose(&mut self, vehicle_name: &str, pose: Pose, _ignore_collision: bool) {
        if let Some(vehicle) = self.vehicles.get_mut(vehicle_name) {
            vehicle.set_pose(pose);
        }
    }

    /// Get collision info
    pub fn get_collision_info(&self, vehicle_name: &str) -> CollisionInfo {
        // Check for collisions (simplified)
        if let Some(_vehicle) = self.vehicles.get(vehicle_name) {
            CollisionInfo::default()
        } else {
            CollisionInfo::default()
        }
    }

    /// Get ground truth kinematics
    pub fn get_ground_truth_kinematics(&self, vehicle_name: &str) -> Option<KinematicsState> {
        self.vehicles.get(vehicle_name).map(|v| *v.kinematics())
    }

    /// Get environment state
    pub fn environment(&self) -> &EnvironmentState {
        self.environment.state()
    }

    /// Get mutable environment
    pub fn environment_mut(&mut self) -> &mut crate::physics::Environment {
        &mut self.environment
    }

    /// Start recording
    pub fn start_recording(&mut self) {
        self.recording = true;
    }

    /// Stop recording
    pub fn stop_recording(&mut self) {
        self.recording = false;
    }

    /// Check if recording
    pub fn is_recording(&self) -> bool {
        self.recording
    }

    /// Reset simulation
    pub fn reset(&mut self) {
        self.sim_time = 0.0;
        self.paused = false;

        // Reset all vehicles
        for (_name, vehicle) in &mut self.vehicles {
            vehicle.reset();
        }
    }

    /// Get simulation time
    pub fn sim_time(&self) -> f64 {
        self.sim_time
    }
}

impl Default for Simulation {
    fn default() -> Self {
        Self::new()
    }
}

/// Simulation builder for easy setup
pub struct SimulationBuilder {
    vehicles: Vec<(String, Box<dyn Vehicle>)>,
    environment: EnvironmentState,
}

impl SimulationBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self {
            vehicles: Vec::new(),
            environment: EnvironmentState::default(),
        }
    }

    /// Add a multirotor
    pub fn with_multirotor(mut self, name: &str) -> Self {
        let mut config = crate::vehicles::multirotor::MultirotorConfig::default();
        config.vehicle.name = name.to_string();
        let multirotor = MultirotorVehicle::new(config);
        self.vehicles.push((name.to_string(), Box::new(multirotor)));
        self
    }

    /// Add a car
    pub fn with_car(mut self, name: &str) -> Self {
        let mut config = crate::vehicles::car::CarConfig::default();
        config.vehicle.name = name.to_string();
        let car = CarVehicle::new(config);
        self.vehicles.push((name.to_string(), Box::new(car)));
        self
    }

    /// Set home location
    pub fn with_home_location(self, _lat: f64, _lon: f64, _alt: f64) -> Self {
        // Update environment home point
        self
    }

    /// Build the simulation
    pub fn build(self) -> Simulation {
        let mut sim = Simulation::new();

        for (name, vehicle) in self.vehicles {
            sim.add_vehicle(&name, vehicle);
        }

        sim
    }
}

impl Default for SimulationBuilder {
    fn default() -> Self {
        Self::new()
    }
}
