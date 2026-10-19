//! Find Missing Files, as Blender does it — by file name, under a folder,
//! recursively (`BKE_bpath_missing_files_find`, `missing_files_find__recursive`
//! in `bpath.cc`: walk the directory, compare the basename, take the first
//! match). Blender runs it on command (File ▸ External Data ▸ Find Missing
//! Files) over a folder the user picks; this runs it automatically over the
//! `.blend`'s own folder — the folder anyone who has just copied a file and
//! its textures together has them in — and over a `textures` folder beside
//! it. That is the divergence, recorded here and in the read report, which
//! says which images were found this way.
//!
//! The CW-POSES files are the case: 304 images, 15 packed, the rest at
//! `E:\Graphigem\…` or `//../../../Female-01-Texture-File-…` — paths of the
//! machine they were made on. Beside the file, or under `textures/`, is
//! where a copy would be put.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

/// How deep under the root the walk goes. Blender's walk is unbounded; a
/// bound keeps an accidental open of a file in a home folder from indexing
/// the disk.
const DEPTH: usize = 4;

/// The one index: the folder it was built for and, by lower-cased file
/// name, the first path found with that name (Blender compares with
/// `BLI_path_cmp`, case-insensitive on Windows).
static INDEX: RwLock<Option<(PathBuf, HashMap<String, PathBuf>)>> = RwLock::new(None);

/// A file named `name` under `dir`'s tree or its `textures` folder, if one
/// is there. The index is built on the first ask for a folder and kept
/// until another folder is asked about.
pub fn find(dir: &Path, name: &str) -> Option<PathBuf> {
    if name.is_empty() {
        return None;
    }
    let key = name.to_lowercase();
    if let Some(hit) = INDEX.read().ok().and_then(|i| i.as_ref().filter(|(d, _)| d == dir).map(|(_, m)| m.get(&key).cloned())) {
        return hit;
    }
    let built = index(dir);
    let hit = built.get(&key).cloned();
    if let Ok(mut i) = INDEX.write() {
        *i = Some((dir.to_path_buf(), built));
    }
    hit
}

/// Every file under `dir` (to `DEPTH`) by lower-cased name; the first seen
/// wins, and `textures/` is walked first so a deliberate copy there beats a
/// stray one elsewhere.
fn index(dir: &Path) -> HashMap<String, PathBuf> {
    let mut out = HashMap::new();
    walk(&dir.join("textures"), 0, &mut out);
    walk(dir, 0, &mut out);
    out
}

fn walk(dir: &Path, depth: usize, out: &mut HashMap<String, PathBuf>) {
    if depth > DEPTH {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    let mut folders = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            folders.push(path);
        } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            out.entry(name.to_lowercase()).or_insert(path);
        }
    }
    for folder in folders {
        walk(&folder, depth + 1, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_texture_moved_into_a_subfolder_is_found_by_its_name_whatever_its_case() {
        let root = std::env::temp_dir().join(format!("phiphx_find_{}", std::process::id()));
        let deep = root.join("textures").join("Female-01").join("skin");
        std::fs::create_dir_all(&deep).unwrap();
        std::fs::write(deep.join("Body_Diffuse.PNG"), b"x").unwrap();
        assert_eq!(find(&root, "body_diffuse.png"), Some(deep.join("Body_Diffuse.PNG")));
        assert_eq!(find(&root, "nothing_here.png"), None);
        // A second folder rebuilds the index rather than answering from the first.
        let other = root.join("elsewhere");
        std::fs::create_dir_all(&other).unwrap();
        assert_eq!(find(&other, "body_diffuse.png"), None);
        let _ = std::fs::remove_dir_all(&root);
    }
}
