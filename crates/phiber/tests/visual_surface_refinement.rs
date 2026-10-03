//! Focused regression coverage for neutral resize gutters and Topbar chrome.
//!
//! Validates: Requirements 3.4, 3.5, 7.3, 7.5-7.8, 10.1, 11.4

use phiber::v2::{
    host::BHostType,
    state::BState,
    theme::{BPaletteName, BSurfaceTone, resolve_ui_style, ui_style_defaults},
};
use std::{fs, path::PathBuf};

fn renderer(relative: &str) -> String {
    let path = module_file(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/v2/view/draw")
        .join(relative));
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

#[test]
fn resize_gutters_share_the_neutral_workspace_surface() {
    for palette in BPaletteName::ALL.iter().copied() {
        let style = resolve_ui_style(ui_style_defaults(palette), &BState::new());
        // The workspace shows the app shade between the area cards, so a
        // seam reads as the window showing through.
        assert_eq!(style.workspace_background(), style.colors.bg_app);
        assert_eq!(
            style.resize_gutter_background(),
            style.workspace_background()
        );
    }

    // The space paints the gutter and an area card its host's body, as
    // `BFill` tokens; each token resolves from the resolved style in one place.
    // The window shade shows only between the cards: an area card on the
    // window shade was the dark Outliner and the dark activity rail.
    let space = renderer("space/mod.rs") + &renderer("space/part.rs");
    let area = renderer("area/mod.rs");
    let tokens = renderer("flex/fill.rs");
    assert!(space.contains(".fill(BFill::Gutter)"));
    assert!(tokens.contains("Self::Gutter => to_hsla(ctx.style.resize_gutter_background()).into()"));
    assert!(area.contains(".fill(BFill::HostBody)"));
    assert!(!area.contains("BFill::Workspace") && !tokens.contains("Workspace,"), "no surface but the seams takes the window shade");
    // A host body is a swept surface now, not a flat colour — the token
    // still resolves in this one place, which is what this pins.
    assert!(tokens.contains("Self::HostBody => resolved_gradient(ctx.style.host_body_surface(ctx.host))"));
    assert!(!space.contains("bg_app"));
}

#[test]
fn resize_hit_bands_stay_wide_while_the_seam_sits_inside_the_gap_at_rest() {
    for palette in BPaletteName::ALL.iter().copied() {
        let style = resolve_ui_style(ui_style_defaults(palette), &BState::new());
        // The seam is the gap less 3 px at rest and the gap plus 2 px under
        // the pointer — one rule for the zone, area and panel seams.
        assert_eq!(style.resize.rest_thickness(), (style.resize.gap - 3.0).max(1.0));
        assert_eq!(style.resize.hover_thickness(), style.resize.gap + 2.0);
        assert_eq!(style.resize.hit_width, 14.0);
        assert!(style.resize.hit_width >= style.resize.hover_thickness());
        // The resting shade is still the theme's border, but the seam no
        // longer paints it: each area draws its own border either side of
        // the gap, so the seam appears only under the pointer (`resize.rs`).
        assert_eq!(style.resize_seam_resting_color(), style.colors.border);
        assert_eq!(style.resize_seam_active_color(), style.colors.accent);
    }

    // Decomposition moved the concrete seam builders out of the workspace
    // and zone composition routers. Audit the focused owners so this remains
    // a post-move regression rather than accidentally checking stale wrappers.
    // The hit band and its inset are one shared geometry now, so a seam
    // module is audited together with the overlay it delegates to.
    let overlay = renderer("resize.rs");
    assert!(
        overlay.contains("ctx.style.resize.hit_width"),
        "the shared seam overlay lost the resolved hit width"
    );
    assert!(
        overlay.contains("div().absolute()"),
        "the shared seam overlay is no longer zero-footprint"
    );
    for (name, relative) in [
        ("zone boundary", "space/seam.rs"),
        ("area boundary", "zone/seam.rs"),
        ("panel edge", "resize.rs"),
    ] {
        let source = renderer(relative);
        assert!(
            source.contains("BSeamLine {") || source.contains("impl BSeamLine"),
            "{name} does not draw the one seam line"
        );
        assert!(source.contains(".on_drag("), "{name} lost drag initiation");
        assert!(
            source.contains("CursorStyle::Resize"),
            "{name} lost resize cursors"
        );
        assert!(source.contains("group: group.clone()"), "{name} lost hover feedback: the handle joins the seam line's hover group");
        if relative == "resize.rs" {
            // A panel edge clamps to its own width range instead of the
            // area-boundary snap model, so it carries no snap threshold.
            assert!(
                source.contains("min: ctx.style.dimensions.sidebar_min_width"),
                "{name} drag payload lost its resolved minimum width"
            );
            continue;
        }
        assert!(
            source.contains("snap_threshold: ctx.style.resize.snap_threshold"),
            "{name} drag payload lost the resolved snap threshold"
        );
        assert!(
            source.contains("min_extent: ctx.style.resize.min_extent"),
            "{name} drag payload lost the resolved minimum extent"
        );
    }
}

#[test]
fn topbar_chrome_and_main_tabs_use_the_requested_rest_hover_active_mapping() {
    for palette in BPaletteName::ALL.iter().copied() {
        let style = resolve_ui_style(ui_style_defaults(palette), &BState::new());
        // The top bar is the chrome shade, opaque, flat; its items hover and
        // activate on the same two shades every host uses.
        let resting = style.resting_host_background(BHostType::Topbar);
        let header = style.host_header_surface(BHostType::Topbar);
        let hover = style.host_item_hover(BHostType::Topbar);
        let active = style.host_item_active_gradient(BHostType::Topbar);
        assert_eq!(resting, style.colors.bg_top);
        assert_eq!(resting.3, 1.0);
        assert!(!header.enabled);
        assert_eq!(header.bottom, style.colors.bg_top);
        assert_eq!(hover, style.colors.bg_hover);
        // Inward is lighter in a dark theme and darker in a light one.
        let inward = |from: f32, to: f32| if style.colors.ladder.light { to < from } else { to > from };
        assert!(inward(resting.2, hover.2), "{palette:?}");
        assert_eq!(active.bottom, style.colors.bg_active);
        assert!(inward(hover.2, active.bottom.2), "{palette:?}");
    }
}

#[test]
fn topbar_zone_controls_use_resolved_rounded_stable_geometry() {
    // The button is square at the button module, rounded as every button is,
    // and its group is pushed to the strip's end — all of it stated as
    // `BZoneTogglePart` rather than measured at the call site.
    let toggles = renderer("toggle.rs");
    assert!(toggles.contains("let size = BLen::Dim(BDim::ButtonHeight);"));
    assert!(toggles.contains(".width(size)") && toggles.contains(".height(size)"));
    assert!(toggles.contains("BRadius::Role(BStyleRole::Button)"));
    assert!(toggles.contains("BJustify::End"));
    assert!(toggles.contains("topbar-zone-toggle-group"));
    assert!(toggles.contains("topbar-zone-toggle-"));
    assert!(toggles.contains(".debug_selector("));

    // Hover paints and never measures. That used to need checking by reading
    // the hover closure for a stray `.w(` / `.h(`; a hover is a `BFill` now,
    // so there is no geometry it could set.
    assert!(toggles.contains(".hovered(BFill::ItemHover)"));
    assert!(!toggles.contains(".hover("), "a hover is a fill token, not a closure that could resize");
}

#[test]
fn zone_assets_have_exact_directional_thirds_and_outline_only_inactive_forms() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/icons");
    for (slug, boundary, active_path, inactive_path) in [
        ("left", "5.6667", "M3 1H5.6667V15", "M5.6667 1.75V14.25"),
        ("right", "10.3333", "M10.3333 1H13", "M10.3333 1.75V14.25"),
        ("bottom", "10.3333", "M1 10.3333H15", "M1.75 10.3333H14.25"),
    ] {
        let active = fs::read_to_string(root.join(format!("zone_{slug}.svg"))).unwrap();
        let inactive = fs::read_to_string(root.join(format!("zone_{slug}_outline.svg"))).unwrap();
        assert!(active.contains(boundary));
        assert!(active.contains(active_path));
        assert!(active.contains("fill=\"currentColor\""));
        assert!(inactive.contains(inactive_path));
        assert!(!inactive.contains("fill=\"currentColor\""));
        assert!(inactive.contains("stroke=\"currentColor\""));
    }
}

