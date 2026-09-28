//! `render.render` — F12 renders the shown frame, Ctrl F12 the whole range,
//! each frame written where Output points.

use super::super::BPhiberApp;
use crate::v2::part::text::info::{self, BReportLevel};
use crate::v2::scene::render_job;
use crate::v2::state::{CtxKey, ModalKey, log, val::V_MODAL_RENDER};
use gpui::Context;
use std::time::Duration;

/// How often the animation render looks for a finished frame.
const FRAME_POLL_MS: u64 = 100;

impl BPhiberApp {
    /// Start a final render of the loaded scene at the shown frame. `false`
    /// when there is nothing to render — no file, or no camera in it
    /// (Blender: "No camera found in scene").
    pub(crate) fn render_now(&mut self) -> bool {
        use crate::v2::scene::render_job;
        let Some(model) = crate::v2::scene::source::with_loaded(|m| m.cloned()) else {
            crate::v2::state::log::push("Nothing to render: open a file first".to_string());
            return false;
        };
        let set = crate::v2::scene::render_set(&self.state);
        let look = self.render_look(&model, set);
        let frame = crate::v2::scene::motion::shown_frame(&self.state);
        let Some(plan) = crate::v2::scene::render_job::plan_at(&model, set, &look, frame) else {
            crate::v2::state::log::push("No camera found in scene".to_string());
            return false;
        };
        // The camera toggle decides a render, not the eye.
        let hidden = crate::v2::part::outliner::key::BHidden::for_render(&self.state);
        let objects: Vec<_> = crate::v2::scene::current_scene(&self.state, Some(&model))
            .into_iter()
            .filter(|o| !matches!(o, crate::v2::scene::BObject3D::Loaded { name, .. } if hidden.hides(&self.state, name)))
            .collect();
        render_job::start(objects, plan, gpui::hsla(0.0, 0.0, 0.8, 1.0));
        true
    }

    /// The world a final render stands in — the file's lamps and its
    /// Background node, the same ones the viewport uses.
    fn render_look(
        &self,
        model: &crate::phiphx::scene::SceneModel,
        set: crate::v2::scene::BRenderSet,
    ) -> crate::v2::view::draw::canvas::view3d::rendered::BRayLook {
        use crate::v2::view::draw::canvas::view3d::rendered::{BRayLook, environment, view_words};
        BRayLook {
            sky: gpui::hsla(0.0, 0.0, 0.14, 1.0),
            grid: gpui::hsla(0.0, 0.0, 0.3, 1.0),
            world: Some(model.world.background.radiance()),
            lights: crate::v2::scene::scene_lights(),
            wire: None,
            view: view_words(),
            environment: environment().map(|(_, rot)| rot),
            set,
            studio: false,
        }
    }

    /// F12: the render window opens and a render starts, every time.
    pub(super) fn render_image(&mut self) {
        self.state.set(ModalKey::Open, V_MODAL_RENDER);
        if render_job::rendering() {
            log::push("Already rendering".to_string());
            return;
        }
        self.render_now();
    }

    /// Ctrl F12: every frame of the range rendered in turn and written where
    /// Output points, numbered (`BLI_path_frame`), until the range ends or
    /// the render is cancelled.
    pub(super) fn render_animation(&mut self, cx: &mut Context<Self>) {
        self.state.set(ModalKey::Open, V_MODAL_RENDER);
        if render_job::rendering() {
            log::push("Already rendering".to_string());
            return;
        }
        let start = crate::v2::state::key::int_or(&self.state, CtxKey::FrameStart, 1);
        let end = crate::v2::state::key::int_or(&self.state, CtxKey::FrameEnd, 250).max(start);
        log::push(format!("Rendering animation: frames {start}–{end}"));
        cx.spawn(async move |this, cx| {
            for frame in start..=end {
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
                if !finished {
                    let _ = this.update(cx, |_, _| log::push("Animation render stopped".to_string()));
                    break;
                }
                let go_on = this
                    .update(cx, |app, cx| {
                        let saved = app.save_frame(frame);
                        cx.notify();
                        saved
                    })
                    .unwrap_or(false);
                if !go_on {
                    break;
                }
            }
        })
        .detach();
    }

    /// The finished shot written as frame `frame` of the output path.
    fn save_frame(&mut self, frame: i32) -> bool {
        let (shot, _) = render_job::shot();
        let Some(shot) = shot else { return false };
        let output = crate::v2::scene::output_set(&self.state);
        let pass = crate::v2::scene::pass_set(&self.state, Default::default());
        let blend = self.state.get(&crate::v2::scene::loaded_path_key()).unwrap_or_default().to_string();
        let folder = std::path::Path::new(&blend).parent().map(std::path::Path::to_path_buf).unwrap_or_default();
        let file = crate::v2::scene::render_save::frame_file(&output, &folder, frame);
        match crate::v2::scene::save_render_to(&shot, &output, pass, &file) {
            Ok(()) => {
                info::push(&mut self.state, BReportLevel::Info, format!("Saved: '{}'", file.display()));
                true
            }
            Err(why) => {
                info::push(&mut self.state, BReportLevel::Error, format!("Cannot save frame {frame}: {why}"));
                false
            }
        }
    }
}
