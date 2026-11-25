//! Where the pictures the web Phiogy names are: its own finished files.
//! A picture's key is `app:path`, as the web writes it
//! (`phiegg:worlds/w_aria.png`), and the path is in that app's public
//! folder; `data/asset.tsv` says where each app's is on this disk. One
//! that is not there is read from the app's server, by its path or by
//! the key it is served under there (`data/media.tsv`), and kept in the
//! person's cache. A picture is read as it is: none is made, changed or
//! stood in for.

use std::path::{Component, PathBuf};

use appkit::pui::core::{PAsset, PFilm};
use appkit::service::PTable;

use crate::data;

/// The public folder of each app whose pictures are named, and where
/// they are on the app's server.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Hosted {
    place: Vec<(String, PathBuf)>,
    /// Each app's pictures on the server: the address they are under,
    /// and whether they are served by their keys there.
    far: Vec<(String, String, bool)>,
    /// Where pictures read from the server are kept.
    cache: Option<PathBuf>,
}

/// The person's cache of the app's pictures. A browser keeps its own.
#[cfg(not(target_family = "wasm"))]
fn cache() -> Option<PathBuf> {
    Some(dirs::cache_dir()?.join(PTable::read(data::APP).get("name")))
}

#[cfg(target_family = "wasm")]
fn cache() -> Option<PathBuf> {
    None
}

/// Whether a path stays inside the folder it is read from: no step
/// back up in it.
fn inside(path: &str) -> bool {
    path.split(['/', '\\']).all(|part| part != "..")
}

/// What a picture of Phiegg's is called on the server: the first rule
/// of `data/media.tsv` that fits its path, else the path itself.
pub fn key(path: &str) -> String {
    let rule = data::row(data::MEDIA).into_iter().find_map(|cell| {
        let rest = path.strip_prefix(*cell.first()?)?;
        let into = cell.get(1)?;
        match (cell.get(2), cell.get(3)) {
            (Some(end), Some(made)) => Some(format!("{into}{}{made}", rest.strip_suffix(*end)?)),
            _ => Some(format!("{into}{rest}")),
        }
    });
    rule.unwrap_or_else(|| path.to_string())
}

impl Hosted {
    /// The folders `data/asset.tsv` gives, from this crate's own, and
    /// the app's server.
    pub fn own() -> Hosted {
        let row = data::row(data::ASSET);
        let place = row.iter().filter(|cell| cell.first() != Some(&"far")).filter_map(|cell| Some((cell.first()?.to_string(), PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(cell.get(1)?))));
        let server = PTable::read(data::APP).get("server").to_string();
        let far = row.iter().filter(|cell| cell.first() == Some(&"far")).filter_map(|cell| Some((cell.get(1)?.to_string(), format!("{server}{}", cell.get(2)?), cell.get(3).is_some())));
        Hosted { place: place.collect(), far: far.collect(), cache: cache() }
    }

    /// The same, with nothing read from this disk: as a copy of the
    /// program on another machine reads its pictures.
    pub fn far_only(self) -> Hosted {
        Hosted { place: Vec::new(), ..self }
    }

    /// Where a key's picture is on the app's server; nothing for a key
    /// of no app there, and for one that would leave its folder.
    pub fn address(&self, key: &str) -> Option<String> {
        let (app, path) = key.split_once(':')?;
        let (_, under, keyed) = self.far.iter().find(|(known, ..)| known == app)?;
        let path = path.trim_start_matches('/');
        let named = match keyed {
            true => self::key(path),
            false => path.to_string(),
        };
        inside(path).then(|| format!("{under}{named}"))
    }

    /// A key's picture read from the server, kept in the cache; read from
    /// the cache where it was kept before.
    fn far(&self, key: &str) -> Option<Vec<u8>> {
        let (app, path) = key.split_once(':')?;
        let kept = self.cache.as_ref().filter(|_| inside(path)).map(|cache| cache.join(app).join(path.trim_start_matches('/')));
        if let Some(bytes) = kept.as_ref().and_then(|file| std::fs::read(file).ok()) {
            return Some(bytes);
        }
        let bytes = crate::net::fetch(&self.address(key)?)?;
        kept.iter().for_each(|file| {
            file.parent().iter().for_each(|folder| _ = std::fs::create_dir_all(folder));
            _ = std::fs::write(file, &bytes);
        });
        Some(bytes)
    }

    /// Where the file of a key is; nothing for a key of no app here, and
    /// for one that would leave its app's folder.
    pub fn path(&self, key: &str) -> Option<PathBuf> {
        let (app, path) = key.split_once(':')?;
        let folder = self.place.iter().find(|(known, _)| known == app).map(|(_, folder)| folder)?;
        inside(path).then(|| folder.join(path.trim_start_matches('/')))
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
    /// From this disk where it is here, else from the app's server.
    fn load(&self, path: &str) -> Option<Vec<u8>> {
        self.path(path).and_then(|file| std::fs::read(file).ok()).or_else(|| self.far(path))
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