#[test]
fn properties_rail_tab_uses_one_inset_paint_layer_for_active_and_hover() {
    for palette in BPaletteName::ALL.iter().copied() {
        let style = resolve_ui_style(ui_style_defaults(palette), &BState::new());
        let inset = style.properties_tab_background_inset();
        let overlap = style.properties_tab_overlap();
        assert_eq!(inset, style.strokes.border * 2.0, "{palette:?}");
        assert_eq!(inset, 2.0, "default rail paint inset changed");
        assert!(overlap > 0.0, "{palette:?}");
        assert_eq!(overlap, style.spacing.gap_xs.max(style.strokes.border));
        assert!(overlap <= style.spacing.gap_sm, "{palette:?}");
    }

    // Which cap this is, is one named question (`tab/shape.rs`), and the
    // Properties tab is the one that paints on its own layer.
    let shape = renderer("tab/shape.rs");
    assert!(shape.contains("(false, _) if host == BHostType::Buts => Self::PropertiesTab"));
    assert!(shape.contains("matches!(self, Self::PropertiesTab)"), "only the Properties tab paints separately");

    let tab = renderer("tab/mod.rs");
    let properties = tab
        .split("fn properties_layers(")
        .nth(1)
        .expect("Properties rail paint layer");
    for geometry in [
        "ctx.style.properties_tab_background_inset()",
        "ctx.style.properties_tab_overlap()",
        "side.round(paint, r)",
    ] {
        assert!(properties.contains(geometry), "paint layer lost {geometry}");
    }
    // The layer spans the cap top to bottom; its left and right come from
    // `BCapSide::paint`.
    assert!(tab.contains("BFlex::row().place(BPlace::Overlay)"), "the paint layer covers its cap");

    // The per-side arms live in the cap module, so a right-hand rail mirrors
    // the left-hand shape instead of repeating it.
    let cap = renderer("tab/cap.rs");
    for arm in [
        "BCapSide::Below => element.rounded_t(r)",
        "BCapSide::Right => element.rounded_l(r)",
        "BCapSide::Left => element.rounded_r(r)",
        "BCapSide::Below => element.self_end()",
        "BCapSide::Right => element.ml(px(gap))",
        "BCapSide::Left => element.mr(px(gap))",
        "BCapSide::Right => element.left(px(inset)).right(px(-overlap))",
        "BCapSide::Left => element.right(px(inset)).left(px(-overlap))",
    ] {
        assert!(cap.contains(arm), "cap geometry lost {arm}");
    }
    let state_only = properties
        .split("paint = if active {")
        .nth(1)
        .expect("active/hover paint switch");
    assert!(state_only.contains("paint.bg(resolved_gradient"));
    assert!(state_only.contains("paint.group_hover"));
    for forbidden_layout_change in [".left(", ".right(", ".w(", ".mx(", ".px(", ".border_l("] {
        assert!(
            !state_only.contains(forbidden_layout_change),
            "state branch changes layout with {forbidden_layout_change}"
        );
    }
    assert!(!properties.contains("colors.accent"));
    // The paint goes under the row, so state paints without moving the icon.
    assert!(properties.contains(".child(paint)\n        .child(row)"));
    assert!(!tab.contains(".left(px(ctx.style.properties_tab_overlap()))"));
    // A Properties tab's rest and hover live on its paint layer, so the cap
    // itself takes no `chrome::paint` fill — that is what keeps the two from
    // painting over each other.
    assert!(tab.contains("if !shape.paints_separately() && !active"));
}

