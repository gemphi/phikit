//! The asset set a browser build carries, embedded in the binary.
//!
//! The desktop build reads `assets/` from disk through `PhiberAssets`. A
//! browser has no disk, and `gpui::AssetSource::load` is synchronous, so it
//! cannot await a fetch either — the bytes have to be in the binary.
//!
//! `assets/` is 460 MB, which would be impossible, except that 455 MB of it
//! is three scene files (`production/`, `CW-Fashion/`). Those are *content*:
//! a browser user opens their own file through `Phx::open_bytes`, so they do
//! not belong in the bundle. What the app actually loads at startup — icons,
//! fonts, the brand marks, matcaps, studio lights and the startup
//! templates — comes to **5.5 MB**, which does fit.
//!
//! Listing the roots one by one rather than embedding `assets/` wholesale is
//! the point: a new folder of content cannot silently add 100 MB to the
//! download, because it has to be named here first.
//!
//! This type is built on every target, not only wasm, so that
//! `web_assets_tests.rs` can check the embedded bytes against the files on
//! disk. An asset set that has never been compared to its source is a
//! guess.

use gpui::{AssetSource, SharedString};
use include_dir::{Dir, include_dir};
use std::borrow::Cow;

static ICONS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets/icons");
static FONTS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets/fonts");
static BRAND: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets/brand");
static MATCAP: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets/matcap");
static BLEND: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets/blend");
static STUDIOLIGHT: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets/studiolight");

/// The embedded roots, by the folder name `assets/` gives them.
pub(crate) const ROOTS: [(&str, &Dir<'static>); 6] = [
    ("icons", &ICONS),
    ("fonts", &FONTS),
    ("brand", &BRAND),
    ("matcap", &MATCAP),
    ("blend", &BLEND),
    ("studiolight", &STUDIOLIGHT),
];

/// `assets/`, embedded — the [`AssetSource`] a browser build hands gpui.
pub struct WebAssets;

/// Split `icons/foo.svg` into its root and the path within it.
///
/// gpui asks for paths relative to `assets/`, the same strings the desktop
/// source joins onto its base folder, so the first segment names the root.
fn split(path: &str) -> Option<(&'static Dir<'static>, &str)> {
    let path = path.trim_start_matches('/');
    let (head, rest) = path.split_once('/').unwrap_or((path, ""));
    let (_, dir) = ROOTS.iter().find(|(name, _)| *name == head)?;
    Some((dir, rest))
}

impl AssetSource for WebAssets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        let Some((dir, rest)) = split(path) else { return Ok(None) };
        // `get_file` is relative to the embedded root, and an unknown path is
        // `None` rather than an error: gpui treats a missing asset as absent,
        // which is what the desktop source's `fs::read` failure means too.
        Ok(dir.get_file(rest).map(|f| Cow::Borrowed(f.contents())))
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
        let Some((dir, rest)) = split(path) else { return Ok(Vec::new()) };
        let listing = if rest.is_empty() { Some(dir) } else { dir.get_dir(rest) };
        Ok(listing
            .map(|d| {
                d.files()
                    .filter_map(|f| f.path().file_name()?.to_str().map(SharedString::from))
                    .collect()
            })
            .unwrap_or_default())
    }
}
