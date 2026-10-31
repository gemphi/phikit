//! Basic usage example for rust-sim-kit
//!
//! This example demonstrates:
//! - Creating a multirotor vehicle with sensors
//! - Setting up the RPC client
//! - Using the recording system
//! - Working with the simulation clock

use simkit::prelude::*;

fn main() -> Result<()> {
    println!("Rust-Sim-Kit v{} Example", VERSION);
    println!("===========================\n");

    // Create a vehicle configuration
    let vehicle_config = VehicleConfig {
        name: "drone_01".to_string(),
        initial_pose: Pose::identity(),
        home_geo_point: GeoPoint::new(47.6062, -122.3321, 0.0), // Seattle
        mass: 1.5,                                              // 1.5 kg
        max_velocity: 20.0,                                     // 20 m/s
        api_control_enabled: true,
    };

    // Create vehicle base
    let mut vehicle = VehicleBase::new(vehicle_config);
    println!("Created vehicle: {}", vehicle.config().name);

    // Add sensors
    let imu = ImuSensor::new("imu");
    vehicle.sensors_mut().add(Box::new(imu));

    let gps = GpsSensor::new("gps");
    vehicle.sensors_mut().add(Box::new(gps));

    let barometer = BarometerSensor::new("barometer");
    vehicle.sensors_mut().add(Box::new(barometer));

    let mut camera_config = CameraConfig::default();
    camera_config.base.name = "front_camera".to_string();
    vehicle.add_camera(CameraSensor::new(camera_config));

    println!("Added sensors: IMU, GPS, Barometer, Camera");
    println!("Cameras: {:?}", vehicle.list_cameras());

    // Set up environment
    let home_geo = GeoPoint::new(47.6062, -122.3321, 0.0);
    let environment = Environment::new(home_geo);
    println!("\nEnvironment setup:");
    println!("  Home: {:?}", environment.home_geo_point());
    println!("  Temperature: {}°C", environment.state().temperature);
    println!("  Pressure: {} Pa", environment.state().pressure);

    // Create simulation clock
    let mut clock = SimulationClock::new_steppable(0.016);
    println!("\nSimulation clock: {:?}", clock.clock_type());

    // Simulate for a few steps
    println!("\n--- Simulation Loop ---");
    for i in 0..5 {
        let dt = 0.016; // 60 FPS
        clock.step(1);
        vehicle.update_sensors(dt, &environment);
        println!("Step {}: time={:.3}s", i + 1, clock.now());
    }

    // Set up recording
    let recording_config = RecordingConfig::with_output_dir("./recordings").enable_all();

    let mut session = RecordingSession::new(recording_config);
    println!("\n--- Recording System ---");
    println!("Recording session created");
    println!("  Output: {:?}", session.config().output_dir);

    if let Err(e) = session.start() {
        eprintln!("  Warning: Could not start recording: {}", e);
    } else {
        println!("  Started");

        // Simulate recording some frames
        for i in 0..3 {
            let mut frame = RecordedFrame::new(i as u64, (i * 100) as u64);
            frame.vehicle_poses.insert("drone_01".to_string(), Pose::identity());
            if let Err(e) = session.record_frame(frame) {
                eprintln!("  Warning: Could not record frame: {}", e);
            }
        }

        if let Err(e) = session.stop() {
            eprintln!("  Warning: Could not stop recording: {}", e);
        } else {
            println!("  Stopped");
        }
    }

    // RPC Client example (stub - would connect to real server)
    println!("\n--- RPC Client ---");
    let client = RpcClient::new("localhost");
    println!("Created RPC client");
    println!("  Address: {}:{}", client.ip_address(), client.port());
    println!("  State: {:?}", client.connection_state());
    println!("  Timeout: {}s", client.timeout());
    println!("  Client Version: {}", client.client_version());

    println!("\n===========================");
    println!("Example completed successfully!");

    Ok(())
}