#[test]
fn view3d_uses_a_lighter_opaque_flat_semantic_surface_across_palettes() {
    for palette in BPaletteName::ALL.iter().copied() {
        let style = resolve_ui_style(ui_style_defaults(palette), &BState::new());
        let base = style.colors.bg_viewport;
        let viewport = style.viewport_surface();
        let semantic = style.surface(BSurfaceTone::Viewport);
        // Inward of the window shade: lighter in a dark theme, darker in a light one.
        let lift = if style.colors.ladder.light { style.colors.bg_app.2 - viewport.2 } else { viewport.2 - style.colors.bg_app.2 };

        assert_eq!((viewport.0, viewport.1), (base.0, base.1), "{palette:?}");
        assert_eq!(viewport.3, 1.0, "{palette:?}");
        assert_eq!(semantic.top, viewport, "{palette:?}");
        assert_eq!(semantic.bottom, viewport, "{palette:?}");
        assert!(!semantic.enabled, "{palette:?}");
        assert!(
            (viewport.2 - (base.2 + style.effects.viewport_lightness_delta * 100.0).min(100.0))
                .abs()
                < 1.0e-5,
            "{palette:?}"
        );
        // The canvas sits clearly above the app chrome, but it is the
        // palette's own viewport tone — no lift on top. Brightening it washed
        // the floor grid and the axes out; the grid's alpha carries that
        // separation instead (`viewport_grid_alpha`).
        assert!((12.0..=55.0).contains(&lift), "{palette:?}: lift={lift}");
        assert!(
            (style.colors.text.2 - viewport.2).abs() >= 25.0,
            "{palette:?} viewport/text contrast became unreadable"
        );
        let grid = style.viewport_grid_stroke();
        // Two steps of the ladder: a tenth of the seed's lightness each, so a
        // near-black theme (Onyx, seed L 16.5) separates its grid by ~3.3.
        assert!((grid.color.2 - viewport.2).abs() >= 3.0, "{palette:?}: {}", (grid.color.2 - viewport.2).abs());
        assert_eq!(grid.color, style.anim().grid_line, "{palette:?}: the floor draws the UV editor's grid line");
    }

    let view3d = renderer("canvas/view3d/mod.rs");
    assert!(view3d.contains("ctx.style.viewport_surface()"), "the tracer's sky is the canvas shade");
    assert!(view3d.contains("ctx.style.viewport_gradient()"), "under the tracer, the canvas sweep");
    assert!(view3d.contains("viewport_grid_stroke()"));
    // The floor's axes are drawn with the overlays (`view3d/guide`).
    assert!(renderer("canvas/view3d/guide/mod.rs").contains("axis_stroke("));
    assert!(!view3d.contains("linear_gradient("));
    assert!(!view3d.contains("linear_color_stop("));
    assert!(!view3d.contains("Horizon band"));
}

