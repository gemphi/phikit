//! Simulation clock and time management
//!
//! Provides AirSim-compatible timing:
//! - Real-time and steppable clock
//! - Scalable simulation speed
//! - Pause/resume functionality
//! - Time synchronization

use std::time::{Duration, Instant};

/// Clock types for different simulation modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ClockType {
    /// Real-time clock (wall clock)
    RealTime,
    /// Steppable clock (manual step control)
    Steppable,
    /// Scalable clock (variable speed)
    Scalable,
}

impl Default for ClockType {
    fn default() -> Self {
        ClockType::RealTime
    }
}

/// Simulation clock for managing time
///
/// Mirrors AirSim's ClockBase and ScalableClock
#[derive(Debug, Clone)]
pub struct SimulationClock {
    clock_type: ClockType,
    /// Current simulation time in seconds
    current_time: f64,
    /// Start time (for elapsed calculation)
    start_time: f64,
    /// Last update time
    last_update: f64,
    /// Time scale factor (1.0 = real-time, 2.0 = 2x speed)
    scale: f64,
    /// Whether clock is paused
    paused: bool,
    /// Step size for steppable clock (seconds)
    step_size: f64,
    /// Real-time clock start instant (for real-time mode)
    real_start: Option<Instant>,
    /// Total elapsed simulation time
    elapsed_sim_time: f64,
    /// Total elapsed real time
    elapsed_real_time: f64,
}

impl Default for SimulationClock {
    fn default() -> Self {
        Self::new_realtime()
    }
}

impl SimulationClock {
    /// Create new real-time clock
    pub fn new_realtime() -> Self {
        Self {
            clock_type: ClockType::RealTime,
            current_time: 0.0,
            start_time: 0.0,
            last_update: 0.0,
            scale: 1.0,
            paused: false,
            step_size: 0.01,
            real_start: Some(Instant::now()),
            elapsed_sim_time: 0.0,
            elapsed_real_time: 0.0,
        }
    }

    /// Create new steppable clock
    pub fn new_steppable(step_size: f64) -> Self {
        Self {
            clock_type: ClockType::Steppable,
            current_time: 0.0,
            start_time: 0.0,
            last_update: 0.0,
            scale: 1.0,
            paused: true, // Step clocks start paused
            step_size,
            real_start: None,
            elapsed_sim_time: 0.0,
            elapsed_real_time: 0.0,
        }
    }

    /// Create new scalable clock
    pub fn new_scalable(scale: f64) -> Self {
        Self {
            clock_type: ClockType::Scalable,
            current_time: 0.0,
            start_time: 0.0,
            last_update: 0.0,
            scale: scale.max(0.0),
            paused: false,
            step_size: 0.01,
            real_start: Some(Instant::now()),
            elapsed_sim_time: 0.0,
            elapsed_real_time: 0.0,
        }
    }

    /// Get current simulation time
    pub fn now(&self) -> f64 {
        self.current_time
    }

    /// Get clock type
    pub fn clock_type(&self) -> ClockType {
        self.clock_type
    }

    /// Get time scale
    pub fn scale(&self) -> f64 {
        self.scale
    }

    /// Set time scale (for scalable clocks)
    pub fn set_scale(&mut self, scale: f64) {
        if self.clock_type == ClockType::Scalable {
            self.scale = scale.max(0.0);
        }
    }

    /// Check if paused
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Pause the clock
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resume the clock
    pub fn resume(&mut self) {
        self.paused = false;
        if self.clock_type == ClockType::RealTime || self.clock_type == ClockType::Scalable {
            self.real_start = Some(Instant::now());
        }
    }

    /// Toggle pause state
    pub fn toggle_pause(&mut self) {
        if self.paused {
            self.resume();
        } else {
            self.pause();
        }
    }

    /// Step forward (for steppable clocks)
    pub fn step(&mut self, steps: u32) {
        if self.clock_type == ClockType::Steppable {
            let dt = self.step_size * steps as f64;
            self.current_time += dt;
            self.elapsed_sim_time += dt;
        }
    }

