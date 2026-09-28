//! Phiber application - GPUI window launch.
//! Views are built in `v2::view`; this module only opens the window.

use std::borrow::Cow;
use std::fs;
use std::path::PathBuf;

use gpui::{
    App, AppContext, AssetSource, Bounds, SharedString, WindowBounds, WindowOptions, px, size,
};
use gpui_platform::application;


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

/// Launch the Phiber GPUI application.
pub fn run() {
    // Armature deform of opened files runs on the GPU (phiray) when available.
    crate::v2::scene::RaySkin::install();
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let assets = PhiberAssets {
                base: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
            };

            application().with_assets(assets).run(move |cx: &mut App| {
                // gpui's theme globals plus our font / density settings.
                crate::v2::theme::vendor::install(cx);

                let bounds = Bounds::centered(None, size(px(1024.0), px(768.0)), cx);
                cx.open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(bounds)),
                        // Phiber draws its own title bar in the theme colours
                        // (Topbar zone + caption controls); hide the native one.
                        titlebar: Some(gpui::TitlebarOptions {
                            title: Some("Phiber".into()),
                            appears_transparent: true,
                            traffic_light_position: Some(gpui::point(px(12.0), px(10.0))),
                        }),
                        window_decorations: Some(gpui::WindowDecorations::Client),
                        window_min_size: Some(size(px(640.0), px(400.0))),
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
