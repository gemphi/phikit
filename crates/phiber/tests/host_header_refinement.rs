//! Focused regressions for Properties sections and host-header controls.
//!
//! Validates: Requirements 3.4, 3.5, 5.1-5.7, 10.1, 11.3, 12.7

use phiber::v2::{
    area::current,
    host::BHostType,
    pane::BPaneType,
    part::{BPartType, modebar::set::modes_for, switcher::BSwitcher, zonebar::BZonebar},
    region::{BRegionItem, BRegionType},
    space::BSpaceType,
    state::{BState, V_TRUE, action::apply_action, default_state, key::K_SPACE},
    view::{BAction, BPopoverSide, BView, BViewItem, BZoneToggle},
    zone::{BZoneType, visibility},
};
use std::{fs, path::PathBuf};

fn source(relative: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/v2")
        .join(relative);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn layout_center_area() -> phiber::v2::area::BAreaSpec {
    *BSpaceType::Layout
        .spec()
        .areas
        .iter()
        .find(|area| area.default_host == BHostType::View3D)
        .expect("Layout must retain its switchable center area")
}

fn open_switcher(state: &BState) -> BView {
    let area = layout_center_area();
    current::with_space(BSpaceType::Layout, || {
        current::with_area(area, || BSwitcher::new(BHostType::View3D).build(state))
    })
}

fn popup_columns(view: &BView) -> (&[BView], &[BView]) {
    let BView::Popover {
        body,
        open: true,
        side: BPopoverSide::BelowEnd,
        ..
    } = view
    else {
        panic!("switcher must be an opened, right-aligned popover")
    };
    let [BView::Row { children }] = body.as_slice() else {
        panic!("switcher body must have one two-column row")
    };
    let [
        BView::Column {
            children: categories,
        },
        BView::Column { children: hosts },
    ] = children.as_slice()
    else {
        panic!("switcher row must contain category and host columns")
    };
    (categories, hosts)
}

fn labels(views: &[BView]) -> Vec<&str> {
    views
        .iter()
        .filter_map(|view| match view {
            BView::MenuItem(ctl) => ctl.label.as_deref(),
            _ => None,
        })
        .collect()
}

#[test]
fn properties_rows_are_transparent_but_header_and_outer_card_remain_surfaces() {
    // After the prim.rs → prim/mod.rs split, panel logic lives in prim/panel/*.
    // Section transparency is in section.rs; card surfaces are in card.rs.
    let section = source("view/render/gpui/prim/panel/section.rs");
    assert!(section.contains("if c.host != crate::v2::host::BHostType::Buts"));
    // CardHeader is in heading.rs — verifiable via the panel mod router
    let panel_mod = source("view/render/gpui/prim/panel/mod.rs");
    assert!(panel_mod.contains("BPanelRole::CardHeader =>"));
    let card = source("view/render/gpui/prim/panel/card.rs");
    assert!(
        card.contains(".border(px(c.style.strokes.border))"),
        "outer accordion border was lost"
    );
    assert!(
        card.contains(".rounded("),
        "outer accordion radius was lost"
    );
    assert!(
        card.contains("panel_bg(BPanelRole::Card, c)"),
        "outer surface was lost"
    );
}

#[test]
fn active_geometry_is_fill_only_for_every_host() {
    // Contract: no tab / menu anchor uses an accent border strip on its
    // active state — the paper-tab illusion is delivered by fill only, so
    // the active cap and the surface beneath it (popover, pane body) form
    // one continuous filled surface. This applies to Topbar and every
    // editor host uniformly.
    for relative in [
        "view/render/gpui/atom/tab.rs",
        "view/render/gpui/atom/anchor.rs",
    ] {
        let renderer = source(relative);
        assert!(
            !renderer.contains("border_b(px(c.style.strokes.border * 2.0))"),
            "{relative} still paints an accent underline on active state"
        );
        assert!(
            !renderer.contains("border_l(px(c.style.strokes.border * 2.0))"),
            "{relative} still paints an accent left border on active state"
        );
    }
}

#[test]
fn switcher_uses_category_and_host_columns_without_replacing_host_metadata() {
    let area = layout_center_area();
    let mut state = BState::new();
    state.set(&current::switcher_key(BSpaceType::Layout, &area), V_TRUE);

    let view = open_switcher(&state);
    let (categories, hosts) = popup_columns(&view);
    assert_eq!(
        labels(categories),
        vec!["General", "Animation", "Scripting", "Data"]
    );
    assert!(labels(hosts).contains(&"3D Viewport"));
    assert!(!labels(hosts).contains(&"Properties"));

    let data_action = categories
        .iter()
        .find_map(|view| match view {
            BView::MenuItem(ctl) if ctl.label.as_deref() == Some("Data") => ctl.action.clone(),
            _ => None,
        })
        .expect("Data category must be selectable");
    assert!(matches!(data_action, BAction::Open(_)));
    apply_action(&mut state, &data_action);
    assert_eq!(
        state.get(&current::switcher_key(BSpaceType::Layout, &area)),
        Some(V_TRUE)
    );

    let rebuilt = open_switcher(&state);
    let (_, hosts) = popup_columns(&rebuilt);
    assert!(labels(hosts).contains(&"Properties"));
    assert!(!labels(hosts).contains(&"3D Viewport"));
}

#[test]
fn every_declared_host_header_has_one_rightmost_switcher_and_its_mode_control() {
    for &host in BHostType::ALL {
        for &region in host
            .spec()
            .regions
            .iter()
            .filter(|region| region.kind == BRegionType::Header)
        {
            let BView::Region {
                children: panes, ..
            } = BRegionItem::new(host, region).build(&BState::new())
            else {
                panic!("{host:?} header did not build")
            };
            let parts = |pane_kind| {
                panes
                    .iter()
                    .filter_map(move |pane| match pane {
                        BView::Pane { kind, children } if *kind == pane_kind => Some(children),
                        _ => None,
                    })
                    .flatten()
            };
            assert_eq!(
                parts(BPaneType::Right)
                    .filter(|view| matches!(
                        view,
                        BView::Part {
                            kind: BPartType::Switcher,
                            ..
                        }
                    ))
                    .count(),
                1,
                "{host:?} header must expose exactly one switcher"
            );
            assert!(
                matches!(
                    parts(BPaneType::Right).last(),
                    Some(BView::Part {
                        kind: BPartType::Switcher,
                        ..
                    })
                ),
                "{host:?} switcher must be the rightmost header control"
            );

            if !modes_for(host).is_empty() {
                assert_eq!(
                    parts(BPaneType::Center)
                        .filter(|view| matches!(
                            view,
                            BView::Part {
                                kind: BPartType::Modebar,
                                ..
                            }
                        ))
                        .count(),
                    1,
                    "{host:?} lost its mode control"
                );
            }
        }
    }
}

#[test]
fn host_switch_popup_aligns_its_right_edge_and_expands_left() {
    let popover = source("view/render/gpui/popover.rs");
    assert!(popover.contains("BPopoverSide::BelowEnd"));
    assert!(popover.contains(".top_full().right_0()"));
    assert!(!popover.contains("BPopoverSide::BelowEnd => el.absolute().top_full().left_0()"));
}

fn zone_controls(space: BSpaceType, state: &mut BState) -> Vec<BZoneToggle> {
    state.set(&K_SPACE, space.slug());
    let BView::ZoneToggleGroup { controls } = BZonebar::new(BHostType::Topbar).build(state) else {
        panic!("Topbar Zonebar must build one focused toggle group")
    };
    controls
}

#[test]
fn topbar_zone_controls_match_declared_zones_without_placeholders() {
    for (space, expected) in [
        (
            BSpaceType::Layout,
            vec![BZoneType::Left, BZoneType::Right, BZoneType::Bottom],
        ),
        (
            BSpaceType::UvEditing,
            vec![BZoneType::Left, BZoneType::Bottom],
        ),
        (BSpaceType::Shading, vec![BZoneType::Left, BZoneType::Right]),
        (BSpaceType::Compositing, vec![BZoneType::Bottom]),
    ] {
        let mut state = default_state();
        let actual: Vec<_> = zone_controls(space, &mut state)
            .into_iter()
            .map(|control| control.zone)
            .collect();
        assert_eq!(
            actual, expected,
            "{space:?} emitted absent-zone placeholders"
        );
        assert_eq!(BZonebar::declared_zones(space), expected);
    }
}

#[test]
fn topbar_zone_actions_use_canonical_visibility_keys_and_toggle_immediately() {
    let mut state = default_state();
    for control in zone_controls(BSpaceType::Layout, &mut state) {
        let key = visibility::key(control.zone).expect("toggleable zone key");
        assert_eq!(control.action, BAction::Toggle(key.clone()));
        assert!(control.visible);

        apply_action(&mut state, &control.action);
        let rebuilt = zone_controls(BSpaceType::Layout, &mut state);
        assert!(
            !rebuilt
                .iter()
                .find(|item| item.zone == control.zone)
                .unwrap()
                .visible
        );

        apply_action(&mut state, &control.action);
        let rebuilt = zone_controls(BSpaceType::Layout, &mut state);
        assert!(
            rebuilt
                .iter()
                .find(|item| item.zone == control.zone)
                .unwrap()
                .visible
        );
    }
}

#[test]
fn topbar_zone_group_is_in_the_right_pane_before_the_preserved_switcher() {
    let region = BHostType::Topbar
        .spec()
        .regions
        .iter()
        .find(|region| region.kind == BRegionType::Header)
        .expect("Topbar header");
    let BView::Region {
        children: panes, ..
    } = BRegionItem::new(BHostType::Topbar, *region).build(&default_state())
    else {
        panic!("Topbar header did not build")
    };
    let right = panes
        .iter()
        .find_map(|pane| match pane {
            BView::Pane {
                kind: BPaneType::Right,
                children,
            } => Some(children),
            _ => None,
        })
        .expect("Topbar right pane");

    assert_eq!(
        right
            .iter()
            .filter(|part| matches!(
                part,
                BView::Part {
                    kind: BPartType::Zonebar,
                    ..
                }
            ))
            .count(),
        1
    );
    let zonebar = right
        .iter()
        .position(|part| {
            matches!(
                part,
                BView::Part {
                    kind: BPartType::Zonebar,
                    ..
                }
            )
        })
        .unwrap();
    let switcher = right
        .iter()
        .position(|part| {
            matches!(
                part,
                BView::Part {
                    kind: BPartType::Switcher,
                    ..
                }
            )
        })
        .unwrap();
    assert!(zonebar < switcher);
    assert_eq!(
        switcher,
        right.len() - 1,
        "host switcher must remain rightmost"
    );
}

#[test]
fn accordion_heading_container_owns_the_full_width_hand_cursor_and_only_handler() {
    let focused = source("view/render/gpui/prim/accordion.rs");
    assert_eq!(
        focused.matches(".on_click(").count(),
        1,
        "the heading container must be the sole toggle handler"
    );
    assert_eq!(
        focused.matches("(Rc::clone(&handler))(&action").count(),
        1,
        "one click must dispatch exactly one action"
    );
    assert!(focused.contains(".w_full()"));
    assert!(focused.contains(".cursor(gpui::CursorStyle::PointingHand)"));
    assert!(focused.contains("accordion-header-"));
    assert!(focused.contains(".debug_selector("));
    assert!(focused.contains("accordion-title-"));
    assert!(focused.contains("accordion-disclosure-"));
}

#[test]
fn representative_host_stacks_allocate_zero_seam_gap_and_one_header_divider() {
    for host in [
        BHostType::View3D,
        BHostType::Buts,
        BHostType::Outliner,
        BHostType::Node,
        BHostType::Image,
    ] {
        let regions = host.spec().regions;
        assert_eq!(
            regions
                .iter()
                .filter(|region| region.kind == BRegionType::Header)
                .count(),
            1,
            "{host:?} must retain one host header"
        );
        assert!(
            regions
                .iter()
                .any(|region| region.kind != BRegionType::Header),
            "{host:?} must retain content below its header"
        );
    }

    let host = source("view/render/gpui/host.rs");
    assert!(
        !host.contains(".gap("),
        "host stacking must allocate zero gap"
    );
    assert!(
        !host.contains(".bg("),
        "generic host wrapper must be transparent"
    );
    assert!(host.contains(".flex_1()"));
    assert!(host.contains(".min_h_0()"));

    let region = source("view/render/gpui/region.rs");
    assert_eq!(
        region
            .matches(".border_b(px(c.style.strokes.border))")
            .count(),
        1,
        "Header must own exactly one semantic bottom divider"
    );
    assert!(
        !region.contains(".border_t("),
        "content must not double the seam"
    );
    assert!(region.contains("Some(c.style.dimensions.header_height)"));
    assert!(region.contains("el.flex_1().min_h_0().h_full().w_full()"));
}

#[test]
fn unified_switcher_surface_contains_two_transparent_columns() {
    let popover = source("view/render/gpui/popover.rs");
    assert!(popover.contains("fn switcher_columns("));
    assert!(popover.matches("BView::Column").count() >= 2);
    assert_eq!(popover.matches(".border_r(").count(), 1);
    assert_eq!(popover.matches(".border_1()").count(), 1);
    assert_eq!(popover.matches(".shadow_xl()").count(), 1);
    assert_eq!(popover.matches(".overflow_hidden()").count(), 1);
    assert!(popover.contains("compatible_inner_radius("));
    assert!(popover.contains(".top_full().right_0()"));

    let columns = popover
        .split("fn switcher_columns(")
        .nth(1)
        .expect("focused switcher column renderer")
        .split("pub fn view(")
        .next()
        .expect("switcher column renderer end");
    assert!(
        !columns.contains(".bg("),
        "inner switcher columns must be transparent"
    );
}

#[test]
fn properties_accordions_share_one_pane_surface_without_losing_interaction() {
    // Panel shades resolve through the singular family-scoped style module.
    let style = source("view/render/gpui/prim/panel/style.rs");
    assert!(style.contains("let properties = c.host == BHostType::Buts;"));
    assert!(style.contains("BPanelRole::Sidebar if properties => BSurfaceTone::Body"));
    assert!(style.contains("BPanelRole::Card if properties => BSurfaceTone::Body"));
    assert!(style.contains("BPanelRole::Body => BSurfaceTone::Body"));

    let section = source("view/render/gpui/prim/panel/section.rs");
    assert!(section.contains("if c.host != crate::v2::host::BHostType::Buts"));

    let card = source("view/render/gpui/prim/panel/card.rs");
    assert!(card.contains(".rounded("));
    assert!(card.contains(".border(px(c.style.strokes.border))"));
    assert!(card.contains("panel_bg(BPanelRole::Card, c)"));
    assert!(!card.contains(".overflow_hidden()"));

    let body = source("view/render/gpui/prim/panel/body.rs");
    for required in [
        ".flex_1()",
        ".min_h_0()",
        ".min_w_0()",
        ".overflow_y_scroll()",
    ] {
        assert!(body.contains(required), "Body lost {required}");
    }

    let heading = source("view/render/gpui/prim/accordion.rs");
    assert!(heading.contains(".cursor(gpui::CursorStyle::PointingHand)"));
    assert_eq!(heading.matches(".on_click(").count(), 1);
}

#[test]
fn ordinary_regions_are_transparent_while_specialized_canvases_keep_surface_ownership() {
    let region = source("view/render/gpui/region.rs");
    assert!(region.contains("_ => gpui::transparent_black().into()"));
    assert!(region.contains("BRegionType::Header | BRegionType::ToolHeader"));
    assert!(region.contains("strip_gradient(c.host)"));

    // After the canvas.rs → canvas/mod.rs split, the fallback lives in
    // atom/canvas/fallback.rs; focused canvases remain in their sub-modules.
    let _canvas_mod = source("view/render/gpui/atom/canvas/mod.rs");
    let host_canvas = source("view/render/gpui/atom/canvas/fallback.rs");
    let view3d = source("view/render/gpui/atom/canvas/view3d.rs");
    let node = source("view/render/gpui/atom/canvas/node.rs");
    let image = source("view/render/gpui/atom/canvas/image.rs");
    // The fallback bg_viewport fill is in fallback.rs — uses the resolved
    // semantic viewport surface via c.style.viewport_surface() (migrated form).
    assert!(host_canvas.contains("c.style.viewport_surface()"));
    assert!(!host_canvas.contains("c.p."));
    assert!(view3d.contains("c.style.viewport_surface()"));
    assert!(view3d.contains(".bg(vp)"));
    assert!(node.contains(".bg(bg)"));
    assert!(image.contains(".bg(bg)"));
    assert!(image.contains("if (row + col) % 2 == 0"));
}

#[test]
fn properties_region_stays_edge_to_edge_while_bounded_body_owns_one_pane_colored_inset() {
    let region = source("view/render/gpui/region.rs");
    let properties_center = region
        .split("else if c.host == BHostType::Buts && kind == BRegionType::Center")
        .nth(1)
        .expect("Properties Center region edge-to-edge branch")
        .split("} else {")
        .next()
        .expect("Properties Center branch end");
    assert!(
        !properties_center.contains(".p("),
        "Properties Center must not duplicate the bounded owner's inset"
    );
    assert!(
        region.contains("el.p(px(c.style.spacing.pad_md))"),
        "ordinary host content must retain resolved region padding"
    );
    assert!(
        region.contains("el.px(px(c.style.spacing.pad_md))"),
        "host headers must retain their horizontal content gutter"
    );

    // After the prim.rs → prim/mod.rs split, sidebar/body live in focused panel modules.
    let sidebar = source("view/render/gpui/prim/panel/sidebar.rs");
    let body = source("view/render/gpui/prim/panel/body.rs");
    for (name, block) in [("Sidebar", sidebar.as_str()), ("Body", body.as_str())] {
        assert!(
            block.contains("let outer_padding = c.style.spacing.pad_sm;"),
            "{name} must source the collection inset from resolved spacing"
        );
        assert_eq!(
            block.matches(".p(px(outer_padding))").count(),
            1,
            "{name} must apply exactly one collection-level padding layer"
        );
        assert!(
            block.contains("panel_bg(BPanelRole::"),
            "{name} padding must be painted by the pane surface"
        );
    }
    for required in [
        ".flex_1()",
        ".min_h_0()",
        ".min_w_0()",
        ".overflow_y_scroll()",
        ".gap(px(c.style.spacing.gap_xs))",
    ] {
        assert!(body.contains(required), "Body lost {required}");
    }

    // Surface shade checks are in the singular family style module.
    let style = source("view/render/gpui/prim/panel/style.rs");
    assert!(style.contains("BPanelRole::Body => BSurfaceTone::Body"));
    assert!(style.contains("BPanelRole::Card if properties => BSurfaceTone::Body"));

    // Section transparency check in section.rs.
    let section = source("view/render/gpui/prim/panel/section.rs");
    assert!(section.contains("if c.host != crate::v2::host::BHostType::Buts"));
    assert!(section.contains(".px(px(text_gutter(c)))"));

    let card = source("view/render/gpui/prim/panel/card.rs");
    assert!(card.contains(".rounded("));
    assert!(card.contains(".border(px(c.style.strokes.border))"));
    assert!(card.contains(".gap(px(c.style.spacing.gap_xs))"));
    assert!(card.contains("panel_bg(BPanelRole::Card, c)"));

    let heading = source("view/render/gpui/prim/accordion.rs");
    assert!(heading.contains(".text_size(px(c.style.typography.body))"));
    assert!(heading.contains("dimensions.header_height - 2.0 * c.style.spacing.pad_xs"));
    assert!(heading.contains(".cursor(gpui::CursorStyle::PointingHand)"));
    assert_eq!(heading.matches(".on_click(").count(), 1);

    let host = source("view/render/gpui/host.rs");
    assert!(
        !host.contains(".gap("),
        "header and content must stay adjacent"
    );
    assert_eq!(
        region
            .matches(".border_b(px(c.style.strokes.border))")
            .count(),
        1,
        "the header/content seam must retain one divider"
    );
}
