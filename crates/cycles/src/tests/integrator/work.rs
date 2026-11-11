// SPDX-License-Identifier: Apache-2.0
use crate::integrator::*;

#[test]
fn empty_work_is_no_work() {
    let work = RenderWork::default();
    assert!(!work.any() && work.step_list().is_empty());
    assert!(work.display.use_denoised_result && work.resolution_divider == 1.0);
}

#[test]
fn the_steps_come_in_the_order_of_the_pipeline() {
    let work = RenderWork {
        init_render_buffers: true,
        path_trace: WorkPathTrace { start_sample: 0, sample_count: 4, sample_offset: 0 },
        adaptive_sampling: WorkAdaptive { filter: true, threshold: 0.01, reset: false },
        tile: WorkTile { write: true, denoise: true },
        display: WorkDisplay { update: true, use_denoised_result: true },
        ..RenderWork::default()
    };
    assert!(work.any());
    assert_eq!(
        work.step_list(),
        [
            RenderStep::InitRenderBuffer,
            RenderStep::PathTrace,
            RenderStep::AdaptiveSample,
            RenderStep::Denoise,
            RenderStep::WriteTileBuffer,
            RenderStep::UpdateDisplay,
        ]
    );
}

#[test]
fn a_display_update_alone_is_work() {
    let work = RenderWork { display: WorkDisplay { update: true, use_denoised_result: false }, ..RenderWork::default() };
    assert!(work.any());
    assert_eq!(work.step_list(), [RenderStep::UpdateDisplay]);
}
