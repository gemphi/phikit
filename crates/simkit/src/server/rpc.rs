//! RPC Server Implementation
//!
//! Handles incoming msgpack-rpc requests from clients.

use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;

use crate::server::{ConnectionInfo, ConnectionManager, ServerConfig, ServerState};
use crate::{Result, SimKitError};

/// RPC Server that handles client requests
pub struct RpcServer {
    /// Server configuration
    config: ServerConfig,
    /// Current server state
    state: Arc<RwLock<ServerState>>,
    /// Connection manager
    connections: Arc<ConnectionManager>,
    /// Simulation handle
    simulation: Arc<RwLock<crate::server::Simulation>>,
}

impl RpcServer {
    /// Create new RPC server
    pub fn new(config: ServerConfig, simulation: crate::server::Simulation) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(ServerState::Initializing)),
            connections: Arc::new(ConnectionManager::new()),
            simulation: Arc::new(RwLock::new(simulation)),
        }
    }

    /// Start the RPC server
    pub async fn start(&self) -> Result<()> {
        // A serving run is the headless one: the journal first, so every
        // record below — and a panic — lands in a file beside the executable
        // for a session nobody was watching. Only this path arms it.
        crate::server::journal::arm();
        let addr = format!("0.0.0.0:{}", self.config.port);
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| SimKitError::Connection(format!("Failed to bind: {}", e)))?;

        *self.state.write().await = ServerState::Running;
        log::info!("RPC Server listening on {}", addr);

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    let state = self.state.clone();
                    let connections = self.connections.clone();
                    let simulation = self.simulation.clone();

                    tokio::spawn(async move {
                        let peer_addr = addr.to_string();
                        connections
                            .register(peer_addr.clone(), ConnectionInfo::new(peer_addr.clone()));
                        // How long this client was on, filed when it leaves.
                        let _span = philog::BSpan::open(format!("Client {}", peer_addr));
                        log::info!("Client {} connected", peer_addr);

                        if let Err(e) = handle_client(stream, state, simulation).await {
                            log::error!("Client {} error: {}", peer_addr, e);
                        }

                        connections.remove(&peer_addr);
                    });
                }
                Err(e) => {
                    log::error!("Accept error: {}", e);
                }
            }

            // Check if we should stop
            if *self.state.read().await == ServerState::ShuttingDown {
                break;
            }
        }

        *self.state.write().await = ServerState::Stopped;
        Ok(())
    }

    /// Stop the server
    pub async fn stop(&self) {
        *self.state.write().await = ServerState::ShuttingDown;
    }

    /// Get current server state
    pub async fn state(&self) -> ServerState {
        *self.state.read().await
    }

    /// Get connection count
    pub fn connection_count(&self) -> usize {
        self.connections.count()
    }
}

/// Handle a single client connection
async fn handle_client(
    mut stream: TcpStream,
    state: Arc<RwLock<ServerState>>,
    simulation: Arc<RwLock<crate::server::Simulation>>,
) -> Result<()> {
    let mut buf = vec![0u8; 4096];

    loop {
        // Check server state
        if *state.read().await == ServerState::ShuttingDown {
            break;
        }

        // Read request
        let n = match stream.read(&mut buf).await {
            Ok(0) => break, // Connection closed
            Ok(n) => n,
            Err(e) => return Err(SimKitError::Connection(format!("Read error: {}", e))),
        };

        // Parse msgpack-rpc request
        let request: rmpv::Value = match rmp_serde::from_slice(&buf[..n]) {
            Ok(req) => req,
            Err(e) => {
                log::error!("Failed to parse request: {}", e);
                continue;
            }
        };

        // Process request and send response
        let response = process_request(request, &simulation).await;

        let response_bytes =
            rmp_serde::to_vec(&response).map_err(|e| SimKitError::Serialization(e.to_string()))?;

        stream
            .write_all(&response_bytes)
            .await
            .map_err(|e| SimKitError::Connection(format!("Write error: {}", e)))?;
    }

    Ok(())
}

