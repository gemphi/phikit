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
        side: BPopoverSide::Switcher,
        ..
    } = view
    else {
        panic!("switcher must be an opened two-column picker popover")
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
    // A property row is transparent in every host: a label/value row is
    // text and a control on whatever it sits on (`BSplitPart::Row`).
    assert!(!section.contains("BSurfaceTone::Section"), "no section tone behind a row");
    // A panel's header and frame are the card's (`card.rs`): the header a
    // step over the card it heads, the frame outlined, rounded and on the
    // panel tone; a sub-panel's header a step over its own well.
    let card = source("view/draw/card.rs");
    assert!(card.contains("strip.fill(BFill::Heading(BSurfaceTone::Card))"), "panel header surface was lost");
    assert!(card.contains("BFill::Heading(BSurfaceTone::Nested(*depth))"), "sub-panel header surface was lost");
    // A panel is a ground with no border round it: a group is a BG, not a box.
    assert!(!card.contains(".border(BEdge::All)"), "a panel takes no outer border");
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
    // The window's own bars (top bar, status bar) are not editors: no
    // editor type to switch them to, so no switcher.
    for &host in BHostType::ALL.iter().filter(|host| !host.is_fixed_bar()) {
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

            // Left, centre, right — the same three cells in every header, and
            // the same thing in each: what the editor IS on the left, what you
            // are LOOKING FOR in the middle, how it is FILTERED on the right
            // (`part::filter`'s header note). So search is never anywhere but
            // the centre pane, and the filter never anywhere but the right —
            // a control that drifted into the left cell would read as part of
            // the editor's identity.
            // A browser's header is the columns under it read across, so its
            // search stands over the details column, at the right.
            let search_home = if host.heads_its_columns() { BPaneType::Right } else { BPaneType::Center };
            for (part, home) in [(BPartType::Search, search_home), (BPartType::Filter, BPaneType::Right)] {
                for pane in [BPaneType::Left, BPaneType::Center, BPaneType::Right] {
                    let count = parts(pane).filter(|v| is(v, part)).count();
                    let allowed = usize::from(pane == home);
                    assert!(
                        count <= allowed,
                        "{host:?} header puts {part:?} in its {pane:?} pane; it belongs in {home:?}"
                    );
                }
            }
        }
    }
}

/// The header row itself: the three panes sit at the start, the centre and
/// the end, stated as `BJustify::Between` rather than left to the centre
/// pane's growth — a host with nothing in its middle paints no Center pane,
/// which stranded its right-hand controls mid-row. Both the one-line header
/// and the wrapping top bar say it, so the rule does not depend on width.
#[test]
fn a_header_row_pushes_its_right_cell_to_the_right() {
    let region = source("view/draw/region/mod.rs");
    let rows = region
        .split("let flex = match (axis, kind.placement()) {")
        .nth(1)
        .expect("the region's row flex")
        .split("(BAxis::Row, _) => BFlex::row(),")
        .next()
        .expect("the header arms come before the plain row");
    assert_eq!(
        rows.matches("justify(BJustify::Between)").count(),
        2,
        "both header arms — wrapping and not — must push their right cell to the right"
    );
    // And the right cell never gives its width up, at any window size: the
    // zone toggles and the window controls stay reachable.
    let pane = source("view/draw/pane.rs");
    assert!(pane.contains("cell.keeps_size().min_width(BLen::Measured(floor)).justify(BJustify::End)"));
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
        BPlacement::corner_for(BPopoverSide::Switcher, &style),
        gpui::Anchor::TopLeft,
        "the switcher must open into its area, not out of it"
    );
    // A header popover's trigger sits at the OTHER end of the header, so
    // "into its area" is the other corner: trailing edges flush, growing
    // back across the header rather than off the edge of the host.
    assert_eq!(
        BPlacement::corner_for(BPopoverSide::BelowEnd, &style),
        gpui::Anchor::TopRight,
        "a header popover must grow back across its header"
    );
    assert_eq!(
        BPlacement::corner_for(BPopoverSide::Below, &style),
        gpui::Anchor::TopLeft,
        "a menubar drop still hangs from its trigger's leading edge"
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
    // The content scrolls past that cap: the scroller is the popover's
    // render half (`popover/render.rs`), the box above it this module.
    assert!(source("view/draw/popover/render.rs").contains(".scroller("));
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
