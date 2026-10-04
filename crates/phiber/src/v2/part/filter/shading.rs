//! The viewport shading popover — `VIEW3D_PT_shading` and the panels under
//! it (`bl_ui/space_view3d.py`), the chevron after the shading radio. Its
//! rows follow the shading in use as Blender's `poll`s do: **Options**
//! (with Shadow and its settings — Blender's gear popover
//! `VIEW3D_PT_shading_options_shadow`, drawn inline — for Solid)
//! (`VIEW3D_PT_shading_options`: the object outline and its colour, X-ray
//! and its alpha — for Solid and Wireframe) and **Cavity**
//! (`VIEW3D_PT_shading_cavity`: the switch, the type, World Space and
//! Screen Space ridge / valley, and World's SSAO settings — Blender's gear
//! popover `VIEW3D_PT_shading_options_ssao`, drawn inline — Solid). The
//! rows key on what the canvas
//! reads (`scene::solid_fx::BSolidFx`, `scene::shading::BXray`), so the
//! popover, a file's settings and the kernel are one set of values.
//!
//! **Lighting** (`VIEW3D_PT_shading_lighting`, Solid: Studio / MatCap /
//! Flat — MatCap draws as Studio until matcap images are in, and says so)
//! and **Color** (`VIEW3D_PT_shading_color`, Solid: Material / Object /
//! Attribute / Single / Random / Texture with the single colour's swatch;
//! Background for Solid and Wireframe: Theme / World / Viewport with its
//! swatch) key on `scene::solid_shade::BSolidShade`.
//!
//! Not here yet, in Blender's order: the studio-light picker and
//! world-space lighting, the wireframe colour type and Depth of Field —
//! each waits on the kernel drawing it, so the popover does not offer a
//! box that does nothing.

use crate::v2::scene::solid_fx::{BSolidFx, RNA};
use crate::v2::scene::solid_shade::{BACKGROUND_LIST, BSolidShade, COLOR_LIST, LIGHT_LIST, RNA as SHADE};
use crate::v2::scene::{BShading, BXray, view_shading};
use crate::v2::state::val::BValue;
use crate::v2::state::{BKey as _, BState, View3dKey};
use crate::v2::view::prim::{BCtrl, BCtrlKind};
use crate::v2::view::{BView, BViewItem, BViewList};

/// `(value, label)` of `cavity_type`'s items (`rna_space.cc`).
pub const CAVITY_TYPE_LIST: [(&str, &str); 3] = [("WORLD", "World"), ("SCREEN", "Screen"), ("BOTH", "Both")];

/// The popover's panels (`VIEW3D_PT_shading_*`), each a list of rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BShadingPanel {
    Lighting,
    Color,
    Options,
    Cavity,
}

impl BShadingPanel {
    /// The panels the shading in use shows, in Blender's order — its
    /// `poll`s: Lighting and Cavity for Solid, Color and Options for Solid
    /// and Wireframe, none for Material Preview and Rendered.
    pub fn shown(shading: BShading) -> Vec<Self> {
        match shading {
            BShading::Solid => vec![Self::Lighting, Self::Color, Self::Options, Self::Cavity],
            BShading::Wireframe => vec![Self::Color, Self::Options],
            _ => Vec::new(),
        }
    }

    /// A factor field, `0 ..= 2.5` as Blender's are.
    fn factor(s: &BState, heading: &str, rna: &'static str, label: &str, default: f32) -> BView {
        BView::prop_row(heading, BCtrl::read(BCtrlKind::Number { min: 0.0, max: 2.5, step: 0.1 }, View3dKey::Shade(rna).path(), BValue::Float(default), s).with_label(label))
    }

    /// A colour swatch row under `heading`.
    fn swatch(s: &BState, heading: &str, rna: &'static str, default: [f32; 3], tip: &str) -> BView {
        BView::prop_row(heading, BCtrl::read(BCtrlKind::Color, View3dKey::Shade(rna).path(), BValue::Color([default[0], default[1], default[2], 1.0]), s).with_tooltip(tip))
    }

