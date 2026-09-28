//! Viewport shading mode — Blender's `View3DShading.type`
//! (`V3D_SHADING_WIREFRAME / SOLID / MATERIAL / RENDERED`, `DNA_view3d_enums.h`).
//! Solid is Blender's default and colours objects with `Material.r/g/b`
//! (`color_type = MATERIAL`); Material Preview / Rendered use the shader
//! graph's Base Color. State key: `ctx.view3d.shading`.

use crate::v2::state::{BPath, BState};

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
            Self::Wireframe => "circle_dashed",
            Self::Solid => "circle",
            Self::Material => "sphere",
            Self::Rendered => "sun",
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

    /// Does a selection see through the surfaces (`XRAY_ENABLED`)?
    /// Wireframe does: `show_xray_wireframe` is on in the factory file, so
    /// a box select there takes what is behind as well.
    pub fn sees_through(self) -> bool {
        self == Self::Wireframe
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
