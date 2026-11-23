//! An app's assets as gpui asks for them: `PAsset` behind gpui's
//! `AssetSource`.

use std::borrow::Cow;

use gpui::{AssetSource, Result, SharedString};

use crate::pui::core::PAsset;

/// An app's assets, for gpui.
pub struct Source<A>(pub A);

impl<A: PAsset + Send + Sync + 'static> AssetSource for Source<A> {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        Ok(self.0.load(path).map(Cow::Owned))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(self.0.list(path).into_iter().map(SharedString::from).collect())
    }
}
