// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/integrator/render_scheduler.h` (`RenderWork`)
//! and the order of `PathTrace::render_pipeline`
//! (`integrator/path_trace.cpp:184-256`). The scheduler fills a
//! `RenderWork` in (`scheduler/`); the path tracer carries it out (`trace/`).

/// `RenderWork::path_trace`: the samples to trace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WorkPathTrace {
    pub start_sample: i32,
    /// `num_samples`.
    pub sample_count: i32,
    pub sample_offset: i32,
}

/// `RenderWork::adaptive_sampling`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct WorkAdaptive {
    pub filter: bool,
    pub threshold: f32,
    pub reset: bool,
}

/// `RenderWork::tile`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WorkTile {
    pub write: bool,
    pub denoise: bool,
}

/// `RenderWork::display`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkDisplay {
    pub update: bool,
    pub use_denoised_result: bool,
}

impl Default for WorkDisplay {
    fn default() -> Self {
        Self { update: false, use_denoised_result: true }
    }
}

/// `RenderWork`: one unit of work the scheduler hands the path tracer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderWork {
    pub resolution_divider: f32,
    pub denoised_resolution_divider: f32,
    pub init_render_buffers: bool,
    pub path_trace: WorkPathTrace,
    pub adaptive_sampling: WorkAdaptive,
    /// `cryptomatte.postprocess`.
    pub cryptomatte_postprocess: bool,
    pub tile: WorkTile,
    /// `full.write`.
    pub full_write: bool,
    pub display: WorkDisplay,
    pub rebalance: bool,
    pub volume_guiding_denoise: bool,
}

impl Default for RenderWork {
    fn default() -> Self {
        Self {
            resolution_divider: 1.0,
            denoised_resolution_divider: 1.0,
            init_render_buffers: false,
            path_trace: WorkPathTrace::default(),
            adaptive_sampling: WorkAdaptive::default(),
            cryptomatte_postprocess: false,
            tile: WorkTile::default(),
            full_write: false,
            display: WorkDisplay::default(),
            rebalance: false,
            volume_guiding_denoise: false,
        }
    }
}

/// One step of `PathTrace::render_pipeline`, in the order it takes them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderStep {
    /// `init_render_buffers`.
    InitRenderBuffer,
    /// `rebalance`: share work between devices again.
    Rebalance,
    /// `path_trace`.
    PathTrace,
    /// `adaptive_sample`: filter, and find the pixels still open.
    AdaptiveSample,
    /// `cryptomatte_postprocess`.
    CryptomattePostprocess,
    /// `denoise`.
    Denoise,
    /// `write_tile_buffer`.
    WriteTileBuffer,
    /// `update_display`.
    UpdateDisplay,
    /// `finalize_full_buffer_on_disk`.
    FinalizeFullBuffer,
}

impl RenderWork {
    /// `operator bool`: whether there is anything to do.
    pub fn any(&self) -> bool {
        self.path_trace.sample_count != 0
            || self.adaptive_sampling.filter
            || self.display.update
            || self.tile.denoise
            || self.tile.write
            || self.full_write
    }

    /// The steps this work asks for, in the pipeline's order. Each step
    /// returns at once in Cycles when its flag is off; here it is left out.
    pub fn step_list(&self) -> Vec<RenderStep> {
        [
            (self.init_render_buffers, RenderStep::InitRenderBuffer),
            (self.rebalance, RenderStep::Rebalance),
            (self.path_trace.sample_count != 0, RenderStep::PathTrace),
            (self.adaptive_sampling.filter, RenderStep::AdaptiveSample),
            (self.cryptomatte_postprocess, RenderStep::CryptomattePostprocess),
            (self.tile.denoise, RenderStep::Denoise),
            (self.tile.write, RenderStep::WriteTileBuffer),
            (self.display.update, RenderStep::UpdateDisplay),
            (self.full_write, RenderStep::FinalizeFullBuffer),
        ]
        .into_iter()
        .filter_map(|(asked, step)| asked.then_some(step))
        .collect()
    }
}
