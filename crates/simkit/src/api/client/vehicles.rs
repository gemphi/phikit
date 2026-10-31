use super::RpcClient;
use crate::api::{CarControls, CarState, DrivetrainType, MultirotorState, RotorStates, YawMode};
use crate::Result;
impl RpcClient {
    /// API Endpoint
    pub fn start_recording(&mut self) -> Result<()> {
        self.block_on(async { self.send_request("startRecording", ()).await })
    }

    /// API Endpoint
    pub fn stop_recording(&mut self) -> Result<()> {
        self.block_on(async { self.send_request("stopRecording", ()).await })
    }

    /// API Endpoint
    pub fn is_recording(&mut self) -> Result<bool> {
        self.block_on(async { self.send_request("isRecording", ()).await })
    }

    /// API Endpoint
    pub fn list_vehicles(&mut self) -> Result<Vec<String>> {
        self.block_on(async { self.send_request("listVehicles", ()).await })
    }

    // ==================== MULTIROTOR CONTROL ====================

    /// API Endpoint
    pub fn takeoff(&mut self, timeout_sec: f64, vehicle_name: &str) -> Result<()> {
        self.block_on(async {
            self.send_request("takeoff", (timeout_sec, vehicle_name))
                .await
        })
    }

    /// API Endpoint
    pub fn land(&mut self, timeout_sec: f64, vehicle_name: &str) -> Result<()> {
        self.block_on(async { self.send_request("land", (timeout_sec, vehicle_name)).await })
    }

    /// API Endpoint
    pub fn hover(&mut self, vehicle_name: &str) -> Result<()> {
        self.block_on(async { self.send_request("hover", (vehicle_name,)).await })
    }

    /// API Endpoint
    pub fn go_home(&mut self, timeout_sec: f64, vehicle_name: &str) -> Result<()> {
        self.block_on(async {
            self.send_request("goHome", (timeout_sec, vehicle_name))
                .await
        })
    }

    /// API Endpoint
    pub fn move_to_gps(
        &mut self,
        latitude: f32,
        longitude: f32,
        altitude: f32,
        velocity: f32,
        timeout_sec: f64,
        drivetrain: DrivetrainType,
        yaw_mode: YawMode,
        lookahead: f32,
        adaptive_lookahead: f32,
        vehicle_name: &str,
    ) -> Result<()> {
        self.block_on(async {
            self.send_request(
                "moveToGPS",
                (
                    latitude,
                    longitude,
                    altitude,
                    velocity,
                    timeout_sec,
                    drivetrain as u8,
                    yaw_mode,
                    lookahead,
                    adaptive_lookahead,
                    vehicle_name,
                ),
            )
            .await
        })
    }

    /// API Endpoint
    pub fn move_to_position(
        &mut self,
        x: f32,
        y: f32,
        z: f32,
        velocity: f32,
        timeout_sec: f64,
        drivetrain: DrivetrainType,
        yaw_mode: YawMode,
        vehicle_name: &str,
    ) -> Result<()> {
        self.block_on(async {
            self.send_request(
                "moveToPosition",
                (
                    x,
                    y,
                    z,
                    velocity,
                    timeout_sec,
                    drivetrain as u8,
                    yaw_mode as u8,
                    vehicle_name,
                ),
            )
            .await
        })
    }

    /// API Endpoint
    pub fn move_by_velocity(
        &mut self,
        vx: f32,
        vy: f32,
        vz: f32,
        duration: f64,
        drivetrain: DrivetrainType,
        yaw_mode: YawMode,
        vehicle_name: &str,
    ) -> Result<()> {
        self.block_on(async {
            self.send_request(
                "moveByVelocity",
                (
                    vx,
                    vy,
                    vz,
                    duration,
                    drivetrain as u8,
                    yaw_mode as u8,
                    vehicle_name,
                ),
            )
            .await
        })
    }

    /// API Endpoint
    pub fn move_by_velocity_z(
        &mut self,
        vx: f32,
        vy: f32,
        z: f32,
        duration: f64,
        drivetrain: DrivetrainType,
        yaw_mode: YawMode,
        vehicle_name: &str,
    ) -> Result<()> {
        self.block_on(async {
            self.send_request(
                "moveByVelocityZ",
                (
                    vx,
                    vy,
                    z,
                    duration,
                    drivetrain as u8,
                    yaw_mode,
                    vehicle_name,
                ),
            )
            .await
        })
    }

    /// API Endpoint
    pub fn move_by_velocity_body_frame(
        &mut self,
        vx: f32,
        vy: f32,
        vz: f32,
        duration: f64,
        drivetrain: DrivetrainType,
        yaw_mode: YawMode,
        vehicle_name: &str,
    ) -> Result<()> {
        self.block_on(async {
            self.send_request(
                "moveByVelocityBodyFrame",
                (
                    vx,
                    vy,
                    vz,
                    duration,
                    drivetrain as u8,
                    yaw_mode,
                    vehicle_name,
                ),
            )
            .await
        })
    }

