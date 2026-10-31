use crate::mavlink::messages::core::*;
use crate::mavlink::messages::navigation::*;
use crate::mavlink::messages::control::*;
use crate::mavlink::*;

/// Extended sys state message (ID: 245)
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct ExtendedSysState {
    /// VTOL state
    pub vtol_state: MavVtolState,
    /// Landed state
    pub landed_state: MavLandedState,
}

impl MavMessage for ExtendedSysState {
    fn message_id(&self) -> MessageId {
        245
    }

    fn serialize(&self, _version: MavlinkVersion) -> Vec<u8> {
        vec![self.vtol_state as u8, self.landed_state as u8]
    }

    fn priority(&self) -> MessagePriority {
        MessagePriority::Important
    }
}

/// Battery status message (ID: 147)
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct BatteryStatus {
    /// Battery ID
    pub id: u8,
    /// Battery function
    pub battery_function: MavBatteryFunction,
    /// Battery type
    pub battery_type: MavBatteryType,
    /// Temperature (degC, INT16_MAX if unknown)
    pub temperature: i16,
    /// Voltages (mV, UINT16_MAX if not populated)
    pub voltages: [u16; 10],
    /// Current battery (cA, -1 if unknown)
    pub current_battery: i16,
    /// Consumed charge (mAh, -1 if unknown)
    pub current_consumed: i32,
    /// Energy consumed (hJ, -1 if unknown)
    pub energy_consumed: i32,
    /// Remaining battery (%)
    pub battery_remaining: i8,
}

impl MavMessage for BatteryStatus {
    fn message_id(&self) -> MessageId {
        147
    }

    fn serialize(&self, _version: MavlinkVersion) -> Vec<u8> {
        let mut buf = Vec::with_capacity(41);
        buf.push(self.id);
        buf.push(self.battery_function as u8);
        buf.push(self.battery_type as u8);
        buf.extend_from_slice(&self.temperature.to_le_bytes());
        for v in &self.voltages {
            buf.extend_from_slice(&v.to_le_bytes());
        }
        buf.extend_from_slice(&self.current_battery.to_le_bytes());
        buf.extend_from_slice(&self.current_consumed.to_le_bytes());
        buf.extend_from_slice(&self.energy_consumed.to_le_bytes());
        buf.push(self.battery_remaining as u8);
        buf
    }

    fn priority(&self) -> MessagePriority {
        MessagePriority::Important
    }
}

/// Enum for all message types
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Message {
    /// Heartbeat
    Heartbeat(Heartbeat),
    /// System status
    SysStatus(SysStatus),
    /// GPS raw
    GpsRawInt(GpsRawInt),
    /// Attitude
    Attitude(Attitude),
    /// Global position
    GlobalPositionInt(GlobalPositionInt),
    /// RC channels
    RcChannelsScaled(RcChannelsScaled),
    /// Servo output
    ServoOutputRaw(ServoOutputRaw),
    /// Extended system state
    ExtendedSysState(ExtendedSysState),
    /// Battery status
    BatteryStatus(BatteryStatus),
}

impl Message {
    /// Get message ID
    pub fn message_id(&self) -> MessageId {
        match self {
            Message::Heartbeat(_) => 0,
            Message::SysStatus(_) => 1,
            Message::GpsRawInt(_) => 24,
            Message::Attitude(_) => 30,
            Message::GlobalPositionInt(_) => 33,
            Message::RcChannelsScaled(_) => 34,
            Message::ServoOutputRaw(_) => 36,
            Message::ExtendedSysState(_) => 245,
            Message::BatteryStatus(_) => 147,
        }
    }

    /// Serialize to bytes
    pub fn serialize(&self, version: MavlinkVersion) -> Vec<u8> {
        match self {
            Message::Heartbeat(m) => m.serialize(version),
            Message::SysStatus(m) => m.serialize(version),
            Message::GpsRawInt(m) => m.serialize(version),
            Message::Attitude(m) => m.serialize(version),
            Message::GlobalPositionInt(m) => m.serialize(version),
            Message::RcChannelsScaled(m) => m.serialize(version),
            Message::ServoOutputRaw(m) => m.serialize(version),
            Message::ExtendedSysState(m) => m.serialize(version),
            Message::BatteryStatus(m) => m.serialize(version),
        }
    }

    /// Get priority
    pub fn priority(&self) -> MessagePriority {
        match self {
            Message::Heartbeat(_) => MessagePriority::Critical,
            Message::SysStatus(_) => MessagePriority::Important,
            Message::GpsRawInt(_) => MessagePriority::Normal,
            Message::Attitude(_) => MessagePriority::Important,
            Message::GlobalPositionInt(_) => MessagePriority::Important,
            Message::RcChannelsScaled(_) => MessagePriority::Normal,
            Message::ServoOutputRaw(_) => MessagePriority::Normal,
            Message::ExtendedSysState(_) => MessagePriority::Important,
            Message::BatteryStatus(_) => MessagePriority::Important,
        }
    }
}
