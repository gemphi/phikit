//! Integration tests for rust-sim-kit

use simkit::prelude::*;

#[test]
fn test_vehicle_creation() {
    let config = VehicleConfig {
        name: "test_drone".to_string(),
        initial_pose: Pose::identity(),
        home_geo_point: GeoPoint::new(0.0, 0.0, 0.0),
        mass: 1.0,
        max_velocity: 10.0,
        api_control_enabled: false,
    };

    let vehicle = VehicleBase::new(config);
    assert_eq!(vehicle.config().name, "test_drone");
    assert!(!vehicle.is_armed());
    assert!(!vehicle.is_api_control_enabled());
}

#[test]
fn test_sensor_collection() {
    let mut collection = SensorCollection::new();

    let imu = ImuSensor::new("imu");
    collection.add(Box::new(imu));

    let gps = GpsSensor::new("gps");
    collection.add(Box::new(gps));

    assert_eq!(collection.all().len(), 2);
    assert!(collection.get("imu").is_some());
    assert!(collection.get("gps").is_some());
    assert!(collection.get("nonexistent").is_none());
}

#[test]
fn test_pose_identity() {
    let pose = Pose::identity();
    assert_eq!(pose.position, glam::Vec3::ZERO);
    assert_eq!(pose.orientation, glam::Quat::IDENTITY);
}

#[test]
fn test_geopoint_creation() {
    let geo = GeoPoint::new(47.6062, -122.3321, 100.0);
    assert_eq!(geo.latitude, 47.6062);
    assert_eq!(geo.longitude, -122.3321);
    assert_eq!(geo.altitude, 100.0);
}

#[test]
fn test_kinematics_state_default() {
    let kin = KinematicsState::default();
    assert_eq!(kin.pose.position, glam::Vec3::ZERO);
    assert_eq!(kin.twist.linear, glam::Vec3::ZERO);
    assert_eq!(kin.accelerations.linear, glam::Vec3::ZERO);
}

#[test]
fn test_environment_default() {
    let home = GeoPoint::new(0.0, 0.0, 0.0);
    let env = Environment::new(home);
    assert_eq!(env.state().temperature, 20.0);
    assert_eq!(env.state().pressure, 101325.0);
}

#[test]
fn test_simulation_clock() {
    let mut clock = SimulationClock::new_steppable(0.5);
    assert_eq!(clock.clock_type(), ClockType::Steppable);
    assert_eq!(clock.now(), 0.0);

    clock.step(2);
    assert_eq!(clock.now(), 1.0);
}

#[test]
fn test_recording_config() {
    let config = RecordingConfig::default();
    assert!(!config.enabled);
    assert!(config.record_images);
    assert!(config.record_sensor_data);
    assert!(config.record_pose);
}

#[test]
fn test_recording_session() {
    let config = RecordingConfig::with_output_dir("./test_recordings").enable_all();
    let mut session = RecordingSession::new(config);

    assert_eq!(session.state(), RecordingState::Stopped);

    session.start().expect("Failed to start recording");
    assert_eq!(session.state(), RecordingState::Recording);

    let mut frame = RecordedFrame::new(0, 0);
    frame.vehicle_poses.insert("drone".to_string(), Pose::identity());

    session.record_frame(frame).expect("Failed to record frame");
    assert_eq!(session.stats().total_frames, 1);

    let frames = session.stop().expect("Failed to stop recording");
    assert_eq!(frames.len(), 1);
    assert_eq!(session.state(), RecordingState::Stopped);
}

#[test]
fn test_rpc_client_creation() {
    let client = RpcClient::new("localhost");
    assert_eq!(client.ip_address(), "localhost");
    assert_eq!(client.port(), DEFAULT_RPC_PORT);
    assert_eq!(client.connection_state(), ConnectionState::Initial);
}

#[test]
fn test_imu_sensor() {
    let imu = ImuSensor::new("test_imu");
    assert_eq!(imu.name(), "test_imu");
    assert_eq!(imu.sensor_type(), SensorType::Imu);
    assert!(imu.enabled());
}

#[test]
fn test_gps_sensor() {
    let gps = GpsSensor::new("test_gps");
    assert_eq!(gps.name(), "test_gps");
    assert_eq!(gps.sensor_type(), SensorType::Gps);
}

#[test]
fn test_camera_sensor() {
    let mut config = CameraConfig::default();
    config.base.name = "test_camera".to_string();
    let camera = CameraSensor::new(config);
    assert_eq!(camera.name(), "test_camera");
    assert_eq!(camera.sensor_type(), SensorType::Camera);
}

#[test]
fn test_image_request_creation() {
    let request = ImageRequest::new("camera1", ImageType::Scene);
    assert_eq!(request.camera_name, "camera1");
    assert_eq!(request.image_type, ImageType::Scene);
    assert!(!request.pixels_as_float);
    assert!(request.compress);
}

#[test]
fn test_wrench_arithmetic() {
    let w1 = Wrench {
        force: glam::Vec3::new(1.0, 0.0, 0.0),
        torque: glam::Vec3::new(0.0, 1.0, 0.0),
    };

    let w2 = Wrench {
        force: glam::Vec3::new(0.0, 1.0, 0.0),
        torque: glam::Vec3::new(1.0, 0.0, 0.0),
    };

    let sum = w1 + w2;
    assert_eq!(sum.force, glam::Vec3::new(1.0, 1.0, 0.0));
    assert_eq!(sum.torque, glam::Vec3::new(1.0, 1.0, 0.0));
}

#[test]
fn test_collision_info_default() {
    // `common::CollisionInfo` is the one an AirSim client reads over the
    // wire; `physics::CollisionInfo` is the contact the engine produces.
    let info = simkit::common::CollisionInfo::default();
    assert!(!info.has_collided);
    assert_eq!(info.collision_count, 0);
}
