//! Workbench's Solid effects as the Shading popover has them
//! (`VIEW3D_PT_shading_options`): the object outline
//! (`View3DShading.show_object_outline`, `object_outline_color`) and
//! Cavity (`show_cavity`, `cavity_type`, `cavity_valley_factor`,
//! `cavity_ridge_factor`, `curvature_ridge_factor`,
//! `curvature_valley_factor`) with its World-space settings
//! (`Scene.display.matcap_ssao_samples`, `_distance`, `_attenuation`,
//! `VIEW3D_PT_shading_options_ssao`), and Shadow (`show_shadows`,
//! `shadow_intensity`; `Scene.display.light_direction`, `shadow_shift`,
//! `shadow_focus`, `VIEW3D_PT_shading_options_shadow`), with Blender's
//! startup file's values until a file says otherwise, read by the viewport
//! into `phiray::Camera::fx`, `::outline`, `::cavity`, `::ssao`, `::shadow`
//! and `::shadow_fade`.

use crate::phiphx::scene::SceneSolidFx;
use crate::v2::state::val::{BValue, BValueKind, V_FALSE, V_TRUE};
use crate::v2::state::{BKey as _, BState, View3dKey};

/// The effects as the state has them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BSolidFx {
    pub outline: bool,
    /// Display-space, as the popover's colour swatch shows it (kept as a
    /// `BValue::Color`, so the swatch edits it).
    pub outline_color: [f32; 3],
    pub cavity: bool,
    /// `WORLD` (screen-space ambient occlusion), `SCREEN` (curvature) or `BOTH`.
    pub cavity_type: &'static str,
    pub cavity_valley_factor: f32,
    pub cavity_ridge_factor: f32,
    pub curvature_ridge_factor: f32,
    pub curvature_valley_factor: f32,
    /// Cavity ▸ World's SSAO (`Scene.display`): the disk samples a frame
    /// (1 ..= 500), how far out it reaches (Blender units) and its
    /// attenuation — kept beside the view's shading keys though they are
    /// the scene's.
    pub ssao_samples: u32,
    pub ssao_distance: f32,
    pub ssao_attenuation: f32,
    /// Shadow and its intensity (`View3DShading`), the light's direction as
    /// the panel shows it, Offset and Focus (`Scene.display`, kept beside
    /// the view's keys as the SSAO settings are).
    pub shadow: bool,
    pub shadow_intensity: f32,
    pub light_direction: [f32; 3],
    pub shadow_shift: f32,
    pub shadow_focus: f32,
    /// `show_backface_culling`, `show_specular_highlight`.
    pub backface_culling: bool,
    pub specular: bool,
}

/// The RNA names, which are the keys' tails (`view3d.shade.<rna>`); the
/// direction is three keys, one an axis.
pub const RNA: [&str; 20] = ["show_object_outline", "object_outline_color", "show_cavity", "cavity_type", "cavity_valley_factor", "cavity_ridge_factor", "curvature_ridge_factor", "curvature_valley_factor", "matcap_ssao_samples", "matcap_ssao_distance", "matcap_ssao_attenuation", "show_shadows", "shadow_intensity", "light_direction_x", "light_direction_y", "light_direction_z", "shadow_shift", "shadow_focus", "show_backface_culling", "show_specular_highlight"];

impl BSolidFx {
    /// The startup file's: the outline on and black, Cavity off, its type
    /// Screen (curvature), every factor 1.
    pub const FACTORY: Self = Self { outline: true, outline_color: [0.0; 3], cavity: false, cavity_type: "SCREEN", cavity_valley_factor: 1.0, cavity_ridge_factor: 1.0, curvature_ridge_factor: 1.0, curvature_valley_factor: 1.0, ssao_samples: 16, ssao_distance: 0.2, ssao_attenuation: 1.0, shadow: false, shadow_intensity: 0.5, light_direction: [0.577_350_3; 3], shadow_shift: 0.1, shadow_focus: 0.0, backface_culling: false, specular: true };

