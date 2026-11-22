//! Where the pictures the web Phiogy names are: its own finished files,
//! on disk. A picture's key is `app:path`, as the web writes it
//! (`phiegg:worlds/w_aria.png`), and the path is in that app's public
//! folder; `data/asset.tsv` says where each app's is. A picture is read
//! as it is: none is made, changed or stood in for.

use std::path::{Component, PathBuf};

use appkit::pui::core::{PAsset, PFilm};

use crate::data;

/// The public folder of each app whose pictures are named.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Hosted {
    place: Vec<(String, PathBuf)>,
}

impl Hosted {
    /// The folders `data/asset.tsv` gives, from this crate's own.
    pub fn own() -> Hosted {
        let row = data::row(data::ASSET).into_iter().filter_map(|cell| Some((cell.first()?.to_string(), PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(cell.get(1)?))));
        Hosted { place: row.collect() }
    }

    /// Where the file of a key is; nothing for a key of no app here, and
    /// for one that would leave its app's folder.
    pub fn path(&self, key: &str) -> Option<PathBuf> {
        let (app, path) = key.split_once(':')?;
        let folder = self.place.iter().find(|(known, _)| known == app).map(|(_, folder)| folder)?;
        let inside = path.split(['/', '\\']).all(|part| part != "..");
        inside.then(|| folder.join(path.trim_start_matches('/')))
    }

    /// Where the file of a key is, as a person reads it and the system
    /// takes it: with no step back up left in it.
    pub fn place(&self, key: &str) -> Option<String> {
        let mut clean = PathBuf::new();
        for part in self.path(key)?.components() {
            match part {
                Component::ParentDir => _ = clean.pop(),
                Component::CurDir => {}
                other => clean.push(other),
            }
        }
        Some(clean.to_string_lossy().to_string())
    }
}

impl PAsset for Hosted {
    fn load(&self, path: &str) -> Option<Vec<u8>> {
        std::fs::read(self.path(path)?).ok()
    }
}

/// The pictures of a call, which move: an asset source with no file,
/// and the window's film.
pub struct Moving(pub PFilm);

impl PAsset for Moving {
    fn load(&self, _path: &str) -> Option<Vec<u8>> {
        None
    }

    fn film(&self) -> Option<PFilm> {
        Some(self.0.clone())
    }
}
