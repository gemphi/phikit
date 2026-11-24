//! A place of the library: where a creator keeps things. Phiogy's own
//! shapes, the person's Phiegg space, and the folders the system keeps
//! for them (Downloads, Videos, Pictures, Documents, the Desktop), so
//! the library sees what the person sees. `data/place.tsv` names each.

use std::path::PathBuf;

use crate::data;

/// One place: its key, its words (a word of the text), its icon, its
/// folder where the system has one, and whether it is the person's own
/// to make when it is not there yet.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Place {
    pub key: String,
    pub word: String,
    pub icon: String,
    pub folder: Option<PathBuf>,
    pub make: bool,
}

impl Place {
    /// The places of `data/place.tsv`, each where this system keeps it.
    pub fn own() -> Vec<Place> {
        data::row(data::PLACE).into_iter().filter(|cell| cell.len() > 4).map(|cell| {
            let folder = root(cell[3]).map(|root| match cell[4] {
                "-" => root,
                under => root.join(under),
            });
            Place { key: cell[0].to_string(), word: cell[1].to_string(), icon: cell[2].to_string(), folder, make: cell.get(5) == Some(&"make") }
        }).collect()
    }

    /// A place of that key at that folder: one a test makes.
    pub fn at(key: &str, folder: PathBuf, make: bool) -> Place {
        Place { key: key.to_string(), word: format!("place.{key}"), icon: String::new(), folder: Some(folder), make }
    }
}

/// The folder a place's table names: this program's own, the person's
/// own, or one the system keeps for them.
fn root(name: &str) -> Option<PathBuf> {
    match name {
        "asset" => Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("asset")),
        "home" => dirs::home_dir(),
        "download" => dirs::download_dir(),
        "video" => dirs::video_dir(),
        "picture" => dirs::picture_dir(),
        "document" => dirs::document_dir(),
        "desktop" => dirs::desktop_dir(),
        "audio" => dirs::audio_dir(),
        _ => None,
    }
}
