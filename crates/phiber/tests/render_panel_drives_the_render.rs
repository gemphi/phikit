//! Properties → Render is wired to the renderer, not decoration.
//!
//! The panel's rows are RNA properties stored at `ctx.buts.<key>`. These check
//! that what the panel says is what phiray is asked for: the defaults are
//! Blender's and an edit in the panel reaches `BRenderSet`. That the bounce
//! count then reaches the `Camera` is checked in-crate, in
//! `view/draw/canvas/view3d/rendered/tests.rs`, where `BRayLook` lives.

use phiber::v2::scene::{BRenderSet, render_set};
use phiber::v2::state::{BState, ButsKey};

fn state_with(key: &'static str, value: &str) -> BState {
    let mut s = BState::new();
    s.set(ButsKey::Value(key), value);
    s
}

#[test]
fn an_untouched_panel_reads_as_blenders_defaults() {
    let read = render_set(&BState::new());
    assert_eq!(read, BRenderSet::default());
    // Spelt out, because "the default" is the thing being claimed: Cycles
    // ships 4096 render samples, 1024 in the viewport, 4 diffuse bounces and
    // denoising on.
    assert_eq!(read.samples, 4096);
    assert_eq!(read.preview_samples, 1024);
    assert_eq!(read.diffuse_bounces, 4);
    assert!(read.denoise);
}

#[test]
fn editing_a_row_changes_what_the_renderer_is_asked_for() {
    assert_eq!(render_set(&state_with("cycles.samples", "128")).samples, 128);
    assert_eq!(render_set(&state_with("cycles.preview_samples", "16")).preview_samples, 16);
    assert_eq!(render_set(&state_with("cycles.diffuse_bounces", "7")).diffuse_bounces, 7);
    assert!(!render_set(&state_with("cycles.use_denoising", "false")).denoise);
}

#[test]
fn a_value_the_panel_could_not_have_written_falls_back() {
    // The store is strings; anything that is not the property's own category
    // has to read as the default rather than as zero.
    assert_eq!(render_set(&state_with("cycles.samples", "")).samples, 4096);
    assert_eq!(render_set(&state_with("cycles.samples", "lots")).samples, 4096);
    assert!(render_set(&state_with("cycles.use_denoising", "yes")).denoise);
}
