//! `BlendImporter`: reads a `.blend` for the asset pipeline and for `Phx::open`.

use crate::device::blend::{BlendFile, BlendScene};
use crate::Result;
use std::path::Path;

/// Structure encapsulating all Blender (.blend) asset import capabilities.
pub struct BlendImporter;

impl BlendImporter {
    /// Read a .blend file and extract a BlendScene.
    /// Zstd-compressed files (Blender 4.5+ "compressed" saves, and all of the
    /// upstream test suite) are decompressed transparently, like blenloader.
    pub fn import_blend_native(path: &Path) -> Result<BlendScene> {
        Self::import_blend_native_with_progress(path, &|_, _| true)
    }

    /// Read a .blend file, reporting progress through `progress(fraction,
    /// stage)`. The callback returns `false` to cancel; a cancelled read
    /// returns [`PointCloudError::Cancelled`].
    ///
    /// Opening a file has **no side effects on disk** (Blender's
    /// `WM_file_read` never writes beside the file). The asset-import
    /// pipeline that wants a `.phx` sidecar calls
    /// [`Self::import_blend_native_with_sidecar`] instead.
    ///
    /// Stage fractions (mirroring the phases of `blo_read_file_internal`):
    /// 0.00–0.05 read bytes · 0.05–0.45 block table + DNA · 0.45–0.90 IDs
    /// (objects/meshes/materials) · 0.90–1.00 scene assembly.
    pub fn import_blend_native_with_progress(
        path: &Path,
        progress: &(dyn Fn(f32, &str) -> bool + Sync),
    ) -> Result<BlendScene> {
        use crate::PointCloudError;
        if !progress(0.0, "Reading file") {
            return Err(PointCloudError::Cancelled);
        }
        let data = std::fs::read(path)?;
        Self::import_blend_bytes_with_progress(&data, progress)
    }

    /// The same read, from bytes that are already in memory.
    ///
    /// This is where the path stopped being needed: the parser underneath
    /// has always been `parse_bytes_with_progress`, and the only thing the
    /// path did here was `std::fs::read`. Splitting the two is what lets a
    /// `.blend` arrive from somewhere that has no filesystem — a browser's
    /// file picker, an HTTP body, an asset server — without a second reader.
    ///
    /// Stage fractions start at 0.05: 0.00-0.05 was reading the file, and
    /// for a caller holding the bytes that stage has already happened.
    pub fn import_blend_bytes_with_progress(
        data: &[u8],
        progress: &(dyn Fn(f32, &str) -> bool + Sync),
    ) -> Result<BlendScene> {
        use crate::PointCloudError;
        let total = data.len().max(1) as f64;
        if !progress(0.05, "Indexing blocks") {
            return Err(PointCloudError::Cancelled);
        }

        let cancelled = std::sync::atomic::AtomicBool::new(false);
        let mut on_block = |blocks: usize, bytes: u64| {
            // Compressed files decompress to more bytes than `total`; clamp.
            let frac = (bytes as f64 / total).min(1.0) as f32;
            let label = format!("Indexing blocks ({blocks})");
            if !progress(0.05 + 0.40 * frac, &label) {
                cancelled.store(true, std::sync::atomic::Ordering::Relaxed);
            }
        };
        let blend = BlendFile::parse_bytes_with_progress(data, &mut on_block)?;
        if cancelled.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(PointCloudError::Cancelled);
        }

        let scene = blend.extract_scene_with_progress(&|f, s| progress(0.45 + 0.45 * f, s))?;
        if !progress(0.95, "Building scene") {
            return Err(PointCloudError::Cancelled);
        }
        Ok(scene)
    }

    /// Asset-import variant: read the file **and** emit a portable `.phx`
    /// sidecar beside it (best-effort; never fails the import). Only the
    /// asset pipeline (`AssetImporter`) should call this — opening a file in
    /// the editor must not write next to the user's data.
    pub fn import_blend_native_with_sidecar(path: &Path) -> Result<BlendScene> {
        let scene = Self::import_blend_native(path)?;
        scene.write_sidecar(path);
        Ok(scene)
    }
}

