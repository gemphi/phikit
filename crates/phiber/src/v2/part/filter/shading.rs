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
//! swatch) key on `scene::solid_shade::BSolidShade`. Attribute and Texture
//! draw as Material until the kernel has them, and say so.
//!
//! Not here yet, in Blender's order: the studio-light picker and
//! world-space lighting, the wireframe colour type, Backface Culling,
//! Specular Lighting, Shadow and Depth of Field — each waits on the kernel
//! drawing it, so the popover does not offer a box that does nothing.

use crate::v2::scene::solid_fx::{BSolidFx, RNA};
use crate::v2::scene::solid_shade::{BACKGROUNDS, BSolidShade, COLORS, LIGHTS, RNA as SHADE};
use crate::v2::scene::{BShading, BXray, view_shading};
use crate::v2::state::val::BValue;
use crate::v2::state::{BKey as _, BState, View3dKey};
use crate::v2::view::BView;
use crate::v2::view::prim::{BCtrl, BCtrlKind};

/// `(value, label)` of `cavity_type`'s items (`rna_space.cc`).
pub const CAVITY_TYPES: [(&str, &str); 3] = [("WORLD", "World"), ("SCREEN", "Screen"), ("BOTH", "Both")];

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
    let mut rows = vec![BView::select_row(s, "Lighting", View3dKey::Shade(SHADE[0]).path(), "light", &LIGHTS, f.light)];
    if BSolidShade::now(s).light == "MATCAP" {
        rows.push(BView::label("MatCap draws as Studio: no matcap images yet"));
    }
    rows
}

/// `VIEW3D_PT_shading_color`'s rows: Object colour (Solid) and Background
/// (Solid and Wireframe).
fn color(s: &BState, shading: BShading) -> Vec<BView> {
    let f = BSolidShade::FACTORY;
    let now = BSolidShade::now(s);
    let mut rows = Vec::new();
    if shading == BShading::Solid {
        rows.push(BView::select_row(s, "Color", View3dKey::Shade(SHADE[1]).path(), "color_type", &COLORS, f.color_type));
        if now.color_type == "SINGLE" {
            rows.push(swatch(s, "", SHADE[2], f.single_color, "Single Color"));
        }
        if matches!(now.color_type, "VERTEX" | "TEXTURE") {
            rows.push(BView::label("Attribute and Texture draw as Material for now"));
        }
    }
    rows.push(BView::select_row(s, "Background", View3dKey::Shade(SHADE[3]).path(), "background_type", &BACKGROUNDS, f.background_type));
    if now.background_type == "VIEWPORT" {
        rows.push(swatch(s, "", SHADE[4], f.background_color, "Background Color"));
    }
    rows
}

/// `VIEW3D_PT_shading_options`' rows for `shading`.
fn options(s: &BState, shading: BShading) -> Vec<BView> {
    let f = BSolidFx::FACTORY;
    let x = BXray::FACTORY;
    let mut rows = vec![
        BView::check_row(s, "Options", View3dKey::Shade(RNA[0]).path(), f.outline, "Outline"),
        BView::prop_row("", BCtrl::read(BCtrlKind::Color, View3dKey::Shade(RNA[1]).path(), BValue::Color([0.0, 0.0, 0.0, 1.0]), s).with_tooltip("Object Outline Color")),
    ];
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
    }
    rows
}

/// `VIEW3D_PT_shading_cavity`'s rows: the header's switch and type, then
/// the factors of the type in use.
fn cavity(s: &BState) -> Vec<BView> {
    let fx = BSolidFx::now(s);
    let mut rows = vec![
        BView::check_row(s, "Cavity", View3dKey::Shade(RNA[2]).path(), BSolidFx::FACTORY.cavity, "Cavity"),
        BView::select_row(s, "", View3dKey::Shade(RNA[3]).path(), "cavity_type", &CAVITY_TYPES, BSolidFx::FACTORY.cavity_type),
    ];
    if matches!(fx.cavity_type, "WORLD" | "BOTH") {
        rows.push(factor(s, "World Space", RNA[5], "Ridge", 1.0));
        rows.push(factor(s, "", RNA[4], "Valley", 1.0));
        // `VIEW3D_PT_shading_options_ssao`, as `scene.display` has it.
        let f = BSolidFx::FACTORY;
        rows.push(BView::prop_row("SSAO Settings", BCtrl::read(BCtrlKind::Int { min: 1, max: 500 }, View3dKey::Shade(RNA[8]).path(), BValue::Int(i64::from(f.ssao_samples)), s).with_label("Samples")));
        rows.push(BView::prop_row("", BCtrl::read(BCtrlKind::Number { min: 0.0, max: 100.0, step: 0.1 }, View3dKey::Shade(RNA[9]).path(), BValue::Float(f.ssao_distance), s).with_label("Distance")));
        rows.push(BView::prop_row("", BCtrl::read(BCtrlKind::Number { min: 0.0, max: 100.0, step: 0.1 }, View3dKey::Shade(RNA[10]).path(), BValue::Float(f.ssao_attenuation), s).with_label("Attenuation")));
    }
    if matches!(fx.cavity_type, "SCREEN" | "BOTH") {
        rows.push(factor(s, "Screen Space", RNA[6], "Ridge", 1.0));
        rows.push(factor(s, "", RNA[7], "Valley", 1.0));
    }
    rows
}

