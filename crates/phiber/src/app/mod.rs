//! Phiber application - GPUI window launch.
//! Views are built in `v2::view`; this module only opens the window.
//!
//! Two ways in, because the desktop and the browser differ in three ways
//! that cannot be papered over: where the assets come from
//! (`fs_assets` / `web_assets`), whether the app may run on a thread of its
//! own, and whether there is a disk to journal to. [`boot`] holds everything
//! they share, so the window itself has one implementation.

use gpui::{App, AppContext, Bounds, WindowBounds, WindowOptions, px, size};

/// `assets/` read from disk (desktop).
pub mod fs_assets;
/// `assets/` embedded in the binary, for a build with no filesystem.
pub mod web_assets;
#[cfg(test)]
mod web_assets_tests;

/// Launch the Phiber GPUI application.
pub fn run(file: Option<String>) {
    #[cfg(not(target_family = "wasm"))]
    desktop(file);
    #[cfg(target_family = "wasm")]
    web(file);
}

/// The desktop app: journal and GPU skinning armed first, then the window on
/// a thread with a stack big enough for the view tree.
#[cfg(not(target_family = "wasm"))]
fn desktop(file: Option<String>) {
    // Journal first, with the `log` crate bridged into it, so the GPU adapter
    // and backend each context picks is on disk from the very first line —
    // before phiray's skinning context and before the window's GPU context.
    crate::v2::app::journal::BJournal::arm();
    crate::v2::app::journal::BJournal::bridge_log();
    // Armature deform of opened files runs on the GPU (phiray) when available.
    crate::v2::scene::RaySkin::install();
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(move || {
            gpui_platform::application()
                .with_assets(fs_assets::PhiberAssets::beside_the_crate())
                .run(move |cx: &mut App| boot(cx, Persist::Yes, file));
        })
        .unwrap()
        .join()
        .unwrap();
}

/// The browser app.
///
/// Everything the desktop path does first is unavailable or pointless here.
/// `std::thread::Builder::spawn` returns `Err` on `wasm32-unknown-unknown`,
/// so the app runs on the one thread there is — which is also why the
/// platform comes from `single_threaded_web` rather than `application`.
/// `RaySkin::install` spawns a warm-up thread for the same reason, and
/// phiphx's CPU skinner does the work instead. The journal, the durable
/// session and the health log all write files, so they stay off: see
/// [`Persist`].
#[cfg(target_family = "wasm")]
fn web(file: Option<String>) {
    gpui_platform::web_init();
    gpui_platform::single_threaded_web()
        .with_assets(web_assets::WebAssets)
        .run(move |cx: &mut App| boot(cx, Persist::No, file));
}

/// Whether this build has a disk to write to.
///
/// Not a cosmetic flag: `BJournal`, `BSession` and `BHealth` each open a file,
/// and a browser has nowhere to put one. Naming it this way keeps the
/// difference between the two builds in one readable place instead of
/// scattered `cfg!` calls inside the window setup.
#[derive(Clone, Copy, PartialEq)]
enum Persist {
    Yes,
    /// Only [`web`] constructs this, and that is compiled out everywhere
    /// else — hence the allow, rather than a cfg that would make `boot`
    /// read differently on the two targets.
    #[cfg_attr(not(target_family = "wasm"), allow(dead_code))]
    No,
}

/// Open the window and wire it up — the part both builds share. `file` is
/// the command line's `.blend`, opened as `blender file.blend` opens it.
fn boot(cx: &mut App, persist: Persist, file: Option<String>) {
    // gpui's theme globals plus our font / density settings.
    crate::v2::theme::vendor::install(cx);

    let bounds = Bounds::centered(None, size(px(1024.0), px(768.0)), cx);
    let window = cx
        .open_window(
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

    // Persist the durable session from here on (a no-op until now, so
    // nothing a test did was written), and resume the last one: the window
    // opens where it was left and reopens the last file with Load UI off.
    // A browser has no file to keep it in.
    if persist == Persist::Yes {
        crate::v2::app::session::BSession::arm();
    }

    // Closing the window asks about unsaved changes as File ▸ Quit does
    // (`op::close::may_close`).
    let _ = window.update(cx, |view, w, cx| {
        let app = cx.entity();
        w.on_window_should_close(cx, move |window, cx| {
            app.update(cx, |app, cx| app.may_close(window, cx))
        });
        // A file on the command line is opened as Blender opens it — its
        // own UI, no splash — instead of resuming the last session.
        match file {
            Some(path) => view.open_argument(path, cx),
            None if persist == Persist::Yes => view.restore_session(cx),
            None => {}
        }
    });

    // The session journal, then the Info log's first lines: the build, where
    // the journal is, and the render device once phiray has it. All three
    // open files, so they are desktop-only.
    if persist == Persist::Yes {
        crate::v2::app::journal::BJournal::arm();
        crate::v2::app::journal::BJournal::bridge_log();
        crate::v2::app::health::BHealth::arm();
    }

    cx.activate(true);
}
