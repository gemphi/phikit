//! The embedded asset set matches the files on disk, and stays small.

use super::web_assets::{ROOTS, WebAssets};
use gpui::AssetSource;
use std::path::PathBuf;

fn assets() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
}

/// Every embedded root holds exactly the files its folder holds, byte for
/// byte. This is the check that makes the embedded set trustworthy: a browser
/// build that silently shipped stale or truncated icons would look like a
/// rendering bug, not an asset bug.
#[test]
fn the_embedded_assets_match_the_files_on_disk() {
    let web = WebAssets;
    let mut checked = 0;
    for (name, _) in ROOTS {
        let dir = assets().join(name);
        let entries = std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("read {name}: {e}"));
        for entry in entries.flatten().filter(|e| e.path().is_file()) {
            let file = entry.file_name().into_string().expect("utf-8 name");
            let want = std::fs::read(entry.path()).expect("read file");
            let got = web
                .load(&format!("{name}/{file}"))
                .expect("load")
                .unwrap_or_else(|| panic!("{name}/{file} is not embedded"));
            assert_eq!(got.as_ref(), want.as_slice(), "{name}/{file} differs from disk");
            checked += 1;
        }
    }
    assert!(checked > 50, "only {checked} files checked — the roots look wrong");
}

/// `list` names the files in a root, as the desktop source's `read_dir` does.
#[test]
fn listing_a_root_names_its_files() {
    let web = WebAssets;
    let icons = web.list("icons").expect("list icons");
    assert!(!icons.is_empty(), "icons listed nothing");
    let on_disk = std::fs::read_dir(assets().join("icons"))
        .expect("read icons")
        .flatten()
        .filter(|e| e.path().is_file())
        .count();
    assert_eq!(icons.len(), on_disk, "listing and folder disagree");
}

/// An unknown path and an unembedded root are absent, not an error — the
/// desktop source's failed `fs::read` means the same thing to gpui.
#[test]
fn an_unknown_asset_is_absent_rather_than_an_error() {
    let web = WebAssets;
    assert!(web.load("icons/nothing-here.svg").expect("load").is_none());
    assert!(web.load("production/LAND-TYPE-15-01.blend").expect("load").is_none());
    assert!(web.list("production").expect("list").is_empty());
}

/// The bundle stays a download somebody will wait for. The three scene files
/// under `production/` and `CW-Fashion/` are 455 MB between them; if one of
/// those folders ever gets embedded, this is what says so.
#[test]
fn the_embedded_set_stays_small() {
    let total: usize = ROOTS
        .iter()
        .map(|(_, dir)| {
            fn bytes(d: &include_dir::Dir<'_>) -> usize {
                d.files().map(|f| f.contents().len()).sum::<usize>()
                    + d.dirs().map(bytes).sum::<usize>()
            }
            bytes(dir)
        })
        .sum();
    let mb = total as f64 / 1_048_576.0;
    assert!(mb < 12.0, "the embedded assets are {mb:.1} MB — too much to download");
    assert!(mb > 1.0, "only {mb:.1} MB embedded — a root is probably missing");
}
