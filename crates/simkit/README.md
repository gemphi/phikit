---
id: simkit
title: SimKit
custom_edit_url: https://github.com/gemphi/phi/edit/main/crates/simkit/README.md
---

# Rust-Sim-Kit

An AirSim-inspired simulation toolkit for autonomous vehicles (drones, cars) written in Rust.

## Features

- **RPC Client**: AirSim-compatible msgpack-rpc client for simulation server communication
- **Sensors**: IMU, GPS, Lidar, Barometer, Magnetometer, Distance, Camera
- **Vehicles**: Multirotor (drone) and Car vehicle types
- **Recording**: Frame-based recording system for simulation data and images
- **Physics**: Kinematics, environment simulation, and time management
- **Clock**: Real-time, steppable, and scalable simulation clocks
- **Input**: Gamepad, flight-stick and keyboard control, through a
  remappable binding table with dead zones, expo curves and trim
- **ECS**: The vehicles as a [phiecs](../phicore/phiecs) world — a vehicle
  is a word, its pose and control state are aspects, the controller and
  the physics are systems in stage order

## Quick Start

```rust
use simkit::prelude::*;

fn main() -> Result<()> {
    // Create a vehicle
    let config = VehicleConfig {
        name: "drone_01".to_string(),
        initial_pose: Pose::identity(),
        home_geo_point: GeoPoint::new(47.6062, -122.3321, 0.0),
        mass: 1.5,
        max_velocity: 20.0,
        api_control_enabled: true,
    };
    
    let mut vehicle = VehicleBase::new(config);
    
    // Add sensors
    vehicle.sensors_mut().add(Box::new(ImuSensor::new("imu")));
    vehicle.sensors_mut().add(Box::new(GpsSensor::new("gps")));
    vehicle.add_camera(CameraSensor::new(CameraConfig::default()));
    
    // Connect to simulation server
    let mut client = RpcClient::new("localhost");
    client.confirm_connection()?;
    
    // Control the simulation
    client.enable_api_control(true, "drone_01")?;
    client.arm_disarm(true, "drone_01")?;
    
    // Get sensor data
    let imu_data = client.get_imu_data("imu", "drone_01")?;
    let gps_data = client.get_gps_data("gps", "drone_01")?;
    
    // Capture images
    let image = client.sim_get_image("front_camera", ImageType::Scene, "drone_01")?;
    
    Ok(())
}
```

## Architecture

This crate mirrors the structure of AirSim's AirLib:

- `api`: RPC client APIs for connecting to simulation servers
- `common`: Shared types, structs, clock, and utilities
- `physics`: Physics simulation types (kinematics, environment, wrench)
- `sensors`: Sensor implementations (7 types + camera)
- `vehicles`: Vehicle types (multirotor/drone, car)
- `recording`: Recording system for simulation data
- `input`: The game-control layer: a device, a binding table, an axis
  shape and the device-independent `ControlState` they produce
- `ecs`: The phiecs bridge, and the one `ControlLink` a stick and an
  AirSim client both command through

## Flying it

A drone flown from a keyboard and then from a client, in one run:

```sh
cargo run -p simkit --example fly_with_a_stick
```

```rust
use simkit::ecs::{build, link_of, Driver, Plan};
use simkit::input::{Keys, Pilot};

let mut world = build(Plan::new().with_drone("drone"));
let link = link_of(&world);
let mut driver = Driver::new(&world, "drone", Keys::flight(), Pilot::gamepad());

// A host's whole loop: poll the device, step the world.
loop {
    driver.poll(1.0 / 60.0);
    world.step(1.0 / 60.0);
}
```

An AirSim client commands the same vehicle through the same link, so the
two take turns rather than fighting — `enableApiControl` decides which,
exactly as in AirSim.

## Dependencies

- `tokio`: Async runtime for RPC client
- `rmp-serde` + `rmpv`: MessagePack serialization for RPC
- `glam`: Math library for 3D types
- `phiecs`: The ECS the vehicles live in
- `serde`: Serialization framework
- `thiserror`: Error handling
- `chrono`: Time handling
- `image`: Image processing

## License

MIT
