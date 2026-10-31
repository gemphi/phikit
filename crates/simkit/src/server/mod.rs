//! Simulation Server Module
//!
//! Implements the RPC server that responds to client requests.
//! This is the server-side counterpart to the RPC client.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// The session journal a serving run leaves: `simkit-session.log` beside the
// executable, armed by `RpcServer::start` and by nothing else.
pub mod journal;
pub mod rpc;
pub mod simulation;

pub use rpc::RpcServer;
pub use simulation::Simulation;

/// Server state management
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerState {
    /// Server is initializing
    Initializing,
    /// Server is running and accepting connections
    Running,
    /// Server is paused
    Paused,
    /// Server is shutting down
    ShuttingDown,
    /// Server has stopped
    Stopped,
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Port to listen on
    pub port: u16,
    /// Enable API control by default
    pub enable_api_control: bool,
    /// Default home position
    pub home_position: (f64, f64, f64),
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 41451,
            enable_api_control: true,
            home_position: (0.0, 0.0, 0.0),
        }
    }
}

impl ServerConfig {
    /// Create new config with specific port
    pub fn with_port(port: u16) -> Self {
        Self {
            port,
            ..Default::default()
        }
    }

    /// Set home position
    pub fn with_home_position(mut self, lat: f64, lon: f64, alt: f64) -> Self {
        self.home_position = (lat, lon, alt);
        self
    }
}

/// Active connection tracking
pub struct ConnectionManager {
    connections: Arc<Mutex<HashMap<String, ConnectionInfo>>>,
}

impl ConnectionManager {
    /// Create new connection manager
    pub fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a new connection
    pub fn register(&self, id: String, info: ConnectionInfo) {
        if let Ok(mut conns) = self.connections.lock() {
            conns.insert(id, info);
        }
    }

    /// Remove a connection
    pub fn remove(&self, id: &str) {
        if let Ok(mut conns) = self.connections.lock() {
            conns.remove(id);
        }
    }

    /// Get active connection count
    pub fn count(&self) -> usize {
        self.connections.lock().map(|c| c.len()).unwrap_or(0)
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about a client connection
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    /// Client address
    pub address: String,
    /// Connection timestamp
    pub connected_at: std::time::Instant,
    /// Last activity
    pub last_activity: std::time::Instant,
}

impl ConnectionInfo {
    /// Create new connection info
    pub fn new(address: String) -> Self {
        let now = std::time::Instant::now();
        Self {
            address,
            connected_at: now,
            last_activity: now,
        }
    }
}
