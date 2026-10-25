//! The only place phiber touches the vendored `theme` crate (`crates/phiui`):
//! it registers the gpui theme globals and answers gpui's font and density
//! questions. Every colour the app draws comes from `v2::theme`, not from here.

use gpui::{App, Font, FontFeatures, FontStyle, FontWeight, Pixels, px};
use theme::{
    LoadThemes, ThemeSettingsProvider, UiDensity, init as theme_init, set_theme_settings_provider,
};

/// Fonts and density gpui asks for (tooltips and font queries panic without it).
struct BThemeSettings {
    ui: Font,
    buffer: Font,
}

impl ThemeSettingsProvider for BThemeSettings {
    fn ui_font<'a>(&'a self, _cx: &'a App) -> &'a Font {
        &self.ui
    }
    fn buffer_font<'a>(&'a self, _cx: &'a App) -> &'a Font {
        &self.buffer
    }
    fn ui_font_size(&self, _cx: &App) -> Pixels {
        px(13.0)
    }
    fn buffer_font_size(&self, _cx: &App) -> Pixels {
        px(14.0)
    }
    fn ui_density(&self, _cx: &App) -> UiDensity {
        UiDensity::Compact
    }
}

fn sans() -> Font {
    Font {
        family: super::style::font::BFontFamily::PhiSoft.family().into(),
        features: FontFeatures::default(),
        weight: FontWeight::NORMAL,
        style: FontStyle::Normal,
        fallbacks: None,
    }
}

/// Register the gpui theme globals and our settings provider, once at start-up
/// before any window opens.
pub fn install(cx: &mut App) {
    // Phi-Soft Condensed, the interface's face, from inside the binary:
    // registered before the theme so the first frame already draws in it.
    let faces = super::style::font::PHI_SOFT_FACES.iter().map(|face| std::borrow::Cow::Borrowed(*face));
    if let Err(error) = cx.text_system().add_fonts(faces.collect()) {
        log::warn!("Phi-Soft Condensed could not be registered: {error}");
    }
    // Ask under which name the faces were filed: a name the text system does
    // not know draws every letter in the fallback face instead.
    let known = cx.text_system().all_font_names();
    let filed = super::style::font::PHI_SOFT_NAMES.into_iter().find(|name| known.iter().any(|seen| seen == name));
    match filed {
        Some(name) => {
            let _ = super::style::font::PHI_SOFT_FILED.set(name);
            log::info!("Phi-Soft Condensed is filed as {name:?}");
        }
        None => log::warn!("Phi-Soft Condensed is not among the faces this machine knows"),
    }
    theme_init(LoadThemes::JustBase, cx);
    set_theme_settings_provider(Box::new(BThemeSettings { ui: sans(), buffer: sans() }), cx);
}
