//! The Shading popover's rows follow the shading in use as Blender's
//! `poll`s do, and its boxes write what the canvas reads.

use super::*;
use crate::v2::scene::shading_key;

#[test]
fn the_rows_follow_the_shading_in_use_as_blenders_polls_do() {
    let mut s = BState::new();
    // Solid: Lighting (1), Color (2: the type and Background), Options
    // (Backface Culling, Outline and its colour, X-Ray and its alpha,
    // Shadow, its intensity and five settings, Specular Lighting: 13)
    // and Cavity — its type Screen, so one pair.
    assert_eq!(BShadingPopover.items(&s).len(), 1 + 2 + 13 + 2 + 2);
    s.set(View3dKey::Shade("cavity_type"), "BOTH");
    assert_eq!(BShadingPopover.items(&s).len(), 1 + 2 + 13 + 2 + 7, "Both: World Space factors and SSAO settings, Screen Space factors");
    s.set(View3dKey::Shade("cavity_type"), "WORLD");
    assert_eq!(BShadingPopover.items(&s).len(), 1 + 2 + 13 + 2 + 5, "World: its factors and the three SSAO settings");
    s.set(View3dKey::Shade("cavity_type"), "BOTH");
    s.set(View3dKey::Shade("color_type"), "SINGLE");
    s.set(View3dKey::Shade("background_type"), "VIEWPORT");
    assert_eq!(BShadingPopover.items(&s).len(), 1 + 4 + 13 + 2 + 7, "Single and Viewport each add a swatch");
    s.set(View3dKey::Shade("light"), "MATCAP");
    assert_eq!(BShadingPopover.items(&s).len(), 3 + 4 + 13 + 2 + 7, "MatCap: the matcap and its flip");
    s.set(View3dKey::Shade("light"), "FLAT");
    assert_eq!(BShadingPopover.items(&s).len(), 1 + 4 + 12 + 2 + 7, "Flat: no Specular Lighting row");
    s.set(View3dKey::Shade("light"), "STUDIO");
    s.set(&shading_key(), "wireframe");
    assert_eq!(BShadingPopover.items(&s).len(), 2 + 4, "Wireframe: Background and Options, no Lighting, Color, Shadow or Cavity");
    s.set(&shading_key(), "material");
    assert_eq!(BShadingPopover.items(&s).len(), 1, "Material Preview: nothing drawable yet, said so");
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