/// Process an RPC request and return a response
async fn process_request(
    request: rmpv::Value,
    simulation: &Arc<RwLock<crate::server::Simulation>>,
) -> rmpv::Value {
    // Parse request format: [type, msgid, method, params]
    let arr = match request.as_array() {
        Some(a) if a.len() >= 4 => a,
        _ => return error_response(0, "Invalid request format"),
    };

    let _msg_type = arr[0].as_u64().unwrap_or(0) as u32;
    let msg_id = arr[1].as_u64().unwrap_or(0) as u32;
    let method = arr[2].as_str().unwrap_or("");
    let params = &arr[3];

    log::debug!("RPC request: {} (id={})", method, msg_id);

    // What a slow flight is made of: the journal gets a line for any call
    // that took long enough to be felt by the drone (`journal::SLOW_MS`).
    let began = std::time::Instant::now();
    let sim = simulation.read().await;

    // A call that commands a vehicle goes to the world's link, never
    // straight at a vehicle: the same door a gamepad pushes through, so
    // neither can bypass the other. Only when no world is attached does
    // the dispatch below answer for itself.
    let result = match commanded(&sim, method, params) {
        Some(answered) => answered,
        None => dispatch(&sim, method, params),
    };
    crate::server::journal::slow_request(method, msg_id, began.elapsed());

    match result {
        Ok(val) => success_response(msg_id, val),
        Err(e) => error_response(msg_id, &e),
    }
}

/// An AirSim call's parameters, as [`ecs::wire::Params`] reads them.
///
/// [`ecs::wire::Params`]: crate::ecs::wire::Params
struct WireParams<'a>(&'a [rmpv::Value]);

impl crate::ecs::wire::Params for WireParams<'_> {
    fn number(&self, index: usize) -> Option<f64> {
        let value = self.0.get(index)?;
        value
            .as_f64()
            .or_else(|| value.as_i64().map(|i| i as f64))
            .or_else(|| value.as_bool().map(|b| f64::from(u8::from(b))))
    }

    fn flag(&self, index: usize) -> Option<bool> {
        let value = self.0.get(index)?;
        value.as_bool().or_else(|| value.as_i64().map(|i| i != 0))
    }

    fn text(&self, index: usize) -> Option<String> {
        self.0.get(index)?.as_str().map(str::to_string)
    }

    fn count(&self) -> usize {
        self.0.len()
    }
}

/// `method` translated onto the world's link, or `None` when it is not a
/// command or there is no world to send it to.
fn commanded(
    sim: &crate::server::Simulation,
    method: &str,
    params: &rmpv::Value,
) -> Option<Result<rmpv::Value, String>> {
    let link = sim.link()?;
    let empty: Vec<rmpv::Value> = Vec::new();
    let list = params.as_array().unwrap_or(&empty);
    let call = crate::ecs::wire::read(method, &WireParams(list))?;
    match crate::ecs::wire::send(link, call, &sim.only_vehicle()) {
        true => Some(Ok(rmpv::Value::Boolean(true))),
        false => Some(Err(format!("{method}: no such vehicle"))),
    }
}

