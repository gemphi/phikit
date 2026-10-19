//! Which engine a final render draws with (`Scene.r.engine`): the path
//! tracer for EEVEE, Cycles and Phiray, or — `BLENDER_WORKBENCH` — Solid /
//! Wireframe with `Scene.display.shading`, as Blender's Workbench render
//! engine draws F12 (`workbench_render.cc`: the scene's display shading,
//! not the viewport's, and `render_aa` for the samples).
//!
//! The engine is the **panel's**, not the file's. Properties ▸ Render ▸
//! Engine was a dropdown a render never read: a file saved with Workbench
//! rendered Workbench, and changing the row changed nothing — least of all
//! `PHIRAY`, which no `.blend` can name and which could therefore only ever
//! be chosen there. `BRenderSet` carries it now, so the row behaves like
//! every other row in that panel.

use super::BRenderSet;
use crate::phiphx::scene::SceneModel;
use crate::v2::view::draw::canvas::view3d::rendered::BStillLook;
use phiray::Camera;

/// Properties ▸ Render ▸ Engine (`part/props/render.rs`'s `ENGINE`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BRenderEngine {
    /// `BLENDER_EEVEE_NEXT`, the engine a fresh Blender scene names.
    Eevee,
    /// `BLENDER_WORKBENCH`.
    Workbench,
    /// `CYCLES`.
    Cycles,
    /// `PHIRAY` — phiber's own tracer, named as itself rather than as
    /// Cycles. No `.blend` carries it, and it is **the default**: it is
    /// the only engine phiber has for a traced frame
    /// (`render_quality::BRenderQuality::ENGINE`).
    #[default]
    Phiray,
}

impl BRenderEngine {
    /// The enum's identifier as the state and the file both spell it; an
    /// unknown name is EEVEE, as Blender falls back on a missing engine.
    pub fn of(name: &str) -> Self {
        match name {
            "BLENDER_WORKBENCH" => Self::Workbench,
            "CYCLES" => Self::Cycles,
            "PHIRAY" => Self::Phiray,
            _ => Self::Eevee,
        }
    }

    /// The identifier the state and a `.blend` both spell it with — the
    /// inverse of [`Self::of`], so a value read out of the Engine row and
    /// one written into it agree. The row stores identifiers
    /// (`props::render`'s `ENGINE`), not labels.
    pub fn ident(self) -> &'static str {
        match self {
            Self::Eevee => "BLENDER_EEVEE_NEXT",
            Self::Workbench => "BLENDER_WORKBENCH",
            Self::Cycles => "CYCLES",
            Self::Phiray => "PHIRAY",
        }
    }

    /// The name shown in the Info report and the Engine row.
    pub fn label(self) -> &'static str {
        match self {
            Self::Eevee => "EEVEE",
            Self::Workbench => "Workbench",
            Self::Cycles => "Cycles",
            Self::Phiray => "Phiray",
        }
    }

    /// Whether a final render path-traces: every engine but Workbench,
    /// which rasterizes `Scene.display.shading` instead.
    pub fn traces(self) -> bool {
        self != Self::Workbench
    }
}

impl BStillLook {
    /// `BLENDER_WORKBENCH`'s render: `Scene.display.shading` on the camera's
    /// words (its effects, lighting, colour, shadow and depth of field through
    /// `lens`, the scene camera's — the viewport's state plays no part),
    /// `render_aa` as the sample count, and no denoise; every
    /// other engine renders through the path tracer at the panel's settings.
    pub fn for_engine(model: &SceneModel, set: BRenderSet, lens: super::BCameraLens, camera: &mut Camera, plain: gpui::Hsla) -> (Self, u32, bool) {
        if set.engine.traces() {
            // The sample floor is applied where the panel is seeded
            // (`app::seed::seed_render_props`), not here: that way Max
            // Samples *shows* 2048 and a count typed into the panel is
            // still the count that renders.
            return (Self::new(crate::v2::scene::BShading::Rendered, plain), set.samples.max(1), set.denoise);
        }
        let d = &model.render.display;
        let fx = super::BSolidFx::of_file(&d.solid_fx, &model.render.ssao, &model.render.shadow);
        let shade = super::BSolidShade::of_file(&d.solid_shade);
        let shading = if d.shading == "wireframe" { crate::v2::scene::BShading::Wireframe } else { crate::v2::scene::BShading::Solid };
        camera.fx = fx.fx_words(1.0);
        camera.outline = fx.outline_words();
        camera.cavity = fx.cavity_words();
        camera.ssao = fx.ssao_words(d.samples);
        camera.shadow = fx.shadow_words();
        camera.shadow_fade = fx.shadow_fade_words();
        camera.shade = fx.shade_words(shade.shade_words());
        camera.single = shade.single_words();
        camera.matcap = shade.matcap_words();
        // Depth of Field through the scene camera, the frame the sensor's.
        camera.dof = fx.dof_words(Some(lens), 1.0);
        camera.see_through = super::BXray { solid: d.xray.solid, alpha: d.xray.alpha, wireframe: d.xray.wireframe, alpha_wireframe: d.xray.alpha_wireframe }.see_through(shading);
        if let Some([r, g, b]) = shade.background(Some(model.world.horizon_color)) {
            camera.sky = [r, g, b, 0.0];
        }
        // `render_aa` as it is: 0 and 1 are Workbench's off and single pass.
        (Self { shading, plain, matcap: shade.matcap_for_build(), texture: shade.texture_for_build() }, d.samples, false)
    }
}
