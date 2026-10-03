//! Focused File Browser table and Properties accordion regressions.
//!
//! Validates: Requirements 6.1-6.6, 10.1, 11.3, 12.7, 13.8

use phiber::v2::state::BKey as _;
use phiber::v2::state::FileKey;
use phiber::v2::{
    host::BHostType,
    part::{
        file::{BFileKind, BFileRow, BFileSort, listing, table},
        props::{BPanel, BPanelItem, BPropSpec, set::sec},
    },
    state::{BState, V_FALSE, action::apply_action},
    view::{
        BAction, BColAlign, BColWidth, BRowShade, BSortDirection, BView, BViewItem,
    },
};
use std::{fs, path::PathBuf};

fn source(relative: &str) -> String {
    let path = module_file(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative));
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn file(name: &str, size: u64, modified: u64) -> BFileRow {
    BFileRow {
        name: name.to_string(),
        path: format!("/fixture/{name}"),
        kind: BFileKind::Text,
        size,
        modified: Some(modified),
    }
}

fn folder(name: &str) -> BFileRow {
    BFileRow {
        name: name.to_string(),
        path: format!("/fixture/{name}"),
        kind: BFileKind::Folder,
        size: 0,
        modified: Some(0),
    }
}

fn header_cells(state: &BState) -> &[phiber::v2::view::BTableCell] {
    let header = Box::leak(Box::new(table::header(state)));
    let BView::TableHeader { cell_list } = header else {
        panic!("shared File Browser header must use TableHeader")
    };
    cell_list
}

fn sort_header(cell: &phiber::v2::view::BTableCell) -> (&str, Option<BSortDirection>, &BAction) {
    let BView::SortHeader {
        label,
        direction,
        action,
        ..
    } = cell.view.as_ref()
    else {
        panic!("table cell must use the stable SortHeader primitive")
    };
    (label, *direction, action)
}

#[test]
fn normal_file_rows_are_flat_and_keep_full_row_selection_identity() {
    let row = file("alpha.txt", 11, 22);
    let BView::TableRow {
        cell_list,
        shade,
        action,
    } = table::row(&row, "")
    else {
        panic!("file entries must use TableRow")
    };
    assert_eq!(shade, BRowShade::Flat);
    assert_eq!(
        action,
        Some(BAction::Set(FileKey::Selected.path(), row.path.clone()))
    );
    assert_eq!(cell_list.len(), 3);

    let BView::TableRow { shade, .. } = table::row(&row, &row.path) else {
        panic!()
    };
    assert_eq!(shade, BRowShade::Selected);

    // After the prim.rs → prim.rs split, table row logic lives in table.rs.
    let renderer = source("src/v2/view/draw/table.rs");
    let table_row = renderer
        .split("pub fn view_row(")
        .nth(1)
        .unwrap_or_else(|| renderer.as_str())
        .split("pub fn view_header(")
        .next()
        .unwrap_or_else(|| renderer.as_str());
    // The row's fill and hover are the tokens the tree, the list and the
    // table share (`BFill::Row` / `BFill::RowHover` → `tone::row_fill`),
    // not a mapping of its own.
    assert!(renderer.contains("BFill::Row(shade)"));
    assert!(renderer.contains("BFill::RowHover(shade)"));
    assert!(!table_row.contains("BRowShade::Alt"));
    assert!(!table_row.contains("METRICS"));
    assert!(!table_row.contains("ctx.p"));
    assert!(table_row.contains(".id(id)"));
    assert!(table_row.contains(".on_click("));
}

