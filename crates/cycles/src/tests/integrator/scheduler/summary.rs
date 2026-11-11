// SPDX-License-Identifier: Apache-2.0
use super::*;
use crate::device::{DenoiseParam, DenoiserPass};

/// A 4-sample background render that took 2 seconds, half a second of it
/// path tracing the first sample.
fn finished(setup: impl FnOnce(&mut Rig)) -> String {
    let mut rig = rig(background(), SMALL, 4);
    setup(&mut rig);
    let first = rig.work();
    rig.scheduler.report_work_begin(&first);
    rig.scheduler.report_path_trace_time(&first, 0.5, false);
    rig.clock.advance(2.0);
    while rig.work().any() {}
    rig.scheduler.full_report()
}

/// A row of the time table as Cycles prints it: `"  %20s %20f %20f\n"`.
fn row(name: &str, wall: &str, average: &str) -> String {
    let pad = |cell: &str| " ".repeat(20 - cell.len()) + cell;
    format!("  {} {} {}\n", pad(name), pad(wall), pad(average))
}

#[test]
fn the_report_of_a_plain_background_render() {
    let report = finished(|_| {});
    let expect = [
        "\nRender Scheduler Summary\n\n",
        "Mode: Background\n",
        "Resolution: 64x64\n",
        "\nAdaptive sampling:\n  Use: False\n",
        "\nDenoiser:\n  Use: False\n",
        "\nTime (in seconds):\n",
        &row("", "Wall", "Average"),
        &row("Path Tracing", "0.500000", "0.500000"),
        &row("Display Update", "0.000000", "0.000000"),
        "\n  Total: 0.500000\n",
        "\nRendered 4 samples in 2.000000 seconds\n",
        "Average time per sample: 0.500000 seconds\n",
    ];
    assert_eq!(report, expect.concat());
}

#[test]
fn the_report_names_the_denoiser_and_its_passes() {
    let pass_mask = DenoiserPass::ALBEDO | DenoiserPass::NORMAL | DenoiserPass::DEPTH;
    let report = finished(|rig| rig.scheduler.set_denoise_param(DenoiseParam { enabled: true, start_sample: 2, pass_mask, ..DenoiseParam::default() }));
    assert!(report.contains("\nDenoiser:\n  Use: True\n  Type: OpenImageDenoise\n  Start Sample: 2\n  Passes: Color, Albedo, Normal, Depth\n"), "{report}");
    assert!(report.contains(&row("Denoiser", "0.000000", "0.000000")), "{report}");
}

#[test]
fn the_report_of_adaptive_sampling_has_no_time_per_sample() {
    let adaptive = AdaptiveSampling { enabled: true, adaptive_step: 4, min_sample: 11, threshold: 0.01 };
    let report = finished(|rig| rig.scheduler.set_adaptive_sampling(adaptive));
    assert!(report.contains("\nAdaptive sampling:\n  Use: True\n  Step: 4\n  Min Samples: 11\n  Threshold: 0.010000\n"), "{report}");
    assert!(report.contains(&row("Adaptive Filter", "0.000000", "0.000000")), "{report}");
    assert!(!report.contains("Average time per sample"), "{report}");
}

#[test]
fn the_report_counts_rebalances_once_one_was_asked_for() {
    let report = finished(|rig| rig.scheduler.set_need_schedule_rebalance(true));
    assert!(report.contains("\nRebalancer:\n  Number of requested rebalances: 1\n  Number of performed rebalances: 0\n"), "{report}");
    assert!(report.contains(&row("Rebalance", "0.000000", "0.000000")), "{report}");
}

#[test]
fn the_report_names_the_mode() {
    let mode = |param| rig(param, SMALL, 4).scheduler.full_report();
    assert!(mode(headless()).contains("Mode: Headless\n"));
    assert!(mode(viewport()).contains("Mode: Interactive\n"));
}
