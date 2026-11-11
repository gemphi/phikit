// SPDX-License-Identifier: Apache-2.0
use super::super::scheduler::{tiled, viewport};
use super::*;
use crate::device::DenoiseParam;

#[test]
fn a_render_goes_through_the_steps_in_the_pipeline_s_order() {
    let mut bench = single(rig(background(), SMALL, 2), 0.5);
    bench.round();
    assert_eq!(bench.take(), ["init", "buffer", "zero", "read", "trace 0+1", "display noisy", "progress", "deinit"]);
    bench.round();
    // The buffers are sized once: the second round goes straight to tracing.
    assert_eq!(bench.take(), ["init", "trace 1+1", "write host", "display noisy", "progress", "deinit"]);
    assert!(bench.trace.state().tile_written);
    assert!(!bench.rig.work().any());
}

#[test]
fn the_scheduler_is_told_the_time_and_the_occupancy_of_the_devices() {
    let rig = rig(background(), SMALL, 100);
    let log = Log::default();
    let mut busy = device(&rig, &log, 0.5);
    let mut idle = device(&rig, &log, 0.5);
    (busy.occupancy, idle.occupancy) = (0.75, 0.25);
    let mut bench = bench(vec![busy, idle], rig, log);
    bench.round();
    let scheduler = &bench.rig.scheduler;
    // One device after the other, half a second each.
    assert_eq!(scheduler.task_time().path_trace.wall(), 1.0);
    assert_eq!((scheduler.state().occupancy_sample_count, scheduler.state().occupancy), (1, 0.5));
    assert_eq!(scheduler.state().start_render_time, 100.0);
    let spent: Vec<f64> = bench.trace.balance().iter().map(|info| info.time_spent).collect();
    assert_eq!(spent, [0.5, 0.5]);
}

#[test]
fn a_device_with_an_error_traces_nothing() {
    let rig = rig(background(), SMALL, 2);
    let log = Log::default();
    let mut broken = device(&rig, &log, 0.5);
    broken.error = true;
    let mut bench = bench(vec![broken], rig, log);
    bench.round();
    assert!(!bench.take().iter().any(|line| line.starts_with("trace")));
}

/// A viewport with adaptive sampling, rendered up to the round whose work
/// filters; the device reports `active` open pixels, a call each.
fn filtering(threshold: f32, active: &[u32]) -> Bench {
    let mut rig = rig(viewport(), SMALL, 100);
    rig.scheduler.set_adaptive_sampling(AdaptiveSampling { enabled: true, adaptive_step: 2, min_sample: 0, threshold });
    let log = Log::default();
    let mut device = device(&rig, &log, 0.5);
    device.active = active.iter().copied().collect();
    let mut bench = bench(vec![device], rig, log);
    bench.round();
    bench.take();
    bench
}

fn filter_call(log: &[String]) -> Vec<&str> {
    log.iter().map(String::as_str).filter(|line| line.starts_with("filter")).collect()
}

#[test]
fn converged_pixels_go_on_with_a_lower_threshold_and_an_idle_device_once_more() {
    let mut bench = filtering(0.01, &[0, 100, 100]);
    assert!(bench.round().adaptive_sampling.filter);
    assert_eq!(filter_call(&bench.take()), ["filter 0.4 false", "filter 0.2 true", "filter 0.1 true"]);
}

#[test]
fn many_open_pixels_are_filtered_once() {
    let mut bench = filtering(0.01, &[20_000, 0]);
    bench.round();
    assert_eq!(filter_call(&bench.take()), ["filter 0.4 false"]);
}

#[test]
fn converging_at_the_final_threshold_ends_path_tracing_and_shows_the_result() {
    let mut bench = filtering(0.4, &[0]);
    bench.round();
    let log = bench.take();
    assert_eq!(filter_call(&log), ["filter 0.4 false"]);
    assert!(log.iter().any(|line| line == "display noisy"));
    assert!(bench.rig.scheduler.state().path_trace_finished);
}

#[test]
fn nothing_is_rendered_once_a_cancel_was_asked_for() {
    let mut bench = single(rig(background(), SMALL, 2), 0.5);
    bench.trace.request_cancel();
    bench.round();
    assert!(bench.take().is_empty());
}

#[test]
fn an_instant_cancel_stops_the_pipeline_after_path_tracing() {
    let mut bench = single(rig(background(), SMALL, 2), 0.5);
    bench.host.raise = Some(bench.trace.cancel_handle());
    bench.round();
    assert_eq!(bench.take(), ["init", "buffer", "zero", "read", "trace 0+1", "deinit"]);
    // Cancelled samples count toward the wall time alone.
    let time = bench.rig.scheduler.task_time().path_trace;
    assert_eq!((time.wall(), time.average()), (0.5, 0.0));
}

