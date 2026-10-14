//! Every icon-only button in a strip is a square tile — the transport's
//! record, jump and play buttons, a header's filter and overlay toggles —
//! the same size as each other and painted with the control shade at rest
//! (`view/draw/button.rs`). Half-weight on a header the tile vanished, so a
//! transport read as loose glyphs beside the one button that happened to be
//! lit, and nothing said which marks were buttons.

use gpui::{TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::state::{CtxKey, ModalKey};
use phiber::v2::theme::{BControlState, BPaletteName, BResolvedUiStyle, ui_style_defaults};

#[gpui::test]
async fn the_transport_buttons_are_equal_squares(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1600.0), px(900.0)));
    app.update(cx, |app, cx| {
        app.state.remove(ModalKey::Open);
        app.state.set(CtxKey::Space, "layout");
        cx.notify();
    });
    cx.run_until_parked();
    let found: Vec<_> = ["btn-icon-set-ctx.frame_step-start-skip_start", "btn-icon-set-ctx.playing-true-play", "btn-icon-toggle-ctx.auto_key-record"]
        .into_iter()
        .filter_map(|id| cx.debug_bounds(id).map(|b| (id, b)))
        .collect();
    assert!(found.len() >= 2, "the transport's buttons are painted with their ids: {found:?}");
    let side = f32::from(found[0].1.size.width);
    for (id, b) in &found {
        assert!((f32::from(b.size.width) - f32::from(b.size.height)).abs() < 0.51, "{id} is a square: {b:?}");
        assert!((f32::from(b.size.width) - side).abs() < 0.51, "{id} is the same square as the others: {b:?}");
    }
}

#[test]
fn a_tile_at_rest_is_the_whole_control_shade_not_a_wash() {
    // What the tile paints: the floating rest fill is `bg_control` whole; the
    // docked one is the same shade at half weight, which a header swallows.
    let style = BResolvedUiStyle::from(ui_style_defaults(BPaletteName::DEFAULT));
    let tile = style.control_fill(BControlState::Rest, true).unwrap();
    let wash = style.control_fill(BControlState::Rest, false).unwrap();
    assert_eq!((tile.0, tile.1, tile.2), (wash.0, wash.1, wash.2), "one shade");
    assert!(tile.3 > wash.3 * 1.5, "at its whole weight, not the wash: {} vs {}", tile.3, wash.3);
    // And the button draws a strip icon with that fill (`button.rs`).
    let button = include_str!("../src/v2/view/draw/button.rs");
    assert!(button.contains("let tile = label.is_none() && chrome::in_strip(ctx);"));
    assert!(button.contains("BRenderCtx { floating: true, ..ctx.clone() }"));
}
