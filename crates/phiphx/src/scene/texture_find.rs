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
        return hit.or_else(|| pointed(&key));
    }
    let built = index(dir);
    let hit = built.get(&key).cloned();
    if let Ok(mut i) = INDEX.write() {
        *i = Some((dir.to_path_buf(), built));
    }
    hit.or_else(|| pointed(&key))
}

/// The folders the user pointed at (File - External Data - Find Missing
/// Files), each with its files by lower-cased name: Blender's command,
/// over a folder of the user's choosing. They are searched after the
/// `.blend`'s own folder and stay for the session, so a file reopened, or
/// the next one from the same set, finds its images there too.
static POINTED: RwLock<Vec<(PathBuf, HashMap<String, PathBuf>)>> = RwLock::new(Vec::new());

/// Search `dir` too from here on. Answers how many files were listed
/// under it; a folder already pointed at is listed again, so files copied
/// into it since are seen.
pub fn point_at(dir: &Path) -> usize {
    let mut listed = HashMap::new();
    walk(dir, 0, &mut listed);
    let count = listed.len();
    if let Ok(mut all) = POINTED.write() {
        all.retain(|(known, _)| known != dir);
        all.push((dir.to_path_buf(), listed));
    }
    // The file's own index answered "not there" for these names before.
    if let Ok(mut i) = INDEX.write() {
        *i = None;
    }
    count
}

/// `key` (a lower-cased file name) under a pointed-at folder, newest first.
fn pointed(key: &str) -> Option<PathBuf> {
    POINTED.read().ok()?.iter().rev().find_map(|(_, listed)| listed.get(key).cloned())
}

/// Every file under `dir` (to `DEPTH`) by lower-cased name; the first seen
/// wins, and `textures/` is walked first so a deliberate copy there beats a
/// stray one elsewhere.
fn index(dir: &Path) -> HashMap<String, PathBuf> {
    let dir = rooted(dir);
    let mut out = HashMap::new();
    walk(&dir.join("textures"), 0, &mut out);
    walk(dir, 0, &mut out);
    out
}

/// The folder to list for `dir`. A file opened by its bare name
/// (`phiber -b pose.blend`) has the empty path for a parent, which joins
/// well (`textures` beside it was found) and cannot be listed: the walk of
/// the file's own folder found nothing, so a sky image in `sky/` beside the
/// file was reported missing. The empty path is the current folder.
fn rooted(dir: &Path) -> &Path {
    match dir.as_os_str().is_empty() {
        true => Path::new("."),
        false => dir,
    }
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

    /// A file opened by its bare name has the empty path for a folder; the
    /// walk lists the current folder for it, and leaves any other alone.
    #[test]
    fn the_empty_folder_is_the_current_one() {
        assert_eq!(rooted(Path::new("")), Path::new("."));
        assert_eq!(rooted(Path::new("poses")), Path::new("poses"));
        assert!(std::fs::read_dir(rooted(Path::new(""))).is_ok(), "and it can be listed");
    }
}