    pub fn now(s: &BState) -> Self {
        let f = Self::FACTORY;
        let on = |rna: &'static str, default: bool| s.get(View3dKey::Shade(rna)).map_or(default, |v| v == V_TRUE);
        let factor = |rna: &'static str, default: f32| s.get(View3dKey::Shade(rna)).and_then(|v| v.parse::<f32>().ok()).map_or(default, |v| v.clamp(0.0, 2.5));
        // `matcap_ssao_distance` and `_attenuation` range `0 ..= 100000`.
        let wide = |rna: &'static str, default: f32| s.get(View3dKey::Shade(rna)).and_then(|v| v.parse::<f32>().ok()).map_or(default, |v| v.clamp(0.0, 100_000.0));
        let unit = |rna: &'static str, default: f32| s.get(View3dKey::Shade(rna)).and_then(|v| v.parse::<f32>().ok()).map_or(default, |v| v.clamp(0.0, 1.0));
        let any = |rna: &'static str, default: f32| s.get(View3dKey::Shade(rna)).and_then(|v| v.parse::<f32>().ok()).unwrap_or(default);
        let color = match s.value(&View3dKey::Shade(RNA[1]).path(), BValueKind::Color) {
            Some(BValue::Color([r, g, b, _])) => [r, g, b],
            _ => f.outline_color,
        };
        let cavity_type = match s.get(View3dKey::Shade(RNA[3])).as_deref() {
            Some("WORLD") => "WORLD",
            Some("BOTH") => "BOTH",
            Some("SCREEN") => "SCREEN",
            _ => f.cavity_type,
        };
        Self {
            outline: on(RNA[0], f.outline),
            outline_color: color,
            cavity: on(RNA[2], f.cavity),
            cavity_type,
            cavity_valley_factor: factor(RNA[4], f.cavity_valley_factor),
            cavity_ridge_factor: factor(RNA[5], f.cavity_ridge_factor),
            curvature_ridge_factor: factor(RNA[6], f.curvature_ridge_factor),
            curvature_valley_factor: factor(RNA[7], f.curvature_valley_factor),
            ssao_samples: s.get(View3dKey::Shade(RNA[8])).and_then(|v| v.parse::<f32>().ok()).map_or(f.ssao_samples, |v| (v as i64).clamp(1, 500) as u32),
            ssao_distance: wide(RNA[9], f.ssao_distance),
            ssao_attenuation: wide(RNA[10], f.ssao_attenuation),
            shadow: on(RNA[11], f.shadow),
            shadow_intensity: unit(RNA[12], f.shadow_intensity),
            light_direction: [any(RNA[13], f.light_direction[0]), any(RNA[14], f.light_direction[1]), any(RNA[15], f.light_direction[2])],
            shadow_shift: unit(RNA[16], f.shadow_shift),
            shadow_focus: unit(RNA[17], f.shadow_focus),
            backface_culling: on(RNA[18], f.backface_culling),
            specular: on(RNA[19], f.specular),
        }
    }

    /// A file's shading as it stands — the Workbench render engine's
    /// `Scene.display.shading` with the scene's SSAO and shadow settings —
    /// without the state (`scene::render_job`).
    pub fn of_file(fx: &SceneSolidFx, ssao: &crate::phiphx::scene::SceneSsao, shadow: &crate::phiphx::scene::SceneShadow) -> Self {
        Self {
            outline: fx.outline,
            outline_color: fx.outline_color,
            cavity: fx.cavity,
            cavity_type: fx.cavity_type,
            cavity_valley_factor: fx.cavity_valley_factor,
            cavity_ridge_factor: fx.cavity_ridge_factor,
            curvature_ridge_factor: fx.curvature_ridge_factor,
            curvature_valley_factor: fx.curvature_valley_factor,
            ssao_samples: ssao.samples.clamp(1, 500),
            ssao_distance: ssao.distance,
            ssao_attenuation: ssao.attenuation,
            shadow: fx.shadow,
            shadow_intensity: fx.shadow_intensity,
            light_direction: shadow.light_direction,
            shadow_shift: shadow.shift,
            shadow_focus: shadow.focus,
            backface_culling: fx.backface_culling,
            specular: fx.specular,
        }
    }

    /// A file's view's effects, into the state (`app::load_ui`).
    pub fn seed(s: &mut BState, fx: &SceneSolidFx) {
        let on = |b: bool| if b { V_TRUE } else { V_FALSE };
        s.set(View3dKey::Shade(RNA[0]), on(fx.outline));
        s.set(View3dKey::Shade(RNA[1]), BValue::Color([fx.outline_color[0], fx.outline_color[1], fx.outline_color[2], 1.0]).encode());
        s.set(View3dKey::Shade(RNA[2]), on(fx.cavity));
        s.set(View3dKey::Shade(RNA[3]), fx.cavity_type.to_string());
        for (rna, v) in RNA[4..8].iter().zip([fx.cavity_valley_factor, fx.cavity_ridge_factor, fx.curvature_ridge_factor, fx.curvature_valley_factor]) {
            s.set(View3dKey::Shade(rna), v.to_string());
        }
        s.set(View3dKey::Shade(RNA[11]), on(fx.shadow));
        s.set(View3dKey::Shade(RNA[12]), fx.shadow_intensity.to_string());
        s.set(View3dKey::Shade(RNA[18]), on(fx.backface_culling));
        s.set(View3dKey::Shade(RNA[19]), on(fx.specular));
    }

