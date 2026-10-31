//! Rust-Sim-Kit
//!
//! An AirSim-inspired simulation toolkit for autonomous vehicles (drones, cars).
//! Provides sensor models, physics simulation types, vehicle controllers, and
//! RPC APIs for simulation environments.
//!
//! # Architecture
//!
//! This crate mirrors the structure of AirSim's AirLib:
//! - `api`: RPC client APIs for connecting to simulation servers
//! - `common`: Shared types, structs, and utilities
//! - `config`: Simulation configuration and settings
//! - `physics`: Physics simulation types (kinematics, environment)
//! - `recording`: Data recording system
//! - `safety`: Geofencing and collision avoidance
//! - `sensors`: Sensor implementations (IMU, GPS, Lidar, Barometer, Magnetometer, Distance)
//! - `server`: RPC server for simulation control
//! - `vehicles`: Vehicle types (multirotor/drone, car)

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

pub mod api;
pub mod common;
pub mod config;
pub mod control;
pub mod controllers;
pub mod ecs;
pub mod input;
pub mod mavlink;
pub mod physics;
pub mod recording;
pub mod safety;
pub mod sensors;
pub mod server;
pub mod vehicles;

// Common types and structs
pub use common::*;

// Physics simulation types
pub use physics::{
    Accelerations, CollisionInfo, Environment, EnvironmentState,
    FastPhysicsEngine, InertiaTensor, KinematicsState, PhysicsBody, PhysicsEngine,
    PhysicsParams, PhysicsWorld, Wrench,
};

// Control systems
pub use control::{
    AttitudeSetpoint, ControlMode, Controller, ControllerGains, MotorOutputs, PidConfig,
    PidController, PidController3, PidOutput, PositionConfig, PositionController, PositionSetpoint,
    RateConfig, RateController, RateSetpoint, VelocitySetpoint,
};

// Flight controllers
pub use controllers::{
    CascadeConfig, CascadeController, ControllerInput, ControllerOutput, FlightCommand,
    FlightController, FlightMode, SimpleFlightConfig, SimpleFlightController, Trajectory,
    TrajectoryPoint,
};

// MAVLink protocol
pub use mavlink::{
    Attitude, BatteryStatus, ComponentId, ConnectionStats, ConnectionType, ExtendedSysState,
    GlobalPositionInt, GpsFixType, GpsRawInt, Heartbeat, MavArmedState, MavAutopilot,
    MavBatteryFunction, MavBatteryStatus, MavBatteryType, MavExtendedState, MavLandedState,
    MavMode, MavResult, MavState, MavSystem, MavType, MavVtolState, MavlinkConnection,
    MavlinkVersion, Message, MessageId, MessagePriority, Sequence, SysStatus, SystemId,
};

// Sensor types and implementations
pub use sensors::{
    BarometerOutput, BarometerSensor, CameraCaptureQueue, CameraConfig, CameraSensor,
    DistanceSensor, GpsOutput, GpsSensor, GroundTruth, ImuOutput, ImuSensor, LidarSensor,
    MagnetometerOutput, MagnetometerSensor, Sensor, SensorBase, SensorCollection, SensorConfig,
    SensorType,
};

// DistanceSensorData is in common module
pub use common::DistanceSensorData;

// Vehicle types and traits
pub use vehicles::{CarVehicle, MultirotorVehicle, Vehicle, VehicleBase, VehicleConfig};

// Recording system
pub use recording::{
    FileRecordingWriter, RecordedFrame, RecordingConfig, RecordingError, RecordingFormat,
    RecordingSession, RecordingState, RecordingStats, RecordingWriter,
};

// Configuration and settings
pub use config::{
    CameraSettings, CarSettings, LidarSettings, MultirotorSettings, SensorSettings, SettingsLoader,
    SimSettings, SimulationVehicleConfig, VehicleSettings,
};

// Safety system
pub use safety::{
    CollisionAvoidance, Geofence, GeofenceManager, GeofenceType, ProximityAlert, SafetyCheck,
    SafetySystem, SafetyViolation,
};