fn dispatch(
    sim: &crate::server::Simulation,
    method: &str,
    params: &rmpv::Value,
) -> Result<rmpv::Value, String> {
    match method {
        "ping" => handle_ping(),
        "getServerVersion" => handle_get_server_version(),
        "reset" => handle_reset(sim),
        "simPause" => handle_sim_pause(sim, params),
        "simIsPaused" => handle_sim_is_paused(sim),
        "enableApiControl" => handle_enable_api_control(sim, params),
        "isApiControlEnabled" => handle_is_api_control_enabled(sim, params),
        "armDisarm" => handle_arm_disarm(sim, params),
        "takeoff" => handle_takeoff(sim, params),
        "land" => handle_land(sim, params),
        "hover" => handle_hover(sim, params),
        "moveToPosition" => handle_move_to_position(sim, params),
        "moveByVelocity" => handle_move_by_velocity(sim, params),
        "getHomeGeoPoint" => handle_get_home_geo_point(sim, params),
        "simGetVehiclePose" => handle_get_vehicle_pose(sim, params),
        "simSetVehiclePose" => handle_set_vehicle_pose(sim, params),
        "simGetGroundTruthKinematics" => handle_get_ground_truth_kinematics(sim, params),
        "simGetImages" => handle_get_images(sim, params),
        "simGetImage" => handle_get_image(sim, params),
        "simGetCameraInfo" => handle_get_camera_info(sim, params),
        "simSetCameraPose" => handle_set_camera_pose(sim, params),
        "simSetCameraFov" => handle_set_camera_fov(sim, params),
        "getLidarData" => handle_get_lidar_data(sim, params),
        "getImuData" => handle_get_imu_data(sim, params),
        "getGpsData" => handle_get_gps_data(sim, params),
        "getBarometerData" => handle_get_barometer_data(sim, params),
        "getMagnetometerData" => handle_get_magnetometer_data(sim, params),
        "getDistanceSensorData" => handle_get_distance_sensor_data(sim, params),
        "simGetCollisionInfo" => handle_get_collision_info(sim, params),
        "listVehicles" => handle_list_vehicles(sim),
        "startRecording" => handle_start_recording(sim),
        "stopRecording" => handle_stop_recording(sim),
        "isRecording" => handle_is_recording(sim),
        _ => Err(format!("Unknown method: {}", method)),
    }
}

fn success_response(msg_id: u32, result: rmpv::Value) -> rmpv::Value {
    // msgpack-rpc response format: [1, msgid, null, result]
    rmpv::Value::Array(vec![
        rmpv::Value::Integer(1.into()),
        rmpv::Value::Integer(msg_id.into()),
        rmpv::Value::Nil,
        result,
    ])
}

fn error_response(msg_id: u32, error: &str) -> rmpv::Value {
    // msgpack-rpc error response: [1, msgid, error, null]
    rmpv::Value::Array(vec![
        rmpv::Value::Integer(1.into()),
        rmpv::Value::Integer(msg_id.into()),
        rmpv::Value::String(error.into()),
        rmpv::Value::Nil,
    ])
}

// ==================== REQUEST HANDLERS ====================

fn handle_ping() -> Result<rmpv::Value, String> {
    Ok(rmpv::Value::String("pong".into()))
}

fn handle_get_server_version() -> Result<rmpv::Value, String> {
    Ok(rmpv::Value::Integer(1.into()))
}

fn handle_reset(_sim: &crate::server::Simulation) -> Result<rmpv::Value, String> {
    // Reset simulation state
    Ok(rmpv::Value::Nil)
}

fn handle_sim_pause(
    _sim: &crate::server::Simulation,
    params: &rmpv::Value,
) -> Result<rmpv::Value, String> {
    let paused = params
        .as_array()
        .and_then(|a| a.get(0))
        .and_then(|v| v.as_bool())
        .ok_or("Invalid params")?;

    log::info!("Simulation pause: {}", paused);
    Ok(rmpv::Value::Nil)
}

fn handle_sim_is_paused(_sim: &crate::server::Simulation) -> Result<rmpv::Value, String> {
    Ok(rmpv::Value::Boolean(false))
}

fn handle_enable_api_control(
    _sim: &crate::server::Simulation,
    _params: &rmpv::Value,
) -> Result<rmpv::Value, String> {
    Ok(rmpv::Value::Nil)
}

fn handle_is_api_control_enabled(
    sim: &crate::server::Simulation,
    params: &rmpv::Value,
) -> Result<rmpv::Value, String> {
    // Whether the world says the client holds the vehicle, not whether
    // the server would like it to.
    let held = match sim.report(&named_vehicle(params)) {
        Some(report) => report.api_control,
        None => true,
    };
    Ok(rmpv::Value::Boolean(held))
}

fn handle_arm_disarm(
    _sim: &crate::server::Simulation,
    _params: &rmpv::Value,
) -> Result<rmpv::Value, String> {
    Ok(rmpv::Value::Boolean(true))
}

fn handle_takeoff(
    _sim: &crate::server::Simulation,
    _params: &rmpv::Value,
) -> Result<rmpv::Value, String> {
    Ok(rmpv::Value::Nil)
}