    /// `VIEW3D_PT_shading_lighting`'s rows (Solid).
    fn lighting(s: &BState) -> Vec<BView> {
        let f = BSolidShade::FACTORY;
        let mut rows = vec![BView::select_row(s, "Lighting", View3dKey::Shade(SHADE[0]).path(), "light", &LIGHT_LIST, f.light)];
        if BSolidShade::now(s).light == "MATCAP" {
            // Blender's matcap preview grid, as a list of the shipped set, and
            // its flip (`V3D_SHADING_MATCAP_FLIP_X`).
            let names: Vec<(&str, &str)> = crate::v2::scene::matcap::Matcap::name_list().map(|n| (n, n)).collect();
            rows.push(BView::select_row(s, "", View3dKey::Shade(SHADE[5]).path(), "matcap", &names, f.matcap));
            rows.push(BView::check_row(s, "", View3dKey::Shade(SHADE[6]).path(), f.matcap_flip_x, "Flip MatCap"));
        }
        rows
    }

    /// `VIEW3D_PT_shading_color`'s rows: Object colour (Solid) and
    /// Background (Solid and Wireframe).
    fn color(s: &BState, shading: BShading) -> Vec<BView> {
        let f = BSolidShade::FACTORY;
        let now = BSolidShade::now(s);
        let mut rows = Vec::new();
        if shading == BShading::Solid {
            rows.push(BView::select_row(s, "Color", View3dKey::Shade(SHADE[1]).path(), "color_type", &COLOR_LIST, f.color_type));
            if now.color_type == "SINGLE" {
                rows.push(Self::swatch(s, "", SHADE[2], f.single_color, "Single Color"));
            }
        }
        rows.push(BView::select_row(s, "Background", View3dKey::Shade(SHADE[3]).path(), "background_type", &BACKGROUND_LIST, f.background_type));
        if now.background_type == "VIEWPORT" {
            rows.push(Self::swatch(s, "", SHADE[4], f.background_color, "Background Color"));
        }
        rows
    }

    /// `VIEW3D_PT_shading_options`' rows for `shading`.
    fn options(s: &BState, shading: BShading) -> Vec<BView> {
        let f = BSolidFx::FACTORY;
        let x = BXray::FACTORY;
        let mut rows = Vec::new();
        // Solid leads with Backface Culling (`show_backface_culling`).
        if shading == BShading::Solid {
            rows.push(BView::check_row(s, "Options", View3dKey::Shade(RNA[18]).path(), f.backface_culling, "Backface Culling"));
        }
        rows.push(BView::check_row(s, if rows.is_empty() { "Options" } else { "" }, View3dKey::Shade(RNA[0]).path(), f.outline, "Outline"));
        rows.push(BView::prop_row("", BCtrl::read(BCtrlKind::Color, View3dKey::Shade(RNA[1]).path(), BValue::Color([0.0, 0.0, 0.0, 1.0]), s).with_tooltip("Object Outline Color")));
        let (on, alpha, default_on, default_alpha) = if shading == BShading::Wireframe {
            (View3dKey::XrayWireframe, View3dKey::XrayAlphaWireframe, x.wireframe, x.alpha_wireframe)
        } else {
            (View3dKey::Xray, View3dKey::XrayAlpha, x.solid, x.alpha)
        };
        rows.push(BView::check_row(s, "", on.path(), default_on, "X-Ray"));
        rows.push(BView::prop_row("", BCtrl::read(BCtrlKind::Slider { min: 0.0, max: 1.0 }, alpha.path(), BValue::Float(default_alpha), s).with_label("X-Ray")));
        if shading == BShading::Solid {
            // Shadow, its intensity, and `VIEW3D_PT_shading_options_shadow`'s
            // Direction, Offset and Focus (`scene.display`).
            rows.push(BView::check_row(s, "", View3dKey::Shade(RNA[11]).path(), f.shadow, "Shadow"));
            rows.push(BView::prop_row("", BCtrl::read(BCtrlKind::Slider { min: 0.0, max: 1.0 }, View3dKey::Shade(RNA[12]).path(), BValue::Float(f.shadow_intensity), s).with_label("Shadow")));
            for (k, (rna, axis)) in RNA[13..16].iter().zip(["X", "Y", "Z"]).enumerate() {
                let heading = if k == 0 { "Shadow Settings" } else { "" };
                rows.push(BView::prop_row(heading, BCtrl::read(BCtrlKind::Number { min: -1.0, max: 1.0, step: 0.05 }, View3dKey::Shade(rna).path(), BValue::Float(f.light_direction[k]), s).with_label(axis)));
            }
            rows.push(BView::prop_row("", BCtrl::read(BCtrlKind::Slider { min: 0.0, max: 1.0 }, View3dKey::Shade(RNA[16]).path(), BValue::Float(f.shadow_shift), s).with_label("Offset")));
            rows.push(BView::prop_row("", BCtrl::read(BCtrlKind::Slider { min: 0.0, max: 1.0 }, View3dKey::Shade(RNA[17]).path(), BValue::Float(f.shadow_focus), s).with_label("Focus")));
            // Depth of Field (`use_dof`): through the camera, with its own.
            rows.push(BView::check_row(s, "", View3dKey::Shade(RNA[20]).path(), f.dof, "Depth of Field"));
            // Specular Lighting (`show_specular_highlight`): Studio and MatCap only.
            if BSolidShade::now(s).light != "FLAT" {
                rows.push(BView::check_row(s, "", View3dKey::Shade(RNA[19]).path(), f.specular, "Specular Lighting"));
            }
        }
        rows
    }

