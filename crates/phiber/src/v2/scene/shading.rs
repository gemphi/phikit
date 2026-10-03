//! Viewport shading mode — Blender's `View3DShading.type`
//! (`V3D_SHADING_WIREFRAME / SOLID / MATERIAL / RENDERED`, `DNA_view3d_enums.h`).
//! Solid is Blender's default and colours objects with `Material.r/g/b`
//! (`color_type = MATERIAL`); Material Preview / Rendered use the shader
//! graph's Base Color. State key: `ctx.view3d.shading`.
//!
//! X-ray (`BXray`, Alt Z) belongs to the shading: Solid and Wireframe each
//! keep their own switch and alpha, as `View3DShading` does.

use crate::v2::state::{BPath, BState, View3dKey, val::V_TRUE};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BShading {
    Wireframe,
    #[default]
    Solid,
    Material,
    Rendered,
}

impl BShading {
    pub const ALL: [BShading; 4] = [Self::Wireframe, Self::Solid, Self::Material, Self::Rendered];

    pub fn slug(self) -> &'static str {
        match self {
            Self::Wireframe => "wireframe",
            Self::Solid => "solid",
            Self::Material => "material",
            Self::Rendered => "rendered",
        }
    }

    pub fn from_slug(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|m| m.slug() == s)
    }

    /// Header icon (Blender `SHADING_WIRE / _SOLID / _TEXTURE / _RENDERED`,
    /// mapped onto Phiber's `assets/icons`).
    pub fn icon(self) -> &'static str {
        match self {
            // Blender's four: a wire cage, a lit sphere, a material ball, a
            // camera render. The generic circles these were drawn with said
            // nothing about which mode they picked — two of them were the
            // same circle with a different dash. `globe` and `grid_3x3` read
            // better for the wire mode but are both still the placeholder
            // drawing (`icon_coverage`), so the cage is `box`.
            Self::Wireframe => "box",
            Self::Solid => "shading_solid",
            Self::Material => "material",
            Self::Rendered => "render",
        }
    }

    /// Tooltip / label, Blender's `View3DShading.type` enum names.
    pub fn label(self) -> &'static str {
        match self {
            Self::Wireframe => "Wireframe",
            Self::Solid => "Solid",
            Self::Material => "Material Preview",
            Self::Rendered => "Rendered",
        }
    }

    /// Whether object colour comes from the material's viewport display
    /// colour (Wireframe) or the theme's solid shade (Solid) rather than
    /// the shader Base Color — either way, no texture.
    pub fn uses_viewport_color(self) -> bool {
        matches!(self, Self::Wireframe | Self::Solid)
    }
}

pub fn shading_key() -> BPath {
    use crate::v2::state::BKey as _;
    crate::v2::state::View3dKey::Shading.path()
}

pub fn view_shading(s: &BState) -> BShading {
    s.get(&shading_key())
        .and_then(BShading::from_slug)
        .unwrap_or_default()
}

/// X-ray — `View3DShading.show_xray` / `xray_alpha` for Solid,
/// `show_xray_wireframe` / `xray_alpha_wireframe` for Wireframe — with the
/// factory file's values until a file or Alt Z says otherwise.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BXray {
    pub solid: bool,
    pub alpha: f32,
    pub wireframe: bool,
    pub alpha_wireframe: f32,
}

impl BXray {
    /// The factory file's: off in Solid at 0.5, on in Wireframe at 0.
    pub const FACTORY: Self = Self { solid: false, alpha: 0.5, wireframe: true, alpha_wireframe: 0.0 };

    pub fn now(s: &BState) -> Self {
        let on = |key: View3dKey, default: bool| s.get(key).map_or(default, |v| v == V_TRUE);
        let alpha = |key: View3dKey, default: f32| s.get(key).and_then(|v| v.parse::<f32>().ok()).map_or(default, |v| v.clamp(0.0, 1.0));
        let f = Self::FACTORY;
        Self {
            solid: on(View3dKey::Xray, f.solid),
            alpha: alpha(View3dKey::XrayAlpha, f.alpha),
            wireframe: on(View3dKey::XrayWireframe, f.wireframe),
            alpha_wireframe: alpha(View3dKey::XrayAlphaWireframe, f.alpha_wireframe),
        }
    }

    /// The switch `view3d.toggle_xray` flips in `shading`
    /// (`SHADING_XRAY_FLAG`): Wireframe's own, else Solid's.
    pub fn key(shading: BShading) -> View3dKey {
        if shading == BShading::Wireframe { View3dKey::XrayWireframe } else { View3dKey::Xray }
    }

    /// Is the switch for `shading` on (`XRAY_FLAG_ENABLED`) — the header
    /// button's pressed state.
    pub fn on(&self, shading: BShading) -> bool {
        if shading == BShading::Wireframe { self.wireframe } else { self.solid }
    }

    /// `SHADING_XRAY_ALPHA`.
    pub fn alpha(&self, shading: BShading) -> f32 {
        if shading == BShading::Wireframe { self.alpha_wireframe } else { self.alpha }
    }

    /// `XRAY_ACTIVE`: on, the alpha under 1, and a Workbench shading — what
    /// draws through and what a selection sees through.
    pub fn active(&self, shading: BShading) -> bool {
        self.on(shading) && self.alpha(shading) < 1.0 && matches!(shading, BShading::Wireframe | BShading::Solid)
    }

    /// Can X-ray show in `shading` at all (the header button is dimmed where
    /// it cannot: Material Preview and Rendered).
    pub fn available(shading: BShading) -> bool {
        matches!(shading, BShading::Wireframe | BShading::Solid)
    }

    /// How much every surface lets through, for the renderer: `1 − alpha`
    /// while active, 0 otherwise (`phiray::Camera::see_through`).
    pub fn see_through(&self, shading: BShading) -> f32 {
        if self.active(shading) { 1.0 - self.alpha(shading) } else { 0.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xray_is_on_in_wireframe_and_off_in_solid_until_toggled() {
        let mut s = BState::new();
        let x = BXray::now(&s);
        assert!(x.active(BShading::Wireframe) && !x.active(BShading::Solid), "the factory file's");
        assert_eq!(x.see_through(BShading::Wireframe), 1.0, "alpha 0: every edge, whole");
        s.set(BXray::key(BShading::Solid), V_TRUE);
        let x = BXray::now(&s);
        assert!(x.active(BShading::Solid) && !x.active(BShading::Material), "Material Preview has none");
        assert_eq!(x.see_through(BShading::Solid), 0.5);
        s.set(View3dKey::XrayAlpha, "1");
        assert!(!BXray::now(&s).active(BShading::Solid), "alpha 1 is X-ray off (`XRAY_ENABLED`)");
    }
}
