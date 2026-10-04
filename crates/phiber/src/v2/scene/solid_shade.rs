//! Solid's Lighting, Color and Background as the Shading popover has them
//! (`VIEW3D_PT_shading_lighting`, `_color`): `View3DShading.light`
//! (`STUDIO`, `MATCAP`, `FLAT`), `color_type` (`MATERIAL`, `RANDOM`,
//! `SINGLE`, `TEXTURE`, `OBJECT`, `VERTEX`) with `single_color`, and
//! `background_type` (`THEME`, `WORLD`, `VIEWPORT`) with `background_color`,
//! and MatCap's image (`matcap`, `V3D_SHADING_MATCAP_FLIP_X`; `scene::matcap`)
//! — Blender's defaults until a file says otherwise, read by the viewport
//! into `phiray::Camera::shade` / `::single` / `::matcap` (`solid.wgsl`) and
//! the sky (`rendered/world.rs`).

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
    /// The matcap shown under MatCap lighting — one of `Matcap::name_list()`,
    /// the default for a name the set lacks — and whether it is flipped.
    pub matcap: &'static str,
    pub matcap_flip_x: bool,
}

/// The RNA names, the keys' tails (`view3d.shade.<rna>`).
pub const RNA: [&str; 7] = ["light", "color_type", "single_color", "background_type", "background_color", "matcap", "matcap_flip_x"];
pub const LIGHT_LIST: [(&str, &str); 3] = [("STUDIO", "Studio"), ("MATCAP", "MatCap"), ("FLAT", "Flat")];
pub const COLOR_LIST: [(&str, &str); 6] = [("MATERIAL", "Material"), ("OBJECT", "Object"), ("VERTEX", "Attribute"), ("SINGLE", "Single"), ("RANDOM", "Random"), ("TEXTURE", "Texture")];
pub const BACKGROUND_LIST: [(&str, &str); 3] = [("THEME", "Theme"), ("WORLD", "World"), ("VIEWPORT", "Viewport")];

