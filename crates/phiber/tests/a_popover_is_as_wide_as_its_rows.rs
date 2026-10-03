//! A popover grows to its content; it does not clip it.
//!
//! Both columns of a split row were `fills()`, which divides the row in half.
//! Blender's `use_property_split` does not: the label takes 40% and the value
//! 60% (`UI_ITEM_PROP_SEP_DIVIDE`). Half a row spent 110px on the word "Sync"
//! and left the value column too narrow for what stood opposite it, so the
//! Playback popover clipped "Limit to Frame Range" to "Limit to Frame R", and
//! five rows beside it: "Follow Current F", "Properties and S", "Movie Clip
//! Edito". A heading is one or two words; what faces it is a tick and a
//! sentence.
//!
//! Measured against each row's own text, not against a number: a width cannot
//! be asserted by arithmetic without re-implementing the shaper, but a control
//! that fits its column is one whose column is at least as wide as the control
//! inside it, and a label that is cut is one whose text ran past the panel.

use gpui::{TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::host::BHostType;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{BKey as _, BState, CtxKey, MenuKey, val::V_TRUE};

/// Open the Timeline's Playback popover — the longest set of split rows in
/// the app, and the one in the report.
macro_rules! playback_open {
    ($cx:ident) => {{
        let (app, cx) = $cx.add_window_view(|_, _| BPhiberApp::new());
        cx.simulate_resize(size(px(1600.0), px(900.0)));
        app.update(cx, |app, cx| {
            app.state = BState::new();
            app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
            app.state.set(&MenuKey::Panel(BHostType::Time.slug(), "Playback").path(), V_TRUE);
            cx.notify();
        });
        cx.run_until_parked();
        (app, cx)
    }};
}

#[gpui::test]
async fn the_panel_grows_to_its_widest_row(cx: &mut TestAppContext) {
    let (_app, cx) = playback_open!(cx);
    let bounds = cx.all_debug_bounds();
    let panel = bounds.iter().find(|(s, _)| s == "popover-panel").map(|(_, b)| *b).expect("the Playback popover is open");
    let style = phiber::v2::theme::resolve_ui_style(
        phiber::v2::theme::ui_style_defaults_in(
            phiber::v2::theme::BPaletteName::DEFAULT,
            phiber::v2::theme::BThemeMode::DEFAULT,
        ),
        &BState::new(),
    );
    let floor = style.dimensions.panel_width;
    let width = f32::from(panel.size.width);
    assert!(
        width > floor,
        "the panel takes what its rows need ({width}), not the minimum it may be ({floor})"
    );

    // And the widest row's value column really can hold its label. Measured
    // against a conservative lower bound for the text — no proportional UI
    // face averages under 0.45 em per character — plus the tick beside it,
    // because the shaper's own extents are not in the painted scene.
    let longest = "Limit to Frame Range";
    let needed = longest.len() as f32 * 0.45 * style.typography.body + style.dimensions.icon_size_sm;
    // Only this popover's own rows: `section-*` is every split row in the
    // window, and a popover floats over the rest of them.
    let inside = |b: &gpui::Bounds<gpui::Pixels>| {
        f32::from(b.origin.x) >= f32::from(panel.origin.x) - 0.5
            && f32::from(b.origin.x) + f32::from(b.size.width) <= f32::from(panel.origin.x + panel.size.width) + 0.5
            && f32::from(b.origin.y) >= f32::from(panel.origin.y) - 0.5
            && f32::from(b.origin.y) + f32::from(b.size.height) <= f32::from(panel.origin.y + panel.size.height) + 0.5
    };
    let columns: Vec<(&str, f32)> = bounds
        .iter()
        .filter(|(s, b)| s.ends_with("-value") && s.starts_with("section-") && inside(b))
        .map(|(s, b)| (s.as_str(), f32::from(b.size.width)))
        .collect();
    // Sync, Audio, Playback, Loop, Play In, Show, and the unheaded one.
    assert_eq!(columns.len(), 7, "the Playback popover's heading rows: {columns:?}");
    let narrow: Vec<&(&str, f32)> = columns.iter().filter(|(_, w)| *w < needed).collect();
    assert!(
        narrow.is_empty(),
        "a value column narrower than {needed:.0}, which is what \"{longest}\" needs: {narrow:?}"
    );
}

#[gpui::test]
async fn a_panel_in_a_region_still_gives_its_width_up(cx: &mut TestAppContext) {
    // The other half of the rule: Properties has a width the user dragged it
    // to, and a long property name must not push it wider. Its split columns
    // keep `min_width(Zero)` and shrink, which is why the rule is scoped
    // rather than removed.
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1600.0), px(900.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
        cx.notify();
    });
    cx.run_until_parked();
    let bounds = cx.all_debug_bounds();
    let pane = bounds.iter().find(|(s, _)| s == "pane-Right-Buts").map(|(_, b)| *b).expect("Properties is on screen");
    let rows: Vec<(&str, f32, f32)> = bounds
        .iter()
        .filter(|(s, _)| s.starts_with("section-") && !s.ends_with("-label") && !s.ends_with("-value"))
        .map(|(s, b)| (s.as_str(), f32::from(b.origin.x), f32::from(b.size.width)))
        .collect();
    assert!(!rows.is_empty(), "Properties draws split rows");
    let right = f32::from(pane.origin.x + pane.size.width);
    for (sel, x, w) in &rows {
        assert!(x + w <= right + 0.5, "{sel} runs past the pane it is in ({:.0} > {right:.0})", x + w);
    }
}

