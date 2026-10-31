//! MAVLink connection handling for rust-sim-kit
//!
//! Provides connection management for MAVLink communication over serial, UDP, TCP.

use super::*;
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream, UdpSocket};
use std::time::{Duration, Instant};

/// Connection type
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ConnectionType {
    /// Serial port connection
    Serial,
    /// UDP connection
    Udp,
    /// TCP connection
    Tcp,
}

impl Default for ConnectionType {
    fn default() -> Self {
        ConnectionType::Udp
    }
}

/// MAVLink connection trait
pub trait Connection: Send + Sync {
    /// Read a message from the connection
    fn recv(&mut self) -> io::Result<Vec<u8>>;

    /// Write a message to the connection
    fn send(&mut self, data: &[u8]) -> io::Result<()>;

    /// Check if connection is active
    fn is_connected(&self) -> bool;

    /// Close the connection
    fn close(&mut self);

    /// Get connection type
    fn connection_type(&self) -> ConnectionType;
}

/// MAVLink connection wrapper
pub struct MavlinkConnection {
    /// Connection type
    connection_type: ConnectionType,
    /// System info
    system: MavSystem,
    /// Sequence counter
    sequence: Sequence,
    /// Last heartbeat time
    last_heartbeat: Instant,
    /// Heartbeat interval
    heartbeat_interval: Duration,
    /// Is connected
    connected: bool,
    /// Pending outgoing messages
    outgoing_queue: Vec<Message>,
    /// Message statistics
    stats: ConnectionStats,
}

/// Connection statistics
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct ConnectionStats {
    /// Messages sent
    pub messages_sent: u64,
    /// Messages received
    pub messages_received: u64,
    /// Bytes sent
    pub bytes_sent: u64,
    /// Bytes received
    pub bytes_received: u64,
    /// Errors
    pub errors: u64,
    /// Last activity time (seconds since start)
    pub last_activity: Option<f64>,
}

impl MavlinkConnection {
    /// Create new MAVLink connection
    pub fn new(connection_type: ConnectionType, system: MavSystem) -> Self {
        Self {
            connection_type,
            system,
            sequence: 0,
            last_heartbeat: Instant::now(),
            heartbeat_interval: Duration::from_secs(1),
            connected: false,
            outgoing_queue: Vec::new(),
            stats: ConnectionStats::default(),
        }
    }

    /// Create UDP connection
    pub fn udp(system: MavSystem, _bind_addr: &str) -> Self {
        Self::new(ConnectionType::Udp, system)
    }

    /// Create TCP connection
    pub fn tcp(system: MavSystem, _addr: &str) -> Self {
        Self::new(ConnectionType::Tcp, system)
    }

    /// Get system info
    pub fn system(&self) -> &MavSystem {
        &self.system
    }

    /// Get mutable system info
    pub fn system_mut(&mut self) -> &mut MavSystem {
        &mut self.system
    }

    /// Get next sequence number
    pub fn next_sequence(&mut self) -> Sequence {
        let seq = self.sequence;
        self.sequence = self.sequence.wrapping_add(1);
        seq
    }

    /// Queue a message for sending
    pub fn queue_message(&mut self, message: Message) {
        self.outgoing_queue.push(message);
    }

    /// Create and queue a heartbeat message
    pub fn send_heartbeat(&mut self) {
        let heartbeat = Heartbeat::new(&self.system);
        self.queue_message(Message::Heartbeat(heartbeat));
    }

    /// Check if heartbeat should be sent
    pub fn should_send_heartbeat(&self) -> bool {
        self.last_heartbeat.elapsed() >= self.heartbeat_interval
    }

