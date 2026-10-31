//! MAVLink protocol support for rust-sim-kit
//!
//! Provides MAVLink message types and communication support for integration
//! with PX4, ArduPilot, and other autopilot systems.
//!
//! This is a simplified implementation for simulation purposes.

pub mod connection;
pub mod messages;

pub use connection::{ConnectionStats, ConnectionType, MavlinkConnection};
pub use messages::*;

/// MAVLink system ID
pub type SystemId = u8;

/// MAVLink component ID
pub type ComponentId = u8;

/// MAVLink message ID
pub type MessageId = u32;

/// MAVLink message sequence number
pub type Sequence = u8;

/// MAVLink protocol version
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum MavlinkVersion {
    /// MAVLink 1.0
    V1 = 1,
    /// MAVLink 2.0 (default)
    #[default]
    V2 = 2,
}

impl MavlinkVersion {
    /// Get magic number for version
    pub fn magic(&self) -> u8 {
        match self {
            MavlinkVersion::V1 => 0xFE,
            MavlinkVersion::V2 => 0xFD,
        }
    }

    /// Parse from value
    pub fn from_value(value: u8) -> Option<Self> {
        match value {
            1 => Some(MavlinkVersion::V1),
            2 => Some(MavlinkVersion::V2),
            _ => None,
        }
    }
}

/// MAVLink message priority
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum MessagePriority {
    /// Critical (highest)
    Critical = 0,
    /// Important
    Important = 1,
    /// Normal
    Normal = 2,
    /// Low
    Low = 3,
    /// Debug (lowest)
    Debug = 4,
}

impl Default for MessagePriority {
    fn default() -> Self {
        MessagePriority::Normal
    }
}

/// MAVLink system status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum MavState {
    /// Uninitialized
    #[default]
    Uninit = 0,
    /// Booting up
    Boot = 1,
    /// Calibrating sensors
    Calibrating = 2,
    /// Active and operational
    Active = 3,
    /// Standby mode
    Standby = 4,
    /// Critical state (may not be controllable)
    Critical = 5,
    /// Emergency (may be crashing)
    Emergency = 6,
    /// Powered off
    Poweroff = 7,
    /// Flight termination (intentional crash)
    FlightTermination = 8,
}

/// MAVLink autopilot types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum MavAutopilot {
    /// Generic autopilot (placeholder)
    #[default]
    Generic = 0,
    /// Reserved for SLUGS
    Reserved = 1,
    /// ArduPilot (ArduPlane, ArduCopter, etc.)
    Ardupilotmega = 3,
    /// OpenPilot
    Openpilot = 4,
    /// Generic autopilot with waypoint support
    GenericWaypoints = 5,
    /// Generic autopilot with waypoint and simple navigation
    GenericWaypointsAndSimpleNavigation = 6,
    /// Generic autopilot with waypoint and full navigation
    GenericWaypointsAndFullNavigation = 7,
    /// No autopilot
    NoAutopilot = 8,
    /// PPZ UAV
    Ppz = 9,
    /// UAV Dev Board
    Udb = 10,
    /// FlexiPilot
    Flexipilot = 11,
    /// PX4
    Px4 = 12,
    /// SMACCMPilot
    Smaccmpilot = 13,
    /// AutoQuad
    Autoquad = 14,
    /// Armazila
    Armazila = 15,
    /// Aerob
    Aerob = 16,
    /// ASLUAV autopilot
    Asluav = 17,
    /// SmartAP Autopilot
    Smartap = 18,
    /// AirSim (simulated autopilot)
    Airsim = 19,
    /// TDRAGONS
    Tdragon = 20,
    /// Vigilant Aerospace Systems
    Vigilant = 21,
    /// Aerobotics aircraft
    Aion = 22,
}