    /// Step forward by specific time delta
    pub fn step_by(&mut self, delta: f64) {
        if self.clock_type == ClockType::Steppable {
            self.current_time += delta;
            self.elapsed_sim_time += delta;
        }
    }

    /// Update the clock (call each simulation frame)
    pub fn update(&mut self) -> f64 {
        if self.paused {
            return 0.0; // No time elapsed
        }

        let dt = match self.clock_type {
            ClockType::RealTime => {
                if let Some(start) = self.real_start {
                    let elapsed = start.elapsed().as_secs_f64();
                    self.real_start = Some(Instant::now());
                    elapsed
                } else {
                    0.0
                }
            }
            ClockType::Scalable => {
                if let Some(start) = self.real_start {
                    let real_dt = start.elapsed().as_secs_f64();
                    self.real_start = Some(Instant::now());
                    self.elapsed_real_time += real_dt;
                    real_dt * self.scale
                } else {
                    0.0
                }
            }
            ClockType::Steppable => {
                // Steppable clock only advances via step() calls
                0.0
            }
        };

        self.current_time += dt;
        self.elapsed_sim_time += dt;
        self.last_update = self.current_time;

        dt
    }

    /// Get elapsed simulation time since start
    pub fn elapsed_sim_time(&self) -> f64 {
        self.elapsed_sim_time
    }

    /// Get elapsed real time since start
    pub fn elapsed_real_time(&self) -> f64 {
        self.elapsed_real_time
    }

    /// Reset clock to initial state
    pub fn reset(&mut self) {
        self.current_time = self.start_time;
        self.last_update = self.start_time;
        self.elapsed_sim_time = 0.0;
        self.elapsed_real_time = 0.0;
        self.paused = false;
        if self.clock_type == ClockType::RealTime || self.clock_type == ClockType::Scalable {
            self.real_start = Some(Instant::now());
        }
    }

    /// Set current time directly (for initialization/synchronization)
    pub fn set_time(&mut self, time: f64) {
        self.current_time = time;
        self.last_update = time;
    }

    /// Get step size (for steppable clocks)
    pub fn step_size(&self) -> f64 {
        self.step_size
    }

    /// Set step size
    pub fn set_step_size(&mut self, size: f64) {
        self.step_size = size.max(0.0001);
    }

    /// Sleep to maintain real-time rate (for real-time clocks)
    pub fn sleep_for_real_time(&self, target_dt: f64) {
        if self.clock_type == ClockType::RealTime && !self.paused {
            let sleep_duration = Duration::from_secs_f64(target_dt.max(0.0));
            std::thread::sleep(sleep_duration);
        }
    }
}

/// Time provider trait for dependency injection
pub trait ClockProvider: Send + Sync {
    /// Get current simulation time
    fn now(&self) -> f64;
    /// Get time delta since last call
    fn delta_time(&self) -> f64;
    /// Check if paused
    fn is_paused(&self) -> bool;
}

impl ClockProvider for SimulationClock {
    fn now(&self) -> f64 {
        self.now()
    }

    fn delta_time(&self) -> f64 {
        // Return approximate delta based on scale
        match self.clock_type {
            ClockType::RealTime => 0.016, // ~60 FPS
            ClockType::Scalable => 0.016 * self.scale,
            ClockType::Steppable => self.step_size,
        }
    }

    fn is_paused(&self) -> bool {
        self.is_paused()
    }
}

/// Wall clock for real-time synchronization
#[derive(Debug, Clone, Copy)]
pub struct WallClock;

impl WallClock {
    /// Get current system time in seconds since epoch
    pub fn now() -> f64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64()
    }

    /// Get monotonic time (for precise intervals)
    pub fn monotonic() -> f64 {
        Instant::now().elapsed().as_secs_f64()
    }

    /// Sleep for duration
    pub fn sleep(seconds: f64) {
        if seconds > 0.0 {
            std::thread::sleep(Duration::from_secs_f64(seconds));
        }
    }
}

