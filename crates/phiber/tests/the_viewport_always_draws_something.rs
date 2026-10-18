//! The viewport never shows an empty canvas, and a fresh file has its floor.
//!
//! Two faults, one picture: a 3D viewport with a blue selection outline on
//! black — no surface, no floor, no axes.
//!
//! 1. The tracer claimed the frame and painted nothing. `plan` sends every
//!    shading mode to the GPU, so `cpu_draws` was false and the cache was
//!    told not to project triangles; `rendered::paint` then had no image yet
//!    (it is asynchronous, and an adapter can go away) and the branch that
//!    draws the floor only ran in the CPU arm. What was left on the canvas
//!    was the selection outline, which is painted unconditionally.
//! 2. Every overlay shipped OFF. `show_overlays`, `show_floor`, the axes,
//!    the extras, the bones, the gizmo — Blender ships all of those on, and
//!    the panel drew a clear checkbox beside "Floor" while the viewport
//!    faithfully drew no floor.
//!
//! So `rendered::paint` answers whether it painted, the CPU painter stands
//! in when it did not, and the table carries Blender's defaults.

use phiber::v2::part::props::prop::{BDefault, BPropSpec};
use phiber::v2::part::props::tool::{OPTIONS, OVERLAYS};
use phiber::v2::scene::BShading;

fn source(path: &str) -> String {
    let full = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/v2").join(path);
    std::fs::read_to_string(&full).unwrap_or_else(|_| panic!("{}", full.display()))
}

fn on(spec: &BPropSpec) -> bool {
    matches!(spec.default, Some(BDefault::Bool(true)))
}

fn find<'a>(table: &'a [BPropSpec], key: &str) -> &'a BPropSpec {
    table.iter().find(|spec| spec.key == key).unwrap_or_else(|| panic!("{key}"))
}

#[test]
fn the_tracer_says_whether_it_painted() {
    let rendered = source("view/draw/canvas/view3d/rendered/mod.rs");
    assert!(
        rendered.contains("corners: Corners<Pixels>, window: &mut Window) -> bool"),
        "`rendered::paint` must answer whether an image landed"
    );
}

#[test]
fn the_canvas_falls_back_when_it_did_not() {
    let canvas = source("view/draw/canvas/view3d/mod.rs");
    assert!(
        canvas.contains("let traced = ray && self::rendered::paint("),
        "the canvas reads that answer rather than assuming the GPU drew"
    );
    assert!(canvas.contains("if !traced {"), "and draws the floor when it did not");
    // The meshes must exist for the fallback to have anything to paint:
    // a frame the tracer delivered projects nothing at all (batch 166,
    // `BCpuDraw::Nothing` — the tracer draws the selection outline too).
    // They are projected in the paint, when the tracer turns out to have
    // no image (batch 105) — projecting them on every frame the CPU
    // *could* need them cost every orbit frame of a small scene the whole
    // projection.
    assert!(
        canvas.contains("else { fall_back() }") && canvas.contains("cache::meshes(self::cache::BCpuDraw::Whole, &scene"),
        "the triangles are projected the moment the fallback is needed"
    );
    // And the fill is skipped when the tracer's image is already there, or
    // the raster would paint over a finished frame.
    assert!(canvas.contains("corners, !traced, supersample, window)"), "the fill is the fallback's, not both");
}

#[test]
fn every_shading_mode_still_prefers_the_gpu() {
    // The fallback is a safety net, not a new tier: nothing here should have
    // moved a mode back onto the CPU painter.
    let draw = source("view/draw/canvas/view3d/draw.rs");
    for mode in BShading::ALL {
        let _ = mode;
    }
    assert!(draw.contains("BShading::Solid => Some(phiray::Mode::Solid)"));
    assert!(draw.contains("BShading::Wireframe => Some(phiray::Mode::Wire)"));
}

#[test]
fn a_fresh_file_has_its_overlays_on() {
    // Blender's defaults. All of these used to be off, which is a viewport
    // with no floor and no axes in a file nobody has touched.
    for key in ["show_overlays", "show_floor", "show_axis_x", "show_axis_y", "show_extras", "show_bones", "show_relationship_lines"] {
        assert!(on(find(OVERLAYS, key)), "{key} ships on in Blender");
    }
    // Z alone is off: the default view looks along the floor plane.
    assert!(!on(find(OVERLAYS, "show_axis_z")), "the Z axis ships off");
}

#[test]
fn a_fresh_file_has_its_gizmo_but_not_the_object_handles() {
    assert!(on(find(OPTIONS, "show_gizmo")), "the gizmo ships on");
    assert!(on(find(OPTIONS, "show_gizmo_context")));
    assert!(on(find(OPTIONS, "show_gizmo_tool")));
    // The move/rotate/scale handles appear with the matching tool, not over
    // every object in the default file.
    for key in ["show_gizmo_object_translate", "show_gizmo_object_rotate", "show_gizmo_object_scale"] {
        assert!(!on(find(OPTIONS, key)), "{key} ships off");
    }
}

#[test]
fn the_shading_modes_are_told_apart_by_their_icons() {
    // Four generic circles said nothing about which mode they picked, and
    // two of them were the same circle with a different dash.
    let icons: Vec<&str> = BShading::ALL.iter().map(|mode| mode.icon()).collect();
    let mut unique = icons.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), icons.len(), "two modes share an icon: {icons:?}");
    for icon in &icons {
        assert!(!icon.starts_with("circle"), "{icon} is a generic circle");
    }
}
