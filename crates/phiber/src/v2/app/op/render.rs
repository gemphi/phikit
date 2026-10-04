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
        let plain = gpui::hsla(0.0, 0.0, 0.8, 1.0);
        let Some(plan) = crate::v2::scene::render_job::plan_at(&model, set, &look, plain, frame) else {
            crate::v2::state::log::push("No camera found in scene".to_string());
            return false;
        };
        // The camera toggle decides a render, not the eye.
        let hidden = crate::v2::part::outliner::key::BHidden::for_render(&self.state);
        let objects: Vec<_> = crate::v2::scene::current_scene(&self.state, Some(&model))
            .into_iter()
            .filter(|o| !matches!(o, crate::v2::scene::BObject3D::Loaded { name, .. } if hidden.hides(&self.state, name)))
            .collect();
        render_job::start(objects, plan);
        true
    }

    /// The world a final render stands in — the file's lamps and its
    /// Background node, the same ones the viewport uses.
    ///
    /// And the same theme. The sky and the grid here were two fixed greys,
    /// `hsla(0, 0, 0.14)` and `hsla(0, 0, 0.3)`, so a final render came back
    /// on a near-black ground under every palette — the viewport beside it
    /// followed the theme and F12 did not. They are the same two tokens the
    /// viewport reads now (`view3d::BView3D::render`), alpha and all: the
    /// sky's carries the sweep lift, which is a lightness delta rather than an
    /// opacity.
    fn render_look(
        &self,
        model: &crate::phiphx::scene::SceneModel,
        set: crate::v2::scene::BRenderSet,
    ) -> crate::v2::view::draw::canvas::view3d::rendered::BRayLook {
        use crate::v2::view::draw::canvas::view3d::rendered::{BRayLook, environment};
        use crate::v2::view::draw::color::to_hsla;
        // The theme in force AND the overrides on it, the same two steps the
        // renderer takes: a colour edited in Preferences has to reach a render
        // too (`view::draw::backend`).
        let style = crate::v2::theme::resolve_ui_style(crate::v2::theme::ui_style_defaults_for(&self.state), &self.state);
        let mut sky = to_hsla(style.viewport_surface());
        sky.a = style.viewport_sweep_lift();
        BRayLook {
            sky,
            grid: to_hsla(style.viewport_grid_stroke().color),
            grid_hide: 0,
            world: Some(model.world.background.radiance()),
            lights: crate::v2::scene::scene_lights(),
            wire: None,
            view: crate::v2::scene::render_set::BColorManagement::now(&self.state, Some(model)).words(),
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

    /// Opening the Rendering workspace: its Image editor shows the Render
    /// Result, so with none yet the scene is rendered into it at once —
    /// through the active camera, at Output's size, with Render's samples,
    /// as F12 would, without F12's window. Blender opens the workspace on
    /// whatever Render Result there is, or none.
    pub(crate) fn render_on_entry(&mut self) {
        let rendering = self.state.get(CtxKey::Space) == Some(crate::v2::space::BSpaceType::Rendering.slug());
        if !rendering || render_job::rendering() || render_job::shot().0.is_some() || self.scene_cache.is_none() {
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
    pub(super) fn render_animation(&mut self, cx: &mut Context<Self>) {
        self.state.set(ModalKey::Open, V_MODAL_RENDER);
        if render_job::rendering() {
            log::push("Already rendering".to_string());
            return;
        }
        let start = crate::v2::state::key::int_or(&self.state, CtxKey::FrameStart, 1);
        let end = crate::v2::state::key::int_or(&self.state, CtxKey::FrameEnd, 250).max(start);
        log::push(format!("Rendering animation: frames {start}–{end}"));
        let mut movie = self.movie_job(start, end);
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

    /// The movie an FFmpeg Video output gathers its frames into, when that
    /// is the format asked for.
    fn movie_job(&self, start: i32, end: i32) -> Option<crate::v2::scene::video::BMovieJob> {
        let output = crate::v2::scene::output_set(&self.state);
        if !crate::v2::scene::output_set::wants_movie(&output) {
            return None;
        }
        let blend = self.state.get(&crate::v2::scene::loaded_path_key()).unwrap_or_default().to_string();
        let folder = std::path::Path::new(&blend).parent().map(std::path::Path::to_path_buf).unwrap_or_default();
        let still = crate::v2::scene::output_set::as_still(&output).file_of(&folder);
        let movie = crate::v2::scene::output_set::movie_set(&self.state, self.scene_cache.as_ref().map(|(_, m)| m.render.movie).unwrap_or_default());
        let fps = crate::v2::state::key::float_or(&self.state, CtxKey::Fps, 24.0);
        match crate::v2::scene::video::BMovieJob::begin(&still, &output, movie, fps, start, end) {
            Ok(job) => Some(job),
            Err(why) => {
                log::push(format!("Cannot gather the movie's frames: {why}"));
                None
            }
        }
    }

    /// What the encode came to, told in the Info editor.
    fn movie_made(&mut self, made: Result<crate::v2::scene::video::BMovieMade, String>) {
        use crate::v2::scene::video::BMovieMade;
        match made {
            Ok(BMovieMade::Ffmpeg(file)) => info::push(&mut self.state, BReportLevel::Info, format!("Saved movie: '{}'", file.display())),
            Ok(BMovieMade::Avi(file)) => info::push(
                &mut self.state,
                BReportLevel::Warning,
                format!("ffmpeg is not installed: saved a Motion-JPEG AVI instead, '{}' (install ffmpeg for the H.264 / container the Output panel asks for)", file.display()),
            ),
            Err(why) => info::push(&mut self.state, BReportLevel::Error, format!("Cannot write the movie: {why}")),
        }
    }

    /// The finished shot written as frame `frame` of the output path — or,
    /// `into` a movie's frame file, as a PNG on its way to the encoder.
    fn save_frame(&mut self, frame: i32, into: Option<std::path::PathBuf>) -> bool {
        let (shot, _) = render_job::shot();
        let Some(shot) = shot else { return false };
        let output = crate::v2::scene::output_set::as_still(&crate::v2::scene::output_set(&self.state));
        let pass = crate::v2::scene::pass_set(&self.state, Default::default());
        let blend = self.state.get(&crate::v2::scene::loaded_path_key()).unwrap_or_default().to_string();
        let folder = std::path::Path::new(&blend).parent().map(std::path::Path::to_path_buf).unwrap_or_default();
        let movie = into.is_some();
        let file = into.unwrap_or_else(|| crate::v2::scene::render_save::frame_file(&output, &folder, frame));
        match crate::v2::scene::save_render_to(&shot, &output, pass, &file) {
            Ok(()) => {
                if !movie {
                    info::push(&mut self.state, BReportLevel::Info, format!("Saved: '{}'", file.display()));
                }
                true
            }
            Err(why) => {
                info::push(&mut self.state, BReportLevel::Error, format!("Cannot save frame {frame}: {why}"));
                false
            }
        }
    }
}
