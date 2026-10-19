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
        // What the render is actually handed, and why it is never handed
        // nothing (`BRenderCast`). The restrictions alone emptied the frame
        // on pose 26 and again on the crystal-caves city — 1985 objects in
        // the file, none in the render — and the tracer's only answer was
        // "the scene has nothing in it". The cast takes the second look and
        // names the rule that emptied it. Read here, while `loaded` is still
        // alive; `model` takes it below.
        let cast = crate::v2::scene::render_cast::BRenderCast::of(&self.state, loaded.as_ref());
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
        // The camera toggle decides a render, not the eye — `render_scene`
        // has already applied it, so this no longer filters a second time.
        // When it leaves nothing, the cast says which rule did it and hands
        // over what the viewport shows instead.
        if let Some(note) = cast.note {
            crate::v2::state::log::push(note);
        }
        render_job::start(cast.objects, plan);
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
}

#[path = "render_entry.rs"]
mod render_entry;

#[path = "render_save.rs"]
mod render_save;

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
#[path = "render_tests.rs"]
mod show_render_result_tests;
