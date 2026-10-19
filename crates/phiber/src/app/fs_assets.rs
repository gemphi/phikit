//! `assets/` read from disk — the desktop [`AssetSource`].
//!
//! The browser build cannot use this (no filesystem) and carries an embedded
//! set instead: `app/web_assets.rs`.

use gpui::{AssetSource, SharedString};
use std::borrow::Cow;
use std::fs;
use std::path::PathBuf;

pub struct PhiberAssets {
    pub base: PathBuf,
}

impl PhiberAssets {
    /// The `assets/` folder beside the crate.
    pub fn beside_the_crate() -> Self {
        Self { base: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets") }
    }
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