fn handle_land(
    _sim: &crate::server::Simulation,
    _params: &rmpv::Value,
) -> Result<rmpv::Value, String> {
    Ok(rmpv::Value::Nil)
}

fn handle_hover(
    _sim: &crate::server::Simulation,
    _params: &rmpv::Value,
) -> Result<rmpv::Value, String> {
    Ok(rmpv::Value::Nil)
}

fn handle_move_to_position(
    _sim: &crate::server::Simulation,
    _params: &rmpv::Value,
) -> Result<rmpv::Value, String> {
    Ok(rmpv::Value::Nil)
}

fn handle_move_by_velocity(
    _sim: &crate::server::Simulation,
    _params: &rmpv::Value,
) -> Result<rmpv::Value, String> {
    Ok(rmpv::Value::Nil)
}

fn handle_get_home_geo_point(
    _sim: &crate::server::Simulation,
    _params: &rmpv::Value,
) -> Result<rmpv::Value, String> {
    // Return default home point
    let home = vec![
        rmpv::Value::F64(47.6062),
        rmpv::Value::F64(-122.3321),
        rmpv::Value::F64(0.0),
    ];
    Ok(rmpv::Value::Array(home))
}

/// The vehicle a read call names, which AirSim puts first on its getters.
fn named_vehicle(params: &rmpv::Value) -> String {
    params
        .as_array()
        .and_then(|a| a.first())
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn handle_get_vehicle_pose(
    sim: &crate::server::Simulation,
    params: &rmpv::Value,
) -> Result<rmpv::Value, String> {
    // The pose the world last published, in NED, so a client reads the
    // very frame the pilot just flew. Without a world attached, the
    // identity pose, as before.
    let pose = match sim.report(&named_vehicle(params)) {
        Some(report) => crate::ecs::wire::pose_reply(&report),
        None => [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0],
    };
    Ok(rmpv::Value::Array(pose.iter().map(|n| rmpv::Value::F64(*n)).collect()))
}

fn handle_set_vehicle_pose(
    _sim: &crate::server::Simulation,
    _params: &rmpv::Value,
) -> Result<rmpv::Value, String> {
    Ok(rmpv::Value::Nil)
}

fn handle_get_ground_truth_kinematics(
    sim: &crate::server::Simulation,
    params: &rmpv::Value,
) -> Result<rmpv::Value, String> {
    let vectors = match sim.report(&named_vehicle(params)) {
        Some(report) => crate::ecs::wire::kinematics_reply(&report),
        None => [[0.0; 3]; 3],
    };
    let triple = |v: &[f64; 3]| {
        rmpv::Value::Array(v.iter().map(|n| rmpv::Value::F64(*n)).collect())
    };
    // Position, linear velocity and linear acceleration, in NED.
    Ok(rmpv::Value::Array(vectors.iter().map(triple).collect()))
}

fn handle_get_images(
    _sim: &crate::server::Simulation,
    _params: &rmpv::Value,
) -> Result<rmpv::Value, String> {
    // Return empty image array
    Ok(rmpv::Value::Array(vec![]))
}

fn handle_get_image(
    _sim: &crate::server::Simulation,
    _params: &rmpv::Value,
) -> Result<rmpv::Value, String> {
    // Return empty image data
    Ok(rmpv::Value::Binary(vec![]))
}

fn handle_get_camera_info(
    _sim: &crate::server::Simulation,
    _params: &rmpv::Value,
) -> Result<rmpv::Value, String> {
    // Return default camera info
    Ok(rmpv::Value::Nil)
}

fn handle_set_camera_pose(
    _sim: &crate::server::Simulation,
    _params: &rmpv::Value,
) -> Result<rmpv::Value, String> {
    Ok(rmpv::Value::Nil)
}

fn handle_set_camera_fov(
    _sim: &crate::server::Simulation,
    _params: &rmpv::Value,
) -> Result<rmpv::Value, String> {
    Ok(rmpv::Value::Nil)
}

fn handle_get_lidar_data(
    _sim: &crate::server::Simulation,
    _params: &rmpv::Value,
) -> Result<rmpv::Value, String> {
    // Return empty point cloud
    let data = vec![
        rmpv::Value::Array(vec![]),     // points
        rmpv::Value::Array(vec![]),     // timestamps
        rmpv::Value::Integer(0.into()), // pose
    ];
    Ok(rmpv::Value::Array(data))
}

fn handle_get_imu_data(
    _sim: &crate::server::Simulation,
    _params: &rmpv::Value,
) -> Result<rmpv::Value, String> {
    // Return default IMU data
    let data = vec![
        rmpv::Value::Array(vec![0.0.into(), 0.0.into(), 0.0.into()]), // angular velocity
        rmpv::Value::Array(vec![0.0.into(), 0.0.into(), 9.81.into()]), // linear acceleration
        rmpv::Value::Array(vec![0.0.into(), 0.0.into(), 0.0.into(), 1.0.into()]), // orientation
        rmpv::Value::Integer(0.into()),                               // timestamp
    ];
    Ok(rmpv::Value::Array(data))
}

fn handle_get_gps_data(
    _sim: &crate::server::Simulation,
    _params: &rmpv::Value,
) -> Result<rmpv::Value, String> {
    // Return default GPS data
    let data = vec![
        rmpv::Value::Integer(0.into()), // timestamp
        rmpv::Value::F64(47.6062),      // latitude
        rmpv::Value::F64(-122.3321),    // longitude
        rmpv::Value::F64(0.0),          // altitude
    ];
    Ok(rmpv::Value::Array(data))
}

fn handle_get_barometer_data(
    _sim: &crate::server::Simulation,
    _params: &rmpv::Value,
) -> Result<rmpv::Value, String> {
    // Return default barometer data
    let data = vec![
        rmpv::Value::Integer(0.into()), // timestamp
        rmpv::Value::F64(1013.25),      // pressure
        rmpv::Value::F64(0.0),          // altitude
    ];
    Ok(rmpv::Value::Array(data))
}

fn handle_get_magnetometer_data(
    _sim: &crate::server::Simulation,
    _params: &rmpv::Value,
) -> Result<rmpv::Value, String> {
    // Return default magnetometer data
    let data = vec![
        rmpv::Value::Integer(0.into()), // timestamp
        rmpv::Value::Array(vec![0.0.into(), 0.0.into(), 1.0.into()]), // magnetic field
    ];
    Ok(rmpv::Value::Array(data))
}

fn handle_get_distance_sensor_data(
    _sim: &crate::server::Simulation,
    _params: &rmpv::Value,
) -> Result<rmpv::Value, String> {
    // Return default distance
    let data = vec![
        rmpv::Value::Integer(0.into()), // timestamp
        rmpv::Value::F64(10.0),         // distance
        rmpv::Value::F64(0.0),          // min distance
        rmpv::Value::F64(100.0),        // max distance
    ];
    Ok(rmpv::Value::Array(data))
}

fn handle_get_collision_info(
    _sim: &crate::server::Simulation,
    _params: &rmpv::Value,
) -> Result<rmpv::Value, String> {
    // Return no collision
    Ok(rmpv::Value::Boolean(false))
}

fn handle_list_vehicles(sim: &crate::server::Simulation) -> Result<rmpv::Value, String> {
    // The vehicles the world has reported, which are the ones a client
    // can actually command.
    let named = match sim.link() {
        Some(link) => link.reported(),
        None => sim.list_vehicles(),
    };
    Ok(rmpv::Value::Array(
        named.into_iter().map(|n| rmpv::Value::String(n.into())).collect(),
    ))
}

fn handle_start_recording(_sim: &crate::server::Simulation) -> Result<rmpv::Value, String> {
    Ok(rmpv::Value::Nil)
}

fn handle_stop_recording(_sim: &crate::server::Simulation) -> Result<rmpv::Value, String> {
    Ok(rmpv::Value::Nil)
}

fn handle_is_recording(_sim: &crate::server::Simulation) -> Result<rmpv::Value, String> {
    Ok(rmpv::Value::Boolean(false))
}
