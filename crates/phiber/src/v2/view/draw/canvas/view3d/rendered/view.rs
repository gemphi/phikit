//! What the viewport asks the tracer for this frame (`BRayView`).

use super::world;
use crate::v2::scene::camera::BViewSetup;
use crate::v2::view::draw::ctx::BRenderCtx;
use gpui::Hsla;
use std::sync::Arc;

/// What the painter needs this frame.
#[derive(Clone)]
pub(in crate::v2::view::draw::canvas::view3d) struct BRayView {
    pub yaw: f32,
    pub pitch: f32,
    pub pan: (f32, f32),
    pub zoom: f32,
    pub setup: BViewSetup,
    pub sky: Hsla,
    pub grid: Hsla,
    /// Material Preview: studio lighting, file lights and world ignored.
    pub studio: bool,
    /// The scene's lamps the viewport's eye leaves on, where they stand
    /// this frame (`pose::lights_in_viewport`): what the Rendered mode is
    /// lit by. Empty for Material Preview.
    pub lights: Vec<crate::phiphx::scene::SceneLight>,
    /// Wireframe: the wire's colour.
    pub wire: Hsla,
    /// Which phiray kernel draws it — Workbench for Solid, the tracer
    /// otherwise.
    pub mode: phiray::Mode,
    /// What Properties → Render asks for: how far to refine, how many diffuse
    /// bounces, whether to denoise (`scene::render_set`).
    pub set: crate::v2::scene::BRenderSet,
    /// X-ray: how much every surface lets through (`BXray::see_through`).
    pub see_through: f32,
    /// The floor and axes the Overlays popover leaves out (`Camera::grid_hide`).
    pub grid_hide: u32,
    /// Color Management as the panel has it (`Camera::view`).
    pub view: [f32; 4],
    /// Workbench's Solid effects (`Camera::fx`, `::outline`, `::cavity`,
    /// `::ssao`; `BSolidFx`).
    pub fx: [f32; 4],
    pub outline: [f32; 4],
    pub cavity: [f32; 4],
    pub ssao: [u32; 4],
    pub shadow: [f32; 4],
    pub shadow_fade: [f32; 4],
    /// Solid's Lighting and Color (`Camera::shade`, `::single`; `BSolidShade`).
    pub shade: [u32; 4],
    pub single: [f32; 4],
    pub matcap: [u32; 4],
    /// Solid's Depth of Field through the camera (`Camera::dof`; `frame::dof_words`).
    pub dof: [f32; 4],
    /// The viewport's own render region (`View3D.render_border`, fractions
    /// y up) when the Rendered shading fills only it (`BViewBorder::fills`).
    pub border: Option<[f32; 4]>,
    /// Viewport Anti-Aliasing's count (`scene::viewport_aa`): the jittered
    /// samples Solid and Wireframe draw, 0 none, 1 a single pass.
    pub aa: u32,
    /// The selection outline (`Camera::outline_fx`, `::outline_pick`,
    /// `::outline_active`, `::outline_move`; `phiray::trace::outline`).
    pub outline_fx: [f32; 4],
    pub outline_pick: [f32; 4],
    pub outline_active: [f32; 4],
    pub outline_move: [f32; 4],
    /// Which class each object of the scene is in, in the order
    /// `rendered::scene::build` instances them: what the tracer's own
    /// overlay pass draws around (`Tracer::outline`).
    pub outline_class: Arc<[phiray::trace::Outline]>,
    /// While playing, how long the paint waits for this frame's image
    /// (one frame period): `None` when still, and in the browser, which
    /// has no thread to wait on.
    pub sync: Option<std::time::Duration>,
}

