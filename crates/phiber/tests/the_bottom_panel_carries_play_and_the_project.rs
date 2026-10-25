//! End to end: the Layout's bottom panel opens with Reality Composer Pro's
//! editor tabs over its editor; switched to Play it shows the picker — the
//! match, the sets, the chosen set's cards — and a card picked is the
//! match changed; switched to the Project Browser it shows its navigator,
//! its toolbar and its inspector.

use gpui::{Modifiers, TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::host::BHostType;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{CtxKey, PlayKey};

fn selectors(cx: &mut gpui::VisualTestContext) -> Vec<String> {
    cx.all_debug_bounds().into_iter().map(|(selector, _)| selector).collect()
}

fn show(app: &gpui::Entity<BPhiberApp>, cx: &mut gpui::VisualTestContext, host: BHostType) {
    let space = BSpaceType::Layout;
    let area = space.spec().areas.iter().find(|a| a.default_host == BHostType::Time).copied().expect("the Timeline area");
    assert!(area.hosts.contains(&host), "{host:?} is offered beside the Timeline");
    app.update(cx, |app, cx| {
        app.state.remove(phiber::v2::state::ModalKey::Open);
        app.state.set(CtxKey::Space, space.slug());
        app.state.set(&phiber::v2::area::area_key(space, &area), host.slug());
        cx.notify();
    });
    cx.run_until_parked();
}

#[gpui::test]
async fn play_is_a_tab_of_the_bottom_panel_and_a_card_picks_the_match(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(900.0)));
    show(&app, cx, BHostType::Time);
    let tabs = cx.debug_bounds("region-Tabs-Time").expect("the bottom panel's editor tabs");
    let header = cx.debug_bounds("region-Header-Time").expect("the Timeline's header");
    assert!(tabs.origin.y < header.origin.y, "the tabs sit over the editor's header");

    show(&app, cx, BHostType::Play);
    assert!(cx.debug_bounds("region-Tabs-Play").is_some(), "the tabs stay over Play");
    let names = selectors(cx);
    for sport in ["Basketball", "Netball", "Volleyball", "Beach volleyball", "Soccer"] {
        assert!(names.iter().any(|n| n == &format!("thumb-{sport}")), "a card for {sport}: {names:?}");
    }
    let soccer = cx.debug_bounds("thumb-Soccer").expect("the Soccer card");
    cx.simulate_click(soccer.center(), Modifiers::none());
    cx.run_until_parked();
    let picked = app.read_with(cx, |a, _| a.state.get(PlayKey::Choice).map(phiplay::pick::decode)).expect("a match picked");
    assert_eq!(picked.sport, phiplay::sport::BSport::Soccer);

    // Another set: the home side's sixteen teams.
    app.update(cx, |app, cx| {
        app.state.set(PlayKey::Row, "home");
        cx.notify();
    });
    cx.run_until_parked();
    let names = selectors(cx);
    assert_eq!(names.iter().filter(|n| n.starts_with("thumb-")).count(), 16, "sixteen teams");
}

#[gpui::test]
async fn the_project_browser_is_a_tab_with_its_three_columns(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(900.0)));
    show(&app, cx, BHostType::Project);
    for region in ["region-Tabs-Project", "region-Header-Project", "region-ToolHeader-Project", "region-Tools-Project", "region-Center-Project", "region-UI-Project"] {
        assert!(cx.debug_bounds(region).is_some(), "{region}: {:?}", selectors(cx));
    }
    let nav = cx.debug_bounds("region-Tools-Project").unwrap();
    let cards = cx.debug_bounds("region-Center-Project").unwrap();
    let info = cx.debug_bounds("region-UI-Project").unwrap();
    assert!(nav.origin.x < cards.origin.x && cards.origin.x < info.origin.x, "navigator, cards, inspector, left to right: {nav:?} {cards:?} {info:?}");
}
