use glam::{Quat, Vec3};
use crate::mavlink::messages::core::*;
use crate::mavlink::*;
/// GPS raw int message (ID: 24)
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct GpsRawInt {
    /// Timestamp (us)
    pub time_usec: u64,
    /// Fix type
    pub fix_type: GpsFixType,
    /// Latitude (degE7)
    pub lat: i32,
    /// Longitude (degE7)
    pub lon: i32,
    /// Altitude (mm)
    pub alt: i32,
    /// HDOP (cm)
    pub eph: u16,
    /// VDOP (cm)
    pub epv: u16,
    /// Velocity (cm/s)
    pub vel: u16,
    /// Course over ground (degE2)
    pub cog: u16,
    /// Visible satellites
    pub satellites_visible: u8,
    /// Altitude above ellipsoid (mm)
    pub alt_ellipsoid: i32,
    /// Horizontal uncertainty (mm)
    pub h_acc: u32,
    /// Vertical uncertainty (mm)
    pub v_acc: u32,
    /// Velocity uncertainty (mm/s)
    pub vel_acc: u32,
    /// Heading uncertainty (degE5)
    pub hdg_acc: u32,
    /// Yaw (degE2)
    pub yaw: u16,
}

impl MavMessage for GpsRawInt {
    fn message_id(&self) -> MessageId {
        24
    }

    fn serialize(&self, _version: MavlinkVersion) -> Vec<u8> {
        let mut buf = Vec::with_capacity(52);
        buf.extend_from_slice(&self.time_usec.to_le_bytes());
        buf.extend_from_slice(&self.lat.to_le_bytes());
        buf.extend_from_slice(&self.lon.to_le_bytes());
        buf.extend_from_slice(&self.alt.to_le_bytes());
        buf.extend_from_slice(&self.eph.to_le_bytes());
        buf.extend_from_slice(&self.epv.to_le_bytes());
        buf.extend_from_slice(&self.vel.to_le_bytes());
        buf.extend_from_slice(&self.cog.to_le_bytes());
        buf.push(self.satellites_visible);
        buf.extend_from_slice(&self.alt_ellipsoid.to_le_bytes());
        buf.extend_from_slice(&self.h_acc.to_le_bytes());
        buf.extend_from_slice(&self.v_acc.to_le_bytes());
        buf.extend_from_slice(&self.vel_acc.to_le_bytes());
        buf.extend_from_slice(&self.hdg_acc.to_le_bytes());
        buf.extend_from_slice(&self.yaw.to_le_bytes());
        buf.push(self.fix_type as u8);
        buf
    }

    fn priority(&self) -> MessagePriority {
        MessagePriority::Normal
    }
}

/// Attitude message (ID: 30)
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct Attitude {
    /// Timestamp (ms)
    pub time_boot_ms: u32,
    /// Roll (rad)
    pub roll: f32,
    /// Pitch (rad)
    pub pitch: f32,
    /// Yaw (rad)
    pub yaw: f32,
    /// Roll rate (rad/s)
    pub rollspeed: f32,
    /// Pitch rate (rad/s)
    pub pitchspeed: f32,
    /// Yaw rate (rad/s)
    pub yawspeed: f32,
}

impl Attitude {
    /// Create from orientation quaternion
    pub fn from_quat(time_boot_ms: u32, orientation: Quat, angular_velocity: Vec3) -> Self {
        let (roll, pitch, yaw) = orientation.to_euler(glam::EulerRot::XYZ);
        Self {
            time_boot_ms,
            roll,
            pitch,
            yaw,
            rollspeed: angular_velocity.x,
            pitchspeed: angular_velocity.y,
            yawspeed: angular_velocity.z,
        }
    }
}

impl MavMessage for Attitude {
    fn message_id(&self) -> MessageId {
        30
    }

    fn serialize(&self, _version: MavlinkVersion) -> Vec<u8> {
        let mut buf = Vec::with_capacity(28);
        buf.extend_from_slice(&self.time_boot_ms.to_le_bytes());
        buf.extend_from_slice(&self.roll.to_le_bytes());
        buf.extend_from_slice(&self.pitch.to_le_bytes());
        buf.extend_from_slice(&self.yaw.to_le_bytes());
        buf.extend_from_slice(&self.rollspeed.to_le_bytes());
        buf.extend_from_slice(&self.pitchspeed.to_le_bytes());
        buf.extend_from_slice(&self.yawspeed.to_le_bytes());
        buf
    }

    fn priority(&self) -> MessagePriority {
        MessagePriority::Important
    }
}

/// Global position message (ID: 33)
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct GlobalPositionInt {
    /// Timestamp (ms)
    pub time_boot_ms: u32,
    /// Latitude (degE7)
    pub lat: i32,
    /// Longitude (degE7)
    pub lon: i32,
    /// Altitude (mm, AMSL)
    pub alt: i32,
    /// Relative altitude (mm, above home)
    pub relative_alt: i32,
    /// Velocity North (cm/s)
    pub vx: i16,
    /// Velocity East (cm/s)
    pub vy: i16,
    /// Velocity Down (cm/s)
    pub vz: i16,
    /// Heading (degE2)
    pub hdg: u16,
}

impl GlobalPositionInt {
    /// Create from position data
    pub fn from_position(
        time_boot_ms: u32,
        lat: f64,
        lon: f64,
        alt: f32,
        relative_alt: f32,
        velocity: Vec3,
        heading: f32,
    ) -> Self {
        Self {
            time_boot_ms,
            lat: (lat * 1e7) as i32,
            lon: (lon * 1e7) as i32,
            alt: (alt * 1000.0) as i32,
            relative_alt: (relative_alt * 1000.0) as i32,
            vx: (velocity.x * 100.0) as i16,
            vy: (velocity.y * 100.0) as i16,
            vz: (velocity.z * 100.0) as i16,
            hdg: (heading.to_degrees() * 100.0) as u16,
        }
    }
}

impl MavMessage for GlobalPositionInt {
    fn message_id(&self) -> MessageId {
        33
    }

    fn serialize(&self, _version: MavlinkVersion) -> Vec<u8> {
        let mut buf = Vec::with_capacity(28);
        buf.extend_from_slice(&self.time_boot_ms.to_le_bytes());
        buf.extend_from_slice(&self.lat.to_le_bytes());
        buf.extend_from_slice(&self.lon.to_le_bytes());
        buf.extend_from_slice(&self.alt.to_le_bytes());
        buf.extend_from_slice(&self.relative_alt.to_le_bytes());
        buf.extend_from_slice(&self.vx.to_le_bytes());
        buf.extend_from_slice(&self.vy.to_le_bytes());
        buf.extend_from_slice(&self.vz.to_le_bytes());
        buf.extend_from_slice(&self.hdg.to_le_bytes());
        buf
    }

    fn priority(&self) -> MessagePriority {
        MessagePriority::Important
    }
}