    /// Update heartbeat timer
    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = Instant::now();
    }

    /// Set heartbeat interval
    pub fn set_heartbeat_interval(&mut self, interval: Duration) {
        self.heartbeat_interval = interval;
    }

    /// Get connection type
    pub fn connection_type(&self) -> ConnectionType {
        self.connection_type
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Set connected state
    pub fn set_connected(&mut self, connected: bool) {
        self.connected = connected;
    }

    /// Get statistics
    pub fn stats(&self) -> &ConnectionStats {
        &self.stats
    }

    /// Clear statistics
    pub fn clear_stats(&mut self) {
        self.stats = ConnectionStats::default();
    }

    /// Update stats for sent message
    pub fn record_sent(&mut self, bytes: usize) {
        self.stats.messages_sent += 1;
        self.stats.bytes_sent += bytes as u64;
        self.stats.last_activity = Some(0.0); // Simplified
    }

    /// Update stats for received message
    pub fn record_received(&mut self, bytes: usize) {
        self.stats.messages_received += 1;
        self.stats.bytes_received += bytes as u64;
        self.stats.last_activity = Some(0.0); // Simplified
    }

    /// Record error
    pub fn record_error(&mut self) {
        self.stats.errors += 1;
    }

    /// Get outgoing queue
    pub fn outgoing_queue(&mut self) -> &mut Vec<Message> {
        &mut self.outgoing_queue
    }

    /// Clear outgoing queue
    pub fn clear_outgoing(&mut self) {
        self.outgoing_queue.clear();
    }
}

/// UDP MAVLink connection
pub struct UdpConnection {
    /// UDP socket
    socket: Option<UdpSocket>,
    /// Remote address
    remote_addr: Option<SocketAddr>,
    /// Connection state
    state: ConnectionState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

impl UdpConnection {
    /// Create new UDP connection
    pub fn new(bind_addr: &str) -> io::Result<Self> {
        let socket = UdpSocket::bind(bind_addr)?;
        socket.set_nonblocking(true)?;

        Ok(Self {
            socket: Some(socket),
            remote_addr: None,
            state: ConnectionState::Connected,
        })
    }

    /// Connect to remote address
    pub fn connect(&mut self, addr: SocketAddr) -> io::Result<()> {
        if let Some(socket) = &self.socket {
            socket.connect(addr)?;
            self.remote_addr = Some(addr);
            self.state = ConnectionState::Connected;
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Socket not available",
            ))
        }
    }
}

impl Connection for UdpConnection {
    fn recv(&mut self) -> io::Result<Vec<u8>> {
        if let Some(socket) = &self.socket {
            let mut buf = vec![0u8; 1024];
            match socket.recv(&mut buf) {
                Ok(len) => {
                    buf.truncate(len);
                    Ok(buf)
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => Ok(Vec::new()),
                Err(e) => Err(e),
            }
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Socket not available",
            ))
        }
    }

    fn send(&mut self, data: &[u8]) -> io::Result<()> {
        if let Some(socket) = &self.socket {
            if let Some(addr) = self.remote_addr {
                socket.send_to(data, addr)?;
            } else {
                socket.send(data)?;
            }
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Socket not available",
            ))
        }
    }

    fn is_connected(&self) -> bool {
        self.state == ConnectionState::Connected && self.socket.is_some()
    }

    fn close(&mut self) {
        self.socket = None;
        self.state = ConnectionState::Disconnected;
    }

    fn connection_type(&self) -> ConnectionType {
        ConnectionType::Udp
    }
}

/// TCP MAVLink connection
pub struct TcpConnection {
    /// TCP stream
    stream: Option<TcpStream>,
    /// Connection state
    state: ConnectionState,
}

impl TcpConnection {
    /// Create new TCP connection
    pub fn new() -> Self {
        Self {
            stream: None,
            state: ConnectionState::Disconnected,
        }
    }

    /// Connect to server
    pub fn connect(&mut self, addr: &str) -> io::Result<()> {
        let stream = TcpStream::connect(addr)?;
        stream.set_nonblocking(true)?;
        self.stream = Some(stream);
        self.state = ConnectionState::Connected;
        Ok(())
    }
}

