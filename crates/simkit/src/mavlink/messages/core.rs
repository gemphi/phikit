//! MAVLink message definitions for rust-sim-kit
//!
//! Simplified MAVLink message structures for simulation.

use crate::mavlink::*;


/// MAVLink message trait
pub trait MavMessage: Send + Sync {
    /// Get message ID
    fn message_id(&self) -> MessageId;

    /// Serialize to bytes
    fn serialize(&self, version: MavlinkVersion) -> Vec<u8>;

    /// Get message priority
    fn priority(&self) -> MessagePriority;
}

/// Heartbeat message (ID: 0)
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct Heartbeat {
    /// Type of the MAV
    pub mav_type: MavType,
    /// Autopilot type
    pub autopilot: MavAutopilot,
    /// Base mode
    pub base_mode: u8,
    /// Custom mode
    pub custom_mode: u32,
    /// System status
    pub system_status: MavState,
    /// MAVLink version
    pub mavlink_version: u8,
}

impl Heartbeat {
    /// Create new heartbeat
    pub fn new(system: &MavSystem) -> Self {
        Self {
            mav_type: system.vehicle_type,
            autopilot: system.autopilot,
            base_mode: system.base_mode as u8,
            custom_mode: system.custom_mode,
            system_status: system.state,
            mavlink_version: system.version as u8,
        }
    }

    /// Set armed bit in base_mode
    pub fn set_armed(&mut self, armed: bool) {
        if armed {
            self.base_mode |= 0x80; // MAV_MODE_FLAG_SAFETY_ARMED
        } else {
            self.base_mode &= !0x80;
        }
    }

    /// Check if armed
    pub fn is_armed(&self) -> bool {
        (self.base_mode & 0x80) != 0
    }
}

impl MavMessage for Heartbeat {
    fn message_id(&self) -> MessageId {
        0
    }

    fn serialize(&self, _version: MavlinkVersion) -> Vec<u8> {
        // Simplified serialization
        let mut buf = Vec::with_capacity(9);
        buf.push(self.custom_mode as u8);
        buf.push((self.custom_mode >> 8) as u8);
        buf.push((self.custom_mode >> 16) as u8);
        buf.push((self.custom_mode >> 24) as u8);
        buf.push(self.base_mode);
        buf.push(self.mav_type as u8);
        buf.push(self.autopilot as u8);
        buf.push(self.system_status as u8);
        buf.push(self.mavlink_version);
        buf
    }

    fn priority(&self) -> MessagePriority {
        MessagePriority::Critical
    }
}

/// System status message (ID: 1)
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct SysStatus {
    /// Sensors present
    pub onboard_control_sensors_present: u32,
    /// Sensors enabled
    pub onboard_control_sensors_enabled: u32,
    /// Sensors health
    pub onboard_control_sensors_health: u32,
    /// CPU load (%)
    pub load: u16,
    /// Battery voltage (mV)
    pub voltage_battery: u16,
    /// Battery current (cA)
    pub current_battery: i16,
    /// Remaining battery (%)
    pub battery_remaining: i8,
    /// Communication drop rate (%)
    pub drop_rate_comm: u16,
    /// Communication errors
    pub errors_comm: u16,
    /// Autopilot-specific errors
    pub errors_count1: u16,
    /// Errors count 2
    pub errors_count2: u16,
    /// Errors count 3
    pub errors_count3: u16,
    /// Errors count 4
    pub errors_count4: u16,
}

impl MavMessage for SysStatus {
    fn message_id(&self) -> MessageId {
        1
    }

    fn serialize(&self, _version: MavlinkVersion) -> Vec<u8> {
        let mut buf = Vec::with_capacity(31);
        buf.extend_from_slice(&self.onboard_control_sensors_present.to_le_bytes());
        buf.extend_from_slice(&self.onboard_control_sensors_enabled.to_le_bytes());
        buf.extend_from_slice(&self.onboard_control_sensors_health.to_le_bytes());
        buf.extend_from_slice(&self.load.to_le_bytes());
        buf.extend_from_slice(&self.voltage_battery.to_le_bytes());
        buf.extend_from_slice(&self.current_battery.to_le_bytes());
        buf.push(self.battery_remaining as u8);
        buf.extend_from_slice(&self.drop_rate_comm.to_le_bytes());
        buf.extend_from_slice(&self.errors_comm.to_le_bytes());
        buf.extend_from_slice(&self.errors_count1.to_le_bytes());
        buf.extend_from_slice(&self.errors_count2.to_le_bytes());
        buf.extend_from_slice(&self.errors_count3.to_le_bytes());
        buf.extend_from_slice(&self.errors_count4.to_le_bytes());
        buf
    }

    fn priority(&self) -> MessagePriority {
        MessagePriority::Important
    }
}