/// MAVLink vehicle types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum MavType {
    /// Generic micro air vehicle
    #[default]
    Generic = 0,
    /// Fixed wing aircraft
    FixedWing = 1,
    /// Quadrotor
    Quadrotor = 2,
    /// Coaxial helicopter
    Coaxial = 3,
    /// Normal helicopter with tail rotor
    Helicopter = 4,
    /// Ground vehicle (rover/car)
    GroundRover = 10,
    /// Boat
    SurfaceBoat = 11,
    /// Submarine
    Submarine = 12,
    /// Hexarotor
    Hexarotor = 13,
    /// Octorotor
    Octorotor = 14,
    /// Tricopter
    Tricopter = 15,
    /// Flapping wing
    FlappingWing = 16,
    /// Kite
    Kite = 17,
    /// Onboard computer companion
    OnboardController = 18,
    /// VTOL with differential thrust
    VtolDuoRotor = 19,
    /// VTOL with tiltrotor
    VtolTiltrotor = 20,
    /// VTOL with tailsitter
    VtolTailsitter = 21,
    /// VTOL with tiltwing
    VtolTiltwing = 22,
    /// VTOL with standard airplane configuration
    VtolFixedrotor = 23,
    /// Generic VTOL
    Vtol = 24,
    /// Generic multirotor
    GenericMultirotor = 25,
}

/// MAVLink flight modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum MavMode {
    /// Preflight
    #[default]
    Preflight = 0,
    /// Manual control
    Manual = 64,
    /// Test mode
    Test = 80,
    /// Assisted control
    Assisted = 96,
    /// Auto mode (mission)
    Auto = 112,
    /// Guided mode (position control)
    Guided = 120,
    /// Stabilize mode
    Stabilize = 208,
    /// Hybrid mode
    Hybrid = 216,
}

/// MAVLink armed state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct MavArmedState {
    /// System is armed
    pub armed: bool,
    /// System is ready to arm
    pub prearmed: bool,
}

/// MAVLink extended state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct MavExtendedState {
    /// VTOL state
    pub vtol_state: MavVtolState,
    /// Landed state
    pub landed_state: MavLandedState,
}

/// VTOL state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum MavVtolState {
    /// Undefined
    #[default]
    Undefined = 0,
    /// Transition to fixed wing
    TransitionToFw = 1,
    /// Transition to multirotor
    TransitionToMc = 2,
    /// Fixed wing mode
    Mc = 3,
    /// Multirotor mode
    Fw = 4,
}

/// Landed state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum MavLandedState {
    /// Undefined
    #[default]
    Undefined = 0,
    /// On ground
    OnGround = 1,
    /// In air
    InAir = 2,
    /// Taking off
    Takeoff = 3,
    /// Landing
    Landing = 4,
}

/// MAVLink battery status
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MavBatteryStatus {
    /// Battery ID
    pub id: u8,
    /// Battery function
    pub battery_function: MavBatteryFunction,
    /// Battery type
    pub battery_type: MavBatteryType,
    /// Temperature (degC)
    pub temperature: i16,
    /// Voltages (mV)
    pub voltages: [u16; 10],
    /// Current battery (cA)
    pub current_battery: i16,
    /// Consumed charge (mAh)
    pub current_consumed: i32,
    /// Energy consumed (hJ)
    pub energy_consumed: i32,
    /// Battery remaining (%)
    pub battery_remaining: i8,
}

impl Default for MavBatteryStatus {
    fn default() -> Self {
        Self {
            id: 0,
            battery_function: MavBatteryFunction::Unknown,
            battery_type: MavBatteryType::Unknown,
            temperature: i16::MAX,
            voltages: [u16::MAX; 10],
            current_battery: -1,
            current_consumed: -1,
            energy_consumed: -1,
            battery_remaining: -1,
        }
    }
}

/// Battery function
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum MavBatteryFunction {
    /// Unknown
    #[default]
    Unknown = 0,
    /// All batteries
    All = 1,
    /// Propulsion
    Propulsion = 2,
    /// Avionics
    Avionics = 3,
    /// Payload
    Payload = 4,
}

/// Battery type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum MavBatteryType {
    /// Unknown
    #[default]
    Unknown = 0,
    /// LiPo
    Lipo = 1,
    /// LiFe
    Life = 2,
    /// LiIon
    Liion = 3,
    /// NiMH
    Nimh = 4,
}

/// GPS fix type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum GpsFixType {
    /// No GPS
    #[default]
    NoGps = 0,
    /// No fix
    NoFix = 1,
    /// 2D fix
    Fix2D = 2,
    /// 3D fix
    Fix3D = 3,
    /// DGPS
    Dgps = 4,
    /// RTK float
    RtkFloat = 5,
    /// RTK fixed
    RtkFixed = 6,
    /// Fixed (static)
    Fixed = 7,
}

