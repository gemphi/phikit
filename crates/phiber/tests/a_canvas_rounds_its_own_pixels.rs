//! A canvas rounds the pixels it draws, not just the box they sit in.
//!
//! GPUI's content mask is a plain rectangle, so `overflow_hidden` on a
//! rounded parent cannot round a child — which the renderer knows: every
//! canvas takes the corners it inherits and paints them itself
//! (`canvas::surface::round`). A raster is the case that rule missed. A
//! `paint_image` carries its own `Corners`, and the 3D viewport passed
//! `Corners::default()`, so the rasterised frame and the ray-traced one both
//! painted square over the corners the surface under them had rounded. That
//! is the viewport squaring off the zone it sits in.
//!
//! So the corners are computed once, as numbers, and handed to the paint.

use phiber::v2::host::BHostType;
use phiber::v2::theme::{BPaletteName, BResolvedUiStyle, BStyleRole, ui_style_defaults};

fn source(path: &str) -> String {
    let full = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/v2/view/draw").join(path);
    std::fs::read_to_string(&full).unwrap_or_else(|_| panic!("{}", full.display()))
}

#[test]
fn the_corners_are_answered_in_one_place() {
    let surface = source("canvas/surface.rs");
    assert!(
        surface.contains("pub fn corners(ctx: &BRenderCtx) -> Corners<Pixels>"),
        "one function answers what corners a canvas has"
    );
    // And the styling helper is built from it, so an element and a raster in
    // the same canvas cannot round differently.
    assert!(
        surface.contains("let c = corners(ctx);"),
        "`round` takes its radii from `corners`, not from its own copy of the plan"
    );
}

#[test]
fn no_canvas_paint_draws_a_full_frame_with_square_corners() {
    // A `paint_image` over the canvas bounds is the frame; it must carry the
    // canvas's corners. `Corners::default()` there is the bug.
    for file in ["canvas/view3d/mesh/paint.rs", "canvas/view3d/rendered/mod.rs"] {
        let text = source(file);
        for line in text.lines().filter(|l| l.contains("paint_image(bounds, bounds,")) {
            assert!(
                line.contains("corners"),
                "{file} paints the whole canvas with its own corners: {}",
                line.trim()
            );
            assert!(
                !line.contains("Corners::default()"),
                "{file} still paints the frame square: {}",
                line.trim()
            );
        }
    }
}

#[test]
fn the_image_editors_picture_is_not_rounded_with_the_canvas() {
    // The image editor's `paint_image` draws the picture at its zoom and
    // offset, which is usually smaller than the canvas and nowhere near its
    // corners. Rounding that would round the picture, not the editor — so it
    // keeps square corners on purpose, and this says so.
    let text = source("canvas/image/set.rs");
    let line = text
        .lines()
        .find(|l| l.contains("paint_image(rect, rect,"))
        .expect("the picture's paint");
    assert!(line.contains("Corners::default()"), "the picture is square: {}", line.trim());
}

#[test]
fn the_viewport_canvas_inherits_the_areas_radius() {
    // The radius a canvas rounds to is the host's, held down to what the
    // area's border leaves inside it — never larger, or the arc would cut
    // into the border it sits in.
    let style: BResolvedUiStyle = BResolvedUiStyle::from(ui_style_defaults(BPaletteName::DEFAULT));
    let area = style.radius_for(BStyleRole::Area);
    let host = style.radius_for(BStyleRole::Host);
    let region = style.radius_for(BStyleRole::Region);
    assert!(area > 0.0, "an area is rounded, so there is something to inherit");
    assert!(region.min(host) <= area, "a canvas never rounds wider than its area");
    // And the viewport is a host that fills its content, which is why its
    // canvas is the thing that reaches those corners at all.
    assert!(BHostType::View3D.fills_content(), "the viewport's canvas reaches every edge");
}
