// SPDX-License-Identifier: Apache-2.0
use super::*;
use crate::device::DenoiseParam;

#[test]
fn a_background_render_walks_its_samples_and_then_has_nothing_left() {
    let mut rig = rig(background(), SMALL, 4);
    let work: Vec<RenderWork> = (0..4).map(|_| rig.work()).collect();
    let start: Vec<i32> = work.iter().map(|work| work.path_trace.start_sample).collect();
    let count: Vec<i32> = work.iter().map(|work| work.path_trace.sample_count).collect();
    let init: Vec<bool> = work.iter().map(|work| work.init_render_buffers).collect();
    assert_eq!(start, [0, 1, 2, 3]);
    assert_eq!(count, [1, 1, 1, 1]);
    assert_eq!(init, [true, false, false, false]);
    assert_eq!(work[0].step_list(), [RenderStep::InitRenderBuffer, RenderStep::PathTrace, RenderStep::UpdateDisplay]);
    assert_eq!(work[3].step_list(), [RenderStep::PathTrace, RenderStep::WriteTileBuffer, RenderStep::UpdateDisplay]);
    assert!(!rig.work().any());
    assert_eq!(rig.scheduler.state().end_render_time, 100.0);
}

#[test]
fn a_viewport_renders_small_first_and_then_at_full_size() {
    let mut rig = rig(viewport(), [1024, 512], 100);
    let small = rig.work();
    assert_eq!((small.resolution_divider, small.path_trace.start_sample, small.path_trace.sample_count), (8.0, 0, 4));
    let full = rig.work();
    assert_eq!((full.resolution_divider, full.path_trace.start_sample, full.path_trace.sample_count), (1.0, 0, 1));
    assert!(full.init_render_buffers);
}

#[test]
fn the_tile_is_written_once() {
    let mut rig = rig(background(), SMALL, 2);
    let written: Vec<bool> = (0..4).map(|_| rig.work().tile.write).collect();
    assert_eq!(written, [false, true, false, false]);
}

#[test]
fn cryptomatte_is_post_processed_with_the_last_sample() {
    let mut rig = rig(background(), SMALL, 2);
    rig.scheduler.set_need_schedule_cryptomatte(true);
    let asked: Vec<bool> = (0..3).map(|_| rig.work().cryptomatte_postprocess).collect();
    assert_eq!(asked, [false, true, false]);
}

#[test]
fn volume_guiding_is_denoised_at_powers_of_two_but_not_at_the_end() {
    let mut rig = rig(background(), SMALL, 4);
    let asked: Vec<bool> = (0..4).map(|_| rig.work().volume_guiding_denoise).collect();
    assert_eq!(asked, [true, true, false, false]);
}

#[test]
fn the_time_limit_ends_the_render_and_the_tile_is_still_written() {
    let mut rig = rig(background(), SMALL, 1000);
    rig.scheduler.set_time_limit(5.0);
    let first = rig.work();
    rig.scheduler.report_work_begin(&first);
    rig.clock.advance(6.0);
    let last = rig.work();
    assert_eq!(last.step_list(), [RenderStep::WriteTileBuffer, RenderStep::UpdateDisplay]);
    assert!(rig.scheduler.state().time_limit_reached);
    assert_eq!(rig.scheduler.state().end_render_time, 106.0);
    assert!(!rig.work().any());
}

#[test]
fn an_upscaling_denoiser_lets_the_path_tracer_render_smaller() {
    let mut rig = rig(background(), SMALL, 4);
    rig.scheduler.set_denoise_param(DenoiseParam { enabled: true, upscale_factor: 2.0, ..DenoiseParam::default() });
    let work = rig.work();
    assert_eq!((work.resolution_divider, work.denoised_resolution_divider), (2.0, 1.0));
}

#[test]
fn the_full_frame_is_written_after_the_last_of_many_tiles() {
    let mut rig = tiled(background(), [128, 64], [64, 64], 1);
    assert!(rig.tile.has_multiple_tile());
    assert_eq!(rig.work().step_list(), [RenderStep::InitRenderBuffer, RenderStep::PathTrace, RenderStep::WriteTileBuffer, RenderStep::UpdateDisplay]);
    // A tile is left, so the full frame waits.
    assert!(!rig.work().any());
    assert!(rig.tile.next());
    rig.scheduler.reset_for_next_tile();
    assert!(rig.work().tile.write);
    assert_eq!(rig.work().step_list(), [RenderStep::FinalizeFullBuffer]);
    assert!(!rig.work().any());
}

#[test]
fn a_render_with_no_sample_is_done_at_once() {
    let mut rig = rig(background(), SMALL, 0);
    assert!(rig.scheduler.done());
    assert_eq!(rig.work().path_trace.sample_count, 0);
}
