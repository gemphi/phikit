use std::collections::HashMap;
use crate::graphics::NSColor;
use crate::theme::{keys, ThemeMode};
use crate::theme::types::ThemeData;

// Color helpers
pub fn rgb(r: f32, g: f32, b: f32) -> NSColor { NSColor::new(r, g, b, 1.0) }
pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> NSColor { NSColor::new(r, g, b, a) }

/// Layer: MODE — applies dark or light color scheme
pub(super) fn apply_mode(d: &mut ThemeData, mode: &str) {
    let c = &mut d.colors;
    match mode {
        "light" => {
            c.insert(keys::CLR_BG_WINDOW.into(), rgb(0.96, 0.96, 0.98));
            c.insert(keys::CLR_BG_CARD.into(), rgba(0.0, 0.0, 0.0, 0.04));
            c.insert(keys::CLR_BORDER_CARD.into(), rgba(0.0, 0.0, 0.0, 0.06));
            c.insert(keys::CLR_BG_CONTENT.into(), rgb(1.0, 1.0, 1.0));
            c.insert(keys::CLR_BG_PANEL.into(), rgb(0.97, 0.97, 0.99));
            c.insert(keys::CLR_BG_SIDEBAR.into(), rgb(0.93, 0.93, 0.95));
            c.insert(keys::CLR_BG_HEADER.into(), rgba(0.95, 0.95, 0.97, 0.92));
            c.insert(keys::CLR_BG_CONTROL.into(), rgba(0.0, 0.0, 0.0, 0.06));
            c.insert(keys::CLR_BG_BUTTON.into(), rgba(0.0, 0.0, 0.0, 0.08));
            c.insert(keys::CLR_BG_BUTTON_HOVER.into(), rgba(0.0, 0.0, 0.0, 0.13));
            c.insert(keys::CLR_BG_BUTTON_PRESSED.into(), rgba(0.0, 0.0, 0.0, 0.18));
            c.insert(keys::CLR_BG_SELECTED.into(), rgba(0.0, 0.48, 1.0, 0.20));
            c.insert(keys::CLR_BG_HOVER.into(), rgba(0.0, 0.0, 0.0, 0.05));
            c.insert(keys::CLR_BG_TIMELINE.into(), rgb(0.90, 0.90, 0.93));
            c.insert(keys::CLR_BG_RULER.into(), rgb(0.87, 0.87, 0.90));
            c.insert(keys::CLR_BG_DROPDOWN.into(), rgba(0.98, 0.98, 1.0, 0.97));
            c.insert(keys::CLR_BG_CANVAS.into(), rgb(0.88, 0.88, 0.91));
            c.insert(keys::CLR_BG_GRID.into(), rgb(0.80, 0.80, 0.84));
            c.insert(keys::CLR_BG_PANEL.into(), rgba(0.95, 0.95, 0.97, 0.95));
            c.insert(keys::CLR_BG_TAB_BAR.into(), rgb(0.93, 0.93, 0.95));
            c.insert(keys::CLR_BG_TAB_ACTIVE.into(), rgba(0.0, 0.48, 1.0, 0.15));
            c.insert(keys::CLR_BG_TAB_INACTIVE.into(), rgba(0.0, 0.0, 0.0, 0.0));
            c.insert(keys::CLR_NODE_BODY.into(), rgb(0.92, 0.92, 0.95));
            c.insert(keys::CLR_NODE_HEADER.into(), rgb(0.0, 0.48, 1.0));
            c.insert(keys::CLR_TEXT_LABEL.into(), rgba(0.0, 0.0, 0.0, 0.85));
            c.insert(keys::CLR_TEXT_SECONDARY.into(), rgba(0.0, 0.0, 0.0, 0.50));
            c.insert(keys::CLR_TEXT_TERTIARY.into(), rgba(0.0, 0.0, 0.0, 0.28));
            c.insert(keys::CLR_TEXT_DISABLED.into(), rgba(0.0, 0.0, 0.0, 0.18));
            c.insert(keys::CLR_TEXT_PLACEHOLDER.into(), rgba(0.0, 0.0, 0.0, 0.25));
            c.insert(keys::CLR_TEXT_SELECTED.into(), rgb(0.0, 0.0, 0.0));
            c.insert(keys::CLR_TEXT_ON_ACCENT.into(), rgb(1.0, 1.0, 1.0));
            c.insert(keys::CLR_TAB_TEXT_ACTIVE.into(), rgb(0.0, 0.0, 0.0));
            c.insert(keys::CLR_TAB_TEXT_INACTIVE.into(), rgba(0.0, 0.0, 0.0, 0.50));
            c.insert(keys::CLR_BORDER.into(), rgba(0.0, 0.0, 0.0, 0.12));
            c.insert(keys::CLR_SEPARATOR.into(), rgba(0.0, 0.0, 0.0, 0.10));
            c.insert(keys::CLR_SHADOW.into(), rgba(0.0, 0.0, 0.0, 0.18));
            c.insert(keys::CLR_TITLE_BAR.into(), rgba(0.95, 0.95, 0.97, 0.92));
            c.insert(keys::CLR_TITLE_BAR_TEXT.into(), rgb(0.0, 0.0, 0.0));
            c.insert(keys::CLR_ICON_DEFAULT.into(), rgba(0.35, 0.35, 0.35, 1.0));
            c.insert(keys::CLR_RED.into(), rgb(0.85, 0.20, 0.20));
            c.insert(keys::CLR_GREEN.into(), rgb(0.20, 0.75, 0.20));
            c.insert(keys::CLR_YELLOW.into(), rgb(0.90, 0.80, 0.10));
            c.insert(keys::CLR_BLUE.into(), rgb(0.20, 0.50, 0.90));
            d.id.mode = ThemeMode::Light;
        }
        _ => {
            c.insert(keys::CLR_BG_WINDOW.into(), rgb(0.11, 0.11, 0.12));
            c.insert(keys::CLR_BG_WINDOW_TOP.into(), rgb(0.14, 0.14, 0.16));
            c.insert(keys::CLR_BG_WINDOW_BOTTOM.into(), rgb(0.09, 0.09, 0.10));
            c.insert(keys::CLR_BG_EDITOR_TOP.into(), rgb(0.06, 0.06, 0.07));
            c.insert(keys::CLR_BG_EDITOR_BOTTOM.into(), rgb(0.04, 0.04, 0.05));
            c.insert(keys::CLR_BG_CARD.into(), rgba(1.0, 1.0, 1.0, 0.04));
            c.insert(keys::CLR_BORDER_CARD.into(), rgba(1.0, 1.0, 1.0, 0.06));
            c.insert(keys::CLR_BG_CONTENT.into(), rgb(0.11, 0.11, 0.12));
            c.insert(keys::CLR_BG_PANEL.into(), rgb(0.11, 0.11, 0.12));
            c.insert(keys::CLR_BG_SIDEBAR.into(), rgb(0.11, 0.11, 0.12));
            c.insert(keys::CLR_BG_HEADER.into(), rgba(0.08, 0.08, 0.09, 0.95));
            c.insert(keys::CLR_BG_CONTROL.into(), rgb(0.19, 0.19, 0.21));
            c.insert(keys::CLR_BG_BUTTON.into(), rgba(1.0, 1.0, 1.0, 0.06));
            c.insert(keys::CLR_BG_BUTTON_HOVER.into(), rgba(1.0, 1.0, 1.0, 0.10));
            c.insert(keys::CLR_BG_BUTTON_PRESSED.into(), rgba(1.0, 1.0, 1.0, 0.10));
            c.insert(keys::CLR_BG_SELECTED.into(), rgba(0.0, 0.48, 1.0, 0.35));
            c.insert(keys::CLR_BG_HOVER.into(), rgba(1.0, 1.0, 1.0, 0.06));
            c.insert(keys::CLR_BG_TIMELINE.into(), rgb(0.09, 0.09, 0.10));
            c.insert(keys::CLR_BG_RULER.into(), rgb(0.12, 0.12, 0.13));
            c.insert(keys::CLR_BG_DROPDOWN.into(), rgba(0.14, 0.14, 0.16, 0.98));
            c.insert(keys::CLR_BG_CANVAS.into(), rgb(0.16, 0.16, 0.17));
            c.insert(keys::CLR_BG_GRID.into(), rgb(0.22, 0.22, 0.24));
            c.insert(keys::CLR_BG_PANEL.into(), rgba(0.11, 0.11, 0.12, 0.95));
            c.insert(keys::CLR_BG_TAB_BAR.into(), rgb(0.11, 0.11, 0.12));
            c.insert(keys::CLR_BG_TAB_ACTIVE.into(), rgba(0.0, 0.48, 1.0, 0.25));
            c.insert(keys::CLR_BG_TAB_INACTIVE.into(), rgba(1.0, 1.0, 1.0, 0.0));
            c.insert(keys::CLR_NODE_BODY.into(), rgb(0.18, 0.18, 0.21));
            c.insert(keys::CLR_NODE_HEADER.into(), rgb(0.0, 0.48, 1.0));
            c.insert(keys::CLR_PORT_OUT.into(), rgb(0.35, 0.75, 1.0));
            c.insert(keys::CLR_PORT_IN.into(), rgb(0.70, 0.50, 1.0));
            c.insert(keys::CLR_CONN_LINE.into(), rgba(0.35, 0.75, 1.0, 0.85));
            c.insert(keys::CLR_TEXT_LABEL.into(), rgba(1.0, 1.0, 1.0, 0.90));
            c.insert(keys::CLR_TEXT_SECONDARY.into(), rgba(1.0, 1.0, 1.0, 0.55));
            c.insert(keys::CLR_TEXT_TERTIARY.into(), rgba(1.0, 1.0, 1.0, 0.30));
            c.insert(keys::CLR_TEXT_DISABLED.into(), rgba(1.0, 1.0, 1.0, 0.20));
            c.insert(keys::CLR_TEXT_PLACEHOLDER.into(), rgba(1.0, 1.0, 1.0, 0.25));
            c.insert(keys::CLR_TEXT_SELECTED.into(), rgb(1.0, 1.0, 1.0));
            c.insert(keys::CLR_TEXT_ON_ACCENT.into(), rgb(1.0, 1.0, 1.0));
            c.insert(keys::CLR_TAB_TEXT_ACTIVE.into(), rgb(1.0, 1.0, 1.0));
            c.insert(keys::CLR_TAB_TEXT_INACTIVE.into(), rgba(1.0, 1.0, 1.0, 0.45));
            c.insert(keys::CLR_BORDER.into(), rgba(1.0, 1.0, 1.0, 0.06));
            c.insert(keys::CLR_SEPARATOR.into(), rgba(1.0, 1.0, 1.0, 0.04));
            c.insert(keys::CLR_SHADOW.into(), rgba(0.0, 0.0, 0.0, 0.60));
            c.insert(keys::CLR_TITLE_BAR.into(), rgba(0.10, 0.10, 0.11, 0.95));
            c.insert(keys::CLR_BG_TITLEBAR.into(), rgb(0.11, 0.11, 0.12));
            c.insert(keys::CLR_TITLE_BAR_TEXT.into(), rgb(0.95, 0.95, 0.95));
            c.insert(keys::CLR_ICON_DEFAULT.into(), rgba(0.82, 0.82, 0.82, 1.0));
            c.insert(keys::CLR_ICON_CAMERA.into(), rgb(0.90, 0.80, 0.30));
            c.insert(keys::CLR_ICON_LIGHT.into(), rgb(1.00, 0.90, 0.30));
            c.insert(keys::CLR_ICON_MATERIAL.into(), rgb(0.80, 0.30, 0.80));
            c.insert(keys::CLR_ICON_MESH.into(), rgb(0.50, 0.75, 1.00));
            c.insert(keys::CLR_ICON_ARMATURE.into(), rgb(0.30, 0.85, 0.50));
            c.insert(keys::CLR_RED.into(), rgb(0.90, 0.20, 0.20));
            c.insert(keys::CLR_GREEN.into(), rgb(0.20, 0.90, 0.20));
            c.insert(keys::CLR_YELLOW.into(), rgb(1.00, 0.90, 0.10));
            c.insert(keys::CLR_BLUE.into(), rgb(0.20, 0.50, 1.00));
            d.id.mode = ThemeMode::Dark;
        }
    }
}
