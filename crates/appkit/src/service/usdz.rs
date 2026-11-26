//! A USD file as one package (`.usdz`), as Apple's Quick Look opens it on
//! a Mac, an iPhone or a Vision Pro: a zip whose files are stored as they
//! are, never squeezed, each one's bytes starting on a 64-byte boundary,
//! the root layer first. Into it goes the layer and every file it names
//! that is on disk (the things it references, its textures), and theirs,
//! as deep as they go, each by its own name side by side, the names in
//! the layers put as the package keeps them. A `.usda` root can be turned
//! first, so it faces as it was seen.

use std::path::{Path, PathBuf};

use super::usda;

/// What a package is written as: the ending of its file.
pub const ENDING: &str = "usdz";

/// The package of a USD file, its root turned about the up axis by `yaw`
/// radians where it is a layer of text.
pub fn package(root: &Path, yaw: f32) -> std::io::Result<Vec<u8>> {
    let file = gather(root)?;
    let named: Vec<(PathBuf, String)> = file.iter().map(|(path, name, _)| (path.clone(), name.clone())).collect();
    let mut entry = Vec::new();
    for (index, (path, name, bytes)) in file.into_iter().enumerate() {
        let bytes = match is_layer(&path) {
            true => {
                let text = String::from_utf8_lossy(&bytes).to_string();
                // Each name it gives, put as the package keeps that file.
                let to: Vec<(String, String)> = usda::named(&text).into_iter().filter_map(|given| {
                    let whole = resolve(&path, &given);
                    named.iter().find(|(p, _)| *p == whole).map(|(_, kept)| (given, format!("./{kept}")))
                }).collect();
                let text = usda::renamed(&text, &to);
                let text = match index {
                    0 => usda::turned(&text, yaw),
                    _ => text,
                };
                text.into_bytes()
            }
            false => bytes,
        };
        entry.push((name, bytes));
    }
    Ok(zip(&entry))
}

/// The root and every file it names on disk, as deep as they go: each
/// path, the name the package keeps it by, and its bytes. The root first.
fn gather(root: &Path) -> std::io::Result<Vec<(PathBuf, String, Vec<u8>)>> {
    let mut out: Vec<(PathBuf, String, Vec<u8>)> = Vec::new();
    let mut queue = vec![std::fs::canonicalize(root)?];
    while let Some(path) = queue.pop() {
        if out.iter().any(|(p, _, _)| *p == path) {
            continue;
        }
        let bytes = std::fs::read(&path)?;
        if is_layer(&path) {
            let text = String::from_utf8_lossy(&bytes);
            let mut more: Vec<PathBuf> = usda::named(&text).iter().map(|given| resolve(&path, given)).filter(|p| p.is_file()).collect();
            more.reverse();
            queue.extend(more);
        }
        let name = kept_name(&path, &out);
        out.push((path, name, bytes));
    }
    Ok(out)
}

/// A name for a file no other file of the package has: its own, or
/// with a number before it.
fn kept_name(path: &Path, taken: &[(PathBuf, String, Vec<u8>)]) -> String {
    let own = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    let free = |name: &str| !taken.iter().any(|(_, n, _)| n == name);
    std::iter::once(own.clone()).chain((1..).map(|k| format!("{k}-{own}"))).find(|name| free(name)).unwrap_or(own)
}

/// A name a layer gives, as a path from the layer's own folder.
fn resolve(layer: &Path, given: &str) -> PathBuf {
    let folder = layer.parent().unwrap_or(Path::new(""));
    let joined = folder.join(given.trim_start_matches("./"));
    std::fs::canonicalize(&joined).unwrap_or(joined)
}

fn is_layer(path: &Path) -> bool {
    path.extension().is_some_and(|e| e.eq_ignore_ascii_case("usda"))
}

/// The files as a zip, stored, each one's bytes on a 64-byte boundary.
pub fn zip(entry: &[(String, Vec<u8>)]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut central = Vec::new();
    for (name, bytes) in entry {
        let offset = out.len() as u32;
        let crc = crc32(bytes);
        // A field of padding, so the bytes start on the boundary.
        let before = out.len() + 30 + name.len() + 4;
        let pad = (64 - before % 64) % 64;
        let mut extra = vec![0x86, 0x19];
        extra.extend((pad as u16).to_le_bytes());
        extra.extend(std::iter::repeat_n(0u8, pad));
        let head = |out: &mut Vec<u8>, central: bool| {
            out.extend(match central {
                true => 0x0201_4b50u32,
                false => 0x0403_4b50,
            }.to_le_bytes());
            match central {
                true => out.extend([20u8, 0]),
                false => {}
            }
            // Made by, needed, no flags, stored, a time and a day of 1980.
            out.extend([20u8, 0, 0, 0, 0, 0, 0, 0, 0x21, 0]);
            out.extend(crc.to_le_bytes());
            out.extend((bytes.len() as u32).to_le_bytes());
            out.extend((bytes.len() as u32).to_le_bytes());
            out.extend((name.len() as u16).to_le_bytes());
        };
        head(&mut out, false);
        out.extend((extra.len() as u16).to_le_bytes());
        out.extend(name.as_bytes());
        out.extend(&extra);
        out.extend(bytes);
        head(&mut central, true);
        // No extra field, comment, disk, or attributes; where it starts.
        central.extend([0u8; 2 + 2 + 2 + 2 + 4]);
        central.extend(offset.to_le_bytes());
        central.extend(name.as_bytes());
    }
    let (start, size, count) = (out.len() as u32, central.len() as u32, entry.len() as u16);
    out.extend(central);
    out.extend(0x0605_4b50u32.to_le_bytes());
    out.extend([0u8; 4]);
    out.extend(count.to_le_bytes());
    out.extend(count.to_le_bytes());
    out.extend(size.to_le_bytes());
    out.extend(start.to_le_bytes());
    out.extend([0u8; 2]);
    out
}

/// The zip's check of a file's bytes (CRC-32, the one zip uses).
fn crc32(bytes: &[u8]) -> u32 {
    !bytes.iter().fold(!0u32, |crc, byte| {
        (0..8).fold(crc ^ *byte as u32, |crc, _| match crc & 1 {
            1 => (crc >> 1) ^ 0xEDB8_8320,
            _ => crc >> 1,
        })
    })
}
