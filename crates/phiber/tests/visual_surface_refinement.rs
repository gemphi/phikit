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
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/v2/view/render/gpui")
        .join(relative);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

#[test]
fn resize_gutters_share_the_neutral_workspace_surface() {
    for palette in BPaletteName::ALL.iter().copied() {
        let style = resolve_ui_style(ui_style_defaults(palette), &BState::new());
        assert_eq!(style.workspace_background(), style.colors.bg_panel);
        assert_eq!(
            style.resize_gutter_background(),
            style.workspace_background()
        );
    }

    let space = renderer("space.rs");
    let area = renderer("area.rs");
    assert!(space.contains("c.style.resize_gutter_background()"));
    assert!(area.contains("c.style.workspace_background()"));
    assert!(!space.contains(".bg(to_hsla(c.style.colors.bg_app))"));
}

#[test]
fn resize_hit_bands_stay_wide_while_resting_lines_are_fully_transparent() {
    for palette in BPaletteName::ALL.iter().copied() {
        let style = resolve_ui_style(ui_style_defaults(palette), &BState::new());
        let resting = style.resize_seam_resting_color();
        assert_eq!(style.resize.visible_seam, 2.0);
        assert_eq!(style.resize.hit_width, 14.0);
        assert!(style.resize.visible_seam > style.strokes.border);
        assert!(style.resize.hit_width >= style.resize.visible_seam);
        assert_eq!(
            (resting.0, resting.1, resting.2),
            (
                style.colors.border.0,
                style.colors.border.1,
                style.colors.border.2
            )
        );
        assert_eq!(resting.3, 0.0, "resting resize line must be invisible");
        assert_eq!(style.resize_seam_active_color(), style.colors.accent);
    }

    // Decomposition moved the concrete seam builders out of the workspace
    // and zone composition routers. Audit the focused owners so this remains
    // a post-move regression rather than accidentally checking stale wrappers.
    for (name, relative) in [
        ("zone boundary", "space/seam.rs"),
        ("area boundary", "zone/seam.rs"),
    ] {
        let source = renderer(relative);
        assert!(
            source.contains("c.style.resize.hit_width"),
            "{name} lost resolved hit width"
        );
        assert!(
            source.contains("c.style.resize.visible_seam"),
            "{name} lost resolved seam"
        );
        assert!(
            source.contains("resize_seam_resting_color()"),
            "{name} lost transparent resting line"
        );
        assert!(
            source.contains("resize_seam_active_color()"),
            "{name} lost semantic accent feedback"
        );
        assert!(
            source.contains("div().absolute()"),
            "{name} hit band is no longer zero-footprint"
        );
        assert!(source.contains(".on_drag("), "{name} lost drag initiation");
        assert!(
            source.contains("CursorStyle::Resize"),
            "{name} lost resize cursors"
        );
        assert!(source.contains("group_hover"), "{name} lost hover feedback");
        assert!(
            source.contains("snap_threshold: c.style.resize.snap_threshold"),
            "{name} drag payload lost the resolved snap threshold"
        );
        assert!(
            source.contains("min_extent: c.style.resize.min_extent"),
            "{name} drag payload lost the resolved minimum extent"
        );
    }
}

#[test]
fn topbar_chrome_and_main_tabs_use_the_requested_rest_hover_active_mapping() {
    for palette in BPaletteName::ALL.iter().copied() {
        let style = resolve_ui_style(ui_style_defaults(palette), &BState::new());
        let neutral = style.workspace_background();
        let resting = style.resting_host_background(BHostType::Topbar);
        let header = style.host_header_surface(BHostType::Topbar);
        let hover = style.host_item_hover(BHostType::Topbar);
        let active = style.host_item_active_gradient(BHostType::Topbar);

        assert_eq!(
            (resting.0, resting.1, resting.2),
            (neutral.0, neutral.1, neutral.2)
        );
        assert_eq!(resting.3, 0.0);
        assert!(!header.enabled);
        assert_eq!(header.bottom.3, 0.0);
        assert_eq!((hover.0, hover.1), (neutral.0, neutral.1));
        assert!(hover.2 > neutral.2);
        assert_eq!(hover.3, 1.0);
        assert_eq!(active.bottom, hover);
        assert!(active.top.2 > neutral.2);
        assert_eq!(style.tab_hover(BHostType::Topbar), hover);
        assert_eq!(style.tab_active_gradient(BHostType::Topbar), active);

        assert_eq!(style.resting_host_background(BHostType::View3D).3, 1.0);
        assert_eq!(style.host_header_surface(BHostType::View3D).bottom.3, 1.0);
    }

    let host = renderer("host.rs");
    let region = renderer("region.rs");
    let tab = renderer("atom/tab.rs");
    let anchor = renderer("atom/anchor.rs");
    let item = renderer("atom/menu.rs");
    assert!(
        !host.contains(".bg("),
        "host wrapper must remain transparent"
    );
    assert!(
        !host.contains(".gap("),
        "host bands must remain directly adjacent"
    );
    assert!(region.contains("host_header_surface(c.host)"));
    assert!(region.contains("_ => gpui::transparent_black().into()"));
    assert!(tab.contains("let topbar_main_tab = c.host == BHostType::Topbar && !icon_only;"));
    assert!(tab.contains("} else if topbar_main_tab {"));
    assert!(tab.contains(".bg(hover_fill)"));
    assert!(tab.contains("s.bg(active_fill.clone())"));
    assert!(tab.contains("if !properties_rail_tab && !active && !topbar_main_tab {"));
    assert!(tab.contains("tab_active_gradient(c.host)"));
    assert!(tab.contains("tab_hover(c.host)"));
    // Fill-only selection is now uniform across every host: neither the tab
    // nor the menu anchor paints an accent border on the active state, so the
    // old `if c.host != BHostType::Topbar` accent-border branches are gone.
    // The active cap and the surface beneath it form one continuous fill.
    assert!(
        !tab.contains("border_b(px(c.style.strokes.border * 2.0))"),
        "tab must not paint an accent underline on active state"
    );
    assert!(
        !anchor.contains("border_b(px(c.style.strokes.border * 2.0))"),
        "menu anchor must not paint an accent underline on active state"
    );
    assert!(anchor.contains("host_item_active_gradient(c.host)"));
    assert!(anchor.contains("host_item_hover(c.host)"));
    assert!(item.contains("host_item_hover(c.host)"));
}

