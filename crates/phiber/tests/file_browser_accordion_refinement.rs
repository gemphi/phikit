//! Focused File Browser table and Properties accordion regressions.
//!
//! Validates: Requirements 6.1-6.6, 10.1, 11.3, 12.7, 13.8

use phiber::v2::{
    host::BHostType,
    part::{
        custom::file::{BFileKind, BFileRow, BFileSort, keys, listing, table},
        panel::{BPanel, BPanelItem, BPropSpec, BSectionSpec},
    },
    state::{BState, V_FALSE, action::apply_action},
    view::{
        BAction, BColAlign, BColWidth, BPanelRole, BRowShade, BSortDirection, BView, BViewItem,
    },
};
use std::{fs, path::PathBuf};

fn source(relative: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
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
    let BView::TableHeader { cells } = header else {
        panic!("shared File Browser header must use TableHeader")
    };
    cells
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
        cells,
        shade,
        action,
    } = table::row(&row, "")
    else {
        panic!("file entries must use TableRow")
    };
    assert_eq!(shade, BRowShade::Flat);
    assert_eq!(
        action,
        Some(BAction::Set(keys::k_selected(), row.path.clone()))
    );
    assert_eq!(cells.len(), 3);

    let BView::TableRow { shade, .. } = table::row(&row, &row.path) else {
        panic!()
    };
    assert_eq!(shade, BRowShade::Selected);

    // After the prim.rs → prim/mod.rs split, table row logic lives in prim/table.rs.
    let renderer = source("src/v2/view/render/gpui/prim/table.rs");
    let table_row = renderer
        .split("pub fn view_row(")
        .nth(1)
        .unwrap_or_else(|| renderer.as_str())
        .split("pub fn view_header(")
        .next()
        .unwrap_or_else(|| renderer.as_str());
    assert!(table_row.contains("selected.then_some(c.style.colors.row_selected)"));
    assert!(!table_row.contains("BRowShade::Alt"));
    assert!(!table_row.contains("METRICS"));
    assert!(!table_row.contains("c.p"));
    assert!(table_row.contains(".id(id)"));
    assert!(table_row.contains(".on_click("));
}

#[test]
fn shared_header_and_body_keep_flexible_name_and_fixed_metadata_geometry() {
    let state = BState::new();
    let header = header_cells(&state);
    let BView::TableRow { cells: body, .. } = table::row(&file("wide-name.txt", 4, 5), "") else {
        panic!()
    };
    assert_eq!(header.len(), 3);
    assert_eq!(body.len(), 3);
    assert!(matches!(header[0].width, BColWidth::Flex));
    assert!(matches!(body[0].width, BColWidth::Flex));
    for cells in [header, body.as_slice()] {
        assert_eq!(cells[0].align, BColAlign::Start);
        assert_eq!(cells[1].width, BColWidth::FileDate);
        assert_eq!(cells[1].align, BColAlign::Start);
        assert_eq!(cells[2].width, BColWidth::FileSize);
        assert_eq!(cells[2].align, BColAlign::End);
    }

    // After the prim.rs → prim/mod.rs split, table cell/row logic lives in prim/table.rs.
    let renderer = source("src/v2/view/render/gpui/prim/table.rs");
    assert!(renderer.contains("BColWidth::Flex => element.flex_1().min_w_0()"));
    assert!(renderer.contains(".gap(px(c.style.spacing.gap_md))"));
    assert!(renderer.contains("c.style.dimensions.file_date_column_width"));
    assert!(renderer.contains("c.style.dimensions.file_size_column_width"));
}

#[test]
fn all_headers_sort_and_repeated_clicks_toggle_a_stable_direction_slot() {
    let mut state = BState::new();
    let initial = header_cells(&state);
    let (name, direction, name_action) = sort_header(&initial[0]);
    assert_eq!((name, direction), ("Name", Some(BSortDirection::Ascending)));
    assert_eq!(name_action, &BAction::Toggle(keys::k_sort_invert()));
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
            &BAction::Set(keys::k_sort(), expected.label().to_string())
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

    let focused = source("src/v2/view/render/gpui/prim/sort.rs");
    assert!(focused.contains("let mut indicator = div()"));
    assert!(focused.contains(".size(px(indicator_size))"));
    assert!(focused.contains("if let Some(direction) = self.direction"));
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
    let spec = BSectionSpec {
        title: "Surface",
        props: EMPTY_PROPS,
    };
    let BView::Panel {
        role: BPanelRole::Card,
        children,
    } = BPanelItem::new(BHostType::Buts, "Material", spec, true).build(&BState::new())
    else {
        panic!("section must retain its outer card")
    };
    let [
        BView::AccordionHeader {
            title,
            expanded: true,
            action: BAction::Set(path, value),
        },
    ] = children.as_slice()
    else {
        panic!("one focused heading must own the sole toggle action")
    };
    assert_eq!(title, "Surface");
    assert_eq!(path.encode(), "ctx.buts.material.surface.expanded");
    assert_eq!(value, V_FALSE);

    let focused = source("src/v2/view/render/gpui/prim/accordion.rs");
    assert_eq!(focused.matches(".on_click(").count(), 1);
    assert!(focused.contains(".w_full()"));
    assert!(focused.contains("accordion-header-"));
    assert!(focused.contains("accordion-disclosure-"));
}

