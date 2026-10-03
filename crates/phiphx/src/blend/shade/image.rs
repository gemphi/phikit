//! Image datablocks (#26): the key a material uses to name its texture and
//! the bytes of images packed into the `.blend`.
//!
//! Blender ref (`DNA_image_types.h`, `DNA_packedFile_types.h`,
//! `packedFile.cc`): `Image { filepath[1024], packedfiles (ListBase of
//! ImagePackedFile { *packedfile, filepath, view, tile_number }),
//! *packedfile (legacy) }`, `PackedFile { int size; int seek; *data }`.
//! The data is written as its own block at the old `data` address.

use crate::blend::file::types::*;
use std::sync::Arc;

/// `(image key, packed file bytes)` for every packed image in the file.
pub type PackedImages = Vec<(String, Arc<Vec<u8>>)>;

/// The folder of the `.blend` being opened, so `//`-relative image paths can
/// be checked while materials are folded. `None` (no file) treats every image
/// as present.
static SOURCE_DIR: std::sync::RwLock<Option<std::path::PathBuf>> = std::sync::RwLock::new(None);

/// Record the `.blend` being opened (`Phx::open`).
pub(crate) fn set_source(path: &std::path::Path) {
    if let Ok(mut dir) = SOURCE_DIR.write() {
        *dir = path.parent().map(std::path::Path::to_path_buf);
    }
}

impl BlendFile {
    /// Whether Blender would find this image: packed, or on disk at its path
    /// (`//` resolved against the `.blend`'s folder), or by file name beside
    /// the `.blend` — the order `scene/texture.rs` resolves them in.
    /// Blender draws a missing image as magenta (`IMB` "invalid" colour).
    pub(crate) fn image_present(&self, im: &BlendBlock, dna: &DnaType) -> bool {
        let first = self.ptr(im, self.off("Image", "packedfiles", dna));
        let pf = match self.block_at(first) {
            Some(ipf) => self.ptr(ipf, self.off("ImagePackedFile", "*packedfile", dna)),
            None => self.ptr(im, self.off("Image", "*packedfile", dna)),
        };
        if pf != 0 {
            return true;
        }
        let Some(dir) = SOURCE_DIR.read().ok().and_then(|d| d.clone()) else { return true };
        let path = self.off("Image", "filepath[1024]", dna).or_else(|| self.off("Image", "name[1024]", dna)).map_or(String::new(), |o| cstr(&im.data, o, 1024));
        if path.is_empty() {
            // Generated images (no file) always draw.
            return true;
        }
        let path = path.replace('\\', "/");
        let full = match path.strip_prefix("//") {
            Some(rel) => dir.join(rel),
            None => std::path::PathBuf::from(&path),
        };
        let name = std::path::Path::new(&path).file_name().map(|n| dir.join(n));
        full.exists() || name.is_some_and(|n| n.exists())
    }

    /// How materials refer to an image: its `filepath`, or `ID.name` (no
    /// prefix) for generated / packed images saved without a path.
    pub(crate) fn image_key(&self, im: &BlendBlock, dna: &DnaType) -> String {
        let path = self.off("Image", "filepath[1024]", dna).or_else(|| self.off("Image", "name[1024]", dna)).map_or(String::new(), |o| cstr(&im.data, o, 1024));
        if !path.is_empty() {
            return path;
        }
        let o = self.id_name_offset(dna).unwrap_or(usize::MAX);
        let name = cstr(&im.data, o, 66);
        name.get(2..).unwrap_or_default().to_string()
    }

    fn packed_bytes(&self, pf_addr: u64, dna: &DnaType) -> Option<Vec<u8>> {
        let pf = self.block_at(pf_addr)?;
        let size = self.int_field(pf, "PackedFile", "size", dna) as usize;
        let data = self.block_at(self.ptr(pf, self.off("PackedFile", "*data", dna)))?;
        (size > 0 && size <= data.data.len()).then(|| data.data[..size].to_vec())
    }

    /// Packed image bytes keyed like [`Self::image_key`].
    pub(crate) fn packed_images(&self, dna: &DnaType) -> PackedImages {
        let mut out = PackedImages::new();
        for im in self.blocks.iter().filter(|b| b.code_str == "IM") {
            let bytes = self.scoped(im, || {
                let first = self.ptr(im, self.off("Image", "packedfiles", dna));
                let pf = match self.block_at(first) {
                    Some(ipf) => self.ptr(ipf, self.off("ImagePackedFile", "*packedfile", dna)),
                    None => self.ptr(im, self.off("Image", "*packedfile", dna)),
                };
                self.packed_bytes(pf, dna)
            });
            if let Some(bytes) = bytes {
                out.push((self.image_key(im, dna), Arc::new(bytes)));
            }
        }
        out
    }
}

fn cstr(data: &[u8], off: usize, max: usize) -> String {
    if off >= data.len() {
        return String::new();
    }
    let s = &data[off..(off + max).min(data.len())];
    let n = s.iter().position(|&b| b == 0).unwrap_or(s.len());
    String::from_utf8_lossy(&s[..n]).to_string()
}
