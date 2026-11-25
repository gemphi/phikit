//! A place of the library's tree: where a creator keeps things. The
//! drives of this system (Volumes), the folders the system keeps for the
//! person (Home, Desktop, Documents, Downloads, Pictures, Videos, Music),
//! and Phiogy's own (its shapes, the person's Phiegg space), so the
//! library reads the disk as Blender's file browser does.
//! `data/place.tsv` names all but the drives, which the system has.

use std::path::{Path, PathBuf};

use crate::data;

/// One place: its key, its words (a word of the text, or the drive's own
/// name), its icon, its section of the tree, its folder where the system
/// has one, and whether it is the person's own to make.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Place {
    pub key: String,
    pub word: String,
    pub icon: String,
    pub section: String,
    pub folder: Option<PathBuf>,
    pub make: bool,
}

impl Place {
    /// The drives of this system, then the places of `data/place.tsv`.
    pub fn own() -> Vec<Place> {
        let set = appkit::service::PTable::read(data::SHELF);
        let drive = set.get("drive").chars().map(|letter| format!("{letter}:\\")).filter(|root| Path::new(root).exists());
        let volume = drive.map(|root| Place { key: root.clone(), word: root.trim_end_matches('\\').to_string(), icon: set.get("volume").to_string(), section: "volume".to_string(), folder: Some(PathBuf::from(&root)), make: false });
        let table = data::row(data::PLACE).into_iter().filter(|cell| cell.len() > 5).map(|cell| {
            let folder = root(cell[4]).map(|root| match cell[5] {
                "-" => root,
                under => root.join(under),
            });
            Place { key: cell[0].to_string(), word: cell[1].to_string(), icon: cell[2].to_string(), section: cell[3].to_string(), folder, make: cell.get(6) == Some(&"make") }
        });
        volume.chain(table).collect()
    }

    /// A place of that key at that folder: one a test makes.
    pub fn at(key: &str, folder: PathBuf, make: bool) -> Place {
        Place { key: key.to_string(), word: format!("place.{key}"), icon: String::new(), section: "own".to_string(), folder: Some(folder), make }
    }
}

/// The folder a place's table names: this program's own, the person's
/// own, or one the system keeps for them.
fn root(name: &str) -> Option<PathBuf> {
    match name {
        "asset" => Some(crate::Phiogy::asset()),
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

/// The folders in a folder, by name, hidden ones left out.
pub fn folders(folder: &Path) -> Vec<PathBuf> {
    let Ok(read) = std::fs::read_dir(folder) else { return Vec::new() };
    let mut out: Vec<PathBuf> = read.flatten().filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir())).filter(|entry| !entry.file_name().to_string_lossy().starts_with(['.', '$'])).map(|entry| entry.path()).collect();
    out.sort_by_key(|path| path.file_name().map(|name| name.to_string_lossy().to_lowercase()).unwrap_or_default());
    out
}
