// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/integrator/render_scheduler.h`
//! (`RenderScheduler::TimeWithAverage` and the six timers the scheduler
//! keeps).

/// `TimeWithAverage`: how long one kind of task took. The wall time is what
/// passed; the average estimates the task at the final resolution and is
/// what work is planned from.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TimeAverage {
    total_wall_time: f64,
    average_time_accumulator: f64,
    /// `num_average_times_`.
    average_count: i32,
    last_sample_time: f64,
}

impl TimeAverage {
    pub fn reset(&mut self) {
        *self = TimeAverage::default();
    }

    pub fn add_wall(&mut self, time: f64) {
        self.total_wall_time += time;
    }

    /// `add_average`: `time` covered `measurement_count` measurements.
    pub fn add_average(&mut self, time: f64, measurement_count: i32) {
        self.average_time_accumulator += time;
        self.average_count += measurement_count;
        self.last_sample_time = time / f64::from(measurement_count);
    }

    /// `get_wall`.
    pub fn wall(&self) -> f64 {
        self.total_wall_time
    }

    /// `get_average`: zero until something was measured.
    pub fn average(&self) -> f64 {
        match self.average_count {
            0 => 0.0,
            count => self.average_time_accumulator / f64::from(count),
        }
    }

    /// `get_last_sample_time`.
    pub fn last_sample_time(&self) -> f64 {
        self.last_sample_time
    }

    /// `reset_average`: the wall time and the last sample's time stay.
    pub fn reset_average(&mut self) {
        self.average_time_accumulator = 0.0;
        self.average_count = 0;
    }
}

/// The scheduler's timers, one for each kind of task.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TaskTime {
    pub path_trace: TimeAverage,
    pub adaptive_filter: TimeAverage,
    pub denoise: TimeAverage,
    pub display_update: TimeAverage,
    pub rebalance: TimeAverage,
    pub volume_guiding_denoise: TimeAverage,
}
