//! Focused regressions for Properties sections and host-header controls.
//!
//! Validates: Requirements 3.4, 3.5, 5.1-5.7, 10.1, 11.3, 12.7

use phiber::v2::{
    area::current,
    host::BHostType,
    pane::BPaneType,
    part::{BPartType, modebar::set::modes_for, switcher::BSwitcher},
    region::{BRegionItem, BRegionType},
    space::BSpaceType,
    state::{BState, V_TRUE, action::apply_action},
    view::{BAction, BPopoverSide, BView, BViewItem},
};
use std::{fs, path::PathBuf};

fn source(relative: &str) -> String {
    let path = module_file(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/v2")
        .join(relative));
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
    let [BView::Row { list }] = body.as_slice() else {
        panic!("switcher body must have one two-column row")
    };
    let [
        BView::Column {
            list: categories,
        },
        BView::Column { list: hosts },
    ] = list.as_slice()
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
    // After the prim.rs → prim.rs split, region columns live in prim/{body,sidebar}.rs.
    // Section transparency is in section.rs; card surfaces are in card.rs.
    let section = source("view/draw/split.rs");
    // A property row carries the section tone everywhere but Properties,
    // where the card it sits in already does — `BSplitPart::Row { toned }`.
    assert!(section.contains("toned: ctx.host != crate::v2::host::BHostType::Buts"));
    // A panel's header and frame are the card's (`card.rs`): the header a
    // step over the card it heads, the frame outlined, rounded and on the
    // panel tone; a sub-panel's header a step over its own well.
    let card = source("view/draw/card.rs");
    assert!(card.contains("strip.fill(BFill::Heading(BSurfaceTone::Card))"), "panel header surface was lost");
    assert!(card.contains("BFill::Heading(BSurfaceTone::Nested(*depth))"), "sub-panel header surface was lost");
    assert!(card.contains(".border(BEdge::All)"), "outer panel border was lost");
    assert!(card.contains(".radius(BRadius::Role(BStyleRole::Card))"), "outer panel radius was lost");
    // Through `panel_fill`, which picks swept or flat from the one card tone
    // (`the_activity_strip_is_card_content`).
    assert!(card.contains(".fill(panel_fill(ctx))"), "outer surface was lost");
    assert!(card.contains("BFill::Tone(BSurfaceTone::Card)"), "and it is still the card tone, swept");
}

