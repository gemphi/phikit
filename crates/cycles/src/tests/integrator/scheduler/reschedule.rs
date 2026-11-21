// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use super::*;

fn adaptive(threshold: f32) -> AdaptiveSampling {
    AdaptiveSampling { enabled: true, adaptive_step: 4, min_sample: 0, threshold }
}

#[test]
fn an_idle_device_halves_the_threshold_down_to_the_final_one() {
    let mut rig = rig(viewport(), SMALL, 100);
    rig.scheduler.set_adaptive_sampling(adaptive(0.01));
    let mut work = rig.work();
    let mut threshold = Vec::new();
    while rig.scheduler.reschedule_on_idle(&mut work) {
        assert!(work.adaptive_sampling.reset);
        threshold.push(work.adaptive_sampling.threshold);
    }
    assert_eq!(threshold, [0.2, 0.1, 0.05, 0.025, 0.0125, 0.01]);
}

#[test]
fn a_background_render_keeps_its_threshold() {
    let mut rig = rig(background(), SMALL, 100);
    rig.scheduler.set_adaptive_sampling(adaptive(0.01));
    let mut work = rig.work();
    assert!(!rig.scheduler.reschedule_on_idle(&mut work));
    assert!(!work.adaptive_sampling.reset);
}

#[test]
fn without_adaptive_sampling_there_is_nothing_to_lower() {
    let mut rig = rig(viewport(), SMALL, 100);
    let mut work = rig.work();
    assert!(!rig.scheduler.reschedule_on_idle(&mut work));
}

#[test]
fn converging_at_a_higher_threshold_goes_on_with_a_lower_one() {
    let mut rig = rig(viewport(), SMALL, 100);
    rig.scheduler.set_adaptive_sampling(adaptive(0.01));
    let mut work = rig.work();
    assert!(rig.scheduler.reschedule_on_converge(&mut work, &rig.tile));
    assert_eq!(work.adaptive_sampling.threshold, 0.2);
    assert!(!rig.scheduler.state().path_trace_finished);
}

#[test]
fn converging_at_the_final_threshold_ends_path_tracing() {
    let mut rig = rig(viewport(), SMALL, 100);
    rig.scheduler.set_adaptive_sampling(adaptive(0.4));
    let mut work = rig.work();
    assert!(!rig.scheduler.reschedule_on_converge(&mut work, &rig.tile));
    assert!(rig.scheduler.state().path_trace_finished);
    assert!(work.display.update);
    assert!(rig.scheduler.done());
    // What is left is writing the tile.
    assert_eq!(rig.work().step_list(), [RenderStep::WriteTileBuffer, RenderStep::UpdateDisplay]);
}

#[test]
fn converging_while_the_view_is_small_changes_nothing() {
    let mut rig = rig(viewport(), [1024, 512], 100);
    rig.scheduler.set_adaptive_sampling(adaptive(0.4));
    let mut work = rig.work();
    assert!(!rig.scheduler.reschedule_on_converge(&mut work, &rig.tile));
    assert!(!rig.scheduler.state().path_trace_finished);
}

#[test]
fn a_cancel_before_any_sample_leaves_only_the_full_write() {
    let mut rig = rig(background(), SMALL, 100);
    let mut work = rig.work();
    rig.scheduler.reschedule_on_cancel(&mut work);
    assert_eq!(rig.scheduler.rendered_sample_count(), 0);
    assert_eq!(work.step_list(), [RenderStep::FinalizeFullBuffer]);
}

#[test]
fn a_cancel_after_samples_writes_and_shows_what_was_rendered() {
    let mut rig = rig(background(), SMALL, 100);
    rig.work();
    rig.work();
    let mut work = rig.work();
    rig.scheduler.reschedule_on_cancel(&mut work);
    assert_eq!(rig.scheduler.rendered_sample_count(), 2);
    assert_eq!(work.path_trace.sample_count, 0);
    assert_eq!(work.step_list(), [RenderStep::WriteTileBuffer, RenderStep::UpdateDisplay, RenderStep::FinalizeFullBuffer]);
}
