//! Runtime regressions for the View3D fill chain and Properties accordion.

use gpui::{Modifiers, TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::host::{BHostType, host_slug};
use phiber::v2::part::BPartType;
use phiber::v2::part::panel::BPanelList;
use phiber::v2::space::{BSpaceItem, BSpaceType};
use phiber::v2::state::{BState, V_FALSE, action::apply_action, key::K_SPACE};
use phiber::v2::view::{BAction, BView, BViewItem, BViewList};

fn select_view3d(state: &mut BState, space: BSpaceType) {
    state.set(&K_SPACE, space.slug());
    // The welcome window is seeded open on a fresh state; dismiss it so the
    // workspace under it is what these assertions see.
    state.set(&phiber::v2::state::key::K_MODAL, "");
    // Area slots are keyed per space and per slot, so the editor chosen for
    // Layout's centre area cannot leak into Shading's (see `v2::area::current`).
    for area in space
        .spec()
        .areas
        .iter()
        .filter(|a| a.hosts.contains(&BHostType::View3D))
    {
        state.set(
            &phiber::v2::area::area_key(space, area),
            host_slug(BHostType::View3D),
        );
    }
}

fn has_view3d_custom_canvas(view: &BView) -> bool {
    let BView::Space {
        children: zones, ..
    } = view
    else {
        return false;
    };
    zones.iter().any(|zone| {
        let BView::Zone {
            children: areas, ..
        } = zone
        else {
            return false;
        };
        areas.iter().any(|area| matches_view3d_area(area))
    })
}

fn matches_view3d_area(view: &BView) -> bool {
    let BView::Area { child, .. } = view else {
        return false;
    };
    let BView::Host {
        kind: BHostType::View3D,
        children: regions,
    } = child.as_ref()
    else {
        return false;
    };
    regions.iter().any(|region| {
        matches!(region,
            BView::Region { children: panes, .. } if panes.iter().any(|pane| matches!(pane,
                BView::Pane { children: parts, .. } if parts.iter().any(|part| matches!(part,
                    BView::Part { kind: BPartType::Custom, body }
                        if matches!(body.as_ref(), BView::Canvas { host: BHostType::View3D })
                ))
            ))
        )
    })
}
fn panel_children(state: &BState) -> usize {
    let items = BPanelList::new(BHostType::Buts).items(state);
    let BView::Panel { children, .. } = &items[0] else {
        panic!("first section must be a card")
    };
    children.len()
}

fn panel_header_action(state: &BState) -> BAction {
    let items = BPanelList::new(BHostType::Buts).items(state);
    let BView::Panel { children, .. } = &items[0] else {
        panic!("first section must be a card")
    };
    let BView::AccordionHeader { action, .. } = &children[0] else {
        panic!("first card child must be its focused accordion header")
    };
    action.clone()
}

#[test]
fn every_space_that_declares_view3d_builds_the_complete_custom_canvas_chain() {
    for &space in BSpaceType::ALL {
        let declares_view3d = space
            .spec()
            .areas
            .iter()
            .any(|area| area.hosts.contains(&BHostType::View3D));
        if !declares_view3d {
            continue;
        }
        let mut state = BState::new();
        select_view3d(&mut state, space);
        let tree = BSpaceItem::new(space).build(&state);
        assert!(
            has_view3d_custom_canvas(&tree),
            "{} lost its View3D Custom/Canvas chain",
            space.slug()
        );
    }
}

#[test]
fn rendered_panel_set_action_changes_rebuilt_child_count_both_directions() {
    let mut state = BState::new();
    let initial = panel_children(&state);
    assert!(initial > 1, "expanded section must include property rows");

    let title_action = panel_header_action(&state);
    let BAction::Set(path, value) = &title_action else {
        panic!("accordion must use an explicit Set")
    };
    assert_eq!(path.encode(), "ctx.buts.active_tool.active_tool.expanded");
    assert_eq!(value, V_FALSE);
    apply_action(&mut state, &title_action);
    assert_eq!(
        panel_children(&state),
        1,
        "collapse must remove all property rows"
    );

    let expand_action = panel_header_action(&state);
    apply_action(&mut state, &expand_action);
    assert_eq!(
        panel_children(&state),
        initial,
        "expansion must restore every property row"
    );
}
#[gpui::test]
async fn every_declared_or_selected_view3d_canvas_gets_visible_runtime_bounds(
    cx: &mut TestAppContext,
) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1024.0), px(768.0)));

    for &space in BSpaceType::ALL {
        if !space
            .spec()
            .areas
            .iter()
            .any(|area| area.hosts.contains(&BHostType::View3D))
        {
            continue;
        }
        app.update(cx, |app, cx| {
            select_view3d(&mut app.state, space);
            cx.notify();
        });
        cx.run_until_parked();
        let bounds = cx
            .debug_bounds("view3d-canvas")
            .unwrap_or_else(|| panic!("{} did not prepaint View3D", space.slug()));
        println!(
            "{} View3D bounds: {:.1} x {:.1}",
            space.slug(),
            f32::from(bounds.size.width),
            f32::from(bounds.size.height)
        );
        assert!(
            f32::from(bounds.size.width) > 1.0,
            "{} View3D width was zero",
            space.slug()
        );
        assert!(
            f32::from(bounds.size.height) > 1.0,
            "{} View3D height was zero",
            space.slug()
        );
    }
}

#[gpui::test]
async fn rendered_full_width_heading_toggles_in_every_buts_space(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1024.0), px(768.0)));

    for &space in BSpaceType::ALL {
        let area = space
            .spec()
            .areas
            .iter()
            .find(|area| area.default_host == BHostType::Buts)
            .or_else(|| {
                space
                    .spec()
                    .areas
                    .iter()
                    .find(|area| area.hosts.contains(&BHostType::Buts))
            });
        let Some(area) = area else { continue };
        app.update(cx, |app, cx| {
            app.state = BState::new();
            app.state.set(&K_SPACE, space.slug());
            app.state
                .set(&phiber::v2::area::area_key(space, area), "buts");
            cx.notify();
        });
        cx.run_until_parked();

        let title_action = app.read_with(cx, |app, _| panel_header_action(&app.state));
        let BAction::Set(path, _) = &title_action else {
            panic!("accordion heading action")
        };
        let header_id: &'static str = Box::leak(
            format!("accordion-header-{}", path.encode().replace('.', "-")).into_boxed_str(),
        );
        let header_bounds = cx
            .debug_bounds(header_id)
            .unwrap_or_else(|| panic!("{} accordion heading was not painted", space.slug()));
        println!("{} heading bounds: {:?}", space.slug(), header_bounds);
        let far_right = gpui::point(
            header_bounds.origin.x + header_bounds.size.width - px(2.0),
            header_bounds.center().y,
        );
        cx.simulate_click(far_right, Modifiers::none());
        cx.run_until_parked();
        let n_after = app.read_with(cx, |app, _| panel_children(&app.state));
        println!(
            "{} panel children after title click: {}",
            space.slug(),
            n_after
        );
        assert_eq!(n_after, 1);

        let header_bounds = cx
            .debug_bounds(header_id)
            .unwrap_or_else(|| panic!("{} collapsed heading was not painted", space.slug()));
        let far_left = gpui::point(header_bounds.origin.x + px(2.0), header_bounds.center().y);
        cx.simulate_click(far_left, Modifiers::none());
        cx.run_until_parked();
        assert!(app.read_with(cx, |app, _| panel_children(&app.state)) > 1);
        println!(
            "{} Buts: full-width heading collapsed and expanded",
            space.slug()
        );
    }
}
