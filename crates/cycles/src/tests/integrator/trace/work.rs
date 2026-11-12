// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::integrator::*;
use crate::session::{BufferParam, RenderBuffer};

/// A work with only what the trait insists on.
struct Bare;

impl PathTraceWork for Bare {
    fn init_execution(&mut self) {}
    fn reset_render_buffer(&mut self, _param: &BufferParam) {}
    fn set_effective_buffer_param(&mut self, _big_tile: &BufferParam, _slice: &BufferParam, _denoised_big_tile: &BufferParam, _denoised_slice: &BufferParam) {}
    fn zero_render_buffer(&mut self) {}
    fn copy_to_render_buffer(&mut self, _buffer: &mut RenderBuffer) {}
    fn copy_from_render_buffer(&mut self, _buffer: &RenderBuffer) {}
    fn render_samples(&mut self, _start_sample: i32, _sample_count: i32, _sample_offset: i32) -> RenderStatistic {
        RenderStatistic::default()
    }
    fn adaptive_sampling_converge_filter_count_active(&mut self, _threshold: f32, _reset: bool) -> u32 {
        0
    }
    fn cryptomatte_postprocess(&mut self) {}
    fn denoise_volume_guiding_buffer(&mut self) {}
}

#[test]
fn a_work_that_measures_nothing_reports_a_full_device() {
    assert_eq!(Bare.render_samples(0, 1, 0).occupancy, 1.0);
}

#[test]
fn a_work_has_no_error_unless_it_says_so() {
    let mut work = Bare;
    work.deinit_execution();
    assert!(!work.has_error());
}
