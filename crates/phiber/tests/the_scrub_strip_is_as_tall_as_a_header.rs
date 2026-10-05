//! The timeline's frame-number strip is the theme's header height.
//!
//! It was a constant — `SCRUB_HEIGHT = 23.0`, Blender's `UI_TIME_SCRUB_MARGIN_Y`
//! at scale 1 — while the headers either side of it are the theme's 36. The
//! strip reads as a header for the window below it, so a strip that does not
//! match them is the one thing in an animation editor that lines up with
//! nothing.
//!
//! The first attempt at this changed the theme's own `scrubbing_height` and
//! claimed the fix. It did nothing: no painter here read that metric, and the
//! band was drawn from the const. So this guard is deliberately about the PATH
//! rather than the number — it checks that nothing paints the band from a
//! constant any more, which is the thing that was actually wrong.

const TIME: &str = include_str!("../src/v2/view/draw/canvas/time/set.rs");
const GRID: &str = include_str!("../src/v2/view/draw/canvas/time/grid/set.rs");
const MARKER: &str = include_str!("../src/v2/view/draw/canvas/time/marker/set.rs");
const DRIVER: &str = include_str!("../src/v2/view/draw/canvas/time/driver/set.rs");
const VIEW: &str = include_str!("../src/v2/view/draw/canvas/time/view/mod.rs");
const CANVAS: &str = include_str!("../src/v2/view/draw/canvas/time/mod.rs");

/// Lines of code, with comments and doc comments stripped.
fn code(text: &str) -> String {
    text.lines().map(|l| l.split("//").next().unwrap_or("")).collect::<Vec<_>>().join("\n")
}

#[test]
fn no_painter_draws_the_band_from_a_constant() {
    for (what, source) in
        [("time/set.rs", TIME), ("grid/set.rs", GRID), ("marker/set.rs", MARKER), ("driver/set.rs", DRIVER)]
    {
        assert!(
            !code(source).contains("SCRUB_HEIGHT"),
            "{what} still paints the scrub band from the constant — the theme cannot reach it there",
        );
    }
}

#[test]
fn the_view_carries_the_band_and_the_renderer_fills_it_from_the_theme() {
    // The one place the theme meets this window: `BTimeView::now` builds from
    // the scene and Blender's constants and has no style to read, so the
    // renderer sets the band straight after.
    assert!(code(VIEW).contains("pub scrub_height: f32"), "the view must carry the band");
    assert!(
        code(CANVAS).contains("view.scrub_height = ctx.style.dimensions.header_height"),
        "the renderer must fill it from the theme's header height",
    );
}

#[test]
fn the_constant_survives_only_as_the_default_and_blenders_own_number() {
    // Keeping it is deliberate: it is what `BTimeView::of_range` opens on, and
    // what a test means when it means Blender's number rather than a theme's.
    let view = code(VIEW);
    assert!(view.contains("scrub_height: super::SCRUB_HEIGHT"), "`of_range` keeps Blender's default");
    assert!(code(CANVAS).contains("pub const SCRUB_HEIGHT"), "and the number stays documented");
}