#[test]
fn the_user_s_cancel_lets_the_pipeline_finish_its_work() {
    let mut bench = single(rig(background(), SMALL, 2), 0.5);
    bench.host.cancelled = true;
    bench.round();
    // That cancel is the session's: it cuts the work down before the round.
    assert_eq!(bench.take(), ["init", "buffer", "zero", "read", "trace 0+1", "display noisy", "progress", "deinit"]);
    let time = bench.rig.scheduler.task_time().path_trace;
    assert_eq!((time.wall(), time.average()), (0.5, 0.0));
}

#[test]
fn two_devices_rebalance_when_one_is_slower() {
    let mut rig = rig(background(), SMALL, 100);
    rig.scheduler.set_need_schedule_rebalance(true);
    let log = Log::default();
    let mut bench = bench(vec![device(&rig, &log, 1.0), device(&rig, &log, 3.0)], rig, log);
    assert!(!bench.round().rebalance);
    bench.take();
    assert!(bench.round().rebalance);
    // The tile is copied out, sliced again for both devices, and copied back.
    assert_eq!(bench.take()[..8], ["init", "init", "copy out", "copy out", "buffer", "buffer", "copy in", "copy in"]);
    assert_eq!(bench.rig.scheduler.state().rebalance_change_count, 1);
    let weight: Vec<f64> = bench.trace.balance().iter().map(|info| info.weight).collect();
    assert!(weight[0] > weight[1], "{weight:?}");
}

#[test]
fn one_device_ignores_a_rebalance() {
    let mut rig = rig(background(), SMALL, 100);
    rig.scheduler.set_need_schedule_rebalance(true);
    let mut bench = single(rig, 1.0);
    bench.round();
    assert!(bench.round().rebalance);
    assert!(!bench.take().iter().any(|line| line == "copy out"));
    assert_eq!(bench.rig.scheduler.task_time().rebalance.wall(), 0.0);
}

/// A one-sample background render with the denoiser asked for.
fn denoising(denoiser: Option<bool>) -> Bench {
    let mut rig = rig(background(), SMALL, 1);
    rig.scheduler.set_denoise_param(DenoiseParam { enabled: true, ..DenoiseParam::default() });
    let mut bench = single(rig, 0.5);
    bench.host.denoiser = denoiser;
    bench
}

#[test]
fn a_denoised_result_is_timed_and_shown() {
    let mut bench = denoising(Some(true));
    bench.round();
    assert_eq!(bench.take()[4..], ["trace 0+1", "denoise 1", "write host", "display denoised", "progress", "deinit"]);
    assert!(bench.trace.has_denoised_result());
    assert_eq!(bench.rig.scheduler.task_time().denoise.wall(), 1.0);
}

#[test]
fn without_a_denoiser_the_noisy_result_is_shown_and_nothing_is_timed() {
    let mut bench = denoising(None);
    bench.round();
    assert!(bench.take().iter().any(|line| line == "display noisy"));
    assert!(!bench.trace.has_denoised_result());
    assert_eq!(bench.rig.scheduler.task_time().denoise.wall(), 0.0);
}

#[test]
fn a_tile_of_many_goes_to_the_tile_file_and_the_file_is_finished_at_the_end() {
    let mut bench = single(tiled(background(), [128, 64], [64, 64], 1), 0.5);
    bench.round();
    assert!(bench.take().iter().any(|line| line == "write disk"));
    assert!(bench.rig.tile.next());
    bench.rig.scheduler.reset_for_next_tile();
    bench.round();
    bench.take();
    bench.round();
    assert_eq!(bench.take(), ["init", "progress", "finish", "deinit"]);
}

#[test]
fn cryptomatte_is_post_processed_with_the_last_sample_when_the_film_has_it() {
    let mut bench = single(rig(background(), SMALL, 2), 0.5);
    bench.trace.need_cryptomatte = true;
    // The first round tells the scheduler; the last sample's work has it.
    bench.round();
    assert!(!bench.take().iter().any(|line| line == "cryptomatte"));
    bench.round();
    assert!(bench.take().iter().any(|line| line == "cryptomatte"));
}

#[test]
fn with_volumes_a_work_stops_at_the_next_power_of_two() {
    let mut bench = single(rig(background(), SMALL, 1000), 0.001);
    bench.trace.has_volume = true;
    let count: Vec<i32> = (0..5).map(|_| bench.round().path_trace.sample_count).collect();
    assert_eq!(count, [1, 1, 2, 4, 8]);
    assert!(bench.take().iter().any(|line| line == "guiding"));
}