/// The popover's rows for the shading in use.
pub fn shading_body(s: &BState) -> Vec<BView> {
    let shading = view_shading(s);
    let mut rows = Vec::new();
    if shading == BShading::Solid {
        rows.extend(lighting(s));
    }
    if matches!(shading, BShading::Wireframe | BShading::Solid) {
        rows.extend(color(s, shading));
        rows.extend(options(s, shading));
    }
    if shading == BShading::Solid {
        rows.extend(cavity(s));
    }
    if rows.is_empty() {
        rows.push(BView::label("Lighting and Color are not in Phiber yet"));
    }
    rows
}

/// The shading popover, after the radio.
pub fn shading(s: &BState) -> BView {
    super::view3d::popover(s, "Shading", "chevron_down", shading_body(s))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::scene::shading_key;

    #[test]
    fn the_rows_follow_the_shading_in_use_as_blenders_polls_do() {
        let mut s = BState::new();
        // Solid: Lighting (1), Color (2: the type and Background), Options
        // (4, then Shadow, its intensity and five settings: 11) and Cavity —
        // its type Screen, so one pair.
        assert_eq!(shading_body(&s).len(), 1 + 2 + 11 + 2 + 2);
        s.set(View3dKey::Shade("cavity_type"), "BOTH");
        assert_eq!(shading_body(&s).len(), 1 + 2 + 11 + 2 + 7, "Both: World Space factors and SSAO settings, Screen Space factors");
        s.set(View3dKey::Shade("cavity_type"), "WORLD");
        assert_eq!(shading_body(&s).len(), 1 + 2 + 11 + 2 + 5, "World: its factors and the three SSAO settings");
        s.set(View3dKey::Shade("cavity_type"), "BOTH");
        s.set(View3dKey::Shade("color_type"), "SINGLE");
        s.set(View3dKey::Shade("background_type"), "VIEWPORT");
        assert_eq!(shading_body(&s).len(), 1 + 4 + 11 + 2 + 7, "Single and Viewport each add a swatch");
        s.set(View3dKey::Shade("light"), "MATCAP");
        assert_eq!(shading_body(&s).len(), 2 + 4 + 11 + 2 + 7, "MatCap says it draws as Studio");
        s.set(&shading_key(), "wireframe");
        assert_eq!(shading_body(&s).len(), 2 + 4, "Wireframe: Background and Options, no Lighting, Color, Shadow or Cavity");
        s.set(&shading_key(), "material");
        assert_eq!(shading_body(&s).len(), 1, "Material Preview: nothing drawable yet, said so");
    }

    #[test]
    fn the_boxes_write_what_the_canvas_reads() {
        let mut s = BState::new();
        s.set(View3dKey::Shade("show_object_outline"), crate::v2::state::val::V_FALSE);
        s.set(View3dKey::Shade("object_outline_color"), BValue::Color([1.0, 0.0, 0.0, 1.0]).encode());
        s.set(View3dKey::Shade("show_cavity"), crate::v2::state::val::V_TRUE);
        s.set(View3dKey::Shade("curvature_ridge_factor"), "2");
        let fx = BSolidFx::now(&s);
        assert!(!fx.outline && fx.cavity && fx.curvature());
        assert_eq!(fx.outline_color, [1.0, 0.0, 0.0]);
        assert_eq!(fx.curvature_ridge_factor, 2.0);
    }
}
