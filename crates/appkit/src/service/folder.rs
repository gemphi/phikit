//! An app's assets as folders on disk: the app's own first, then the one
//! every app shares. A file the app has under the same path as a shared
//! one is the one found.

use std::path::PathBuf;

use crate::pui::core::PAsset;

/// Folders on disk, asked in order.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PFolder {
    root: Vec<PathBuf>,
}

impl PFolder {
    /// These folders, the first asked first.
    pub fn new(root: &[&str]) -> PFolder {
        PFolder { root: root.iter().map(PathBuf::from).collect() }
    }

    /// The folder of assets every app shares: `asset/` at appkit's root.
    pub fn shared() -> &'static str {
        concat!(env!("CARGO_MANIFEST_DIR"), "/asset")
    }
}

impl PAsset for PFolder {
    fn load(&self, path: &str) -> Option<Vec<u8>> {
        self.root.iter().find_map(|root| std::fs::read(root.join(path)).ok())
    }

    fn list(&self, folder: &str) -> Vec<String> {
        let entry = self.root.iter().filter_map(|root| std::fs::read_dir(root.join(folder)).ok()).flatten().flatten();
        let mut name: Vec<String> = entry.map(|entry| format!("{folder}/{}", entry.file_name().to_string_lossy())).collect();
        name.sort();
        name.dedup();
        name
    }
}
