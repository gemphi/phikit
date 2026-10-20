//! `render_on_entry` — what happens when the Rendering workspace opens.
//! Blender's F12 implicit when the user picks the workspace: the file's
//! camera through Output's size at Render's samples, straight into the
//! Image editor's Render Result. Split from `render.rs` so each file
//! sits under the 240-line read cap (`tests/level_structure.rs`).

use super::super::BPhiberApp;
use crate::v2::scene::render_job;
use crate::v2::state::{CtxKey, log};
use gpui::Context;
use std::time::Duration;

/// How often the animation render looks for a finished frame.
const FRAME_POLL_MS: u64 = 100;

impl BPhiberApp {


/// Opening the Rendering workspace: its Image editor shows the Render
/// Result, so with none yet the scene is rendered into it at once —
/// through the active camera (the startup camera's stand-in before a
/// file is open), at Output's size, with Render's samples, as F12
/// would, without F12's window. Blender opens the workspace on
/// whatever Render Result there is, or none.
pub(crate) fn render_on_entry(&mut self) {
    let rendering = self.state.get(CtxKey::Space) == Some(crate::v2::space::BSpaceType::Rendering.slug());
    if !rendering {
        return;
    }
    // The render shows in the workspace's Image editor: put one back in
    // the Center if another editor was left there (`render_view_open`).
    super::show_render_result(&mut self.state);
    if render_job::rendering() || render_job::shot().0.is_some() {
        return;
    }
    if self.render_now() {
        log::push("Rendering the scene for the Rendering workspace".to_string());
    }
}

/// Ctrl F12: every frame of the range rendered in turn and written where
/// Output points, numbered (`BLI_path_frame`) — or, with File Format
/// FFmpeg Video, gathered and encoded into one movie
/// (`scene::video`) — until the range ends or the render is cancelled.
/// A cancelled movie render still encodes the frames it has, as
/// Blender's incremental writer leaves them in the file.
pub(crate) fn render_animation(&mut self, cx: &mut Context<Self>) {
    crate::v2::app::op::render_open::render_view_open(&mut self.state);
    if render_job::rendering() {
        log::push("Already rendering".to_string());
        return;
    }
    let start = crate::v2::state::key::int_or(&self.state, CtxKey::FrameStart, 1);
    let end = crate::v2::state::key::int_or(&self.state, CtxKey::FrameEnd, 250).max(start);
    // Output ▸ Frame Range ▸ Step: every n-th frame, as Blender's
    // `render.render(animation=True)` steps by `scene.frame_step`.
    let step = crate::v2::state::key::int_or(&self.state, crate::v2::state::ButsKey::Value("frame_step"), 1).max(1) as usize;
    log::push(match step {
        1 => format!("Rendering animation: frames {start}–{end}"),
        n => format!("Rendering animation: frames {start}–{end}, every {n}"),
    });
    let mut movie = self.movie_job(start, end);
    cx.spawn(async move |this, cx| {
        for frame in (start..=end).step_by(step) {
            let started = this
                .update(cx, |app, cx| {
                    app.state.set(CtxKey::Frame, frame.to_string());
                    let started = app.render_now();
                    cx.notify();
                    started
                })
                .unwrap_or(false);
            if !started {
                break;
            }
            let finished = loop {
                cx.background_executor().timer(Duration::from_millis(FRAME_POLL_MS)).await;
                let (shot, _) = render_job::shot();
                match shot {
                    Some(shot) if shot.done => break shot.has_image(),
                    Some(_) if render_job::rendering() => continue,
                    _ => break false,
                }
            };
            if !finished || render_job::cancelled() {
                let _ = this.update(cx, |_, _| log::push("Animation render stopped".to_string()));
                break;
            }
            let into = movie.as_mut().map(|m| m.next_frame_file());
            let go_on = this
                .update(cx, |app, cx| {
                    let saved = app.save_frame(frame, into);
                    cx.notify();
                    saved
                })
                .unwrap_or(false);
            if !go_on {
                break;
            }
        }
        if let Some(job) = movie.take() {
            let made = job.finish();
            let _ = this.update(cx, |app, cx| {
                app.movie_made(made);
                cx.notify();
            });
        }
    })
    .detach();
}

/// Render ▸ Render Turnaround: the scene from its six axis views —
/// Blender's numpad Front, Back, Right, Left, Top and Bottom
/// (`BStartupCamera::TURNAROUND`) — each framed by a stand-in camera and
/// written beside the Output path as `<name>_front.png` … , the six
/// rendered in turn as an animation's frames are; a cancel stops after
/// the view in hand. An animated scene then renders its animation too
/// (`render_animation`: frames, or a movie when Output says FFmpeg Video),
/// so one command leaves the whole set. Not a Blender operator: it is
/// six `view3d.view_axis` + `render.render` presses, done at once.
pub(crate) fn render_turnaround(&mut self, cx: &mut Context<Self>) {
    use crate::v2::scene::startup_camera::BStartupCamera;
    crate::v2::app::op::render_open::render_view_open(&mut self.state);
    if render_job::rendering() {
        log::push("Already rendering".to_string());
        return;
    }
    let animated = self.scene_cache.as_ref().is_some_and(|(_, m)| m.motion.animated());
    log::push("Rendering turnaround: front, back, right, left, top, bottom".to_string());
    cx.spawn(async move |this, cx| {
        for (name, dir) in BStartupCamera::TURNAROUND {
            let started = this
                .update(cx, |app, cx| {
                    let started = app.render_from(Some(glam::Vec3::from(dir)));
                    cx.notify();
                    started
                })
                .unwrap_or(false);
            if !started {
                break;
            }
            let finished = loop {
                cx.background_executor().timer(Duration::from_millis(FRAME_POLL_MS)).await;
                let (shot, _) = render_job::shot();
                match shot {
                    Some(shot) if shot.done => break shot.has_image(),
                    Some(_) if render_job::rendering() => continue,
                    _ => break false,
                }
            };
            if !finished || render_job::cancelled() {
                let _ = this.update(cx, |_, _| log::push("Turnaround render stopped".to_string()));
                return;
            }
            let go_on = this
                .update(cx, |app, cx| {
                    let saved = app.save_view(name);
                    cx.notify();
                    saved
                })
                .unwrap_or(false);
            if !go_on {
                return;
            }
        }
        let _ = this.update(cx, |app, cx| {
            log::push("Turnaround rendered: six views saved".to_string());
            if animated {
                log::push("The scene is animated: rendering its animation too".to_string());
                app.render_animation(cx);
            }
            cx.notify();
        });
    })
    .detach();
}}