#[test]
fn shared_header_and_body_keep_flexible_name_and_fixed_metadata_geometry() {
    let state = BState::new();
    let header = header_cells(&state);
    let BView::TableRow { cell_list: body, .. } = table::row(&file("wide-name.txt", 4, 5), "") else {
        panic!()
    };
    assert_eq!(header.len(), 3);
    assert_eq!(body.len(), 3);
    assert!(matches!(header[0].width, BColWidth::Flex));
    assert!(matches!(body[0].width, BColWidth::Flex));
    for cell_list in [header, body.as_slice()] {
        assert_eq!(cell_list[0].align, BColAlign::Start);
        assert_eq!(cell_list[1].width, BColWidth::FileDate);
        assert_eq!(cell_list[1].align, BColAlign::Start);
        assert_eq!(cell_list[2].width, BColWidth::FileSize);
        assert_eq!(cell_list[2].align, BColAlign::End);
    }

    // Table cell/row geometry lives in table.rs, described with `BFlex`: a
    // name column takes the free space, a date and a size column keep the
    // width the theme gives them.
    let renderer = source("src/v2/view/draw/table.rs");
    assert!(renderer.contains("BColWidth::Flex => cell.fills().min_width(BLen::Zero)"));
    assert!(renderer.contains(".gap(BSpace::GapMd)"));
    assert!(renderer.contains("ctx.style.dimensions.file_date_column_width"));
    assert!(renderer.contains("ctx.style.dimensions.file_size_column_width"));
    assert!(!renderer.contains("div()"), "the table no longer builds a bare div");
}

#[test]
fn all_headers_sort_and_repeated_clicks_toggle_a_stable_direction_slot() {
    let mut state = BState::new();
    let initial = header_cells(&state);
    let (name, direction, name_action) = sort_header(&initial[0]);
    assert_eq!((name, direction), ("Name", Some(BSortDirection::Ascending)));
    assert_eq!(name_action, &BAction::Toggle(FileKey::SortInvert.path()));
    let name_action = name_action.clone();

    apply_action(&mut state, &name_action);
    assert_eq!(listing::sort_inverted(&state), true);
    assert_eq!(
        sort_header(&header_cells(&state)[0]).1,
        Some(BSortDirection::Descending)
    );

    for (index, expected) in [(1, BFileSort::Time), (2, BFileSort::Size)] {
        let (_, direction, action) = sort_header(&header_cells(&state)[index]);
        assert_eq!(direction, None);
        assert_eq!(
            action,
            &BAction::Set(FileKey::Sort.path(), expected.label().to_string())
        );
        let action = action.clone();
        apply_action(&mut state, &action);
        assert_eq!(listing::sort_mode(&state), expected);
        assert!(
            !listing::sort_inverted(&state),
            "new columns start ascending"
        );
        let (_, direction, repeat) = sort_header(&header_cells(&state)[index]);
        assert_eq!(direction, Some(BSortDirection::Ascending));
        let repeat = repeat.clone();
        apply_action(&mut state, &repeat);
        assert_eq!(
            sort_header(&header_cells(&state)[index]).1,
            Some(BSortDirection::Descending)
        );
    }

    // The heading is described with `BFlex`, not hand-written GPUI: the
    // arrow slot is a `BFlexed` sized from the theme, reserved whether or
    // not this is the sorted column, and the hover is the shared token
    // rather than a `.hover(|s| s.bg(…))` of its own.
    let focused = source("src/v2/view/draw/sort.rs");
    assert!(focused.contains("Self::Indicator =>"), "the arrow slot is a part of the heading enum");
    assert!(focused.contains(".size(px(size))"));
    assert!(focused.contains("let Some(direction) = self.direction else"));
    assert!(focused.contains(".hovered(BFill::ItemHover)"));
    assert!(focused.contains("BFill::ControlActive"));
    assert!(!focused.contains("div()"), "the heading no longer builds a bare div");
}

#[test]
fn ascending_and_descending_rebuild_order_is_deterministic_and_directory_first() {
    let original = vec![
        file("zeta.txt", 30, 300),
        folder("Folder"),
        file("Alpha.txt", 10, 100),
        file("beta.txt", 20, 200),
    ];
    for mode in [BFileSort::Name, BFileSort::Time, BFileSort::Size] {
        let mut ascending = original.clone();
        listing::sort_rows(&mut ascending, mode, false);
        assert_eq!(ascending[0].kind, BFileKind::Folder);
        assert_eq!(ascending[1].name, "Alpha.txt");
        assert_eq!(ascending[2].name, "beta.txt");
        assert_eq!(ascending[3].name, "zeta.txt");

        let mut descending = original.clone();
        listing::sort_rows(&mut descending, mode, true);
        assert_eq!(descending[0].kind, BFileKind::Folder);
        assert_eq!(descending[1].name, "zeta.txt");
        assert_eq!(descending[2].name, "beta.txt");
        assert_eq!(descending[3].name, "Alpha.txt");
    }
}

