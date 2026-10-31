//! Recording system for simulation data and images
//!
//! Provides recording capabilities for:
//! - Camera images (RGB, depth, segmentation)
//! - Sensor data (IMU, GPS, barometer, magnetometer, distance, lidar)
//! - Vehicle pose and kinematics
//! - Timestamps and metadata

use crate::common::{DistanceSensorData, ImageResponse, Pose, TimePoint};
use crate::sensors::{BarometerOutput, GpsOutput, ImuOutput, MagnetometerOutput};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Recording configuration
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RecordingConfig {
    /// Whether recording is enabled
    pub enabled: bool,
    /// Output directory for recordings
    pub output_dir: PathBuf,
    /// Record images
    pub record_images: bool,
    /// Record sensor data
    pub record_sensor_data: bool,
    /// Record pose/kinematics
    pub record_pose: bool,
    /// Image capture interval (every N frames)
    pub image_interval: u32,
    /// Sensor data interval (every N updates)
    pub sensor_interval: u32,
    /// Maximum recording duration in seconds (0 = unlimited)
    pub max_duration_secs: u64,
    /// Maximum file size in MB (0 = unlimited)
    pub max_file_size_mb: u64,
    /// Compress images (PNG) vs raw (PPM)
    pub compress_images: bool,
}

impl Default for RecordingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            output_dir: PathBuf::from("recordings"),
            record_images: true,
            record_sensor_data: true,
            record_pose: true,
            image_interval: 1,
            sensor_interval: 1,
            max_duration_secs: 0,
            max_file_size_mb: 0,
            compress_images: true,
        }
    }
}

impl RecordingConfig {
    /// Create config with output directory
    pub fn with_output_dir(path: impl AsRef<Path>) -> Self {
        Self {
            output_dir: path.as_ref().to_path_buf(),
            ..Default::default()
        }
    }

    /// Enable all recording options
    pub fn enable_all(mut self) -> Self {
        self.enabled = true;
        self.record_images = true;
        self.record_sensor_data = true;
        self.record_pose = true;
        self
    }
}

/// Single recorded frame data
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RecordedFrame {
    /// Frame number
    pub frame_number: u64,
    /// Timestamp
    pub timestamp: TimePoint,
    /// Vehicle poses by name
    pub vehicle_poses: HashMap<String, Pose>,
    /// Images by camera name
    pub images: HashMap<String, ImageResponse>,
    /// IMU data by sensor name
    pub imu_data: HashMap<String, ImuOutput>,
    /// GPS data by sensor name
    pub gps_data: HashMap<String, GpsOutput>,
    /// Barometer data by sensor name
    pub barometer_data: HashMap<String, BarometerOutput>,
    /// Magnetometer data by sensor name
    pub magnetometer_data: HashMap<String, MagnetometerOutput>,
    /// Distance sensor data by sensor name
    pub distance_data: HashMap<String, DistanceSensorData>,
    /// Lidar point clouds by sensor name (stored as bytes or path)
    pub lidar_data: HashMap<String, Vec<u8>>,
}

impl RecordedFrame {
    /// Create new empty frame
    pub fn new(frame_number: u64, timestamp: TimePoint) -> Self {
        Self {
            frame_number,
            timestamp,
            vehicle_poses: HashMap::new(),
            images: HashMap::new(),
            imu_data: HashMap::new(),
            gps_data: HashMap::new(),
            barometer_data: HashMap::new(),
            magnetometer_data: HashMap::new(),
            distance_data: HashMap::new(),
            lidar_data: HashMap::new(),
        }
    }

    /// Check if frame has any data
    pub fn is_empty(&self) -> bool {
        self.vehicle_poses.is_empty()
            && self.images.is_empty()
            && self.imu_data.is_empty()
            && self.gps_data.is_empty()
            && self.barometer_data.is_empty()
            && self.magnetometer_data.is_empty()
            && self.distance_data.is_empty()
            && self.lidar_data.is_empty()
    }
}

/// Recording session state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingState {
    /// Not recording
    Stopped,
    /// Recording in progress
    Recording,
    /// Paused
    Paused,
    /// Error occurred
    Error,
}

impl Default for RecordingState {
    fn default() -> Self {
        RecordingState::Stopped
    }
}

/// Recording statistics
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RecordingStats {
    /// Total frames recorded
    pub total_frames: u64,
    /// Total images recorded
    pub total_images: u64,
    /// Total sensor readings recorded
    pub total_sensor_readings: u64,
    /// Recording duration in seconds
    pub duration_secs: f64,
    /// Current file size in bytes
    pub current_file_size: u64,
    /// Start timestamp
    pub start_time: Option<TimePoint>,
}

/// Recording session
#[derive(Debug)]
pub struct RecordingSession {
    config: RecordingConfig,
    state: RecordingState,
    stats: RecordingStats,
    current_frame: u64,
    frames_buffer: Vec<RecordedFrame>,
    buffer_size: usize,
}