impl Connection for TcpConnection {
    fn recv(&mut self) -> io::Result<Vec<u8>> {
        if let Some(stream) = &mut self.stream {
            let mut buf = vec![0u8; 1024];
            match stream.read(&mut buf) {
                Ok(len) => {
                    if len == 0 {
                        self.state = ConnectionState::Disconnected;
                        return Err(io::Error::new(
                            io::ErrorKind::ConnectionReset,
                            "Connection closed",
                        ));
                    }
                    buf.truncate(len);
                    Ok(buf)
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => Ok(Vec::new()),
                Err(e) => {
                    self.state = ConnectionState::Error;
                    Err(e)
                }
            }
        } else {
            Err(io::Error::new(io::ErrorKind::NotConnected, "Not connected"))
        }
    }

    fn send(&mut self, data: &[u8]) -> io::Result<()> {
        if let Some(stream) = &mut self.stream {
            stream.write_all(data)?;
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::NotConnected, "Not connected"))
        }
    }

    fn is_connected(&self) -> bool {
        self.state == ConnectionState::Connected && self.stream.is_some()
    }

    fn close(&mut self) {
        self.stream = None;
        self.state = ConnectionState::Disconnected;
    }

    fn connection_type(&self) -> ConnectionType {
        ConnectionType::Tcp
    }
}

/// Serial MAVLink connection (stub - requires serial port implementation)
pub struct SerialConnection {
    /// Port name
    port_name: String,
    /// Baud rate
    baud_rate: u32,
    /// Connected state
    connected: bool,
}

impl SerialConnection {
    /// Create new serial connection
    pub fn new(port_name: &str, baud_rate: u32) -> Self {
        Self {
            port_name: port_name.to_string(),
            baud_rate,
            connected: false,
        }
    }
}

impl Connection for SerialConnection {
    fn recv(&mut self) -> io::Result<Vec<u8>> {
        // Stub: would use serialport crate
        Ok(Vec::new())
    }

    fn send(&mut self, _data: &[u8]) -> io::Result<()> {
        // Stub: would use serialport crate
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn close(&mut self) {
        self.connected = false;
    }

    fn connection_type(&self) -> ConnectionType {
        ConnectionType::Serial
    }
}

/// MAVLink packet structure
#[derive(Debug, Clone, PartialEq)]
pub struct MavPacket {
    /// Magic number (0xFE for v1, 0xFD for v2)
    pub magic: u8,
    /// Payload length
    pub len: u8,
    /// Sequence number
    pub seq: u8,
    /// System ID
    pub sysid: SystemId,
    /// Component ID
    pub compid: ComponentId,
    /// Message ID
    pub msgid: MessageId,
    /// Payload data
    pub payload: Vec<u8>,
    /// Checksum
    pub checksum: u16,
}

impl MavPacket {
    /// Create new packet
    pub fn new(
        seq: u8,
        sysid: SystemId,
        compid: ComponentId,
        msgid: MessageId,
        payload: Vec<u8>,
    ) -> Self {
        let len = payload.len() as u8;
        Self {
            magic: 0xFE, // MAVLink v1
            len,
            seq,
            sysid,
            compid,
            msgid,
            payload,
            checksum: 0,
        }
    }

    /// Serialize to bytes
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(8 + self.payload.len() + 2);
        buf.push(self.magic);
        buf.push(self.len);
        buf.push(self.seq);
        buf.push(self.sysid);
        buf.push(self.compid);
        buf.push(self.msgid as u8);
        buf.extend_from_slice(&self.payload);
        // Note: checksum calculation would go here
        buf.extend_from_slice(&self.checksum.to_le_bytes());
        buf
    }

    /// Parse from bytes
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 8 {
            return None;
        }

        let magic = data[0];
        let len = data[1];
        let seq = data[2];
        let sysid = data[3];
        let compid = data[4];
        let msgid = data[5] as MessageId;

        let payload_end = 6 + len as usize;
        if data.len() < payload_end + 2 {
            return None;
        }

        let payload = data[6..payload_end].to_vec();
        let checksum = u16::from_le_bytes([data[payload_end], data[payload_end + 1]]);

        Some(Self {
            magic,
            len,
            seq,
            sysid,
            compid,
            msgid,
            payload,
            checksum,
        })
    }

    /// Check if this is a valid MAVLink v1 packet
    pub fn is_valid_v1(&self) -> bool {
        self.magic == 0xFE
    }

    /// Check if this is a valid MAVLink v2 packet  
    pub fn is_valid_v2(&self) -> bool {
        self.magic == 0xFD
    }
}
