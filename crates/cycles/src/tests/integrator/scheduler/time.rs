// SPDX-License-Identifier: Apache-2.0
use crate::integrator::*;

#[test]
fn nothing_measured_averages_to_zero() {
    assert_eq!(TimeAverage::default().average(), 0.0);
}

#[test]
fn the_average_is_for_each_measurement() {
    let mut time = TimeAverage::default();
    time.add_average(4.0, 2);
    assert_eq!((time.average(), time.last_sample_time()), (2.0, 2.0));
    time.add_average(2.0, 2);
    assert_eq!((time.average(), time.last_sample_time()), (1.5, 1.0));
}

#[test]
fn resetting_the_average_keeps_the_wall_time_and_the_last_sample() {
    let mut time = TimeAverage::default();
    time.add_wall(3.0);
    time.add_average(4.0, 2);
    time.reset_average();
    assert_eq!((time.wall(), time.average(), time.last_sample_time()), (3.0, 0.0, 2.0));
    time.reset();
    assert_eq!(time, TimeAverage::default());
}

#[test]
fn the_task_timers_start_empty() {
    let time = TaskTime::default();
    assert_eq!(time.path_trace.wall() + time.denoise.wall() + time.rebalance.wall(), 0.0);
}
