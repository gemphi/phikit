use std::collections::HashMap;
use crate::graphics::NSColor;
use crate::theme::{keys, ThemeData, ThemeFont, ThemeId, ThemeLayout, ThemeMaterial, ThemeMode, ThemeVariant};
use super::color_palette::{apply_mode, rgb, rgba};

fn shared_traffic(m: &mut HashMap<String, NSColor>) {
    m.insert(keys::CLR_TRAFFIC_CLOSE.into(), rgb(1.00, 0.38, 0.35));
    m.insert(keys::CLR_TRAFFIC_MIN.into(), rgb(1.00, 0.74, 0.20));
    m.insert(keys::CLR_TRAFFIC_ZOOM.into(), rgb(0.15, 0.80, 0.25));
}

fn shared_geometry_standard(g: &mut HashMap<String, f32>) {
    g.insert(keys::RADIUS_BUTTON.into(), 6.0);
    g.insert(keys::RADIUS_PANEL.into(), 12.0);
    g.insert(keys::RADIUS_CONTROL.into(), 4.0);
    g.insert(keys::RADIUS_NODE.into(), 7.0);
    g.insert(keys::RADIUS_TAB.into(), 5.0);
    g.insert(keys::RADIUS_DROPDOWN.into(), 6.0);
    g.insert(keys::RADIUS_TOOLTIP.into(), 4.0);
    g.insert(keys::SPACING_XS.into(), 2.0);
    g.insert(keys::SPACING_SM.into(), 4.0);
    g.insert(keys::SPACING_MD.into(), 8.0);
    g.insert(keys::SPACING_LG.into(), 16.0);
    g.insert(keys::SIZE_CONTROL_HEIGHT.into(), 22.0);
    g.insert(keys::SIZE_TAB_HEIGHT.into(), 28.0);
    g.insert(keys::SIZE_TAB_PAD_X.into(), 12.0);
    g.insert(keys::SIZE_MENU_BAR_HEIGHT.into(), 22.0);
    g.insert(keys::SIZE_TITLE_BAR_HEIGHT.into(), 38.0);
    g.insert(keys::SIZE_SIDEBAR_WIDTH.into(), 220.0);
    g.insert(keys::SIZE_FONT_UI.into(), 14.0);
    g.insert(keys::SIZE_FONT_SMALL.into(), 12.0);
    g.insert(keys::SIZE_CHAR_WIDTH.into(), 7.5);
    g.insert(keys::GLASS_ENABLED.into(), 1.0);
    g.insert(keys::GLASS_OPACITY.into(), 0.82);
    g.insert(keys::SIZE_PANEL_HEADER_H.into(), 32.0);
    g.insert(keys::SIZE_PANEL_TOOLBAR_H.into(), 28.0);
    g.insert(keys::SIZE_HEADER_TAB_H.into(), 24.0);
    g.insert(keys::SIZE_HEADER_TAB_PAD.into(), 10.0);
    g.insert(keys::SIZE_HEADER_TAB_GAP.into(), 4.0);
    g.insert(keys::RADIUS_HEADER_TAB.into(), 8.0);
    g.insert(keys::SIZE_ROW_H.into(), 24.0);
    g.insert(keys::SIZE_ROW_GAP.into(), 2.0);
    g.insert(keys::SIZE_ROW_INDENT.into(), 16.0);
    g.insert(keys::RADIUS_ROW.into(), 4.0);
    g.insert(keys::SIZE_SEARCH_H.into(), 20.0);
    g.insert(keys::SIZE_SEARCH_W.into(), 180.0);
    g.insert(keys::SIZE_SEARCH_MARGIN.into(), 8.0);
    g.insert(keys::SIZE_GIZMO.into(), 28.0);
    g.insert(keys::SIZE_GIZMO_GAP.into(), 4.0);
    g.insert(keys::SIZE_GIZMO_MARGIN.into(), 12.0);
    g.insert(keys::RADIUS_GIZMO.into(), 6.0);
    g.insert(keys::SIZE_ORIENT_GIZMO.into(), 48.0);
    g.insert(keys::SIZE_ORIENT_MARGIN.into(), 12.0);
    g.insert(keys::SIZE_GRID_SPACING.into(), 80.0);
    g.insert(keys::SIZE_GRID_THICKNESS.into(), 1.0);
    g.insert(keys::SIZE_CROSSHAIR_LEN.into(), 20.0);
    g.insert(keys::SIZE_CROSSHAIR_W.into(), 2.0);
    g.insert(keys::SIZE_SECTION_PAD.into(), 8.0);
    g.insert(keys::RADIUS_SECTION.into(), 8.0);
    g.insert(keys::SIZE_LABEL_W.into(), 80.0);
    g.insert(keys::SIZE_CONTENT_PAD.into(), 12.0);
    g.insert(keys::SIZE_PREF_SIDEBAR_W.into(), 200.0);
    g.insert(keys::SIZE_PREF_TAB_H.into(), 32.0);
    g.insert(keys::SIZE_PREF_ROW_H.into(), 32.0);
    g.insert(keys::SIZE_PREF_CONTROL_W.into(), 180.0);
    g.insert(keys::SIZE_PREF_CONTROL_H.into(), 28.0);
    g.insert(keys::SIZE_FONT_TITLE.into(), 20.0);
    g.insert(keys::SIZE_FONT_HEADER.into(), 14.0);
    g.insert(keys::SIZE_FONT_CAPTION.into(), 9.0);
    g.insert(keys::SIZE_ACTIVITY_W.into(), 58.0);
    g.insert(keys::SIZE_ACTIVITY_ICON.into(), 24.0);
    g.insert(keys::SIZE_ACTIVITY_GAP.into(), 14.0);
    g.insert(keys::RADIUS_ACTIVITY_ICON.into(), 6.0);
    g.insert(keys::SIZE_BUTTON_H.into(), 26.0);
    g.insert(keys::SIZE_BUTTON_GAP.into(), 30.0);
}