    /// API Endpoint
    pub fn move_by_velocity_z_body_frame(
        &mut self,
        vx: f32,
        vy: f32,
        z: f32,
        duration: f64,
        drivetrain: DrivetrainType,
        yaw_mode: YawMode,
        vehicle_name: &str,
    ) -> Result<()> {
        self.block_on(async {
            self.send_request(
                "moveByVelocityZBodyFrame",
                (
                    vx,
                    vy,
                    z,
                    duration,
                    drivetrain as u8,
                    yaw_mode,
                    vehicle_name,
                ),
            )
            .await
        })
    }

    /// API Endpoint
    pub fn move_by_motor_pwms(
        &mut self,
        front_right_pwm: f32,
        rear_left_pwm: f32,
        front_left_pwm: f32,
        rear_right_pwm: f32,
        duration: f64,
        vehicle_name: &str,
    ) -> Result<()> {
        self.block_on(async {
            self.send_request(
                "moveByMotorPWMs",
                (
                    front_right_pwm,
                    rear_left_pwm,
                    front_left_pwm,
                    rear_right_pwm,
                    duration,
                    vehicle_name,
                ),
            )
            .await
        })
    }

    /// API Endpoint
    pub fn move_by_roll_pitch_yaw_z(
        &mut self,
        roll: f32,
        pitch: f32,
        yaw: f32,
        z: f32,
        duration: f64,
        vehicle_name: &str,
    ) -> Result<()> {
        self.block_on(async {
            self.send_request(
                "moveByRollPitchYawZ",
                (roll, pitch, yaw, z, duration, vehicle_name),
            )
            .await
        })
    }

    /// API Endpoint
    pub fn move_by_roll_pitch_yaw_throttle(
        &mut self,
        roll: f32,
        pitch: f32,
        yaw: f32,
        throttle: f32,
        duration: f64,
        vehicle_name: &str,
    ) -> Result<()> {
        self.block_on(async {
            self.send_request(
                "moveByRollPitchYawThrottle",
                (roll, pitch, yaw, throttle, duration, vehicle_name),
            )
            .await
        })
    }

    /// API Endpoint
    pub fn move_by_roll_pitch_yawrate_z(
        &mut self,
        roll: f32,
        pitch: f32,
        yaw_rate: f32,
        z: f32,
        duration: f64,
        vehicle_name: &str,
    ) -> Result<()> {
        self.block_on(async {
            self.send_request(
                "moveByRollPitchYawrateZ",
                (roll, pitch, yaw_rate, z, duration, vehicle_name),
            )
            .await
        })
    }

    /// API Endpoint
    pub fn move_by_roll_pitch_yawrate_throttle(
        &mut self,
        roll: f32,
        pitch: f32,
        yaw_rate: f32,
        throttle: f32,
        duration: f64,
        vehicle_name: &str,
    ) -> Result<()> {
        self.block_on(async {
            self.send_request(
                "moveByRollPitchYawrateThrottle",
                (roll, pitch, yaw_rate, throttle, duration, vehicle_name),
            )
            .await
        })
    }

    /// API Endpoint
    pub fn move_by_angle_rates_z(
        &mut self,
        roll_rate: f32,
        pitch_rate: f32,
        yaw_rate: f32,
        z: f32,
        duration: f64,
        vehicle_name: &str,
    ) -> Result<()> {
        self.block_on(async {
            self.send_request(
                "moveByAngleRatesZ",
                (roll_rate, pitch_rate, yaw_rate, z, duration, vehicle_name),
            )
            .await
        })
    }

    /// API Endpoint
    pub fn move_by_angle_rates_throttle(
        &mut self,
        roll_rate: f32,
        pitch_rate: f32,
        yaw_rate: f32,
        throttle: f32,
        duration: f64,
        vehicle_name: &str,
    ) -> Result<()> {
        self.block_on(async {
            self.send_request(
                "moveByAngleRatesThrottle",
                (
                    roll_rate,
                    pitch_rate,
                    yaw_rate,
                    throttle,
                    duration,
                    vehicle_name,
                ),
            )
            .await
        })
    }

    /// API Endpoint
    pub fn move_on_path(
        &mut self,
        path: Vec<glam::Vec3>,
        velocity: f32,
        timeout_sec: f64,
        drivetrain: DrivetrainType,
        yaw_mode: YawMode,
        lookahead: f32,
        adaptive_lookahead: f32,
        vehicle_name: &str,
    ) -> Result<()> {
        self.block_on(async {
            self.send_request(
                "moveOnPath",
                (
                    path,
                    velocity,
                    timeout_sec,
                    drivetrain as u8,
                    yaw_mode,
                    lookahead,
                    adaptive_lookahead,
                    vehicle_name,
                ),
            )
            .await
        })
    }