/// Time conversion utilities
pub mod time_utils {
    /// Convert seconds to formatted string (HH:MM:SS.ms)
    pub fn format_time(seconds: f64) -> String {
        let hours = (seconds / 3600.0) as u32;
        let minutes = ((seconds % 3600.0) / 60.0) as u32;
        let secs = (seconds % 60.0) as u32;
        let millis = ((seconds % 1.0) * 1000.0) as u32;

        format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, secs, millis)
    }

    /// Convert frame count to time at given FPS
    pub fn frames_to_time(frames: u64, fps: f64) -> f64 {
        frames as f64 / fps
    }

    /// Convert time to frame count at given FPS
    pub fn time_to_frames(time: f64, fps: f64) -> u64 {
        (time * fps) as u64
    }

    /// Interpolate between two times
    pub fn lerp_time(t1: f64, t2: f64, alpha: f64) -> f64 {
        t1 + (t2 - t1) * alpha.clamp(0.0, 1.0)
    }
}

/// Timer for tracking intervals
#[derive(Debug, Clone, Copy)]
pub struct Timer {
    duration: f64,
    elapsed: f64,
    loop_mode: bool,
    active: bool,
}

impl Timer {
    /// Create new one-shot timer
    pub fn new(duration: f64) -> Self {
        Self {
            duration,
            elapsed: 0.0,
            loop_mode: false,
            active: false,
        }
    }

    /// Create new looping timer
    pub fn new_looping(duration: f64) -> Self {
        Self {
            duration,
            elapsed: 0.0,
            loop_mode: true,
            active: false,
        }
    }

    /// Start the timer
    pub fn start(&mut self) {
        self.active = true;
        self.elapsed = 0.0;
    }

    /// Stop the timer
    pub fn stop(&mut self) {
        self.active = false;
    }

    /// Reset the timer
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
    }

    /// Update timer
    pub fn update(&mut self, dt: f64) -> bool {
        if !self.active {
            return false;
        }

        self.elapsed += dt;

        if self.elapsed >= self.duration {
            if self.loop_mode {
                self.elapsed -= self.duration;
                true
            } else {
                self.active = false;
                true
            }
        } else {
            false
        }
    }

    /// Check if timer is running
    pub fn is_running(&self) -> bool {
        self.active
    }

    /// Get progress (0.0 to 1.0)
    pub fn progress(&self) -> f64 {
        if self.duration > 0.0 {
            (self.elapsed / self.duration).clamp(0.0, 1.0)
        } else {
            1.0
        }
    }

    /// Get remaining time
    pub fn remaining(&self) -> f64 {
        (self.duration - self.elapsed).max(0.0)
    }
}

/// Frame rate limiter
#[derive(Debug, Clone, Copy)]
pub struct FrameLimiter {
    target_fps: f64,
    target_frame_time: f64,
    last_frame_time: f64,
}

impl FrameLimiter {
    /// Create new frame limiter
    pub fn new(target_fps: f64) -> Self {
        let target_frame_time = 1.0 / target_fps.max(1.0);
        Self {
            target_fps,
            target_frame_time,
            last_frame_time: 0.0,
        }
    }

    /// Get target FPS
    pub fn target_fps(&self) -> f64 {
        self.target_fps
    }

    /// Set target FPS
    pub fn set_target_fps(&mut self, fps: f64) {
        self.target_fps = fps.max(1.0);
        self.target_frame_time = 1.0 / self.target_fps;
    }

    /// Wait to maintain target frame rate
    pub fn limit(&mut self, clock: &mut SimulationClock) -> f64 {
        let elapsed = clock.update();

        if self.target_fps > 0.0 && elapsed < self.target_frame_time {
            let sleep_time = self.target_frame_time - elapsed;
            clock.sleep_for_real_time(sleep_time);
            elapsed + sleep_time
        } else {
            elapsed
        }
    }
}
