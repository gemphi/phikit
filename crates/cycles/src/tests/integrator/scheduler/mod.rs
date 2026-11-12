// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Tests of `src/integrator/scheduler`, one file for each file there. This
//! file tests `mod.rs` and holds what the others, and the tests of `trace`, set a scheduler up with.

mod divider;
mod interval;
mod need;
mod report;
mod reschedule;
mod resolution;
mod sample;
mod schedule;
mod state;
mod summary;
mod time;

use std::rc::Rc;

use crate::integrator::*;
use crate::scene::{Pass, PassMode, PassType};
use crate::session::{BufferParam, SessionParam, TileManager};
use crate::util::ManualClock;

/// A scheduler, the clock it reads and the tiles of its frame.
pub(crate) struct Rig {
    pub(crate) clock: Rc<ManualClock>,
    pub(crate) scheduler: RenderScheduler<Rc<ManualClock>>,
    pub(crate) tile: TileManager,
    pub(crate) buffer: BufferParam,
}

impl Rig {
    /// The scheduler's next work.
    pub(crate) fn work(&mut self) -> RenderWork {
        self.scheduler.render_work(&self.tile)
    }
}

/// A scheduler reset on a `size` frame cut into `tile_size` tiles, the
/// first tile current, its clock at 100 seconds.
pub(crate) fn tiled(param: SessionParam, size: [i32; 2], tile_size: [i32; 2], sample_count: i32) -> Rig {
    let clock = Rc::new(ManualClock::at(100.0));
    let mut scheduler = RenderScheduler::new(clock.clone(), &param);
    let mut buffer = BufferParam::full(size[0], size[1]);
    buffer.update_pass_from(&[Pass::new(PassType::named("PASS_COMBINED").expect("in data/pass.tsv"), PassMode::Noisy)]);
    let mut tile = TileManager::new();
    tile.reset_scheduling(&buffer, tile_size);
    tile.update(&buffer, false, false);
    tile.next();
    scheduler.set_sample_param(sample_count, false, 0, 0);
    scheduler.reset(&buffer);
    Rig { clock, scheduler, tile, buffer }
}

/// The same on a frame of one tile.
pub(crate) fn rig(param: SessionParam, size: [i32; 2], sample_count: i32) -> Rig {
    tiled(param, size, size, sample_count)
}

/// An offline render with a window.
pub(crate) fn background() -> SessionParam {
    SessionParam { background: true, ..SessionParam::default() }
}

/// An offline render from a command line.
pub(crate) fn headless() -> SessionParam {
    SessionParam { headless: true, ..background() }
}

/// An interactive render.
pub(crate) fn viewport() -> SessionParam {
    SessionParam::default()
}

/// A frame too small for a resolution divider.
pub(crate) const SMALL: [i32; 2] = [64, 64];

#[test]
fn the_sample_count_is_clamped_to_the_limit() {
    let mut rig = rig(background(), SMALL, 16);
    assert_eq!((rig.scheduler.sample_offset(), rig.scheduler.sample_count()), (0, 16));
    rig.scheduler.set_sample_param(i32::MAX, false, 0, 0);
    assert_eq!(rig.scheduler.sample_count(), 1 << 24);
}

#[test]
fn a_sample_subset_is_cut_to_the_frame_s_samples() {
    let mut rig = rig(background(), SMALL, 16);
    rig.scheduler.set_sample_param(100, true, 90, 20);
    assert_eq!((rig.scheduler.sample_offset(), rig.scheduler.sample_count()), (90, 10));
    rig.scheduler.set_sample_param(100, true, 120, 10);
    assert_eq!((rig.scheduler.sample_offset(), rig.scheduler.sample_count()), (120, 0));
}

#[test]
fn a_background_render_is_never_made_smaller() {
    let rig = rig(background(), [1024, 512], 16);
    assert_eq!(rig.scheduler.state().resolution_divider, 1);
    assert!(!rig.scheduler.state().user_is_navigating);
    assert!(rig.scheduler.is_background());
}

#[test]
fn a_viewport_starts_small_when_its_frame_is_large_enough() {
    let large = rig(viewport(), [1024, 512], 16);
    assert_eq!(large.scheduler.state().resolution_divider, 8);
    assert!(large.scheduler.state().user_is_navigating);
    let small = rig(viewport(), SMALL, 16);
    assert_eq!(small.scheduler.state().resolution_divider, 1);
    let off = rig(SessionParam { use_resolution_divider: false, ..viewport() }, [1024, 512], 16);
    assert_eq!(off.scheduler.state().resolution_divider, 1);
}

#[test]
fn a_reset_starts_the_noise_floor_high() {
    let rig = rig(viewport(), SMALL, 16);
    assert_eq!(rig.scheduler.state().adaptive_sampling_threshold, 0.4);
}

#[test]
fn the_rendered_sample_is_the_last_one_scheduled() {
    let mut rig = rig(background(), SMALL, 16);
    rig.work();
    rig.work();
    assert_eq!(rig.scheduler.rendered_sample_count(), 2);
    assert_eq!(rig.scheduler.rendered_sample(), 1);
}

#[test]
fn the_next_tile_starts_from_the_first_sample_on_the_same_frame() {
    let mut rig = rig(viewport(), [1024, 512], 16);
    rig.work();
    rig.work();
    rig.scheduler.reset_for_next_tile();
    assert_eq!(rig.scheduler.rendered_sample_count(), 0);
    assert_eq!(rig.scheduler.state().resolution_divider, 8);
    assert_eq!(rig.scheduler.sample_count(), 16);
}

#[test]
fn the_settings_handed_in_are_the_ones_read_back() {
    let mut rig = rig(background(), SMALL, 16);
    assert!(!rig.scheduler.is_adaptive_sampling_used());
    rig.scheduler.set_adaptive_sampling(AdaptiveSampling { enabled: true, adaptive_step: 4, min_sample: 0, threshold: 0.01 });
    assert!(rig.scheduler.is_adaptive_sampling_used());
    rig.scheduler.set_time_limit(30.0);
    assert_eq!(rig.scheduler.time_limit(), 30.0);
    assert!(rig.scheduler.is_denoiser_gpu_used());
}
