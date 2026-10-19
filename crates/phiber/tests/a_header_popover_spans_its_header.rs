//! A header popover opens back across the header it came from.
//!
//! The Filter icon sits at the right end of its header, so a leading-edge
//! drop-down (`BPopoverSide::Below`, which is what the Outliner's used) put
//! the panel off the edge of the host; the fit rule then slid it back to a
//! position that lined up with nothing on either side. A header popover is
//! `BPopoverSide::BelowEnd` instead: trailing edges flush, and one side zone
//! wide (`draw::popover::BPopoverWidth`, which cites Blender's own
//! `UI_POPOVER_WIDTH_UNITS`..14-unit band). In a side zone those two facts
//! together put its leading edge on the header's first icon — the switcher —
//! so the panel reads as the header's own drawer rather than a floating box.

use gpui::{TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{BKey as _, BState, CtxKey, OutlinerKey, val::V_TRUE};

#[gpui::test]
async fn the_outliner_filter_popover_spans_its_side_panel_header(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1600.0), px(900.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
        app.state.set(&OutlinerKey::FilterOpen.path(), V_TRUE);
        cx.notify();
    });
    cx.run_until_parked();

    let painted = cx.all_debug_bounds();
    let find = |name: &str| {
        painted
            .iter()
            .find(|(sel, _)| sel == name)
            .map(|(_, b)| *b)
            .unwrap_or_else(|| panic!("{name} is not painted"))
    };
    let panel = find("popover-panel");
    let lead = find("pane-Left-Outliner");
    let trail = find("pane-Right-Outliner");

    let style = phiber::v2::theme::resolve_ui_style(
        phiber::v2::theme::ui_style_defaults_in(
            phiber::v2::theme::BPaletteName::DEFAULT,
            phiber::v2::theme::BThemeMode::DEFAULT,
        ),
        &BState::new(),
    );
    let width = f32::from(panel.size.width);
    let least = style.dimensions.side_zone_default_width;
    assert!(
        width >= least - 0.5,
        "a header popover is at least one side zone wide: {width} of {least}"
    );

    let (panel_left, panel_right) = (f32::from(panel.origin.x), f32::from(panel.origin.x + panel.size.width));
    let first_icon = f32::from(lead.origin.x);
    let header_end = f32::from(trail.origin.x + trail.size.width);
    assert!(
        panel_left <= first_icon + 0.5,
        "its leading edge reaches the header's first icon: {panel_left} against {first_icon}"
    );
    assert!(
        panel_right <= header_end + 0.5,
        "and its trailing edge stays inside the host: {panel_right} past {header_end}"
    );
}
