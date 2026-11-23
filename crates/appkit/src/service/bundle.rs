//! The bundle: the assets every app shares, built into the program. It
//! is what an app has where there is no disk: in a browser. `build.rs`
//! makes its table from the files under `asset/`.
//!
//! Two sources make one: `(own, shared)` asks the first, then the second,
//! so an app's own file is found before a shared one of the same path.

use crate::pui::core::PAsset;

mod built {
    include!(concat!(env!("OUT_DIR"), "/asset.rs"));
}

/// The shared assets, built into the program.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PBundle;

impl PAsset for PBundle {
    fn load(&self, path: &str) -> Option<Vec<u8>> {
        built::FILE.iter().find(|(name, _)| *name == path).map(|(_, bytes)| bytes.to_vec())
    }

    fn list(&self, folder: &str) -> Vec<String> {
        let inside = |name: &&str| name.strip_prefix(folder).is_some_and(|rest| rest.starts_with('/'));
        built::FILE.iter().map(|(name, _)| *name).filter(inside).map(str::to_string).collect()
    }
}

impl<A: PAsset, B: PAsset> PAsset for (A, B) {
    fn load(&self, path: &str) -> Option<Vec<u8>> {
        self.0.load(path).or_else(|| self.1.load(path))
    }

    fn list(&self, folder: &str) -> Vec<String> {
        let mut name = self.0.list(folder);
        name.extend(self.1.list(folder));
        name.sort();
        name.dedup();
        name
    }

    fn film(&self) -> Option<crate::pui::core::PFilm> {
        self.0.film().or_else(|| self.1.film())
    }
}
