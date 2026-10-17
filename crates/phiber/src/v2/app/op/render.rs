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
        self.render_from(None)
    }

    /// A render from the file's camera — or, with `back` given, from a
    /// stand-in camera standing off the scene in that direction, framing it
    /// (`BStartupCamera::camera_from`): what a turnaround's six views look
    /// through. Returns whether a render started.
    fn render_from(&mut self, back: Option<glam::Vec3>) -> bool {
        use crate::v2::scene::render_job;
        // No file open: the default scene through the startup file's camera
        // (`BStartupCamera`), as a fresh Blender renders its cube.
        let loaded = crate::v2::scene::source::with_loaded(|m| m.cloned());
        // The render-pass camera icon filters this list; the viewport eye
        // doesn't (Blender renders what the viewport hides, `pose::visible_in_render`).
        let scene = crate::v2::scene::render_scene(&self.state, loaded.as_ref());
        let model = loaded.unwrap_or_else(|| crate::v2::scene::startup_camera::BStartupCamera::model(&scene));
        let set = crate::v2::scene::render_set(&self.state);
        let look = self.render_look(&model, set);
        let frame = crate::v2::scene::motion::shown_frame(&self.state);
        let plain = gpui::hsla(0.0, 0.0, 0.8, 1.0);
        // A stand-in camera framing the scene (the startup file's,
        // `BStartupCamera`): asked for by direction (a turnaround view), or
        // when the file has no camera — Blender reports "No camera found in
        // scene" and stops; a render is what this workspace is for, so the
        // stand-in takes the shot and the log says so. Deliberate divergence.
        let stand_in = |dir: glam::Vec3| {
            let mut with = model.clone();
            let (w, h) = crate::v2::scene::render_job::render_size(&with);
            with.cameras.push(crate::v2::scene::startup_camera::BStartupCamera::camera_from(&scene, w as f32 / h.max(1) as f32, dir));
            with.render.camera = crate::v2::scene::startup_camera::BStartupCamera::NAME.into();
            crate::v2::scene::render_job::plan_at(&with, set, &look, plain, frame)
        };
        let plan = match back {
            Some(dir) => stand_in(dir),
            None => crate::v2::scene::render_job::plan_at(&model, set, &look, plain, frame).or_else(|| {
                crate::v2::state::log::push("No camera in the file: rendering from a stand-in camera framing the scene".to_string());
                stand_in(glam::Vec3::from(crate::v2::scene::startup_camera::BStartupCamera::DIRECTION))
            }),
        };
        let Some(plan) = plan else {
            crate::v2::state::log::push("No camera found in scene".to_string());
            return false;
        };
        // The camera toggle decides a render, not the eye.
        let hidden = crate::v2::part::outliner::key::BHidden::for_render(&self.state);
        let objects: Vec<_> = scene
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
        show_render_result(&mut self.state);
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

    /// Render ▸ Render Turnaround: the scene from its six axis views —
    /// Blender's numpad Front, Back, Right, Left, Top and Bottom
    /// (`BStartupCamera::TURNAROUND`) — each framed by a stand-in camera and
    /// written beside the Output path as `<name>_front.png` … , the six
    /// rendered in turn as an animation's frames are; a cancel stops after
    /// the view in hand. An animated scene then renders its animation too
    /// (`render_animation`: frames, or a movie when Output says FFmpeg Video),
    /// so one command leaves the whole set. Not a Blender operator: it is
    /// six `view3d.view_axis` + `render.render` presses, done at once.
    pub(super) fn render_turnaround(&mut self, cx: &mut Context<Self>) {
        use crate::v2::scene::startup_camera::BStartupCamera;
        self.state.set(ModalKey::Open, V_MODAL_RENDER);
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
    }

    /// Write the finished shot as turnaround view `name`'s file.
    fn save_view(&mut self, name: &str) -> bool {
        let (shot, _) = render_job::shot();
        let Some(shot) = shot else { return false };
        let output = crate::v2::scene::output_set::as_still(&crate::v2::scene::output_set(&self.state));
        let pass = crate::v2::scene::pass_set(&self.state, Default::default());
        let blend = self.state.get(&crate::v2::scene::loaded_path_key()).unwrap_or_default().to_string();
        let folder = std::path::Path::new(&blend).parent().map(std::path::Path::to_path_buf).unwrap_or_default();
        let file = crate::v2::scene::render_save::view_file(&output, &folder, name);
        match crate::v2::scene::save_render_to(&shot, &output, pass, &file) {
            Ok(()) => {
                info::push(&mut self.state, BReportLevel::Info, format!("Saved: '{}'", file.display()));
                true
            }
            Err(why) => {
                info::push(&mut self.state, BReportLevel::Error, format!("Cannot save the {name} view: {why}"));
                false
            }
        }
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

/// The Rendering workspace's Center shows the Render Result — in an Image
/// editor. Blender's `render_view_open` (Render Display Type: Image Editor)
/// converts the area to an Image editor when a render is shown; here the
/// workspace renders on entry, so its Center is converted the same way if
/// another editor (the Sequencer, say) was switched into it and resumed —
/// otherwise the render has nowhere to appear and the area looks blank.
/// Returns whether the area was changed.
pub(crate) fn show_render_result(state: &mut crate::v2::state::BState) -> bool {
    use crate::v2::area::current::{active_host, area_key};
    use crate::v2::area::BAreaType;
    use crate::v2::host::BHostType;
    use crate::v2::space::BSpaceType;
    use crate::v2::part::image::shown::RENDER_RESULT;
    use crate::v2::state::ImageKey;
    let space = BSpaceType::Rendering;
    let Some(center) = space.spec().areas.iter().find(|a| a.kind == BAreaType::Center) else { return false };
    let mut changed = false;
    if active_host(space, center, |k| state.get(k).map(str::to_string)) != BHostType::Image {
        state.set(&area_key(space, center), BHostType::Image.slug());
        changed = true;
    }
    // And the editor shows the render, not an image left over from before
    // (`sima->image = Render Result` in `render_view_open`).
    if crate::v2::part::image::shown::BShownImage::now(state) != crate::v2::part::image::shown::BShownImage::RenderResult {
        state.set(ImageKey::Shown, RENDER_RESULT);
        changed = true;
    }
    if changed {
        log::push("Rendering workspace: showing the Render Result in the Image editor".to_string());
    }
    changed
}

#[cfg(test)]
mod show_render_result_tests {
    use super::show_render_result;
    use crate::v2::area::current::{active_host, area_key};
    use crate::v2::area::BAreaType;
    use crate::v2::host::BHostType;
    use crate::v2::space::BSpaceType;
    use crate::v2::state::BState;

    fn center() -> &'static crate::v2::area::BAreaSpec {
        BSpaceType::Rendering.spec().areas.iter().find(|a| a.kind == BAreaType::Center).unwrap()
    }

    #[test]
    fn a_sequencer_left_in_the_rendering_center_gives_way_to_the_image_editor() {
        let mut s = BState::new();
        s.set(&area_key(BSpaceType::Rendering, center()), BHostType::Seq.slug());
        assert_eq!(active_host(BSpaceType::Rendering, center(), |k| s.get(k).map(str::to_string)), BHostType::Seq);
        s.set(crate::v2::state::ImageKey::Shown, "Untitled.001"); // a leftover New image, resumed
        assert!(show_render_result(&mut s), "the area was converted");
        assert_eq!(active_host(BSpaceType::Rendering, center(), |k| s.get(k).map(str::to_string)), BHostType::Image);
        assert_eq!(s.get(crate::v2::state::ImageKey::Shown), Some(crate::v2::part::image::shown::RENDER_RESULT), "and it shows the render, not the leftover");
        // Already an Image editor on the Render Result: nothing to do.
        assert!(!show_render_result(&mut s));
    }

    #[test]
    fn another_workspace_s_center_is_not_touched() {
        // The key is namespaced per space: Layout's Center keeps its Sequencer.
        let mut s = BState::new();
        let layout_center = BSpaceType::Layout.spec().areas.iter().find(|a| a.kind == BAreaType::Center).unwrap();
        s.set(&area_key(BSpaceType::Layout, layout_center), BHostType::Seq.slug());
        show_render_result(&mut s);
        assert_eq!(active_host(BSpaceType::Layout, layout_center, |k| s.get(k).map(str::to_string)), BHostType::Seq);
    }
}
