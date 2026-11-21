// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/session/session.h` and `session.cpp`
//! (`Session`, `Session::run_main_render_loop`, `session.cpp:163-234`).
//!
//! The loop is Cycles' own. Of what it calls below itself the scheduler and
//! the path tracer are ported (`integrator::RenderScheduler`,
//! `integrator::PathTrace`), and `Session::next_work` (`update.rs`) is the
//! part of the update before each round that joins them. The scene update,
//! the progress and the device are not ported, so this loop still reaches
//! what is below it through [`RenderHost`]; each method names the call it
//! stands for.
//! The session thread, pausing and status text are not ported.

use super::{BufferParam, SessionParam, TileManager};
use crate::integrator::RenderWork;

/// What the render loop calls beneath itself.
pub trait RenderHost {
    /// `path_trace_->zero_display()`.
    fn zero_display(&mut self);
    /// `run_update_for_next_iteration()`: update the scene and ask the
    /// scheduler for the next round of work.
    fn next_work(&mut self) -> RenderWork;
    /// `progress.get_cancel()`.
    fn cancelled(&self) -> bool;
    /// `render_scheduler_.render_work_reschedule_on_cancel(work)`: keep only
    /// what must still be done to leave a usable image.
    fn reschedule_on_cancel(&mut self, work: &mut RenderWork);
    /// `run_wait_for_work(work)`: `true` when the loop paused or waited and
    /// should ask for work again.
    fn wait_for_work(&mut self, work: &RenderWork) -> bool;
    /// `device->have_error()` with `device->error_message()`.
    fn error(&self) -> Option<String>;
    /// `path_trace_->render(work)`.
    fn render(&mut self, work: &RenderWork);
}

/// How the render loop ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionEnd {
    /// No work was left, in background mode.
    Finished,
    /// A cancel was asked for and what remained of the work was done.
    Cancelled,
    /// The device reported an error.
    Error(String),
}

/// `Session`: the settings, the tiles and the buffer of one render.
#[derive(Debug, Clone, Default)]
pub struct Session {
    pub param: SessionParam,
    pub tile_manager: TileManager,
    pub buffer: BufferParam,
    /// `delayed_reset_.do_reset`: a reset the next round has to take.
    pub(super) reset_pending: bool,
}

impl Session {
    pub fn new(param: SessionParam) -> Session {
        Session { param, ..Session::default() }
    }

    /// `get_effective_tile_size`, then `tile_manager_.reset_scheduling`
    /// (`session.cpp:549`): take the frame's buffer and cut it into tiles.
    pub fn reset(&mut self, buffer: &BufferParam) {
        self.buffer = buffer.clone();
        let tile_size = TileManager::effective_tile_size(&self.param, buffer.width, buffer.height);
        self.tile_manager.reset_scheduling(buffer, tile_size);
        self.reset_pending = true;
    }

    /// `run_main_render_loop`: ask for work and render it until none is
    /// left, a cancel has been honoured, or the device fails.
    pub fn run_main_render_loop(&mut self, host: &mut impl RenderHost) -> SessionEnd {
        host.zero_display();
        loop {
            let mut work = host.next_work();
            let did_cancel = host.cancelled();
            // With no work left, a background render can stop at once.
            if !work.any() && self.param.background {
                return SessionEnd::Finished;
            }
            match did_cancel {
                true => {
                    host.reschedule_on_cancel(&mut work);
                    if !work.any() {
                        return SessionEnd::Cancelled;
                    }
                }
                false => {
                    if host.wait_for_work(&work) {
                        continue;
                    }
                }
            }
            // An error from updating the scene, before anything is rendered.
            if let Some(message) = host.error() {
                return SessionEnd::Error(message);
            }
            host.render(&work);
            if let Some(message) = host.error() {
                return SessionEnd::Error(message);
            }
            if did_cancel {
                return SessionEnd::Cancelled;
            }
        }
    }
}
