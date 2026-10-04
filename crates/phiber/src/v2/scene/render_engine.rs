//! Which engine a final render draws with (`Scene.r.engine`): the path
//! tracer for EEVEE and Cycles, or — `BLENDER_WORKBENCH` — Solid /
//! Wireframe with `Scene.display.shading`, as Blender's Workbench render
//! engine draws F12 (`workbench_render.cc`: the scene's display shading,
//! not the viewport's, and `render_aa` for the samples).

use super::BRenderSet;
use crate::phiphx::scene::SceneModel;
use crate::v2::view::draw::canvas::view3d::rendered::BStillLook;
use phiray::Camera;

impl BStillLook {
    /// `BLENDER_WORKBENCH`'s render: `Scene.display.shading` on the camera's
    /// words (its effects, lighting, colour and shadow — the viewport's state
    /// plays no part), `render_aa` as the sample count, and no denoise; every
    /// other engine renders through the path tracer at the panel's settings.
    pub fn for_engine(model: &SceneModel, set: BRenderSet, camera: &mut Camera, plain: gpui::Hsla) -> (Self, u32, bool) {
        if model.render.engine != "BLENDER_WORKBENCH" {
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
        camera.see_through = super::BXray { solid: d.xray.solid, alpha: d.xray.alpha, wireframe: d.xray.wireframe, alpha_wireframe: d.xray.alpha_wireframe }.see_through(shading);
        if let Some([r, g, b]) = shade.background(Some(model.world.horizon_color)) {
            camera.sky = [r, g, b, 0.0];
        }
        (Self { shading, plain, matcap: shade.matcap_for_build(), texture: shade.texture_for_build() }, d.samples.max(1), false)
    }
}