/// Base theme: neutral geometry + empty colors. Every theme starts here.
pub(super) fn base_theme() -> ThemeData {
    let mut g = HashMap::new();
    shared_geometry_standard(&mut g);
    let mut c = HashMap::new();
    shared_traffic(&mut c);
    ThemeData {
        id: ThemeId::new(ThemeVariant::Apple, ThemeMode::Dark, ThemeMaterial::Standard),
        colors: c, layout: ThemeLayout { geometry: g },
        font: ThemeFont {
            ui_font: "Inter".into(), ui_font_size: 13.0,
            title_font: "Inter".into(), title_font_size: 14.0,
            monospace_font: "JetBrains Mono".into(), monospace_font_size: 12.0, icon_font: None,
        },
        supports_transparency: true, supports_animations: true,
    }
}

/// Layer: BRAND — applies accent color, identity palette, fonts.
pub(super) fn apply_variant(d: &mut ThemeData, variant: &str) {
    crate::theme::variants::variant_for(variant).apply(d);
}

/// Layer: MATERIAL — applies surface geometry, glass, radii overrides
pub(super) fn apply_material(d: &mut ThemeData, material: &str) {
    let g = &mut d.layout.geometry;
    let c = &mut d.colors;
    match material {
        "glass" => {
            g.insert(keys::RADIUS_PANEL.into(), 12.0);
            g.insert(keys::GLASS_ENABLED.into(), 1.0);
            g.insert(keys::GLASS_OPACITY.into(), 0.35);
            let bg = rgb(0.0706, 0.0549, 0.0902);
            c.insert(keys::CLR_BG_WINDOW.into(), bg);
            c.insert(keys::CLR_BG_PANEL.into(), bg);
            c.insert(keys::CLR_BG_TAB_BAR.into(), bg);
            c.insert(keys::CLR_BG_TITLEBAR.into(), bg);
            let canvas = rgb(0.137, 0.118, 0.161);
            c.insert(keys::CLR_BG_CANVAS.into(), canvas);
            let panel = rgb(0.1059, 0.0863, 0.1216);
            c.insert(keys::CLR_BG_PANEL.into(), panel);
            c.insert(keys::CLR_BG_SIDEBAR.into(), panel);
            c.insert(keys::CLR_BG_HEADER.into(), panel);
            c.insert(keys::CLR_BG_DROPDOWN.into(), panel);
            c.insert(keys::CLR_BG_CONTENT.into(), panel);
            let hover = rgb(0.137, 0.118, 0.161);
            c.insert(keys::CLR_BG_SELECTED.into(), hover);
            c.insert(keys::CLR_BG_HOVER.into(), hover);
            c.insert(keys::CLR_ACCENT.into(), rgb(0.4431, 0.2118, 0.8039));
            c.insert(keys::CLR_BG_TAB_ACTIVE.into(), rgb(0.4431, 0.2118, 0.8039));
            c.insert(keys::CLR_BG_DROPDOWN.into(), hover);
            c.insert(keys::CLR_TEXT_LABEL.into(), rgba(1.0, 1.0, 1.0, 0.7));
            c.insert(keys::CLR_TEXT_SELECTED.into(), rgba(1.0, 1.0, 1.0, 1.0));
            c.insert(keys::CLR_TEXT_DISABLED.into(), rgba(1.0, 1.0, 1.0, 0.35));
            c.insert(keys::CLR_TAB_TEXT_ACTIVE.into(), rgba(1.0, 1.0, 1.0, 1.0));
            c.insert(keys::CLR_TAB_TEXT_INACTIVE.into(), rgba(1.0, 1.0, 1.0, 0.6));
            c.insert(keys::CLR_SEPARATOR.into(), rgba(1.0, 1.0, 1.0, 0.06));
            c.insert(keys::CLR_BORDER.into(), rgba(1.0, 1.0, 1.0, 0.10));
            d.supports_transparency = true;
            d.id.material = ThemeMaterial::Glass;
        }
        "metal" => {
            g.insert(keys::RADIUS_PANEL.into(), 2.0);
            g.insert(keys::RADIUS_BUTTON.into(), 2.0);
            c.insert(keys::CLR_BG_WINDOW.into(), rgb(0.2, 0.22, 0.25));
            d.id.material = ThemeMaterial::Metal;
        }
        "onyx" => {
            g.insert(keys::RADIUS_PANEL.into(), 0.0);
            g.insert(keys::RADIUS_BUTTON.into(), 0.0);
            c.insert(keys::CLR_BG_WINDOW.into(), rgb(0.0, 0.0, 0.0));
            c.insert(keys::CLR_BG_PANEL.into(), rgb(0.05, 0.05, 0.05));
            d.id.material = ThemeMaterial::Onyx;
        }
        "lace" => {
            g.insert(keys::RADIUS_PANEL.into(), 4.0);
            c.insert(keys::CLR_BORDER.into(), rgba(0.0, 0.0, 0.0, 0.2));
            d.id.material = ThemeMaterial::Lace;
        }
        "wetland" => {
            g.insert(keys::RADIUS_PANEL.into(), 18.0);
            c.insert(keys::CLR_BG_WINDOW.into(), rgb(0.1, 0.15, 0.1));
            d.id.material = ThemeMaterial::Wetland;
        }
        "platinum" => {
            g.insert(keys::RADIUS_PANEL.into(), 4.0);
            c.insert(keys::CLR_BG_WINDOW.into(), rgb(0.9, 0.9, 0.92));
            d.id.material = ThemeMaterial::Platinum;
        }
        "holographic" => {
            g.insert(keys::RADIUS_PANEL.into(), 16.0);
            g.insert(keys::GLASS_ENABLED.into(), 1.0);
            g.insert(keys::GLASS_OPACITY.into(), 0.5);
            d.supports_transparency = true;
            d.id.material = ThemeMaterial::Holographic;
        }
        _ => { d.id.material = ThemeMaterial::Standard; }
    }
}
