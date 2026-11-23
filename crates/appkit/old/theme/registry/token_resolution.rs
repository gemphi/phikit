use std::collections::HashMap;

use crate::graphics::NSColor;
use crate::theme::{keys, ThemeData, ThemeFont, ThemeId, ThemeLayout, ThemeMaterial, ThemeMode, ThemeVariant};

use super::color_palette::rgb;

/// Trait for defining theme variations with inheritance and composition.
pub trait ThemeProvider {
    /// The parent theme this theme inherits from, if any.
    fn base(&self) -> Option<&dyn ThemeProvider> {
        None
    }

    /// Override geometry tokens.
    fn override_geometry(&self, _g: &mut HashMap<String, f32>) {}

    /// Override color tokens.
    fn override_colors(&self, _c: &mut HashMap<String, NSColor>) {}

    /// Optionally define the theme ID.
    fn theme_id(&self) -> Option<ThemeId> {
        None
    }

    /// Build the full theme data by resolving inheritance.
    fn build(&self) -> ThemeData {
        let mut d = if let Some(parent) = self.base() {
            parent.build()
        } else {
            BaseTheme.build_base()
        };

        self.override_geometry(&mut d.layout.geometry);
        self.override_colors(&mut d.colors);
        if let Some(id) = self.theme_id() {
            d.id = id;
        }
        d
    }
}

/// Tuple implementation allows composing theme mixins.
impl<A: ThemeProvider, B: ThemeProvider> ThemeProvider for (A, B) {
    fn override_geometry(&self, g: &mut HashMap<String, f32>) {
        self.0.override_geometry(g);
        self.1.override_geometry(g);
    }
    fn override_colors(&self, c: &mut HashMap<String, NSColor>) {
        self.0.override_colors(c);
        self.1.override_colors(c);
    }
}

/// A strongly-typed geometry specification that implements ThemeProvider.
pub struct Geometry {
    pub radius_button: f32,
    pub radius_panel: f32,
    pub radius_control: f32,
    pub spacing_md: f32,
    pub font_size: f32,
    pub glass_opacity: f32,
}

impl ThemeProvider for Geometry {
    fn override_geometry(&self, g: &mut HashMap<String, f32>) {
        g.insert(keys::RADIUS_BUTTON.into(), self.radius_button);
        g.insert(keys::RADIUS_PANEL.into(), self.radius_panel);
        g.insert(keys::RADIUS_CONTROL.into(), self.radius_control);
        g.insert(keys::SPACING_MD.into(), self.spacing_md);
        g.insert(keys::SIZE_FONT_UI.into(), self.font_size);
        g.insert(keys::GLASS_OPACITY.into(), self.glass_opacity);
    }
}

/// A strongly-typed color palette that implements ThemeProvider.
pub struct Palette {
    pub bg_window: NSColor,
    pub bg_content: NSColor,
    pub accent: NSColor,
    pub text_primary: NSColor,
}

impl ThemeProvider for Palette {
    fn override_colors(&self, c: &mut HashMap<String, NSColor>) {
        c.insert(keys::CLR_BG_WINDOW.into(), self.bg_window);
        c.insert(keys::CLR_BG_CONTENT.into(), self.bg_content);
        c.insert(keys::CLR_ACCENT.into(), self.accent);
        c.insert(keys::CLR_TEXT_LABEL.into(), self.text_primary);
    }
}

pub struct CustomTheme {
    pub id: ThemeId,
    pub palette: Palette,
    pub geometry: Geometry,
}

impl ThemeProvider for CustomTheme {
    fn theme_id(&self) -> Option<ThemeId> {
        Some(self.id.clone())
    }
    fn override_geometry(&self, g: &mut HashMap<String, f32>) {
        self.geometry.override_geometry(g);
    }
    fn override_colors(&self, c: &mut HashMap<String, NSColor>) {
        self.palette.override_colors(c);
    }
}

pub(super) struct BaseTheme;
impl BaseTheme {
    pub(super) fn build_base(&self) -> ThemeData {
        let mut c = HashMap::new();
        // Traffic lights
        c.insert(keys::CLR_TRAFFIC_CLOSE.into(), rgb(1.00, 0.38, 0.35));
        c.insert(keys::CLR_TRAFFIC_MIN.into(), rgb(1.00, 0.74, 0.20));
        c.insert(keys::CLR_TRAFFIC_ZOOM.into(), rgb(0.15, 0.80, 0.25));

        let mut g = HashMap::new();
        // Geometry
        g.insert(keys::RADIUS_BUTTON.into(), 8.0);
        g.insert(keys::RADIUS_PANEL.into(), 12.0);
        g.insert(keys::RADIUS_CONTROL.into(), 5.0);
        g.insert(keys::RADIUS_NODE.into(), 8.0);
        g.insert(keys::RADIUS_TAB.into(), 6.0);
        g.insert(keys::RADIUS_DROPDOWN.into(), 7.0);
        g.insert(keys::RADIUS_TOOLTIP.into(), 5.0);
        g.insert(keys::RADIUS_ICON.into(), 4.0); // SF Symbols style rounded corners

        // Spacing & Sizing
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
        g.insert(keys::SIZE_FONT_SMALL.into(), 11.0);
        g.insert(keys::SIZE_CHAR_WIDTH.into(), 7.5);

        g.insert(keys::GLASS_ENABLED.into(), 1.0);
        g.insert(keys::GLASS_OPACITY.into(), 0.82);

        ThemeData {
            id: ThemeId::new(ThemeVariant::Apple, ThemeMode::Dark, ThemeMaterial::Standard),
            colors: c,
            layout: ThemeLayout { geometry: g },
            font: ThemeFont {
                ui_font: "Inter".into(),
                ui_font_size: 14.0,
                title_font: "Inter".into(),
                title_font_size: 15.0,
                monospace_font: "JetBrains Mono".into(),
                monospace_font_size: 13.0,
                icon_font: None,
            },
            supports_transparency: true,
            supports_animations: true,
        }
    }
}
