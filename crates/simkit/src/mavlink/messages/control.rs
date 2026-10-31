use crate::mavlink::messages::core::*;
use crate::mavlink::*;
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
/// RC Channels Scaled Message
pub struct RcChannelsScaled {
    /// Timestamp (ms)
    pub time_boot_ms: u32,
    /// Port/servo index
    pub port: u8,
    /// Channel values (-10000 to 10000)
    pub chan1_scaled: i16,
    /// Channel 2
    pub chan2_scaled: i16,
    /// Channel 3
    pub chan3_scaled: i16,
    /// Channel 4
    pub chan4_scaled: i16,
    /// Channel 5
    pub chan5_scaled: i16,
    /// Channel 6
    pub chan6_scaled: i16,
    /// Channel 7
    pub chan7_scaled: i16,
    /// Channel 8
    pub chan8_scaled: i16,
    /// Receive signal strength indicator (0-255)
    pub rssi: u8,
}

impl MavMessage for RcChannelsScaled {
    fn message_id(&self) -> MessageId {
        34
    }

    fn serialize(&self, _version: MavlinkVersion) -> Vec<u8> {
        let mut buf = Vec::with_capacity(22);
        buf.extend_from_slice(&self.time_boot_ms.to_le_bytes());
        buf.extend_from_slice(&self.chan1_scaled.to_le_bytes());
        buf.extend_from_slice(&self.chan2_scaled.to_le_bytes());
        buf.extend_from_slice(&self.chan3_scaled.to_le_bytes());
        buf.extend_from_slice(&self.chan4_scaled.to_le_bytes());
        buf.extend_from_slice(&self.chan5_scaled.to_le_bytes());
        buf.extend_from_slice(&self.chan6_scaled.to_le_bytes());
        buf.extend_from_slice(&self.chan7_scaled.to_le_bytes());
        buf.extend_from_slice(&self.chan8_scaled.to_le_bytes());
        buf.push(self.port);
        buf.push(self.rssi);
        buf
    }

    fn priority(&self) -> MessagePriority {
        MessagePriority::Normal
    }
}

/// Servo output raw message (ID: 36)
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct ServoOutputRaw {
    /// Timestamp (us)
    pub time_usec: u32,
    /// Port
    pub port: u8,
    /// Servo values (0-10000)
    pub servo1_raw: u16,
    /// Servo 2
    pub servo2_raw: u16,
    /// Servo 3
    pub servo3_raw: u16,
    /// Servo 4
    pub servo4_raw: u16,
    /// Servo 5
    pub servo5_raw: u16,
    /// Servo 6
    pub servo6_raw: u16,
    /// Servo 7
    pub servo7_raw: u16,
    /// Servo 8
    pub servo8_raw: u16,
    /// Servo 9
    pub servo9_raw: u16,
    /// Servo 10
    pub servo10_raw: u16,
    /// Servo 11
    pub servo11_raw: u16,
    /// Servo 12
    pub servo12_raw: u16,
    /// Servo 13
    pub servo13_raw: u16,
    /// Servo 14
    pub servo14_raw: u16,
    /// Servo 15
    pub servo15_raw: u16,
    /// Servo 16
    pub servo16_raw: u16,
}

impl MavMessage for ServoOutputRaw {
    fn message_id(&self) -> MessageId {
        36
    }

    fn serialize(&self, _version: MavlinkVersion) -> Vec<u8> {
        let mut buf = Vec::with_capacity(37);
        buf.extend_from_slice(&self.time_usec.to_le_bytes());
        buf.extend_from_slice(&self.servo1_raw.to_le_bytes());
        buf.extend_from_slice(&self.servo2_raw.to_le_bytes());
        buf.extend_from_slice(&self.servo3_raw.to_le_bytes());
        buf.extend_from_slice(&self.servo4_raw.to_le_bytes());
        buf.extend_from_slice(&self.servo5_raw.to_le_bytes());
        buf.extend_from_slice(&self.servo6_raw.to_le_bytes());
        buf.extend_from_slice(&self.servo7_raw.to_le_bytes());
        buf.extend_from_slice(&self.servo8_raw.to_le_bytes());
        buf.push(self.port);
        buf
    }

    fn priority(&self) -> MessagePriority {
        MessagePriority::Normal
    }
}