#[test]
fn active_geometry_is_fill_only_for_every_host() {
    // Contract: no tab / menu anchor uses an accent border strip on its
    // active state — the paper-tab illusion is delivered by fill only, so
    // the active cap and the surface beneath it (popover, pane body) form
    // one continuous filled surface. This applies to Topbar and every
    // editor host uniformly.
    for relative in [
        "view/draw/tab/mod.rs",
        "view/draw/anchor.rs",
    ] {
        let renderer = source(relative);
        assert!(
            !renderer.contains("border_b(px(ctx.style.strokes.border * 2.0))"),
            "{relative} still paints an accent underline on active state"
        );
        assert!(
            !renderer.contains("border_l(px(ctx.style.strokes.border * 2.0))"),
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
    // Host labels are DISPLAY names — `BHostType::slug()` carries identity —
    // so a relabel updates this expectation. Blender's "Properties" editor is
    // labelled "Inspector" here.
    assert!(labels(hosts).contains(&"3D Viewport"));
    assert!(!labels(hosts).contains(&"Inspector"));

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
    assert!(labels(hosts).contains(&"Inspector"));
    assert!(!labels(hosts).contains(&"3D Viewport"));
}

#[test]
fn every_declared_host_header_leads_with_its_switcher_and_keeps_its_mode_control() {
    for &host in BHostType::ALL {
        for &region in host
            .spec()
            .regions
            .iter()
            .filter(|region| region.kind == BRegionType::Header)
        {
            let BView::Region {
                list: panes, ..
            } = BRegionItem::new(host, region).build(&BState::new())
            else {
                panic!("{host:?} header did not build")
            };
            let parts = |pane_kind| {
                panes
                    .iter()
                    .filter_map(move |pane| match pane {
                        BView::Pane { kind, list } if *kind == pane_kind => Some(list),
                        _ => None,
                    })
                    .flatten()
            };
            let is = |view: &&BView, want: BPartType| {
                matches!(view, BView::Part { kind, .. } if *kind == want)
            };
            // The switcher names the editor, so it heads the header: icon +
            // name on the left, the same place in every host.
            assert_eq!(
                parts(BPaneType::Left).filter(|v| is(v, BPartType::Switcher)).count()
                    + parts(BPaneType::Center).filter(|v| is(v, BPartType::Switcher)).count()
                    + parts(BPaneType::Right).filter(|v| is(v, BPartType::Switcher)).count(),
                1,
                "{host:?} header must expose exactly one switcher"
            );
            assert!(
                matches!(
                    parts(BPaneType::Left).next(),
                    Some(BView::Part { kind: BPartType::Switcher, .. })
                ),
                "{host:?} switcher must lead the header"
            );

            if !modes_for(host).is_empty() {
                assert_eq!(
                    parts(BPaneType::Left).filter(|v| is(v, BPartType::Modebar)).count(),
                    1,
                    "{host:?} lost its mode control (it sits beside the switcher)"
                );
            }
        }
    }
}

#[test]
fn host_switch_popup_opens_inside_its_area() {
    use phiber::v2::view::draw::popover::place::BPlacement;

    // The switcher trigger sits at its area's leading edge, so the picker
    // grows into the area. Anchoring it to the trigger's right edge pushed a
    // two-panel-wide panel leftwards, out of the host entirely.
    let style = phiber::v2::theme::resolve_ui_style(
        phiber::v2::theme::ui_style_defaults(phiber::v2::theme::BPaletteName::DEFAULT),
        &BState::new(),
    );
    assert_eq!(
        BPlacement::corner_for(BPopoverSide::BelowEnd, &style),
        gpui::Anchor::TopLeft,
        "the switcher must open into its area, not out of it"
    );
    // Every popover resolves its side through the one placement rule, so a
    // trigger with no room on its preferred side flips instead of clipping.
    let popover = source("view/draw/popover/mod.rs");
    assert!(
        popover.contains("place::BPlacement::of(self.side, ctx)"),
        "placement must go through the shared rule, not a per-side builder chain"
    );
    // And the panel is bounded on both axes, so a long host list scrolls inside
    // it rather than running past the bottom of the area. The panel says so in
    // `BFlex`, as every other level states its own box, and the content is
    // still a scroller.
    //
    // Two panel widths still bounds the WIDTH — that is the two-column
    // switcher's own measure. It no longer bounds the height: 480px is less
    // than the File menu needs, so capping height by a width meant a menu that
    // could not fit however it was drawn, and what hid that was its rows being
    // squashed (`a_row_is_one_height`, `chrome_geometry`). Height comes from the
    // window now, which is the thing a popover must not run past.
    assert!(popover.contains("let wide = ctx.style.dimensions.panel_width * 2.0;"));
    assert!(popover.contains(".max_width(BLen::Measured(wide))"));
    assert!(popover.contains(".max_height(BLen::Measured("), "the height is still bounded");
    assert!(popover.contains("AppKey::WindowHeight"), "and bounded by the window");
    assert!(
        !popover.contains(".max_height(BLen::Measured(wide))"),
        "but not by a width — that was the cap no long menu could meet"
    );
    assert!(popover.contains(".scroller("));
}

/// A module is a file or a folder; read whichever spelling exists.
fn module_file(path: std::path::PathBuf) -> std::path::PathBuf {
    if path.exists() {
        return path;
    }
    // `x.rs` asked for, `x/mod.rs` exists.
    let folder = path.with_extension("").join("mod.rs");
    if folder.exists() {
        return folder;
    }
    // `x/mod.rs` asked for, `x.rs` exists.
    if path.file_name().is_some_and(|n| n == "mod.rs") {
        if let Some(parent) = path.parent() {
            let file = parent.with_extension("rs");
            if file.exists() {
                return file;
            }
        }
    }
    path
}
