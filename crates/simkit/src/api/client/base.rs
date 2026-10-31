use crate::api::ConnectionState;
use crate::{Result, SimKitError, DEFAULT_RPC_PORT};

use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex as TokioMutex;

#[derive(Debug)]
/// RpcClient Base implementation
pub struct RpcClient {
    pub(crate) ip_address: String,
    pub(crate) port: u16,
    pub(crate) timeout_sec: f32,
    pub(crate) state: std::sync::Mutex<ConnectionState>,
    pub(crate) client_version: i32,
    pub(crate) stream: std::sync::Mutex<Option<Arc<TokioMutex<TcpStream>>>>,
    pub(crate) request_id: std::sync::atomic::AtomicU32,
    pub(crate) runtime: Option<tokio::runtime::Runtime>,
}

impl Default for RpcClient {
    fn default() -> Self {
        Self::new("localhost")
    }
}

impl RpcClient {
    /// API Endpoint
    pub fn new(ip_address: impl Into<String>) -> Self {
        Self::with_port(ip_address, DEFAULT_RPC_PORT)
    }

    /// API Endpoint
    pub fn with_port(ip_address: impl Into<String>, port: u16) -> Self {
        Self {
            ip_address: ip_address.into(),
            port,
            timeout_sec: 60.0,
            state: std::sync::Mutex::new(ConnectionState::Initial),
            client_version: 1,
            stream: std::sync::Mutex::new(None),
            request_id: std::sync::atomic::AtomicU32::new(1),
            runtime: Some(tokio::runtime::Runtime::new().unwrap()),
        }
    }

    /// API Endpoint
    pub fn with_config(ip_address: impl Into<String>, port: u16, timeout_sec: f32) -> Self {
        Self {
            ip_address: ip_address.into(),
            port,
            timeout_sec,
            state: std::sync::Mutex::new(ConnectionState::Initial),
            client_version: 1,
            stream: std::sync::Mutex::new(None),
            request_id: std::sync::atomic::AtomicU32::new(1),
            runtime: Some(tokio::runtime::Runtime::new().unwrap()),
        }
    }

    pub(crate) fn block_on<F, T>(&self, fut: F) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        self.runtime
            .as_ref()
            .ok_or_else(|| SimKitError::Connection("Runtime not available".to_string()))?
            .block_on(fut)
    }

    /// API Endpoint
    pub async fn connect(&self) -> Result<()> {
        let addr = format!("{}:{}", self.ip_address, self.port);
        match tokio::time::timeout(
            tokio::time::Duration::from_secs_f32(self.timeout_sec),
            TcpStream::connect(&addr),
        )
        .await
        {
            Ok(Ok(stream)) => {
                *self.stream.lock().unwrap() = Some(Arc::new(TokioMutex::new(stream)));
                *self.state.lock().unwrap() = ConnectionState::Connected;
                log::info!("Connected to simulation server at {}", addr);
                Ok(())
            }
            Ok(Err(e)) => {
                *self.state.lock().unwrap() = ConnectionState::Disconnected;
                Err(SimKitError::Connection(format!("Failed to connect: {}", e)))
            }
            Err(_) => {
                *self.state.lock().unwrap() = ConnectionState::Disconnected;
                Err(SimKitError::Connection("Connection timeout".to_string()))
            }
        }
    }

    /// API Endpoint
    pub async fn disconnect(&self) -> Result<()> {
        if let Some(stream) = self.stream.lock().unwrap().take() {
            let mut guard = stream.lock().await;
            let _ = guard.shutdown().await;
        }
        *self.state.lock().unwrap() = ConnectionState::Disconnected;
        Ok(())
    }

    pub(crate) async fn send_request<Req: serde::Serialize, Resp: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: Req,
    ) -> Result<Resp> {
        let stream_guard = self.stream.lock().unwrap();
        let stream = stream_guard
            .as_ref()
            .ok_or_else(|| SimKitError::Connection("Not connected".to_string()))?;

        let id = self
            .request_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let request = (0u32, id, method, params);
        let request_bytes =
            rmp_serde::to_vec(&request).map_err(|e| SimKitError::Serialization(e.to_string()))?;

        let mut guard = stream.lock().await;
        guard
            .write_all(&request_bytes)
            .await
            .map_err(|e| SimKitError::Connection(format!("Write error: {}", e)))?;
        guard
            .flush()
            .await
            .map_err(|e| SimKitError::Connection(format!("Flush error: {}", e)))?;
        drop(guard);

        let mut buf = vec![0u8; 4096];
        let mut guard = stream.lock().await;
        let n = tokio::time::timeout(
            tokio::time::Duration::from_secs_f32(self.timeout_sec),
            guard.read(&mut buf),
        )
        .await
        .map_err(|_| SimKitError::Connection("Read timeout".to_string()))?
        .map_err(|e| SimKitError::Connection(format!("Read error: {}", e)))?;
        drop(guard);

        buf.truncate(n);

        let response: (u32, u32, Option<rmpv::Value>, Resp) =
            rmp_serde::from_slice(&buf).map_err(|e| SimKitError::Serialization(e.to_string()))?;

        if let Some(err) = response.2 {
            return Err(SimKitError::Rpc(format!("Server error: {:?}", err)));
        }

        Ok(response.3)
    }

    /// API Endpoint
    pub fn ip_address(&self) -> &str {
        &self.ip_address
    }
    /// API Endpoint
    pub fn port(&self) -> u16 {
        self.port
    }
    /// API Endpoint
    pub fn connection_state(&self) -> ConnectionState {
        *self.state.lock().unwrap()
    }
    /// API Endpoint
    pub fn timeout(&self) -> f32 {
        self.timeout_sec
    }
    /// The client's own version, which AirSim's clients send so a server
    /// can refuse one too old (`getClientVersion`).
    pub fn client_version(&self) -> i32 {
        self.client_version
    }
    /// API Endpoint
    pub fn set_timeout(&mut self, timeout_sec: f32) {
        self.timeout_sec = timeout_sec;
    }

    /// API Endpoint
    pub fn confirm_connection(&mut self) -> Result<()> {
        self.block_on(async {
            self.connect().await?;
            let pong: String = self.send_request("ping", ()).await?;
            if pong == "pong" {
                *self.state.lock().unwrap() = ConnectionState::Connected;
                Ok(())
            } else {
                Err(SimKitError::Connection("Invalid ping response".to_string()))
            }
        })
    }

    /// API Endpoint
    pub fn reset(&mut self) -> Result<()> {
        self.block_on(async {
            *self.state.lock().unwrap() = ConnectionState::Reset;
            let _: () = self.send_request("reset", ()).await?;
            *self.state.lock().unwrap() = ConnectionState::Connected;
            Ok(())
        })
    }

    /// API Endpoint
    pub fn ping(&mut self) -> Result<bool> {
        self.block_on(async {
            let result: Result<String> = self.send_request("ping", ()).await;
            Ok(matches!(result, Ok(pong) if pong == "pong"))
        })
    }
}
