//! Builds the shared assets into the program: a table of every file under
//! `asset/`, by its path. An app then has its icons and its typeface
//! where there is no disk to read them from, in a browser
//! (`src/service/bundle.rs`).

use std::path::{Path, PathBuf};
use std::{env, fs};

/// Every file under a folder, however deep.
fn walk(folder: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(folder).into_iter().flatten().flatten() {
        let path = entry.path();
        match path.is_dir() {
            true => walk(&path, out),
            false => out.push(path),
        }
    }
}

fn main() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("cargo names the crate's folder")).join("asset");
    println!("cargo:rerun-if-changed={}", root.display());
    let mut file = Vec::new();
    walk(&root, &mut file);
    file.sort();
    let row = |path: &PathBuf| {
        let name = path.strip_prefix(&root).expect("a file under asset/").to_string_lossy().replace('\\', "/");
        format!("    ({name:?}, include_bytes!({:?})),\n", path.display().to_string())
    };
    let table = format!("/// Every file under `asset/`, by its path.\npub const FILE: &[(&str, &[u8])] = &[\n{}];\n", file.iter().map(row).collect::<String>());
    let out = PathBuf::from(env::var("OUT_DIR").expect("cargo names the build's folder")).join("asset.rs");
    fs::write(out, table).expect("the table of assets to be written");
}
