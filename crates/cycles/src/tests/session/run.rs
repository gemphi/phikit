// SPDX-License-Identifier: Apache-2.0
use crate::integrator::{RenderWork, WorkDisplay, WorkPathTrace};
use crate::session::*;

/// A host that hands out a fixed number of rounds and records what happened.
#[derive(Default)]
struct Scripted {
    round_left: i32,
    cancel_after: Option<i32>,
    fail_after: Option<i32>,
    rendered: Vec<RenderWork>,
    zeroed: bool,
    waited: i32,
}

fn samples(count: i32) -> RenderWork {
    RenderWork { path_trace: WorkPathTrace { start_sample: 0, sample_count: count, sample_offset: 0 }, ..RenderWork::default() }
}

impl RenderHost for Scripted {
    fn zero_display(&mut self) {
        self.zeroed = true;
    }
    fn next_work(&mut self) -> RenderWork {
        match self.round_left > 0 {
            true => {
                self.round_left -= 1;
                samples(4)
            }
            false => RenderWork::default(),
        }
    }
    fn cancelled(&self) -> bool {
        self.cancel_after.is_some_and(|after| self.rendered.len() as i32 >= after)
    }
    fn reschedule_on_cancel(&mut self, work: &mut RenderWork) {
        // Drop the samples, keep one last display update.
        *work = RenderWork { display: WorkDisplay { update: true, use_denoised_result: false }, ..RenderWork::default() };
    }
    fn wait_for_work(&mut self, _work: &RenderWork) -> bool {
        self.waited += 1;
        false
    }
    fn error(&self) -> Option<String> {
        self.fail_after.filter(|after| self.rendered.len() as i32 >= *after).map(|_| "device lost".to_string())
    }
    fn render(&mut self, work: &RenderWork) {
        self.rendered.push(*work);
    }
}

fn background() -> Session {
    Session::new(SessionParam { background: true, ..SessionParam::default() })
}

#[test]
fn a_background_render_runs_every_round_and_finishes() {
    let mut host = Scripted { round_left: 3, ..Scripted::default() };
    assert_eq!(background().run_main_render_loop(&mut host), SessionEnd::Finished);
    assert!(host.zeroed);
    assert_eq!(host.rendered.len(), 3);
    assert_eq!(host.waited, 3, "it asks whether to wait before each round it renders");
}

#[test]
fn a_cancel_renders_what_the_scheduler_keeps_and_stops() {
    let mut host = Scripted { round_left: 10, cancel_after: Some(2), ..Scripted::default() };
    assert_eq!(background().run_main_render_loop(&mut host), SessionEnd::Cancelled);
    assert_eq!(host.rendered.len(), 3, "two rounds, then the display update kept on cancel");
    assert!(host.rendered[2].display.update && host.rendered[2].path_trace.sample_count == 0);
}

#[test]
fn a_device_error_ends_the_render_with_its_message() {
    let mut host = Scripted { round_left: 10, fail_after: Some(1), ..Scripted::default() };
    assert_eq!(background().run_main_render_loop(&mut host), SessionEnd::Error("device lost".to_string()));
    assert_eq!(host.rendered.len(), 1);
}

#[test]
fn reset_cuts_the_frame_with_the_session_s_tile_size() {
    let mut session = Session::new(SessionParam::default());
    session.reset(&BufferParam::full(5000, 3000));
    assert_eq!(session.tile_manager.tile_count(), 6);
    session.reset(&BufferParam::full(1920, 1080));
    assert_eq!(session.tile_manager.tile_count(), 1);
}
