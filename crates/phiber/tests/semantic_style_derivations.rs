//! Characterization coverage for task 6.1 semantic style derivations.
//! Validates: Requirements 1.1, 1.4, 1.6, 3.4, 8.2, 8.3, 12.1-12.5.

use phiber::v2::{
    host::BHostType,
    theme::{BPaletteName, BResolvedUiStyle, BShadowStyle, BSurfaceTone, ui_style_defaults},
};

fn style(name: BPaletteName) -> BResolvedUiStyle {
    BResolvedUiStyle::from(ui_style_defaults(name))
}

fn close(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 1.0e-5, "{actual} != {expected}");
}

#[test]
fn semantic_surface_ladder_preserves_existing_palette_based_treatments() {
    for name in BPaletteName::ALL.iter().copied() {
        let style = style(name);
        let panel = style.colors.bg_panel;
        let header = style.surface(BSurfaceTone::Header);
        let footer = style.surface(BSurfaceTone::Footer);
        let sidebar = style.surface(BSurfaceTone::Sidebar);
        let card = style.surface(BSurfaceTone::Card);
        let section = style.surface(BSurfaceTone::Section);

        assert_eq!(header.bottom, style.colors.bg_header);
        close(footer.bottom.2, panel.2 + style.effects.footer_lift * 100.0);
        close(
            sidebar.bottom.2,
            panel.2 + style.effects.sidebar_lift * 100.0,
        );
        close(card.bottom.2, panel.2 + style.effects.card_lift * 100.0);
        close(
            section.bottom.2,
            panel.2 + style.effects.section_lift * 100.0,
        );
        close(
            section.top.2 - section.bottom.2,
            style.effects.gradient_lift * 100.0,
        );
    }
}

#[test]
fn disabling_gradients_flattens_every_semantic_gradient_without_changing_base_color() {
    let mut style = style(BPaletteName::AppleDark);
    style.effects.gradients = false;

    for tone in [
        BSurfaceTone::Header,
        BSurfaceTone::Footer,
        BSurfaceTone::Sidebar,
        BSurfaceTone::Body,
        BSurfaceTone::Viewport,
        BSurfaceTone::Modal,
        BSurfaceTone::Card,
        BSurfaceTone::Section,
    ] {
        let gradient = style.surface(tone);
        assert!(!gradient.enabled);
        assert_eq!(gradient.top, gradient.bottom);
    }
    let tab = style.tab_active_gradient(BHostType::Buts);
    assert!(!tab.enabled);
    assert_eq!(tab.top, tab.bottom);
}

#[test]
fn host_and_canvas_derivations_use_resolved_effects_and_colors() {
    let mut style = style(BPaletteName::TokyoNight);
    style.effects.viewport_lightness_delta = 0.08;
    style.effects.node_grid_lift = 0.15;
    style.effects.node_grid_alpha = 0.22;

    assert_eq!(style.host_lightness(BHostType::View3D), 24.0);
    assert_eq!(style.host_lightness(BHostType::Buts), 19.0);
    assert_eq!(style.host_lightness(BHostType::Shader), 10.0);
    assert_eq!(style.host_lightness(BHostType::Topbar), 9.0);
    let viewport = style.viewport_gradient();
    let surface = style.viewport_surface();
    assert!(!viewport.enabled);
    assert_eq!(viewport.top, surface);
    assert_eq!(viewport.bottom, surface);
    close(
        surface.2 - style.colors.bg_viewport.2,
        style.effects.viewport_lightness_delta * 100.0,
    );
    assert_eq!(surface.3, 1.0);
    let node_grid = style.node_grid_stroke();
    close(node_grid.color.2 - style.colors.bg_viewport.2, 15.0);
    close(node_grid.color.3, 0.22);
}

#[test]
fn structural_strokes_sidebar_width_and_shadow_are_resolved_accessors() {
    let mut style = style(BPaletteName::BlenderDark);
    style.strokes.border = 2.0;
    style.strokes.hairline = 0.5;
    style.strokes.grid = 1.5;
    style.dimensions.sidebar_width = 180.0;
    style.dimensions.sidebar_min_width = 160.0;
    style.effects.surface_shadow = BShadowStyle::None;

    assert_eq!(style.border_stroke().width, 2.0);
    assert_eq!(style.border_stroke().color, style.colors.border);
    assert_eq!(style.separator_stroke().width, 0.5);
    assert_eq!(style.viewport_grid_stroke().width, 1.5);
    assert_eq!(style.node_grid_stroke().width, 1.5);
    assert_eq!(style.sidebar_min_width(), 180.0);
    assert_eq!(style.surface_shadow(), BShadowStyle::None);
}

#[test]
fn shade_and_checker_derivations_clamp_to_valid_lightness() {
    let mut style = style(BPaletteName::AppleDark);
    style.colors.bg_viewport = (210.0, 20.0, 98.0, 1.0);
    style.effects.image_light_lift = 0.09;
    style.effects.image_dark_lift = -1.50;

    let checker = style.image_checker_colors();
    assert_eq!(checker.light, (210.0, 20.0, 100.0, 1.0));
    assert_eq!(checker.dark, (210.0, 20.0, 0.0, 1.0));
    assert_eq!(style.selected_hover((0.0, 0.0, 99.0, 0.5)).2, 100.0);
}