impl BRayView {
    /// The view the tracer is asked for, from the camera, the palette and
    /// the theme. Blender draws unselected wires in the theme's wire colour
    /// and the active object in `wire_active`; the kernel carries one wire
    /// colour, so it takes the neutral one.
    pub(in crate::v2::view::draw::canvas::view3d) fn of(camera: (f32, f32, (f32, f32), f32), setup: BViewSetup, sky: Hsla, grid: Hsla, studio: bool, mode: phiray::Mode, dof: [f32; 4], ctx: &BRenderCtx) -> Self {
        let (yaw, pitch, pan, zoom) = camera;
        let (wire, set) = (crate::v2::view::draw::color::to_hsla(ctx.style.colors.text_dim), crate::v2::scene::render_set(ctx.state));
        let see_through = crate::v2::scene::BXray::now(ctx.state).see_through(crate::v2::scene::view_shading(ctx.state));
        let grid_hide = world::BRayLook::grid_hide(crate::v2::scene::BOverlays::now(ctx.state));
        let view = world::view_words(ctx.state);
        let solid_fx = crate::v2::scene::BSolidFx::now(ctx.state);
        let (fx, outline) = (solid_fx.fx_words(1.0), solid_fx.outline_words());
        // The SSAO's disk is run over the anti-aliasing samples (`viewport_aa`,
        // `SceneState::samples_len`: at least the one).
        let aa = crate::v2::scene::viewport_aa::BViewportAa::samples(ctx.state);
        let (cavity, ssao) = (solid_fx.cavity_words(), solid_fx.ssao_words(phiray::trace::taa::count(aa).max(1)));
        let (shadow, shadow_fade) = (solid_fx.shadow_words(), solid_fx.shadow_fade_words());
        let shade = crate::v2::scene::BSolidShade::now(ctx.state);
        // Workbench's Background (`ED_view3d_background_color_get`): the
        // world's colour or the chosen one behind Solid and Wireframe, else
        // the theme's sky with its sweep.
        let sky = match mode {
            phiray::Mode::Solid | phiray::Mode::Wire => shade
                .background(crate::v2::scene::source::with_loaded(|m| m.map(|m| m.world.horizon_color)))
                .map_or(sky, |[r, g, b]| gpui::Rgba { r, g, b, a: 0.0 }.into()),
            _ => sky,
        };
        // The selection outline's colours: Blender's `TH_SELECT` and the
        // active object's `TH_ACTIVE` a step lighter, as the CPU painter
        // drew them. Nothing is marked as being transformed here yet, so
        // the transform colour is the selected one — a class the host
        // never sets.
        let lights = if studio { Vec::new() } else { crate::v2::scene::pose::lights_in_viewport(ctx.state, crate::v2::scene::scene_lights()) };
        let pick = super::scene::rgba(crate::v2::view::draw::color::to_hsla(ctx.style.colors.accent));
        let lit = super::scene::rgba(crate::v2::view::draw::color::to_hsla(ctx.style.step(ctx.style.colors.accent, 2)));
        // Blender draws a frame's shading and its overlays in one pass from
        // one evaluated scene. The tracer is asynchronous, so while playing
        // the paint waits for the frame it posted; otherwise the bones,
        // drawn every frame, run ahead of the meshes in the image.
        let playing = crate::v2::part::playback::set::BTransportButton::playing(ctx.state);
        let fps = crate::v2::state::key::float_or(ctx.state, crate::v2::state::CtxKey::Fps, 24.0).clamp(1.0, 240.0);
        let sync = (playing && cfg!(not(target_family = "wasm"))).then(|| std::time::Duration::from_secs_f32(1.0 / fps));
        Self { yaw, pitch, pan, zoom, setup, sky, grid, studio, lights, wire, mode, set, see_through, grid_hide, view, fx, outline, cavity, ssao, shadow, shadow_fade, shade: solid_fx.shade_words(shade.shade_words()), matcap: shade.matcap_words(), single: shade.single_words(), dof, border: None, aa, outline_fx: [0.0, 0.0, 0.35, 1.0], outline_pick: pick, outline_active: lit, outline_move: pick, outline_class: Arc::from([]), sync }
    }

    /// With the objects the overlay outlines (`BGuide::outlined`), one
    /// class an object in the scene's order: the tracer draws the line
    /// itself, so the CPU projects no hull (`cache::BCpuDraw`). An empty
    /// list, or one of nothing but `None`, leaves the pass out of the
    /// frame.
    pub(in crate::v2::view::draw::canvas::view3d) fn outlining(self, outline_class: Arc<[phiray::trace::Outline]>) -> Self {
        let any = outline_class.iter().any(|c| *c != phiray::trace::Outline::None);
        Self { outline_fx: [f32::from(u8::from(any)), self.outline_fx[1], self.outline_fx[2], self.outline_fx[3]], outline_class, ..self }
    }

    /// With the viewport's own region, for a traced mode: Workbench and
    /// Material Preview draw the whole viewport (`ED_view3d_calc_render_border`).
    pub(in crate::v2::view::draw::canvas::view3d) fn bordered(self, border: Option<[f32; 4]>) -> Self {
        let traced = !matches!(self.mode, phiray::Mode::Wire | phiray::Mode::Solid);
        Self { border: border.filter(|_| traced), ..self }
    }
}