impl BSolidShade {
    /// The item of `list` named `raw`, else `default`.
    fn item<'a>(list: &[(&'a str, &str)], raw: Option<&str>, default: &'a str) -> &'a str {
        list.iter().find(|(k, _)| Some(*k) == raw).map_or(default, |(k, _)| k)
    }

    /// `View3DShading`'s defaults: Studio lighting, the material colour, a
    /// 0.8 grey single colour, the theme background, 0.05 grey.
    pub const FACTORY: Self = Self { light: "STUDIO", color_type: "MATERIAL", single_color: [0.8; 3], background_type: "THEME", background_color: [0.05; 3], matcap: super::matcap::Matcap::DEFAULT, matcap_flip_x: false };

    pub fn now(s: &BState) -> Self {
        let f = Self::FACTORY;
        let color = |rna: &'static str, default: [f32; 3]| match s.value(&View3dKey::Shade(rna).path(), BValueKind::Color) {
            Some(BValue::Color([r, g, b, _])) => [r, g, b],
            _ => default,
        };
        Self {
            light: Self::item(&LIGHT_LIST, s.get(View3dKey::Shade(RNA[0])).as_deref(), f.light),
            color_type: Self::item(&COLOR_LIST, s.get(View3dKey::Shade(RNA[1])).as_deref(), f.color_type),
            single_color: color(RNA[2], f.single_color),
            background_type: Self::item(&BACKGROUND_LIST, s.get(View3dKey::Shade(RNA[3])).as_deref(), f.background_type),
            background_color: color(RNA[4], f.background_color),
            matcap: s.get(View3dKey::Shade(RNA[5])).and_then(|name| super::matcap::Matcap::name_list().find(|n| *n == name)).unwrap_or(f.matcap),
            matcap_flip_x: s.get(View3dKey::Shade(RNA[6])).is_some_and(|v| v == crate::v2::state::val::V_TRUE),
        }
    }

    /// A file's shading as it stands, without the state — the Workbench
    /// render engine's `Scene.display.shading` (`scene::render_job`); a
    /// matcap the set lacks is the default, as `now` resolves it.
    pub fn of_file(sh: &SceneSolidShade) -> Self {
        Self {
            light: Self::item(&LIGHT_LIST, Some(sh.light), Self::FACTORY.light),
            color_type: Self::item(&COLOR_LIST, Some(sh.color_type), Self::FACTORY.color_type),
            single_color: sh.single_color,
            background_type: Self::item(&BACKGROUND_LIST, Some(sh.background_type), Self::FACTORY.background_type),
            background_color: sh.background_color,
            matcap: super::matcap::Matcap::name_list().find(|n| *n == sh.matcap).unwrap_or(super::matcap::Matcap::DEFAULT),
            matcap_flip_x: sh.matcap_flip_x,
        }
    }

    /// A file's view's settings, into the state (`app::load_ui`).
    pub fn seed(s: &mut BState, sh: &SceneSolidShade) {
        s.set(View3dKey::Shade(RNA[0]), sh.light.to_string());
        s.set(View3dKey::Shade(RNA[1]), sh.color_type.to_string());
        s.set(View3dKey::Shade(RNA[2]), BValue::Color([sh.single_color[0], sh.single_color[1], sh.single_color[2], 1.0]).encode());
        s.set(View3dKey::Shade(RNA[3]), sh.background_type.to_string());
        s.set(View3dKey::Shade(RNA[4]), BValue::Color([sh.background_color[0], sh.background_color[1], sh.background_color[2], 1.0]).encode());
        s.set(View3dKey::Shade(RNA[5]), sh.matcap.clone());
        s.set(View3dKey::Shade(RNA[6]), if sh.matcap_flip_x { crate::v2::state::val::V_TRUE } else { crate::v2::state::val::V_FALSE });
    }

    /// `phiray::Camera::matcap`: the atlas slots the build puts the matcap's
    /// diffuse and specular images in, plus one (0: none — Studio, Flat, or
    /// a matcap without a specular pass), and the flip; all 0 off MatCap.
    pub fn matcap_words(&self) -> [u32; 4] {
        if self.light != "MATCAP" {
            return [0; 4];
        }
        let m = super::matcap::Matcap::named(self.matcap);
        [super::matcap::Matcap::DIFFUSE_SLOT + 1, if m.specular.is_some() { super::matcap::Matcap::SPECULAR_SLOT + 1 } else { 0 }, u32::from(self.matcap_flip_x), 0]
    }

    /// The matcap the Solid build uploads: the chosen one under MatCap
    /// lighting, none otherwise (`BMeshPalette::matcap`).
    pub fn matcap_for_build(&self) -> Option<&'static str> {
        (self.light == "MATCAP").then_some(self.matcap)
    }

    /// Whether the Solid build uploads the materials' base-colour images:
    /// Color ▸ Texture (`BMeshPalette::texture`).
    pub fn texture_for_build(&self) -> bool {
        self.color_type == "TEXTURE"
    }

    /// `phiray::Camera::shade`: the kernel's light word (Studio 0, MatCap
    /// 1, Flat 2) and
    /// `eV3DShadingColorType`'s number.
    pub fn shade_words(&self) -> [u32; 4] {
        let light = match self.light { "FLAT" => 2, "MATCAP" => 1, _ => 0 };
        let color = match self.color_type { "RANDOM" => 1, "SINGLE" => 2, "TEXTURE" => 3, "OBJECT" => 4, "VERTEX" => 5, _ => 0 };
        [light, color, 0, 0]
    }

    /// `phiray::Camera::single`: the single colour, scene-linear.
    pub fn single_words(&self) -> [f32; 4] {
        let c = self.single_color.map(phiray::Srgb::to_linear);
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
        assert_eq!(f.matcap_words(), [0; 4], "Studio: no matcap");
        let file = SceneSolidShade { light: "FLAT", color_type: "RANDOM", matcap: "clay_studio".to_string(), matcap_flip_x: true, single_color: [1.0, 0.0, 0.0], background_type: "VIEWPORT", background_color: [0.2, 0.3, 0.4] };
        BSolidShade::seed(&mut s, &file);
        let f = BSolidShade::now(&s);
        assert_eq!((f.light, f.color_type), ("FLAT", "RANDOM"));
        assert_eq!((f.matcap, f.matcap_flip_x), ("clay_studio", true));
        assert_eq!(f.matcap_for_build(), None, "Flat uploads no matcap");
        assert!(!f.texture_for_build());
        s.set(View3dKey::Shade("color_type"), "TEXTURE");
        assert!(BSolidShade::now(&s).texture_for_build(), "Texture uploads the base-colour images");
        s.set(View3dKey::Shade("color_type"), "RANDOM");
        s.set(View3dKey::Shade("light"), "MATCAP");
        let m = BSolidShade::now(&s);
        assert_eq!(m.matcap_for_build(), Some("clay_studio"));
        assert_eq!(m.matcap_words(), [1, 2, 1, 0], "diffuse in slot 0, specular in 1, flipped");
        s.set(View3dKey::Shade("matcap"), "check_normal+y");
        assert_eq!(BSolidShade::now(&s).matcap_words()[1], 0, "no specular pass");
        s.set(View3dKey::Shade("matcap"), "basic_1");
        assert_eq!(BSolidShade::now(&s).matcap, super::super::matcap::Matcap::DEFAULT, "Blender's old default name falls to the first matcap");
        s.set(View3dKey::Shade("light"), "FLAT");
        assert_eq!(f.shade_words(), [2, 1, 0, 0]);
        assert_eq!(f.background(None), Some([0.2, 0.3, 0.4]));
        s.set(View3dKey::Shade("background_type"), "WORLD");
        assert_eq!(BSolidShade::now(&s).background(None), None, "World without a world: the theme");
        assert!((BSolidShade::now(&s).background(Some([0.25; 3])).unwrap()[0] - 0.25f32.powf(1.0 / 2.2)).abs() < 1e-5);
        s.set(View3dKey::Shade("color_type"), "SINGLE");
        assert!((BSolidShade::now(&s).single_words()[0] - 1.0).abs() < 1e-6 && BSolidShade::now(&s).single_words()[1] == 0.0);
        s.set(View3dKey::Shade("light"), "nonsense");
        assert_eq!(BSolidShade::now(&s).light, "STUDIO", "an unknown item is the default");
        let f = BSolidShade::of_file(&SceneSolidShade { light: "MATCAP", color_type: "SINGLE", matcap: "no such".to_string(), matcap_flip_x: false, single_color: [0.5; 3], background_type: "WORLD", background_color: [0.0; 3] });
        assert_eq!((f.light, f.color_type, f.matcap, f.background_type), ("MATCAP", "SINGLE", super::super::matcap::Matcap::DEFAULT, "WORLD"), "a file's shading without the state");
        assert_eq!(f.shade_words(), [1, 2, 0, 0]);
    }
}
