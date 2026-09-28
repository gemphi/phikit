//! Blender 5.2's Dope Sheet (`bl_ui/space_dopesheet.py`): the footer region is
//! `DOPESHEET_HT_playback_controls` — the same `playback_controls` the
//! Timeline draws in its header — and the sidebar's one tab is *Action*
//! (`DOPESHEET_PT_action`, `DOPESHEET_PT_action_slot`). A footer is a strip
//! like the header: the same height, the same surface, its controls on the
//! same centre line (`ED_area_footersize` is `ED_area_headersize`).

use gpui::{TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::host::BHostType;
use phiber::v2::part::sidebar::set::tabs_for;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{CtxKey, ModalKey};
use phiber::v2::theme::METRICS;

fn mid(b: gpui::Bounds<gpui::Pixels>) -> f32 {
    f32::from(b.origin.y) + f32::from(b.size.height) / 2.0
}

#[gpui::test]
async fn the_animation_workspaces_dope_sheet_carries_the_transport_in_its_footer(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1600.0), px(900.0)));
    app.update(cx, |app, cx| {
        app.state.remove(ModalKey::Open);
        app.state.set(CtxKey::Space, BSpaceType::Animation.slug());
        cx.notify();
    });
    cx.run_until_parked();

    let header = cx.debug_bounds("region-Header-Action").expect("the Dope Sheet header");
    let footer = cx.debug_bounds("region-Footer-Action").expect("the Dope Sheet footer");
    assert_eq!(footer.size.height, header.size.height, "a footer is a strip like the header");
    assert_eq!(f32::from(footer.size.height), METRICS.h_header);
    assert!(footer.origin.y > header.origin.y, "the footer is under the window");

    // `playback_controls`: the transport in the middle, the frame fields at
    // the right, every control on the strip's line.
    let play = cx.debug_bounds("btn-icon-set-ctx.playing-true-play").expect("play in the footer");
    let reverse = cx.debug_bounds("btn-icon-set-ctx.playing-reverse-play_back").expect("play reverse in the footer");
    let frame = cx.debug_bounds("btn-icon-set-ctx.edit-ctx.frame-number").expect("the current-frame field");
    let line = f32::from(footer.origin.y) + METRICS.menu_top_gap + (METRICS.h_header - METRICS.menu_top_gap) / 2.0;
    for (name, b) in [("play", play), ("reverse", reverse), ("frame", frame)] {
        assert!(f32::from(b.origin.y) >= f32::from(footer.origin.y) && f32::from(b.origin.y + b.size.height) <= f32::from(footer.origin.y + footer.size.height), "{name} sits in the footer");
        assert!((mid(b) - line).abs() < 0.51, "{name} sits on the footer's line: {} vs {line}", mid(b));
    }
    assert!(reverse.origin.x < play.origin.x && play.origin.x < frame.origin.x, "reverse, play, then the frame fields");

    // The sidebar's tab is Action.
    let tabs: Vec<&str> = tabs_for(BHostType::Action).iter().map(|tab| tab.label).collect();
    assert_eq!(tabs, ["Action"]);
}