    /// `VIEW3D_PT_shading_cavity`'s rows: the header's switch and type, then
    /// the factors of the type in use.
    fn cavity(s: &BState) -> Vec<BView> {
        let fx = BSolidFx::now(s);
        let mut rows = vec![
            BView::check_row(s, "Cavity", View3dKey::Shade(RNA[2]).path(), BSolidFx::FACTORY.cavity, "Cavity"),
            BView::select_row(s, "", View3dKey::Shade(RNA[3]).path(), "cavity_type", &CAVITY_TYPE_LIST, BSolidFx::FACTORY.cavity_type),
        ];
        if matches!(fx.cavity_type, "WORLD" | "BOTH") {
            rows.push(Self::factor(s, "World Space", RNA[5], "Ridge", 1.0));
            rows.push(Self::factor(s, "", RNA[4], "Valley", 1.0));
            // `VIEW3D_PT_shading_options_ssao`, as `scene.display` has it.
            let f = BSolidFx::FACTORY;
            rows.push(BView::prop_row("SSAO Settings", BCtrl::read(BCtrlKind::Int { min: 1, max: 500 }, View3dKey::Shade(RNA[8]).path(), BValue::Int(i64::from(f.ssao_samples)), s).with_label("Samples")));
            rows.push(BView::prop_row("", BCtrl::read(BCtrlKind::Number { min: 0.0, max: 100.0, step: 0.1 }, View3dKey::Shade(RNA[9]).path(), BValue::Float(f.ssao_distance), s).with_label("Distance")));
            rows.push(BView::prop_row("", BCtrl::read(BCtrlKind::Number { min: 0.0, max: 100.0, step: 0.1 }, View3dKey::Shade(RNA[10]).path(), BValue::Float(f.ssao_attenuation), s).with_label("Attenuation")));
        }
        if matches!(fx.cavity_type, "SCREEN" | "BOTH") {
            rows.push(Self::factor(s, "Screen Space", RNA[6], "Ridge", 1.0));
            rows.push(Self::factor(s, "", RNA[7], "Valley", 1.0));
        }
        rows
    }

}

impl BViewList for BShadingPanel {
    fn items(&self, s: &BState) -> Vec<BView> {
        let shading = view_shading(s);
        match self {
            Self::Lighting => Self::lighting(s),
            Self::Color => Self::color(s, shading),
            Self::Options => Self::options(s, shading),
            Self::Cavity => Self::cavity(s),
        }
    }
}

/// The Shading popover after the radio (`VIEW3D_PT_shading`): the panels
/// the shading in use shows, or a line saying nothing is drawable yet.
#[derive(Clone, Copy, Debug, Default)]
pub struct BShadingPopover;

impl BViewList for BShadingPopover {
    fn items(&self, s: &BState) -> Vec<BView> {
        let rows: Vec<BView> = BShadingPanel::shown(view_shading(s)).iter().flat_map(|panel| panel.items(s)).collect();
        if rows.is_empty() {
            return vec![BView::label("Lighting and Color are not in Phiber yet")];
        }
        rows
    }
}

impl BViewItem for BShadingPopover {
    fn build(&self, s: &BState) -> BView {
        super::view3d::popover(s, "Shading", "chevron_down", self.items(s))
    }
}

#[cfg(test)]
#[path = "shading_tests.rs"]
mod tests;
