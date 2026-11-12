// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Work tiles: the units a device's path states are filled from
//! (`intern/cycles/integrator/tile.h`, `work_tile_scheduler.h`). A work
//! tile is not a tile of the frame, which is `session::Tile`.

mod scheduler;
mod size;

pub use scheduler::WorkTileScheduler;
pub use size::TileSize;