#[test]
fn every_menu_row_reserves_its_icon_slot_so_labels_share_one_left_edge() {
    // Blender reserves icon space on every row of a menu, so a row without an
    // icon does not pull its label left of its neighbours'. Ours used to add
    // the icon only when it had one, which put one menu's labels on two
    // different left edges.
    let item = renderer("menu.rs");
    let slot = item.find("let icon = BMenuPart::IconSlot.flex(ctx).div(ctx);").expect("the slot is built unconditionally");
    let branch = item.find("if ic.is_empty() {").expect("the icon is optional, the slot is not");
    assert!(slot < branch, "the slot must exist before the icon decision, not inside it");
    assert!(item.contains(".text_left()"), "labels read from the slot's edge");
    assert!(
        !item.contains("if !ic.is_empty() {\n        row = row.child(\n            svg()"),
        "the icon must not be the thing that decides whether the slot exists"
    );
}

#[test]
fn the_activity_bar_takes_the_toolbars_treatment() {
    // The activity bar and the toolbar are both columns of free buttons, and
    // both take their rest / hover / active fill from the one control mapping
    // (`chrome::paint`). Both now rest on the control shade: a button looks
    // like a button wherever it is, docked or floating, rather than being a
    // tile on the canvas and a bare glyph in a rail. They still part under
    // the pointer — floating takes the hover shade, docked washes the accent
    // over its rail at an alpha. The rail renderer hands the activity kind to
    // the free-button cap rather than the activity part inventing a
    // treatment.
    let rail = renderer("rail.rs");
    assert!(rail.contains("if kind == BRailKind::Activity {"));
    assert!(rail.contains("return tab::rail_button(&ctl, ctx);"));
    let tab = renderer("tab/mod.rs");
    assert!(tab.contains("pub fn rail_button(ctl: &BControl, ctx: &BRenderCtx) -> AnyElement {"));
    assert!(tab.contains("cap_view(ctl, BCapSide::Free, ctx)"));
    let semantic = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/v2/theme/semantic/set.rs"),
    )
    .expect("semantic style");
    // Both rest on the control shade; only what happens under the pointer
    // separates them.
    assert!(semantic.contains("BControlState::Rest => self.colors.bg_control,"));
    assert!(semantic.contains("BControlState::Active => self.colors.accent,"));
    assert!(semantic.contains("BControlState::Rest => return Some(self.colors.bg_control)"));
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