#[test]
fn compact_disclosure_geometry_is_resolved_stable_and_non_nested() {
    let focused = source("src/v2/view/render/gpui/prim/accordion.rs");
    assert!(focused.contains(".button_height"));
    assert!(focused.contains("dimensions.header_height - 2.0 * c.style.spacing.pad_xs"));
    assert!(focused.contains("radius_for(BStyleRole::Button)"));
    assert!(focused.contains(".w(px(chip_size))"));
    assert!(focused.contains(".h(px(chip_size))"));
    assert!(focused.contains(".text_size(px(c.style.typography.body))"));
    assert!(focused.contains("colors.bg_hover"));
    assert!(focused.contains("colors.bg_active"));
    let state_branch = focused
        .split("let chip_fill = if self.expanded")
        .nth(1)
        .unwrap();
    let state_branch = state_branch.split("let hover_fill").next().unwrap();
    assert!(!state_branch.contains(".w("));
    assert!(!state_branch.contains(".h("));
}

#[test]
fn properties_keep_transparent_rows_outer_border_and_bounded_scroll_owner() {
    let BView::Panel {
        role: BPanelRole::Body,
        ..
    } = BPanel::new(BHostType::Buts).build(&BState::new())
    else {
        panic!("Properties content must be owned by a bounded Body scroller")
    };

    // After the prim.rs → prim/mod.rs split, panel role logic lives in prim/panel/.
    // Section (transparent Properties row) is in prim/panel/section.rs.
    let section = source("src/v2/view/render/gpui/prim/panel/section.rs");
    assert!(section.contains("if c.host != crate::v2::host::BHostType::Buts"));

    // Card borders/rounding are in prim/panel/card.rs.
    let card = source("src/v2/view/render/gpui/prim/panel/card.rs");
    assert!(card.contains(".border(px(c.style.strokes.border))"));
    assert!(card.contains(".rounded("));
    assert!(card.contains("panel_bg(BPanelRole::Card, c)"));
    assert!(!card.contains(".overflow_hidden()"));

    // Body scroll ownership is in prim/panel/body.rs.
    let body = source("src/v2/view/render/gpui/prim/panel/body.rs");
    for required in [
        ".flex_1()",
        ".w_full()",
        ".min_h_0()",
        ".min_w_0()",
        ".overflow_y_scroll()",
        "let outer_padding = c.style.spacing.pad_sm;",
        ".p(px(outer_padding))",
        "panel_bg(BPanelRole::Body, c)",
    ] {
        assert!(body.contains(required), "Body lost {required}");
    }
}

