# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added
- **`input`: the game-control layer.** A gamepad, a flight stick or a
  keyboard turned into the demand a vehicle is actually commanded with,
  and not a button code.
  - `ControlState`: `RotorDemand` (roll, pitch, yaw rate, throttle, a mode
    switch and an arm switch) and `CarDemand` (steer, throttle, brake,
    handbrake, gear), after AirSim's `RCData` and
    `CarApiBase::CarControls`. Round-trips through `RCData`, so a
    `moveByRC` off the wire is the same struct a stick produces.
  - `Bindings`: a remappable table of `Source` to `Intent`, with presets
    for a gamepad, a flight stick and a wheel.
  - `AxisShape`: dead zone, expo, scale, trim, inversion and a change of
    `Span` when a sprung stick has to read as a lever — in a radio's order.
  - `Device`: the seam a backend plugs in at, reporting a `Reading`.
    `Keys` is the keyboard implementation (ramped axes, a sprung stick, a
    collective that stays put); `Scripted` plays a flight back.
- **`ecs`: the phiecs bridge.** A vehicle is a `Word`; its `Placement`,
  `Motion`, `Helm`, `Autopilot`, `Senses` and `Craft` are `Aspect`s; the
  controller, the physics, the sensors and the readout are `System`s in
  `Stage` order.
  - `ControlLink`: the one door in and window out. A stick and an AirSim
    client both put an `Order` on it, `Intake` is the only thing that
    drains it, and which side is obeyed is AirSim's own
    `enableApiControl`. A refused order is counted, not dropped quietly.
  - `ecs::wire`: AirSim's msgpack-rpc calls as `Act`s and answers, with
    the NED-to-z-up turn in one place (`flip_ned`).
  - `Simulation::attach` wires the RPC server to a world, so
    `simGetVehiclePose` answers from the frame the pilot just flew.
- `examples/fly_with_a_stick.rs`: a drone and a car flown from a keyboard
  and then from a client, in one run.
- `tests/control_loop.rs` and `tests/control_doors.rs`: a scripted stick
  climbing, holding and landing through phiecs, and the two doors proved
  not to bypass each other.

### Fixed
- The tests, example and benchmark still named the crate `rust_sim_kit`
  and had drifted from the recording, clock and camera APIs, so none of
  them compiled. All three build and pass now, and the benchmark declares
  the `criterion` it was written against.

## [0.1.0] - 2024-05-01

### Added
- Initial release of rust-sim-kit
- **API Module**: AirSim-compatible msgpack-rpc client
  - `RpcClient` with async connection management
  - `SimApi` trait with 30+ simulation control methods
  - Connection state management
  - Full sensor data retrieval API
- **Sensor Implementations** (7 types):
  - `ImuSensor` with accelerometer and gyroscope output
  - `GpsSensor` with position and velocity data
  - `LidarSensor` for point cloud generation
  - `BarometerSensor` for altitude/pressure
  - `MagnetometerSensor` for heading
  - `DistanceSensor` for rangefinding
  - `CameraSensor` for image capture
- **Vehicle Types**:
  - `MultirotorVehicle` for drone simulation
  - `CarVehicle` for ground vehicle simulation
  - `VehicleBase` with sensor management
- **Recording System**:
  - `RecordingSession` with state machine
  - `RecordedFrame` for frame data
  - `RecordingWriter` trait for extensible output
  - Multiple format support (AirSim binary, CSV, JSON)
- **Physics Simulation**:
  - `KinematicsState` with pose, twist, accelerations
  - `Environment` and `EnvironmentState`
  - `Wrench` for force/torque
- **Clock/Time Management**:
  - `SimulationClock` with real-time, steppable, and scalable modes
  - `Timer` for interval tracking
  - `FrameLimiter` for FPS control
  - `WallClock` for system time
- **Common Types**:
  - `Pose`, `Twist`, `Wrench`, `GeoPoint`
  - `CollisionInfo`, `DetectionInfo`, `RcData`
  - `ImageRequest`, `ImageResponse`, `CameraInfo`
  - `DistanceSensorData`, `LidarData`
- **Prelude Module**: Convenient imports via `rust_sim_kit::prelude::*`
- **Examples**: Basic usage example
- **Tests**: Integration test suite
- **Benchmarks**: Performance benchmarks for sensors and core types

### Dependencies
- `tokio` with features: rt-multi-thread, net, time, io-util, macros, sync
- `rmp-serde` and `rmpv` for MessagePack RPC
- `glam` for 3D math
- `serde` for serialization
- `thiserror` for error handling
- `chrono` for time handling
- `image` for image processing