/// MAVLink command result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum MavResult {
    /// Command accepted
    #[default]
    Accepted = 0,
    /// Command temporarily rejected
    TemporarilyRejected = 1,
    /// Command denied
    Denied = 2,
    /// Unsupported
    Unsupported = 3,
    /// Failed
    Failed = 4,
    /// In progress
    InProgress = 5,
    /// Command cancelled
    Cancelled = 6,
    /// Command retry
    CommandRetry = 7,
}

/// MAVLink system information
#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct MavSystem {
    /// System ID
    pub system_id: SystemId,
    /// Component ID
    pub component_id: ComponentId,
    /// Autopilot type
    pub autopilot: MavAutopilot,
    /// Vehicle type
    pub vehicle_type: MavType,
    /// System status
    pub state: MavState,
    /// MAVLink version
    pub version: MavlinkVersion,
    /// Base mode
    pub base_mode: MavMode,
    /// Custom mode (autopilot specific)
    pub custom_mode: u32,
}

impl MavSystem {
    /// Create new system
    pub fn new(system_id: SystemId, component_id: ComponentId) -> Self {
        Self {
            system_id,
            component_id,
            autopilot: MavAutopilot::Airsim,
            vehicle_type: MavType::Quadrotor,
            state: MavState::Active,
            version: MavlinkVersion::V2,
            base_mode: MavMode::Guided,
            custom_mode: 0,
        }
    }

    /// Create for PX4
    pub fn px4(system_id: SystemId, component_id: ComponentId) -> Self {
        Self {
            system_id,
            component_id,
            autopilot: MavAutopilot::Px4,
            vehicle_type: MavType::Quadrotor,
            state: MavState::Active,
            version: MavlinkVersion::V2,
            base_mode: MavMode::Guided,
            custom_mode: 0,
        }
    }

    /// Create for ArduPilot
    pub fn ardupilot(system_id: SystemId, component_id: ComponentId) -> Self {
        Self {
            system_id,
            component_id,
            autopilot: MavAutopilot::Ardupilotmega,
            vehicle_type: MavType::Quadrotor,
            state: MavState::Active,
            version: MavlinkVersion::V2,
            base_mode: MavMode::Stabilize,
            custom_mode: 0,
        }
    }
}

/// MAVLink heartbeat data
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct HeartbeatData {
    /// Autopilot type
    pub autopilot: MavAutopilot,
    /// Vehicle type
    pub vehicle_type: MavType,
    /// Base mode
    pub base_mode: MavMode,
    /// Custom mode
    pub custom_mode: u32,
    /// System status
    pub system_status: MavState,
    /// MAVLink version
    pub mavlink_version: u8,
}

/// Helper functions for MAVLink
pub mod utils {

    /// Convert degrees to centidegrees (MAVLink uses centidegrees for angles)
    pub fn deg_to_cd(deg: f32) -> i16 {
        (deg * 100.0) as i16
    }

    /// Convert centidegrees to degrees
    pub fn cd_to_deg(cd: i16) -> f32 {
        cd as f32 / 100.0
    }

    /// Convert meters to centimeters (MAVLink uses cm for distances)
    pub fn m_to_cm(m: f32) -> i32 {
        (m * 100.0) as i32
    }

    /// Convert centimeters to meters
    pub fn cm_to_m(cm: i32) -> f32 {
        cm as f32 / 100.0
    }

    /// Convert m/s to cm/s
    pub fn ms_to_cms(ms: f32) -> i16 {
        (ms * 100.0) as i16
    }

    /// Convert cm/s to m/s
    pub fn cms_to_ms(cms: i16) -> f32 {
        cms as f32 / 100.0
    }

    /// Compute MAVLink CRC (simplified)
    pub fn compute_crc(data: &[u8]) -> u16 {
        let mut crc: u16 = 0xFFFF;
        for byte in data {
            crc ^= (*byte as u16) << 8;
            for _ in 0..8 {
                if crc & 0x8000 != 0 {
                    crc = (crc << 1) ^ 0x1021;
                } else {
                    crc <<= 1;
                }
            }
        }
        crc & 0xFFFF
    }
}