    /// API Endpoint
    pub fn move_to_z(
        &mut self,
        z: f32,
        velocity: f32,
        timeout_sec: f64,
        yaw_mode: YawMode,
        lookahead: f32,
        adaptive_lookahead: f32,
        vehicle_name: &str,
    ) -> Result<()> {
        self.block_on(async {
            self.send_request(
                "moveToZ",
                (
                    z,
                    velocity,
                    timeout_sec,
                    yaw_mode,
                    lookahead,
                    adaptive_lookahead,
                    vehicle_name,
                ),
            )
            .await
        })
    }

    /// API Endpoint
    pub fn move_by_manual(
        &mut self,
        vx_max: f32,
        vy_max: f32,
        z_min: f32,
        duration: f64,
        drivetrain: DrivetrainType,
        yaw_mode: YawMode,
        vehicle_name: &str,
    ) -> Result<()> {
        self.block_on(async {
            self.send_request(
                "moveByManual",
                (
                    vx_max,
                    vy_max,
                    z_min,
                    duration,
                    drivetrain as u8,
                    yaw_mode,
                    vehicle_name,
                ),
            )
            .await
        })
    }

    /// API Endpoint
    pub fn get_multirotor_state(&mut self, vehicle_name: &str) -> Result<MultirotorState> {
        self.block_on(async {
            self.send_request("getMultirotorState", (vehicle_name,))
                .await
        })
    }

    /// API Endpoint
    pub fn get_rotor_states(&mut self, vehicle_name: &str) -> Result<RotorStates> {
        self.block_on(async { self.send_request("getRotorStates", (vehicle_name,)).await })
    }

    /// API Endpoint
    pub fn set_angle_level_controller_gains(
        &mut self,
        kp: Vec<f32>,
        ki: Vec<f32>,
        kd: Vec<f32>,
        vehicle_name: &str,
    ) -> Result<()> {
        self.block_on(async {
            self.send_request("setAngleLevelControllerGains", (kp, ki, kd, vehicle_name))
                .await
        })
    }

    /// API Endpoint
    pub fn set_angle_rate_controller_gains(
        &mut self,
        kp: Vec<f32>,
        ki: Vec<f32>,
        kd: Vec<f32>,
        vehicle_name: &str,
    ) -> Result<()> {
        self.block_on(async {
            self.send_request("setAngleRateControllerGains", (kp, ki, kd, vehicle_name))
                .await
        })
    }

    /// API Endpoint
    pub fn set_velocity_controller_gains(
        &mut self,
        kp: Vec<f32>,
        ki: Vec<f32>,
        kd: Vec<f32>,
        vehicle_name: &str,
    ) -> Result<()> {
        self.block_on(async {
            self.send_request("setVelocityControllerGains", (kp, ki, kd, vehicle_name))
                .await
        })
    }

    /// API Endpoint
    pub fn set_position_controller_gains(
        &mut self,
        kp: Vec<f32>,
        ki: Vec<f32>,
        kd: Vec<f32>,
        vehicle_name: &str,
    ) -> Result<()> {
        self.block_on(async {
            self.send_request("setPositionControllerGains", (kp, ki, kd, vehicle_name))
                .await
        })
    }

    /// API Endpoint
    pub fn move_by_rc(&mut self, rc_data: crate::common::RcData, vehicle_name: &str) -> Result<()> {
        self.block_on(async { self.send_request("moveByRC", (rc_data, vehicle_name)).await })
    }

    /// API Endpoint
    pub fn rotate_by_yaw_rate(
        &mut self,
        yaw_rate: f32,
        duration: f64,
        vehicle_name: &str,
    ) -> Result<()> {
        self.block_on(async {
            self.send_request("rotateByYawRate", (yaw_rate, duration, vehicle_name))
                .await
        })
    }

    /// API Endpoint
    pub fn rotate_to_yaw(
        &mut self,
        yaw: f32,
        timeout_sec: f64,
        margin: f32,
        vehicle_name: &str,
    ) -> Result<()> {
        self.block_on(async {
            self.send_request("rotateToYaw", (yaw, timeout_sec, margin, vehicle_name))
                .await
        })
    }

    // ==================== CAR CONTROL ====================

    /// API Endpoint
    pub fn set_car_controls(&mut self, controls: CarControls, vehicle_name: &str) -> Result<()> {
        self.block_on(async {
            self.send_request("setCarControls", (controls, vehicle_name))
                .await
        })
    }

    /// API Endpoint
    pub fn get_car_controls(&mut self, vehicle_name: &str) -> Result<CarControls> {
        self.block_on(async { self.send_request("getCarControls", (vehicle_name,)).await })
    }

    /// API Endpoint
    pub fn get_car_state(&mut self, vehicle_name: &str) -> Result<CarState> {
        self.block_on(async { self.send_request("getCarState", (vehicle_name,)).await })
}
    }
