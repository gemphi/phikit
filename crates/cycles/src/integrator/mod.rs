// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! The host side of a render: `intern/cycles/integrator`.

mod adaptive;
mod balance;
mod scheduler;
mod tile;
pub(crate) mod trace;
mod work;

pub use adaptive::AdaptiveSampling;
pub use balance::WorkBalanceInfo;
pub use scheduler::{
    FirstRenderTime, RenderScheduler, SchedulerState, TaskTime, TimeAverage, approximate_final_time, divider_for_resolution, is_power_of_two,
    resolution_for_divider, round_to_power_of_two,
};
pub use tile::{TileSize, WorkTileScheduler};
pub use trace::{PathTrace, PathTraceWork, PathTraceWorkCpu, RenderStatistic, TraceHost, TILE_ARGUMENT, TraceState, scale_buffer_param, slice_buffer_param};
pub use work::{RenderStep, RenderWork, WorkAdaptive, WorkDisplay, WorkPathTrace, WorkTile};