#[test]
fn topbar_zone_controls_use_resolved_rounded_stable_geometry() {
    let toggles = renderer("atom/toggle.rs");
    assert!(toggles.contains("c.style.dimensions.button_height"));
    assert!(toggles.contains("radius_for(BStyleRole::Button)"));
    assert!(toggles.contains(".w(px(size))"));
    assert!(toggles.contains(".h(px(size))"));
    assert!(toggles.contains(".rounded(radius)"));
    assert!(toggles.contains("topbar-zone-toggle-group"));
    assert!(toggles.contains(".justify_end()"));
    assert!(toggles.contains("topbar-zone-toggle-"));
    assert!(toggles.contains(".debug_selector("));

    let hover = toggles.split(".hover(").nth(1).expect("zone toggle hover");
    let hover = hover.split(".child(").next().expect("hover end");
    assert!(!hover.contains(".w("));
    assert!(!hover.contains(".h("));
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

    let tab = renderer("atom/tab.rs");
    assert!(tab.contains("c.host == BHostType::Buts && icon_only"));
    let properties = tab
        .split("let content: AnyElement = if properties_rail_tab {")
        .nth(1)
        .expect("Properties rail paint layer")
        .split("} else {\n        row.into_any_element()")
        .next()
        .expect("end of Properties paint layer");
    for geometry in [
        ".absolute()",
        ".top_0()",
        ".bottom_0()",
        ".left(px(c.style.properties_tab_background_inset()))",
        ".right(px(-c.style.properties_tab_overlap()))",
        ".rounded_l(r)",
    ] {
        assert!(properties.contains(geometry), "paint layer lost {geometry}");
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
    assert!(properties.contains(".child(paint)\n            .child(row)"));
    assert!(!tab.contains(".left(px(c.style.properties_tab_overlap()))"));
    assert!(tab.contains("if !properties_rail_tab && !active && !topbar_main_tab"));
}

#[test]
fn view3d_uses_a_lighter_opaque_flat_semantic_surface_across_palettes() {
    for palette in BPaletteName::ALL.iter().copied() {
        let style = resolve_ui_style(ui_style_defaults(palette), &BState::new());
        let base = style.colors.bg_viewport;
        let viewport = style.viewport_surface();
        let semantic = style.surface(BSurfaceTone::Viewport);
        let lift = viewport.2 - style.colors.bg_app.2;

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
        assert!((18.0..=55.0).contains(&lift), "{palette:?}: lift={lift}");
        assert!(
            (style.colors.text.2 - viewport.2).abs() >= 25.0,
            "{palette:?} viewport/text contrast became unreadable"
        );
        let grid = style.viewport_grid_stroke();
        assert!((grid.color.2 - viewport.2).abs() >= 4.0, "{palette:?}");
        assert!((0.0..=0.3).contains(&grid.color.3), "{palette:?}");
    }

    let view3d = renderer("atom/canvas/view3d.rs");
    assert!(view3d.contains("c.style.viewport_surface()"));
    assert!(view3d.contains(".bg(vp)"));
    assert!(view3d.contains("viewport_grid_stroke()"));
    assert!(view3d.contains("axis_stroke("));
    assert!(!view3d.contains("linear_gradient("));
    assert!(!view3d.contains("linear_color_stop("));
    assert!(!view3d.contains("Horizon band"));
}
