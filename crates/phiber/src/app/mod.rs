//! Phiber application - GPUI window launch.
//! All view-building logic lives in `bui::part`.

use std::borrow::Cow;
use std::fs;
use std::path::PathBuf;

use gpui::{
    App, AppContext, AssetSource, Bounds, SharedString, WindowBounds, WindowOptions, px, size,
};
use gpui_platform::application;
use theme::{
    LoadThemes, ThemeSettingsProvider, UiDensity, init as theme_init, set_theme_settings_provider,
};

pub mod handle;
pub mod interactive;
pub mod render;
pub use interactive::PhiberApp;

struct PhiberAssets {
    base: PathBuf,
}

impl AssetSource for PhiberAssets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        fs::read(self.base.join(path))
            .map(|data| Some(Cow::Owned(data)))
            .map_err(|err| err.into())
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
        fs::read_dir(self.base.join(path))
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| e.file_name().into_string().ok().map(SharedString::from))
                    .collect()
            })
            .map_err(|err| err.into())
    }
}

/// Minimal ThemeSettingsProvider so GPUI tooltips and font queries don't panic.
struct PhiberThemeProvider {
    ui_font: gpui::Font,
    buffer_font: gpui::Font,
}

impl ThemeSettingsProvider for PhiberThemeProvider {
    fn ui_font<'a>(&'a self, _cx: &'a App) -> &'a gpui::Font {
        &self.ui_font
    }
    fn buffer_font<'a>(&'a self, _cx: &'a App) -> &'a gpui::Font {
        &self.buffer_font
    }
    fn ui_font_size(&self, _cx: &App) -> gpui::Pixels {
        px(13.0)
    }
    fn buffer_font_size(&self, _cx: &App) -> gpui::Pixels {
        px(14.0)
    }
    fn ui_density(&self, _cx: &App) -> UiDensity {
        UiDensity::Compact
    }
}

fn default_font() -> gpui::Font {
    gpui::Font {
        family: "sans-serif".into(),
        features: gpui::FontFeatures::default(),
        weight: gpui::FontWeight::NORMAL,
        style: gpui::FontStyle::Normal,
        fallbacks: None,
    }
}

/// Launch the Phiber GPUI application.
pub fn run() {
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let assets = PhiberAssets {
                base: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
            };

            application().with_assets(assets).run(move |cx: &mut App| {
                // Initialize the GPUI theme system — registers GlobalTheme,
                // ThemeRegistry, FontFamilyCache, SystemAppearance.
                theme_init(LoadThemes::JustBase, cx);
                // Register a minimal settings provider for font/density queries.
                set_theme_settings_provider(
                    Box::new(PhiberThemeProvider {
                        ui_font: default_font(),
                        buffer_font: default_font(),
                    }),
                    cx,
                );

                let bounds = Bounds::centered(None, size(px(1024.0), px(768.0)), cx);
                cx.open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(bounds)),
                        ..Default::default()
                    },
                    |_, cx| cx.new(|_cx| crate::v2::app::BPhiberApp::new()),
                )
                .unwrap();

                cx.activate(true);
            });
        })
        .unwrap()
        .join()
        .unwrap();
}