// Server for running simulations
pub use server::{ConnectionManager, RpcServer, ServerConfig, ServerState, Simulation};

// API client and traits
pub use api::{ConnectionState, RpcClient, WeatherParameter};

// Clock and time management
pub use common::{ClockType, FrameLimiter, SimulationClock, Timer, WallClock};

/// Prelude module for convenient imports
///
/// Use with: `use simkit::prelude::*;`
pub mod prelude {
    pub use crate::{
        Accelerations,
        Attitude,
        AttitudeSetpoint,
        BarometerOutput,
        BarometerSensor,
        Box2D,
        Box3D,
        CameraCaptureQueue,

        CameraConfig,
        CameraInfo,

        CameraSensor,
        CarVehicle,

        CascadeConfig,
        // Flight controllers
        CascadeController,
        ClockType,
        CollisionInfo,
        ComponentId,
        ConnectionState,
        ControlMode,
        ControllerGains,
        ControllerInput,
        ControllerOutput,

        DetectionInfo,
        DistanceSensor,
        DistanceSensorData,
        Environment,
        EnvironmentState,
       
        FastPhysicsEngine,
        FileRecordingWriter,
        FlightCommand,
        FlightController,
        FlightMode,
        FrameLimiter,

        GeoPoint,
        GlobalPositionInt,
        GpsOutput,
        GpsSensor,
        GroundTruth,
        Heartbeat,
        ImageRequest,
        ImageResponse,
        ImageType,
        ImuOutput,
        ImuSensor,
        InertiaTensor,
        // Physics
        KinematicsState,
        LidarData,
        LidarSensor,
        MagnetometerOutput,
        MagnetometerSensor,
        MavAutopilot,
        MavState,
        // MAVLink
        MavSystem,
        MavType,
        MavlinkVersion,
        MessageId,

        MotorOutputs,

        MultirotorVehicle,
        PhysicsBody,
        PhysicsEngine,
       
        PhysicsParams,
        PhysicsWorld,
        PidConfig,
        // Control systems
        PidController,
        // Common types
        Pose,
        PositionConfig,
        PositionController,
        PositionSetpoint,
        RateConfig,
        RateController,
        RateSetpoint,
        RcData,
        RecordedFrame,

        // Recording
        RecordingConfig,
        RecordingError,
        RecordingFormat,
        RecordingSession,
        RecordingState,
        RecordingStats,
        RecordingWriter,
        // Core types
        Result,
        RpcClient,
        // Sensors
        Sensor,
        SensorBase,
        SensorCollection,
        SensorConfig,
        SensorType,
        // API

        SimKitError,
        SimpleFlightConfig,
        SimpleFlightController,
        // Clock
        SimulationClock,
        SystemId,
        TimePoint,
        Timer,
        Twist,
        // Vehicles
        Vehicle,
        VehicleBase,
        VehicleConfig,
        VelocitySetpoint,
        WallClock,
        WeatherParameter,
        Wrench,

        DEFAULT_RPC_PORT,

        VERSION,
    };
}

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default RPC port for AirSim-compatible connections
pub const DEFAULT_RPC_PORT: u16 = 41451;

/// Result type alias for rust-sim-kit operations
pub type Result<T, E = SimKitError> = std::result::Result<T, E>;

/// Top-level error type for rust-sim-kit
#[derive(thiserror::Error, Debug)]
pub enum SimKitError {
    /// Connection error
    #[error("Connection error: {0}")]
    Connection(String),

    /// RPC error
    #[error("RPC error: {0}")]
    Rpc(String),

    /// Sensor error
    #[error("Sensor error: {0}")]
    Sensor(String),

    /// Vehicle error
    #[error("Vehicle error: {0}")]
    Vehicle(String),

    /// Physics error
    #[error("Physics error: {0}")]
    Physics(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Generic error from string
    #[error("Error: {0}")]
    Generic(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<&str> for SimKitError {
    fn from(s: &str) -> Self {
        SimKitError::Generic(s.to_string())
    }
}
