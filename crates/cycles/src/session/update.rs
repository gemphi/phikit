// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/session/session.cpp`
//! (`Session::run_update_for_next_iteration`, lines 311-430, and
//! `update_buffers_for_params`, lines 554-580): what happens before each
//! round of the render loop, as far as it concerns the scheduler, the tiles
//! and the path tracer.
//!
//! The rest of that function waits on the scene: updating it, the camera's
//! resolution, loading kernels, the progress and image eviction.

use super::{BufferParam, Session, Tile};
use crate::integrator::{PathTrace, PathTraceWork, RenderScheduler, RenderWork, TraceHost};
use crate::scene::Integrator;
use crate::util::Clock;

impl Session {
    /// The buffer of one tile of the `buffer` frame: the big tile the path
    /// tracer renders (`session.cpp:386-403`).
    pub fn tile_param(buffer: &BufferParam, tile: &Tile) -> BufferParam {
        let mut param = BufferParam {
            width: tile.width,
            height: tile.height,
            window_x: tile.window_x,
            window_y: tile.window_y,
            window_width: tile.window_width,
            window_height: tile.window_height,
            full_x: tile.x + buffer.full_x,
            full_y: tile.y + buffer.full_y,
            ..buffer.clone()
        };
        param.update_offset_stride();
        param
    }

    /// The integrator's settings handed to the scheduler: the denoiser's
    /// and adaptive sampling's (`session.cpp:337-347`). To be called before
    /// [`Self::next_work`] whenever the integrator changed.
    pub fn update_integrator<C: Clock>(scheduler: &mut RenderScheduler<C>, integrator: &Integrator) {
        scheduler.set_denoise_param(integrator.denoise_param());
        scheduler.set_adaptive_sampling(integrator.adaptive_sampling());
    }

    /// `update_buffers_for_params`: after a reset the scheduler and the
    /// tiles start over on the session's buffer. Baking, which turns the
    /// tiles' overscan off, comes with the scene.
    fn update_buffer_for_param<C: Clock>(&mut self, scheduler: &mut RenderScheduler<C>) {
        let param = &self.param;
        scheduler.set_sample_param(param.sample_count, param.use_sample_subset, param.sample_subset_offset, param.sample_subset_length);
        scheduler.reset(&self.buffer);
        self.tile_manager.update(&self.buffer, scheduler.is_adaptive_sampling_used(), false);
    }

    /// `run_update_for_next_iteration`: the next work of the render. When a
    /// tile has nothing left the next tile is started, and the path tracer
    /// is reset onto it. Nothing in the work returned means every tile is
    /// done.
    pub fn next_work<C: Clock, W: PathTraceWork>(&mut self, scheduler: &mut RenderScheduler<C>, trace: &mut PathTrace<W>, host: &mut impl TraceHost) -> RenderWork {
        // A reset asked for since the last round is taken now.
        let reset_buffer = std::mem::take(&mut self.reset_pending);
        let mut have_tile = true;
        let mut switched_to_new_tile = false;
        if reset_buffer {
            self.update_buffer_for_param(scheduler);
            // After a reset the tile manager is at the first big tile.
            have_tile = self.tile_manager.next();
            switched_to_new_tile = true;
        }
        let param = &self.param;
        scheduler.set_need_schedule_rebalance(trace.need_schedule_rebalance());
        scheduler.set_sample_param(param.sample_count, param.use_sample_subset, param.sample_subset_offset, param.sample_subset_length);
        scheduler.set_time_limit(param.time_limit);
        let mut work = RenderWork::default();
        while have_tile {
            work = scheduler.render_work(&self.tile_manager);
            if work.any() {
                break;
            }
            have_tile = self.tile_manager.next();
            if have_tile {
                scheduler.reset_for_next_tile();
                switched_to_new_tile = true;
            }
        }
        if work.any() && switched_to_new_tile {
            let tile_param = Self::tile_param(&self.buffer, &self.tile_manager.current_tile());
            trace.reset(&self.buffer, &tile_param, host, reset_buffer);
        }
        work
    }
}

#[cfg(test)]
mod test;