static EMPTY_PROPS: &[BPropSpec] = &[];

#[test]
fn properties_heading_owns_one_full_width_toggle_action() {
    let spec = sec("Surface", EMPTY_PROPS);
    let BView::Card(card) = BPanelItem::new(BHostType::Buts, "Material", spec, true).build(&BState::new())
    else {
        panic!("section must be a card")
    };
    assert!(card.body.is_empty() && card.is_open());
    let BAction::Set(path, value) = card.fold.as_ref().expect("a panel folds").flip() else {
        panic!("the heading must own one explicit Set")
    };
    assert_eq!(card.title(), Some("Surface"));
    assert_eq!(path.encode(), "ctx.buts.material.surface.expanded");
    assert_eq!(value, V_FALSE);

    let focused = source("src/v2/view/draw/card.rs");
    // The header's gestures (click, Ctrl-click, drag-sweep) live in `fold.rs`.
    let gestures = source("src/v2/view/draw/fold.rs");
    assert_eq!(
        focused.matches(".on_click(").count() + gestures.matches(".on_click(").count(),
        1,
        "the header is the fold's one click target"
    );
    assert!(focused.contains("BLen::Full"));
    assert!(focused.contains("card-head-"));
    assert!(focused.contains("card-fold-"));
}

/// Blender's panel header (`panel_draw_aligned_widgets`): the fold triangle
/// at the left — `ICON_RIGHTARROW` closed, `ICON_DOWNARROW_HLT` open — the
/// title after it, the grip at the right at half alpha. Blender's is
/// `PNL_HEADER` (1.25 units); phiber's is the button height, so a heading
/// and the rows and buttons under it are one module and line up.
#[test]
fn the_card_heading_is_blenders_panel_header() {
    let focused = source("src/v2/view/draw/card.rs");
    assert!(focused.contains(r#"if open { "triangle_down" } else { "triangle_right" }"#));
    assert!(focused.contains(r#"BView::icon("grip")"#));
    assert!(focused.contains("BCardPart::Glyph { dimmed: true }"));
    assert!(focused.contains("BDim::CardHead"), "a card heading is the card-head module, not the area header's");
    assert!(!focused.contains("BDim::HeaderHeight"));
    let style = phiber::v2::theme::ui_style_defaults(phiber::v2::theme::BPaletteName::DEFAULT);
    // The card-head module: the button plus `HEAD_ROOM`, so a lit button in a
    // heading or a row has room over and under it.
    assert_eq!(style.dimensions.card_head_height(), style.dimensions.button_height + phiber::v2::theme::HEAD_ROOM, "a heading is the button plus its head room");
    assert_eq!(phiber::v2::theme::HEAD_ROOM, 4.0);
    let fold = focused.find("card-fold-").unwrap();
    let title = focused.find("card-title-").unwrap();
    let grip = focused.find("card-grip-").unwrap();
    assert!(fold < title && title < grip, "triangle, then title, then grip");
}

#[test]
fn properties_keep_transparent_rows_outer_border_and_bounded_scroll_owner() {
    let BView::Card(column) = BPanel::new(BHostType::Buts).build(&BState::new()) else {
        panic!("Properties content must be owned by a bounded Body scroller")
    };
    assert_eq!(column.kind, phiber::v2::view::prim::BCardKind::Column(phiber::v2::view::prim::BColumn::Body));

    // The property-split row (transparent in Properties) is `split.rs`: the
    // condition is a named part now — `BSplitPart::Row { toned }` — rather
    // than an `if` around a fill.
    let section = source("src/v2/view/draw/split.rs");
    assert!(section.contains("toned: ctx.host != crate::v2::host::BHostType::Buts"));
    assert!(section.contains("if toned { row.fill(BFill::Tone(BSurfaceTone::Section)) } else { row }"));

    // A panel's outline, rounding and surface are the card's frame, and it
    // does not clip (a row's dropdown escapes it).
    let card = source("src/v2/view/draw/card.rs");
    assert!(card.contains(".border(BEdge::All)"));
    assert!(card.contains(".radius(BRadius::Role(BStyleRole::Card))"));
    // The frame still fills from the panel tone — through `panel_fill`, which
    // chooses how that tone is painted: swept for a card lying on a region,
    // flat where the panel IS its region and there is nothing under it to be
    // raised from (`the_activity_strip_is_card_content`).
    assert!(card.contains(".fill(panel_fill(ctx))"));
    assert!(card.contains("BFill::Tone(BSurfaceTone::Card)") && card.contains("fn panel_fill("));
    let frame = card.split("Self::Frame(kind) =>").nth(1).unwrap().split("Self::Head").next().unwrap();
    assert!(!frame.contains("BOverflow::Clip"));

    // Body scroll ownership is in body.rs.
    let body = source("src/v2/view/draw/body.rs");
    for required in [
        ".flex_1()",
        ".w_full()",
        ".min_h_0()",
        ".min_w_0()",
        ".overflow_y_scroll()",
        "let outer_padding = ctx.style.spacing.pad_sm;",
        ".p(px(outer_padding))",
        // The body's surface is named by its tone, not by a helper of its own.
        "BSurfaceTone::Body",
    ] {
        assert!(body.contains(required), "Body lost {required}");
    }
}

#[test]
fn bounded_content_keeps_paint_on_outer_surfaces() {
    for (name, relative, role) in [
        // The surface is named by its tone, not by a helper per caller: a
        // headed body is a card, a plain body and a sidebar are the body
        // tone (a sidebar holds cards, and a card carries its own surface).
        (
            "Body",
            "src/v2/view/draw/body.rs",
            "BFill::Tone(if headed { BSurfaceTone::Card } else { BSurfaceTone::Body }).background(ctx)",
        ),
        (
            "Sidebar",
            "src/v2/view/draw/sidebar.rs",
            "BFill::Tone(BSurfaceTone::Body).background(ctx)",
        ),
    ] {
        let renderer = source(relative);
        let outer = renderer
            .split("let outer = div()")
            .nth(1)
            .expect("focused panel outer surface")
            .split("let mut inner = div()")
            .next()
            .expect("outer surface end");
        let inner = renderer
            .split("let mut inner = div()")
            .nth(1)
            .expect("focused panel inner scroller")
            .split("for ch in self.0")
            .next()
            .expect("inner scroller end");

        for paint in [
            role.to_string(),
            ".border(px(ctx.style.strokes.border))".to_string(),
            ".border_color(to_hsla(ctx.style.colors.border))".to_string(),
            ".rounded(px(ctx.style.radius_for(BStyleRole::Panel)))".to_string(),
        ] {
            assert!(outer.contains(&paint), "{name} outer lost {paint}");
            assert!(!inner.contains(&paint), "{name} inner claimed {paint}");
        }
        assert!(
            !outer.contains(".overflow_y_scroll()"),
            "{name} outer scrolls"
        );
        for bound in [
            ".flex_1()",
            ".min_h_0()",
            ".min_w_0()",
            ".overflow_y_scroll()",
        ] {
            assert!(inner.contains(bound), "{name} inner lost {bound}");
        }
    }

    // The area is described through `BFlex`: its card (`fn card`) carries the
    // paint and never scrolls; the scroller (`BAreaScroll`) fills its slot,
    // may shrink on both axes, and is the only part that scrolls.
    let area = source("src/v2/view/draw/area.rs");
    let area_surface = area
        .split("fn card(")
        .nth(1)
        .expect("area surface")
        .split("/// An area as drawn")
        .next()
        .expect("area surface end");
    for paint in [
        ".fill(BFill::HostBody)",
        ".border(BEdge::All)",
        ".radius(BRadius::Role(BStyleRole::Area))",
    ] {
        assert!(area_surface.contains(paint), "area outer lost {paint}");
    }
    assert!(!area_surface.contains("scroller"));
    // The scroller is a part of the area's own enum (`BAreaPart::Scroll`),
    // not a type of its own.
    let area_scroll = area
        .split("Self::Scroll =>")
        .nth(1)
        .expect("area scroller")
        .split("Self::Frame")
        .next()
        .expect("area scroller end");
    for bound in [".fills()", ".unclamped()"] {
        assert!(area_scroll.contains(bound), "area inner lost {bound}");
    }
    assert!(
        area.contains("BAreaPart::Scroll\n            .flex(ctx)\n            .scroller("),
        "area inner no longer scrolls"
    );

    let card = source("src/v2/view/draw/card.rs");
    for paint in [
        // Through `panel_fill`, which picks swept or flat from the panel tone.
        ".fill(panel_fill(ctx))",
        ".border(BEdge::All)",
        ".radius(BRadius::Role(BStyleRole::Card))",
    ] {
        assert!(card.contains(paint), "card outer lost {paint}");
    }
    assert!(!card.contains("scroller("));

    // A row is a strip along the horizontal axis, so its bounds live with the
    // rest of the strip geometry (`strip`), shared with the rail.
    for relative in [
        "src/v2/view/draw/strip.rs",
        "src/v2/view/draw/column.rs",
    ] {
        let renderer = source(relative);
        // `min_height(BLen::Zero)` is the `BFlex` spelling of `.min_h_0()`:
        // what lets a flex child shrink below its content so the scroller,
        // not the row, absorbs the overflow.
        assert!(
            renderer.contains(".min_h_0()") || renderer.contains("min_height(BLen::Zero)"),
            "{relative} lost height bound"
        );
        assert!(
            renderer.contains(".min_w_0()") || renderer.contains("min_width(BLen::Zero)"),
            "{relative} lost width bound"
        );
    }
}

#[test]
fn focused_renderers_keep_named_ownership_and_delegation_only_routers() {
    let prim = source("src/v2/view/draw/prim.rs");
    let panel = source("src/v2/view/draw/card.rs");
    for router in [&prim] {
        assert!(!router.contains("div()"), "router owns a GPUI builder body");
        assert!(!router.contains("BConcreteRenderer"));
        assert!(!router.contains("render_impl"));
    }
    for (file, owner) in [
        ("row.rs", "BRow"),
        ("column.rs", "BColumn"),
        ("list.rs", "BList"),
        ("table.rs", "BTableRow"),
        ("tree/mod.rs", "BTreeRow"),
    ] {
        let focused = source(&format!("src/v2/view/draw/{file}"));
        assert!(
            focused.contains(&format!("pub struct {owner}")),
            "{file} lost {owner}"
        );
        assert!(
            focused.contains("pub fn render("),
            "{file} lost inherent render coverage"
        );
    }
    // Body and Sidebar are region columns (`BCardKind::Column`): the card
    // renderer hands them to their focused renderers.
    for (module, owner) in [("body.rs", "BBody"), ("sidebar.rs", "BSidebar")] {
        let focused = source(&format!("src/v2/view/draw/{module}"));
        assert!(focused.contains(&format!("pub struct {owner}")), "{module} lost {owner}");
        assert!(panel.contains(&format!("super::{}::{owner}(&card.body).render(ctx)", module.trim_end_matches(".rs"))), "{owner} is not delegated");
    }
}

#[test]
fn task_scoped_renderers_consume_only_the_resolved_style() {
    for relative in [
        // row.rs composes; the strip it delegates to owns the style reads.
        "src/v2/view/draw/column.rs",
        "src/v2/view/draw/list.rs",
        "src/v2/view/draw/table.rs",
        "src/v2/view/draw/tree/mod.rs",
        "src/v2/view/draw/button.rs",
        "src/v2/view/draw/anchor.rs",
        "src/v2/view/draw/strip.rs",
    ] {
        let focused = source(relative);
        assert!(
            !focused.contains("METRICS"),
            "{relative} reads legacy metrics"
        );
        assert!(
            !focused.contains("ctx.p"),
            "{relative} reads the compatibility palette"
        );
        // Either the file reads the resolved style directly, or it describes
        // its containers in `BFlex` tokens and hands the render pass to
        // `.div(ctx)` — which is the same rule, held one level tighter.
        assert!(
            focused.contains("ctx.style") || focused.contains(".div(ctx)"),
            "{relative} does not consume BRenderCtx style"
        );
    }
    for file in [
        "body.rs",
        "sidebar.rs",
    ] {
        let relative = format!("src/v2/view/draw/{file}");
        let focused = source(&relative);
        assert!(
            !focused.contains("METRICS"),
            "{relative} reads legacy metrics"
        );
        assert!(
            !focused.contains("ctx.p"),
            "{relative} reads the compatibility palette"
        );
        assert!(focused.contains("ctx.style") || focused.contains("ctx.style") || focused.contains("panel_bg("));
    }
}

// ── Gate 1 characterization tests (task 6.2) ──────────────────────────────

#[test]
fn gate1_table_has_public_compatibility_wrappers() {
    // Gate 1 requires public compatibility wrappers so the dispatcher can
    // call named entry points without re-owning GPUI composition.
    let table = source("src/v2/view/draw/table.rs");
    assert!(
        table.contains("pub fn view_row("),
        "table.rs must expose pub fn view_row() compatibility wrapper"
    );
    assert!(
        table.contains("pub fn view_header("),
        "table.rs must expose pub fn view_header() compatibility wrapper"
    );
    // The wrappers must delegate to the focused structs, not re-implement.
    assert!(
        table.contains("BTableRow {"),
        "view_row must construct BTableRow"
    );
    assert!(
        table.contains("BTableHeader(cell_list)"),
        "view_header must construct BTableHeader"
    );
}

#[test]
fn gate1_bounded_scroll_ownership_is_characterized() {
    // Characterization: the outer surface div owns paint, border, radius;
    // the bounded inner scroller owns overflow_y_scroll only.
    // Applies to both BBody and BSidebar.
    for (name, path) in [
        ("BBody",    "src/v2/view/draw/body.rs"),
        ("BSidebar", "src/v2/view/draw/sidebar.rs"),
    ] {
        let src = source(path);
        // Outer surface: must own all paint properties.
        // Extract only up to the inner div to avoid reading inner properties.
        let outer_section = src.split("let outer = div()").nth(1)
            .unwrap_or_else(|| panic!("{name}: outer surface div block not found"));
        // Narrow to only the outer div initializer chain (before the inner div).
        let outer_init = outer_section.split("let mut inner = div()").next()
            .unwrap_or(outer_section);
        for prop in [
            ".bg(", ".border(px(", ".border_color(", ".rounded(",
        ] {
            assert!(
                outer_init.contains(prop),
                "{name} outer div must own `{prop}` (bounded-scroll surface ownership)"
            );
        }
        // Outer initializer must NOT own scrolling.
        assert!(
            !outer_init.contains(".overflow_y_scroll()"),
            "{name} outer div must not own overflow_y_scroll"
        );
        // Inner scroller must own the bounded constraints.
        let inner_section = src.split("let mut inner = div()").nth(1)
            .unwrap_or_else(|| panic!("{name}: inner scroll div block not found"));
        for bound in [".flex_1()", ".min_h_0()", ".min_w_0()", ".overflow_y_scroll()"] {
            assert!(
                inner_section.contains(bound),
                "{name} inner div must own `{bound}`"
            );
        }
    }
}

#[test]
fn gate1_all_focused_modules_free_of_legacy_reads() {
    // Characterization: no focused module reads METRICS or c.p palette.
    // Extends task_scoped_renderers_consume_only_the_resolved_style to
    // include all panel role files in a single characterization assertion.
    let all_focused = [
        "src/v2/view/draw/row.rs",
        "src/v2/view/draw/column.rs",
        "src/v2/view/draw/list.rs",
        "src/v2/view/draw/table.rs",
        "src/v2/view/draw/tree/mod.rs",
        "src/v2/view/draw/card.rs",
        "src/v2/view/draw/bar.rs",
        "src/v2/view/draw/sort.rs",
        "src/v2/view/draw/body.rs",
        "src/v2/view/draw/split.rs",
        "src/v2/view/draw/sidebar.rs",
    ];
    for path in all_focused {
        let src = source(path);
        assert!(!src.contains("METRICS"), "{path} reads legacy METRICS");
        assert!(!src.contains("ctx.p."), "{path} reads legacy ctx.p palette");
    }
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
