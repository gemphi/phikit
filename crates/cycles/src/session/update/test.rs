// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use std::rc::Rc;

use crate::integrator::{PathTrace, RenderScheduler};
use crate::scene::Integrator;
use crate::session::*;
use crate::integrator::trace::test::{Device, Host, Log};
use crate::util::ManualClock;

/// A session, its scheduler and a path tracer over one scripted device
/// that takes half a second a sample.
struct Render {
    session: Session,
    scheduler: RenderScheduler<Rc<ManualClock>>,
    trace: PathTrace<Device>,
    host: Host,
    log: Log,
}

fn render(param: SessionParam, size: [i32; 2]) -> Render {
    let clock = Rc::new(ManualClock::at(100.0));
    let log = Log::default();
    let mut session = Session::new(param);
    session.reset(&BufferParam::full(size[0], size[1]));
    let scheduler = RenderScheduler::new(clock.clone(), &session.param);
    Render { session, scheduler, trace: PathTrace::new(vec![Device::new(&log, &clock, 0.5)]), host: Host::new(&log, &clock), log }
}

impl Render {
    /// One round: the session's next work, rendered. `false` when there
    /// was none.
    fn round(&mut self) -> bool {
        let work = self.session.next_work(&mut self.scheduler, &mut self.trace, &mut self.host);
        self.trace.render(&work, &mut self.scheduler, &self.session.tile_manager, &mut self.host);
        work.any()
    }

    /// Render until no work is left; the resets, traces and writes logged.
    fn run(&mut self) -> Vec<String> {
        while self.round() {}
        let kept = ["reset", "trace", "write", "finish"];
        self.log.take().into_iter().filter(|line| kept.iter().any(|start| line.starts_with(start))).collect()
    }
}

fn offline(sample_count: i32, tile_size: i32) -> SessionParam {
    SessionParam { background: true, sample_count, tile_size, ..SessionParam::default() }
}

#[test]
fn a_tile_s_buffer_sits_where_the_tile_is_in_the_frame() {
    let tile = Tile { x: 64, y: 32, width: 64, height: 32, window_x: 0, window_y: 0, window_width: 64, window_height: 32 };
    let param = Session::tile_param(&BufferParam::full(128, 64), &tile);
    assert_eq!((param.width, param.height, param.full_x, param.full_y), (64, 32, 64, 32));
    assert_eq!((param.full_width, param.full_height), (128, 64));
    assert_eq!((param.offset, param.stride), (-(64 + 32 * 64), 64));
}

#[test]
fn a_frame_of_one_tile_is_sampled_and_written_to_the_host() {
    let mut render = render(offline(2, 2048), [64, 64]);
    assert_eq!(render.run(), ["reset true", "trace 0+1", "trace 1+1", "write host"]);
    assert_eq!(render.scheduler.sample_count(), 2);
}

#[test]
fn a_frame_of_two_tiles_is_rendered_tile_by_tile_and_then_finished() {
    let mut render = render(offline(2, 64), [128, 64]);
    assert_eq!(render.session.tile_manager.tile_count(), 2);
    let expect = ["reset true", "trace 0+1", "trace 1+1", "write disk", "reset false", "trace 0+1", "trace 1+1", "write disk", "finish"];
    assert_eq!(render.run(), expect);
    // The device's buffer was one tile, then the other.
    let place: Vec<(i32, i32)> = render.trace.work()[0].allocated.iter().map(|param| (param.full_x, param.width)).collect();
    assert_eq!(place, [(0, 64), (64, 64)]);
}

#[test]
fn a_reset_in_the_middle_starts_the_render_over() {
    let mut render = render(offline(4, 2048), [64, 64]);
    assert!(render.round());
    render.session.reset(&BufferParam::full(64, 64));
    render.log.take();
    assert_eq!(render.run()[..2], ["reset true", "trace 0+1"]);
}

#[test]
fn the_session_s_samples_and_time_limit_reach_the_scheduler() {
    let param = SessionParam { use_sample_subset: true, sample_subset_offset: 6, sample_subset_length: 4, time_limit: 30.0, ..offline(8, 2048) };
    let mut render = render(param, [64, 64]);
    assert!(render.round());
    assert_eq!((render.scheduler.sample_offset(), render.scheduler.sample_count()), (6, 2));
    assert_eq!(render.scheduler.time_limit(), 30.0);
}

#[test]
fn the_integrator_s_settings_reach_the_scheduler() {
    let mut render = render(offline(64, 2048), [64, 64]);
    assert!(!render.scheduler.is_adaptive_sampling_used());
    // Cycles' integrator samples adaptively unless told not to.
    Session::update_integrator(&mut render.scheduler, &Integrator::new());
    assert!(render.scheduler.is_adaptive_sampling_used());
    assert!(render.scheduler.full_report().contains("Min Samples: 64"));
}

#[test]
fn a_finished_render_has_no_work_however_often_it_is_asked() {
    let mut render = render(offline(1, 2048), [64, 64]);
    render.run();
    assert!(!render.round() && !render.round());
    assert!(render.session.tile_manager.done());
}