#[test]
fn bounded_content_keeps_paint_on_outer_surfaces() {
    for (name, relative, role) in [
        (
            "Body",
            "src/v2/view/render/gpui/prim/panel/body.rs",
            "BPanelRole::Body",
        ),
        (
            "Sidebar",
            "src/v2/view/render/gpui/prim/panel/sidebar.rs",
            "BPanelRole::Sidebar",
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
            format!("panel_bg({role}, c)"),
            ".border(px(c.style.strokes.border))".to_string(),
            ".border_color(to_hsla(c.style.colors.border))".to_string(),
            ".rounded(px(c.style.radius_for(BStyleRole::Panel)))".to_string(),
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

    let area = source("src/v2/view/render/gpui/area.rs");
    let area_surface = area
        .split("fn card(")
        .nth(1)
        .expect("area surface")
        .split("pub fn view(")
        .next()
        .expect("area surface end");
    for paint in [".bg(", ".border(", ".border_color(", ".rounded("] {
        assert!(area_surface.contains(paint), "area outer lost {paint}");
    }
    assert!(!area_surface.contains("overflow_y_scroll"));
    for bound in [
        ".flex_1()",
        ".min_h_0()",
        ".min_w_0()",
        ".overflow_y_scroll()",
    ] {
        assert!(area.contains(bound), "area inner lost {bound}");
    }

    let card = source("src/v2/view/render/gpui/prim/panel/card.rs");
    for paint in [
        "panel_bg(BPanelRole::Card, c)",
        ".border(px(c.style.strokes.border))",
        ".rounded(",
    ] {
        assert!(card.contains(paint), "card outer lost {paint}");
    }
    assert!(!card.contains(".overflow_y_scroll()"));

    for relative in [
        "src/v2/view/render/gpui/prim/row.rs",
        "src/v2/view/render/gpui/prim/column.rs",
    ] {
        let renderer = source(relative);
        assert!(
            renderer.contains(".min_h_0()"),
            "{relative} lost height bound"
        );
        assert!(
            renderer.contains(".min_w_0()"),
            "{relative} lost width bound"
        );
    }
}

#[test]
fn focused_renderers_keep_named_ownership_and_delegation_only_routers() {
    let prim = source("src/v2/view/render/gpui/prim/mod.rs");
    let panel = source("src/v2/view/render/gpui/prim/panel/mod.rs");
    for router in [&prim, &panel] {
        assert!(!router.contains("div()"), "router owns a GPUI builder body");
        assert!(!router.contains("BConcreteRenderer"));
        assert!(!router.contains("render_impl"));
    }
    for (file, owner) in [
        ("row.rs", "BRow"),
        ("column.rs", "BColumn"),
        ("list.rs", "BList"),
        ("table.rs", "BTableRow"),
        ("tree.rs", "BTreeRow"),
    ] {
        let focused = source(&format!("src/v2/view/render/gpui/prim/{file}"));
        assert!(
            focused.contains(&format!("pub struct {owner}")),
            "{file} lost {owner}"
        );
        assert!(
            focused.contains("pub fn render("),
            "{file} lost inherent render coverage"
        );
    }
    for (role, module, owner) in [
        ("Section", "section.rs", "BSection"),
        ("CardHeader", "heading.rs", "BCardHeader"),
        ("Card", "card.rs", "BCard"),
        ("Header", "header.rs", "BHeader"),
        ("Toolbar", "toolbar.rs", "BToolbar"),
        ("Sidebar", "sidebar.rs", "BSidebar"),
        ("Footer", "footer.rs", "BFooter"),
        ("Body", "body.rs", "BBody"),
        ("Viewport", "viewport.rs", "BViewport"),
        ("Modal", "modal.rs", "BModal"),
    ] {
        let focused = source(&format!("src/v2/view/render/gpui/prim/panel/{module}"));
        assert!(
            focused.contains(&format!("pub struct {owner}")),
            "{role} lost {owner}"
        );
        assert!(
            panel.contains(&format!("BPanelRole::{role} =>")),
            "{role} is not delegated"
        );
    }
}

#[test]
fn task_scoped_renderers_consume_only_the_resolved_style() {
    for relative in [
        "src/v2/view/render/gpui/prim/row.rs",
        "src/v2/view/render/gpui/prim/column.rs",
        "src/v2/view/render/gpui/prim/list.rs",
        "src/v2/view/render/gpui/prim/table.rs",
        "src/v2/view/render/gpui/prim/tree.rs",
        "src/v2/view/render/gpui/atom/button.rs",
        "src/v2/view/render/gpui/atom/anchor.rs",
    ] {
        let focused = source(relative);
        assert!(
            !focused.contains("METRICS"),
            "{relative} reads legacy metrics"
        );
        assert!(
            !focused.contains("c.p"),
            "{relative} reads the compatibility palette"
        );
        assert!(
            focused.contains("c.style"),
            "{relative} does not consume BRenderCtx style"
        );
    }
    for file in [
        "body.rs",
        "card.rs",
        "footer.rs",
        "header.rs",
        "heading.rs",
        "modal.rs",
        "section.rs",
        "sidebar.rs",
        "toolbar.rs",
        "viewport.rs",
    ] {
        let relative = format!("src/v2/view/render/gpui/prim/panel/{file}");
        let focused = source(&relative);
        assert!(
            !focused.contains("METRICS"),
            "{relative} reads legacy metrics"
        );
        assert!(
            !focused.contains("c.p"),
            "{relative} reads the compatibility palette"
        );
        assert!(focused.contains("c.style") || focused.contains("panel_bg("));
    }
}

// ── Gate 1 characterization tests (task 6.2) ──────────────────────────────

#[test]
fn gate1_contract_module_exists_and_trait_is_not_in_routers() {
    // Gate 1 requires BFocusedRenderer to live in focused modules only,
    // never in the thin-routing dispatchers. The existing router ban test
    // covers mod.rs and panel/mod.rs; this test verifies the trait lives in
    // the render_contract module instead.
    let contract = source("src/v2/view/render/gpui/prim/render_contract.rs");
    assert!(
        contract.contains("pub trait BFocusedRenderer"),
        "render_contract.rs must define BFocusedRenderer trait"
    );
    assert!(
        contract.contains("fn render(&self, c: &BRenderCtx) -> AnyElement"),
        "BFocusedRenderer must declare render(&self, c) -> AnyElement"
    );

    // Routers must NOT contain the trait name (enforced alongside existing bans).
    let prim = source("src/v2/view/render/gpui/prim/mod.rs");
    let panel = source("src/v2/view/render/gpui/prim/panel/mod.rs");
    assert!(!prim.contains("BFocusedRenderer"), "prim/mod.rs must not name BFocusedRenderer");
    assert!(!panel.contains("BFocusedRenderer"), "panel/mod.rs must not name BFocusedRenderer");
}

#[test]
fn gate1_every_focused_struct_implements_bfocusedrenderer() {
    // Verify that every focused module contains an `impl BFocusedRenderer for`
    // block — proving compile-time trait coverage for all 18 named structs.
    for (file, owner) in [
        ("src/v2/view/render/gpui/prim/row.rs",       "BRow"),
        ("src/v2/view/render/gpui/prim/column.rs",    "BColumn"),
        ("src/v2/view/render/gpui/prim/list.rs",      "BList"),
        ("src/v2/view/render/gpui/prim/table.rs",     "BTableRow"),
        ("src/v2/view/render/gpui/prim/table.rs",     "BTableHeader"),
        ("src/v2/view/render/gpui/prim/tree.rs",      "BTreeRow"),
        ("src/v2/view/render/gpui/prim/tree.rs",      "BTreeIndent"),
        ("src/v2/view/render/gpui/prim/accordion.rs", "BAccordion"),
        ("src/v2/view/render/gpui/prim/sort.rs",      "BSort"),
    ] {
        let focused = source(file);
        assert!(
            focused.contains(&format!("impl BFocusedRenderer for {owner}")),
            "{file} missing `impl BFocusedRenderer for {owner}`"
        );
    }
    for (module, owner) in [
        ("body.rs",    "BBody"),
        ("card.rs",    "BCard"),
        ("footer.rs",  "BFooter"),
        ("header.rs",  "BHeader"),
        ("heading.rs", "BCardHeader"),
        ("modal.rs",   "BModal"),
        ("section.rs", "BSection"),
        ("sidebar.rs", "BSidebar"),
        ("toolbar.rs", "BToolbar"),
        ("viewport.rs","BViewport"),
    ] {
        let focused = source(&format!("src/v2/view/render/gpui/prim/panel/{module}"));
        assert!(
            focused.contains(&format!("impl BFocusedRenderer for {owner}")),
            "panel/{module} missing `impl BFocusedRenderer for {owner}`"
        );
    }
}

#[test]
fn gate1_table_has_public_compatibility_wrappers() {
    // Gate 1 requires public compatibility wrappers so the dispatcher can
    // call named entry points without re-owning GPUI composition.
    let table = source("src/v2/view/render/gpui/prim/table.rs");
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
        table.contains("BTableHeader(cells)"),
        "view_header must construct BTableHeader"
    );
}

#[test]
fn gate1_bounded_scroll_ownership_is_characterized() {
    // Characterization: the outer surface div owns paint, border, radius;
    // the bounded inner scroller owns overflow_y_scroll only.
    // Applies to both BBody and BSidebar.
    for (name, path) in [
        ("BBody",    "src/v2/view/render/gpui/prim/panel/body.rs"),
        ("BSidebar", "src/v2/view/render/gpui/prim/panel/sidebar.rs"),
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
        "src/v2/view/render/gpui/prim/row.rs",
        "src/v2/view/render/gpui/prim/column.rs",
        "src/v2/view/render/gpui/prim/list.rs",
        "src/v2/view/render/gpui/prim/table.rs",
        "src/v2/view/render/gpui/prim/tree.rs",
        "src/v2/view/render/gpui/prim/accordion.rs",
        "src/v2/view/render/gpui/prim/sort.rs",
        "src/v2/view/render/gpui/prim/panel/body.rs",
        "src/v2/view/render/gpui/prim/panel/card.rs",
        "src/v2/view/render/gpui/prim/panel/footer.rs",
        "src/v2/view/render/gpui/prim/panel/header.rs",
        "src/v2/view/render/gpui/prim/panel/heading.rs",
        "src/v2/view/render/gpui/prim/panel/modal.rs",
        "src/v2/view/render/gpui/prim/panel/section.rs",
        "src/v2/view/render/gpui/prim/panel/sidebar.rs",
        "src/v2/view/render/gpui/prim/panel/toolbar.rs",
        "src/v2/view/render/gpui/prim/panel/viewport.rs",
    ];
    for path in all_focused {
        let src = source(path);
        assert!(!src.contains("METRICS"), "{path} reads legacy METRICS");
        assert!(!src.contains("c.p."), "{path} reads legacy c.p palette");
    }
}
