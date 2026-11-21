//! The world's clock: one resource, advanced first in every step.
//!
//! Apple's `SceneUpdateContext.deltaTime` (RF:14752) is the only time it
//! gives a system. phiecs adds a count, so a row of history can name the
//! step it belongs to.

/// Where the world is in time. `tick` counts steps, `time` sums their `dt`.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Clock {
    /// Steps taken. 0 before the first `step`; 1 while the first one runs.
    pub tick: u64,
    /// Seconds simulated: the sum of every step's `dt`.
    pub time: f64,
    /// The `dt` of the step now running (or last run).
    pub dt: f32,
    /// Ticks a second the world is meant to run at; a frame rate to map
    /// a Blender frame onto a tick.
    pub fps: f32,
}

impl Default for Clock {
    fn default() -> Self {
        Self { tick: 0, time: 0.0, dt: 0.0, fps: 60.0 }
    }
}

impl Clock {
    /// A clock at tick 0 that means `fps` ticks a second.
    pub fn at_fps(fps: f32) -> Self {
        Self { fps, ..Self::default() }
    }

    /// Moves one step on: `tick + 1`, `time + dt`.
    pub fn advance(&mut self, dt: f32) {
        self.tick += 1;
        self.time += f64::from(dt);
        self.dt = dt;
    }

    /// The tick a frame is, counting from frame `start`, for a scene drawn
    /// at `frame_rate` frames a second: `(frame - start) x fps / frame_rate`.
    pub fn tick_of_frame(&self, frame: f32, start: f32, frame_rate: f32) -> u64 {
        ((frame - start) * self.fps / frame_rate).round().max(0.0) as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advance_counts_and_sums() {
        let mut c = Clock::default();
        c.advance(0.5);
        c.advance(0.25);
        assert_eq!((c.tick, c.time, c.dt), (2, 0.75, 0.25));
    }

    #[test]
    fn a_frame_maps_to_a_tick() {
        let c = Clock::at_fps(48.0);
        assert_eq!(c.tick_of_frame(25.0, 1.0, 24.0), 48);
        assert_eq!(c.tick_of_frame(0.0, 1.0, 24.0), 0, "before the start is tick 0");
    }
}
