//! Solid's Lighting, Color and Background as the Shading popover has them
//! (`VIEW3D_PT_shading_lighting`, `_color`): `View3DShading.light`
//! (`STUDIO`, `MATCAP`, `FLAT`), `color_type` (`MATERIAL`, `RANDOM`,
//! `SINGLE`, `TEXTURE`, `OBJECT`, `VERTEX`) with `single_color`, and
//! `background_type` (`THEME`, `WORLD`, `VIEWPORT`) with `background_color`
//! — Blender's defaults until a file says otherwise, read by the viewport
//! into `phiray::Camera::shade` / `::single` (`solid.wgsl`) and the sky
//! (`rendered/world.rs`).

use crate::phiphx::scene::SceneSolidShade;
use crate::v2::state::val::{BValue, BValueKind};
use crate::v2::state::{BKey as _, BState, View3dKey};

/// The settings as the state has them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BSolidShade {
    pub light: &'static str,
    pub color_type: &'static str,
    /// Display-space, as the swatch shows it.
    pub single_color: [f32; 3],
    pub background_type: &'static str,
    pub background_color: [f32; 3],
}

/// The RNA names, the keys' tails (`view3d.shade.<rna>`).
pub const RNA: [&str; 5] = ["light", "color_type", "single_color", "background_type", "background_color"];
pub const LIGHTS: [(&str, &str); 3] = [("STUDIO", "Studio"), ("MATCAP", "MatCap"), ("FLAT", "Flat")];
pub const COLORS: [(&str, &str); 6] = [("MATERIAL", "Material"), ("OBJECT", "Object"), ("VERTEX", "Attribute"), ("SINGLE", "Single"), ("RANDOM", "Random"), ("TEXTURE", "Texture")];
pub const BACKGROUNDS: [(&str, &str); 3] = [("THEME", "Theme"), ("WORLD", "World"), ("VIEWPORT", "Viewport")];

fn item<'a>(items: &[(&'a str, &str)], raw: Option<&str>, default: &'a str) -> &'a str {
    items.iter().find(|(k, _)| Some(*k) == raw).map_or(default, |(k, _)| k)
}

impl BSolidShade {
    /// `View3DShading`'s defaults: Studio lighting, the material colour, a
    /// 0.8 grey single colour, the theme background, 0.05 grey.
    pub const FACTORY: Self = Self { light: "STUDIO", color_type: "MATERIAL", single_color: [0.8; 3], background_type: "THEME", background_color: [0.05; 3] };

    pub fn now(s: &BState) -> Self {
        let f = Self::FACTORY;
        let color = |rna: &'static str, default: [f32; 3]| match s.value(&View3dKey::Shade(rna).path(), BValueKind::Color) {
            Some(BValue::Color([r, g, b, _])) => [r, g, b],
            _ => default,
        };
        Self {
            light: item(&LIGHTS, s.get(View3dKey::Shade(RNA[0])).as_deref(), f.light),
            color_type: item(&COLORS, s.get(View3dKey::Shade(RNA[1])).as_deref(), f.color_type),
            single_color: color(RNA[2], f.single_color),
            background_type: item(&BACKGROUNDS, s.get(View3dKey::Shade(RNA[3])).as_deref(), f.background_type),
            background_color: color(RNA[4], f.background_color),
        }
    }

    /// A file's view's settings, into the state (`app::load_ui`).
    pub fn seed(s: &mut BState, sh: &SceneSolidShade) {
        s.set(View3dKey::Shade(RNA[0]), sh.light.to_string());
        s.set(View3dKey::Shade(RNA[1]), sh.color_type.to_string());
        s.set(View3dKey::Shade(RNA[2]), BValue::Color([sh.single_color[0], sh.single_color[1], sh.single_color[2], 1.0]).encode());
        s.set(View3dKey::Shade(RNA[3]), sh.background_type.to_string());
        s.set(View3dKey::Shade(RNA[4]), BValue::Color([sh.background_color[0], sh.background_color[1], sh.background_color[2], 1.0]).encode());
    }

    /// `phiray::Camera::shade`: the kernel's light word (Studio 0, MatCap
    /// 1 — drawn as Studio until matcaps are in — Flat 2) and
    /// `eV3DShadingColorType`'s number.
    pub fn shade_words(&self) -> [u32; 4] {
        let light = match self.light { "FLAT" => 2, "MATCAP" => 1, _ => 0 };
        let color = match self.color_type { "RANDOM" => 1, "SINGLE" => 2, "TEXTURE" => 3, "OBJECT" => 4, "VERTEX" => 5, _ => 0 };
        [light, color, 0, 0]
    }

    /// `phiray::Camera::single`: the single colour, scene-linear.
    pub fn single_words(&self) -> [f32; 4] {
        let c = self.single_color.map(to_linear);
        [c[0], c[1], c[2], 1.0]
    }

    /// The sky behind a Solid or Wireframe view, display-space, when the
    /// Background is not the theme's (`ED_view3d_background_color_get`):
    /// the world's horizon colour (`World.horr`) for World, the chosen colour for
    /// Viewport; `None` for Theme, and for World without a world.
    pub fn background(&self, world: Option<[f32; 3]>) -> Option<[f32; 3]> {
        match self.background_type {
            "WORLD" => world.map(|w| w.map(|c| c.max(0.0).powf(1.0 / 2.2))),
            "VIEWPORT" => Some(self.background_color),
            _ => None,
        }
    }
}

fn to_linear(c: f32) -> f32 {
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_factory_is_blenders_and_a_files_values_reach_the_camera_words() {
        let mut s = BState::new();
        let f = BSolidShade::now(&s);
        assert_eq!(f, BSolidShade::FACTORY);
        assert_eq!(f.shade_words(), [0, 0, 0, 0], "Studio, Material: the kernel's defaults");
        assert_eq!(f.background(Some([0.5; 3])), None, "Theme: the theme's sky");
        let file = SceneSolidShade { light: "FLAT", color_type: "RANDOM", single_color: [1.0, 0.0, 0.0], background_type: "VIEWPORT", background_color: [0.2, 0.3, 0.4] };
        BSolidShade::seed(&mut s, &file);
        let f = BSolidShade::now(&s);
        assert_eq!((f.light, f.color_type), ("FLAT", "RANDOM"));
        assert_eq!(f.shade_words(), [2, 1, 0, 0]);
        assert_eq!(f.background(None), Some([0.2, 0.3, 0.4]));
        s.set(View3dKey::Shade("background_type"), "WORLD");
        assert_eq!(BSolidShade::now(&s).background(None), None, "World without a world: the theme");
        assert!((BSolidShade::now(&s).background(Some([0.25; 3])).unwrap()[0] - 0.25f32.powf(1.0 / 2.2)).abs() < 1e-5);
        s.set(View3dKey::Shade("color_type"), "SINGLE");
        assert!((BSolidShade::now(&s).single_words()[0] - 1.0).abs() < 1e-6 && BSolidShade::now(&s).single_words()[1] == 0.0);
        s.set(View3dKey::Shade("light"), "nonsense");
        assert_eq!(BSolidShade::now(&s).light, "STUDIO", "an unknown item is the default");
    }
}
