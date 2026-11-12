// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/integrator/path_trace.h` and `path_trace.cpp`
//! (`PathTrace`): the host loop of the path tracer. It takes one
//! [`RenderWork`](super::RenderWork) and has its works render it.
//!
//! Cycles' `PathTrace` keeps references to the session's scheduler and tile
//! manager; here `render` is handed them. Its works run in parallel, one a
//! device; here one after another. Path guiding is not ported.

mod buffer;
mod cpu;
mod pipeline;
mod slice;
mod work;

pub use cpu::{PathTraceWorkCpu, TILE_ARGUMENT};
pub use slice::{scale_buffer_param, slice_buffer_param};
pub use work::{PathTraceWork, RenderStatistic, TraceHost};

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use super::WorkBalanceInfo;
use crate::session::BufferParam;

/// `PathTrace::render_state_`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TraceState {
    /// The works' buffers are to be sized again: after a reset or a
    /// rebalance.
    pub need_reset_param: bool,
    /// The divider the buffers are sized for. Zero before any work.
    pub resolution_divider: f32,
    /// The big tile as it is path traced.
    pub effective_big_tile_param: BufferParam,
    /// The big tile as it is denoised and shown.
    pub effective_denoised_big_tile_param: BufferParam,
    /// A denoised result is in the buffer.
    pub has_denoised_result: bool,
    /// The current tile was written.
    pub tile_written: bool,
}

/// `PathTrace`.
#[derive(Debug)]
pub struct PathTrace<W: PathTraceWork> {
    /// `path_trace_works_`: one for each device.
    work: Vec<W>,
    /// `work_balance_infos_`: each work's share of the frame.
    balance: Vec<WorkBalanceInfo>,
    /// `full_params_`: the frame.
    full_param: BufferParam,
    /// `big_tile_params_`: the tile of the frame being rendered.
    big_tile_param: BufferParam,
    state: TraceState,
    /// `render_cancel_.is_requested`.
    cancel: Arc<AtomicBool>,
    /// `data.film.cryptomatte_passes` is not zero.
    pub need_cryptomatte: bool,
    /// `data.integrator.use_volumes`.
    pub has_volume: bool,
}

impl<W: PathTraceWork> PathTrace<W> {
    /// A path tracer over `work`, the frame shared evenly between them.
    pub fn new(work: Vec<W>) -> Self {
        let mut balance = vec![WorkBalanceInfo::default(); work.len()];
        WorkBalanceInfo::share_evenly(&mut balance);
        Self { work, balance, full_param: BufferParam::default(), big_tile_param: BufferParam::default(), state: TraceState::default(), cancel: Arc::default(), need_cryptomatte: false, has_volume: false }
    }

    /// The works, to read.
    pub fn work(&self) -> &[W] {
        &self.work
    }

    /// Each work's share of the frame.
    pub fn balance(&self) -> &[WorkBalanceInfo] {
        &self.balance
    }

    pub fn state(&self) -> &TraceState {
        &self.state
    }

    /// The frame, as `reset` was handed it.
    pub fn full_param(&self) -> &BufferParam {
        &self.full_param
    }

    /// The tile being rendered, as `reset` was handed it.
    pub fn big_tile_param(&self) -> &BufferParam {
        &self.big_tile_param
    }

    /// More than one device shares the frame, so the scheduler is to
    /// plan rebalances. Cycles tells it so when the path tracer is made.
    pub fn need_schedule_rebalance(&self) -> bool {
        self.work.len() > 1
    }

    /// `has_denoised_result`.
    pub fn has_denoised_result(&self) -> bool {
        self.state.has_denoised_result
    }

    /// The flag `cancel` raises, for a thread other than the render's.
    pub fn cancel_handle(&self) -> Arc<AtomicBool> {
        self.cancel.clone()
    }

    /// `cancel`: stop as soon as the step being run returns. Cycles then
    /// waits for the render thread and lowers the flag; here whoever asked
    /// calls [`Self::clear_cancel`].
    pub fn request_cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }

    pub fn clear_cancel(&self) {
        self.cancel.store(false, Ordering::Relaxed);
    }

    /// `render_cancel_.is_requested`: the instant cancel alone.
    pub(crate) fn cancel_requested(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }

    /// `is_cancel_requested`: the instant cancel, or the user's through the
    /// progress.
    pub fn is_cancel_requested(&self, host: &impl TraceHost) -> bool {
        self.cancel_requested() || host.cancelled()
    }
}