    /// `phiray::Camera::shade`'s last two words over `BSolidShade`'s first
    /// two: backface culling on, specular lighting off (the kernel's words
    /// are 0 at Blender's defaults).
    pub fn shade_words(&self, shade: [u32; 4]) -> [u32; 4] {
        [shade[0], shade[1], u32::from(self.backface_culling), u32::from(!self.specular)]
    }

    /// A file's `Scene.display` shadow settings, into the state (`app::load_ui`).
    pub fn seed_shadow(s: &mut BState, sh: &crate::phiphx::scene::SceneShadow) {
        for (rna, v) in RNA[13..].iter().zip([sh.light_direction[0], sh.light_direction[1], sh.light_direction[2], sh.shift, sh.focus]) {
            s.set(View3dKey::Shade(rna), v.to_string());
        }
    }

    /// `phiray::Camera::shadow`: `ShadowPass::init`'s `direction_ws` — the
    /// panel's direction with y and z swapped and x and z negated, unit — and
    /// `shadow_intensity`, 0 when Shadow is off.
    pub fn shadow_words(&self) -> [f32; 4] {
        let [x, y, z] = self.light_direction;
        let d = glam::Vec3::new(-x, z, -y).try_normalize().unwrap_or(glam::Vec3::new(-1.0, 1.0, -1.0).normalize());
        [d.x, d.y, d.z, if self.shadow { self.shadow_intensity } else { 0.0 }]
    }

    /// `phiray::Camera::shadow_fade`: `shadow_shift` and `1 − clamp(focus,
    /// 0.0001, 0.99999) (1 − shift)`, held above the shift so the
    /// `smoothstep` has a width.
    pub fn shadow_fade_words(&self) -> [f32; 4] {
        let focus = 1.0 - self.shadow_focus.clamp(0.0001, 0.99999) * (1.0 - self.shadow_shift);
        [self.shadow_shift, focus.max(self.shadow_shift + 1e-5), 0.0, 0.0]
    }

    /// A file's `Scene.display` SSAO settings, into the state (`app::load_ui`).
    pub fn seed_ssao(s: &mut BState, ssao: &crate::phiphx::scene::SceneSsao) {
        s.set(View3dKey::Shade(RNA[8]), BValue::Int(i64::from(ssao.samples)).encode());
        s.set(View3dKey::Shade(RNA[9]), ssao.distance.to_string());
        s.set(View3dKey::Shade(RNA[10]), ssao.attenuation.to_string());
    }

    /// Whether the cavity (SSAO) effect draws (`SceneState::draw_cavity`):
    /// Cavity on and its type World or Both.
    pub fn ssao(&self) -> bool {
        self.cavity && matches!(self.cavity_type, "WORLD" | "BOTH")
    }

    /// `phiray::Camera::cavity`: `CavityEffect::init`'s `cavity_valley_factor`,
    /// `cavity_ridge_factor`, `cavity_attenuation` and `cavity_distance`.
    pub fn cavity_words(&self) -> [f32; 4] {
        [self.cavity_valley_factor, self.cavity_ridge_factor, self.ssao_attenuation, self.ssao_distance]
    }

    /// `phiray::Camera::ssao`: the disk samples a frame — 0 when the effect
    /// is off — and the anti-aliasing sample count the frames run over.
    pub fn ssao_words(&self, samples_len: u32) -> [u32; 4] {
        [if self.ssao() { self.ssao_samples } else { 0 }, samples_len.max(1), 0, 0]
    }

    /// Whether the curvature effect draws (`SceneState::draw_curvature`):
    /// Cavity on and its type Screen or Both.
    pub fn curvature(&self) -> bool {
        self.cavity && matches!(self.cavity_type, "SCREEN" | "BOTH")
    }

    /// `phiray::Camera::fx`: the outline switch and `CavityEffect::init`'s
    /// curvature words — `0.5 / ridge²`, `0.7 / valley²`, 0 when the effect
    /// is off — and the pixel size.
    pub fn fx_words(&self, pixel_size: f32) -> [f32; 4] {
        let (ridge, valley) = if self.curvature() { (0.5 / self.curvature_ridge_factor.powi(2).max(1e-4), 0.7 / self.curvature_valley_factor.powi(2).max(1e-4)) } else { (0.0, 0.0) };
        [f32::from(u8::from(self.outline)), ridge, valley, pixel_size.max(1.0)]
    }

    /// `phiray::Camera::outline`: the outline colour, scene-linear, as
    /// Workbench blends it (`object_outline_color` is a `PROP_COLOR`).
    pub fn outline_words(&self) -> [f32; 4] {
        let lin = phiray::Srgb::to_linear;
        [lin(self.outline_color[0]), lin(self.outline_color[1]), lin(self.outline_color[2]), 1.0]
    }
}

#[cfg(test)]
#[path = "solid_fx_tests.rs"]
mod tests;