/// And the rule that makes it so, named where it is decided.
#[test]
fn the_shrink_rule_is_scoped_to_where_a_width_exists() {
    let split = include_str!("../src/v2/view/draw/split.rs");
    assert!(split.contains("const VALUE_SHARE: f32 = 0.6;"), "Blender's split, named");
    assert!(split.contains("BLen::Share(if value { VALUE_SHARE } else { 1.0 - VALUE_SHARE })"), "and applied to both columns");
    assert!(!split.contains("BFlex::row().fills().min_width(BLen::Zero)"), "the equal halves are gone");
}

/// Every checkbox in the popover is one width — its column's — rather than
/// a pill cut to its own label, so the column reads as a column. The row
/// inside the control filled its slot; the wrapper around it hugged, and
/// "Sync" sat in a pill a third the width of "Limit to Frame Range".
#[gpui::test]
async fn the_checkboxes_in_a_popover_are_one_width(cx: &mut TestAppContext) {
    let (_app, cx) = playback_open!(cx);
    let bounds = cx.all_debug_bounds();
    let panel = bounds.iter().find(|(s, _)| s == "popover-panel").map(|(_, b)| *b).expect("the Playback popover is open");
    let inside = |b: &gpui::Bounds<gpui::Pixels>| {
        f32::from(b.origin.x) >= f32::from(panel.origin.x) - 0.5
            && f32::from(b.origin.y) >= f32::from(panel.origin.y) - 0.5
            && f32::from(b.origin.y) + f32::from(b.size.height) <= f32::from(panel.origin.y + panel.size.height) + 0.5
    };
    let widths: Vec<(&str, f32)> = bounds
        .iter()
        // A checkbox is a labelled button whose action sets its key to the
        // other bool (`BCtrl::flip`): `btn-label-set-ctx.…-true|false-text`.
        .filter(|(s, b)| s.starts_with("btn-label-set-") && (s.ends_with("-true-text") || s.ends_with("-false-text")) && !s.contains("ctx.header.") && inside(b))
        .map(|(s, b)| (s.as_str(), (f32::from(b.size.width) * 2.0).round() / 2.0))
        .collect();
    assert!(widths.len() >= 5, "the popover's checkboxes: {widths:?}");
    let first = widths[0].1;
    let odd: Vec<&(&str, f32)> = widths.iter().filter(|(_, w)| (*w - first).abs() > 0.5).collect();
    assert!(odd.is_empty(), "every checkbox as wide as the first ({first}): {odd:?}");
}