impl RecordingSession {
    /// Create new recording session
    pub fn new(config: RecordingConfig) -> Self {
        let buffer_size = if config.enabled { 100 } else { 0 };
        Self {
            config,
            state: RecordingState::Stopped,
            stats: RecordingStats::default(),
            current_frame: 0,
            frames_buffer: Vec::with_capacity(buffer_size),
            buffer_size,
        }
    }

    /// Start recording
    pub fn start(&mut self) -> Result<(), RecordingError> {
        if self.state == RecordingState::Recording {
            return Err(RecordingError::AlreadyRecording);
        }

        self.state = RecordingState::Recording;
        self.stats.start_time = Some(0); // Would use actual time
        self.current_frame = 0;
        self.frames_buffer.clear();

        Ok(())
    }

    /// Stop recording
    pub fn stop(&mut self) -> Result<Vec<RecordedFrame>, RecordingError> {
        if self.state != RecordingState::Recording && self.state != RecordingState::Paused {
            return Err(RecordingError::NotRecording);
        }

        self.state = RecordingState::Stopped;

        // Return buffered frames
        let frames = std::mem::take(&mut self.frames_buffer);
        Ok(frames)
    }

    /// Pause recording
    pub fn pause(&mut self) -> Result<(), RecordingError> {
        if self.state != RecordingState::Recording {
            return Err(RecordingError::NotRecording);
        }

        self.state = RecordingState::Paused;
        Ok(())
    }

    /// Resume recording
    pub fn resume(&mut self) -> Result<(), RecordingError> {
        if self.state != RecordingState::Paused {
            return Err(RecordingError::NotPaused);
        }

        self.state = RecordingState::Recording;
        Ok(())
    }

    /// Record a frame
    pub fn record_frame(&mut self, frame: RecordedFrame) -> Result<(), RecordingError> {
        if self.state != RecordingState::Recording {
            return Err(RecordingError::NotRecording);
        }

        self.current_frame += 1;
        self.stats.total_frames += 1;

        if !frame.images.is_empty() {
            self.stats.total_images += frame.images.len() as u64;
        }

        let sensor_count = frame.imu_data.len()
            + frame.gps_data.len()
            + frame.barometer_data.len()
            + frame.magnetometer_data.len()
            + frame.distance_data.len()
            + frame.lidar_data.len();
        self.stats.total_sensor_readings += sensor_count as u64;

        // Buffer frame (in real implementation, would write to disk)
        if self.frames_buffer.len() >= self.buffer_size {
            // Flush oldest frames
            self.flush_buffer();
        }

        self.frames_buffer.push(frame);

        Ok(())
    }

    /// Get current state
    pub fn state(&self) -> RecordingState {
        self.state
    }

    /// Check if recording
    pub fn is_recording(&self) -> bool {
        self.state == RecordingState::Recording
    }

    /// Get statistics
    pub fn stats(&self) -> &RecordingStats {
        &self.stats
    }

    /// Get configuration
    pub fn config(&self) -> &RecordingConfig {
        &self.config
    }

    /// Flush buffer to storage (stub)
    fn flush_buffer(&mut self) {
        // In real implementation: write to disk, database, or stream
        // For now, just clear half the buffer
        let keep_count = self.buffer_size / 2;
        self.frames_buffer
            .drain(0..self.frames_buffer.len() - keep_count);
    }
}

impl Default for RecordingSession {
    fn default() -> Self {
        Self::new(RecordingConfig::default())
    }
}

/// Recording errors
#[derive(Debug, thiserror::Error)]
pub enum RecordingError {
    /// Not currently recording
    #[error("Not recording")]
    NotRecording,
    /// Already recording
    #[error("Already recording")]
    AlreadyRecording,
    /// Not paused
    #[error("Not paused")]
    NotPaused,
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
    /// Config error
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Recording file format
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RecordingFormat {
    /// AirSim native format (binary)
    AirSim,
    /// CSV for sensor data
    Csv,
    /// JSON for metadata
    Json,
    /// ROS bag format
    RosBag,
}

impl Default for RecordingFormat {
    fn default() -> Self {
        RecordingFormat::AirSim
    }
}

/// Recording writer trait
pub trait RecordingWriter: Send {
    /// Write a frame
    fn write_frame(&mut self, frame: &RecordedFrame) -> Result<(), RecordingError>;
    /// Flush any buffered data
    fn flush(&mut self) -> Result<(), RecordingError>;
    /// Close the writer
    fn close(&mut self) -> Result<(), RecordingError>;
}

/// Simple file-based recording writer
pub struct FileRecordingWriter {
    base_path: PathBuf,
    format: RecordingFormat,
    frame_count: u64,
}

impl FileRecordingWriter {
    /// Create new file writer
    pub fn new(base_path: impl AsRef<Path>, format: RecordingFormat) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
            format,
            frame_count: 0,
        }
    }
}

impl RecordingWriter for FileRecordingWriter {
    fn write_frame(&mut self, _frame: &RecordedFrame) -> Result<(), RecordingError> {
        // Stub: would write to file
        self.frame_count += 1;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), RecordingError> {
        Ok(())
    }

    fn close(&mut self) -> Result<(), RecordingError> {
        Ok(())
    }
}
