//! **The phiphx front door.** Everything a host does with a file goes
//! through [`Phx`]: open it into a [`SceneModel`], peek at its preview
//! without parsing, or detect its format. Parts live in `crate::device` (`device::blend` for `.blend`),
//! `crate::import`, `crate::scene`; callers never reach into them.
//! Status and next steps: `phx/jobs.md` moved to `src/jobs.md`.

use crate::device::blend::BlendFile;
use crate::import::FileFormat;
use std::path::Path;

pub use crate::device::blend::BlendQuickInfo as Peek;
pub use crate::device::blend::{CpuSkin, SKIN_MIN, SkinJob, SkinPair, Skinner};
pub use crate::scene::{SceneLoadError as PhxError, SceneModel};

/// Progress sink: `(fraction 0..=1, stage)`; return `false` to cancel.
pub type Progress<'a> = &'a (dyn Fn(f32, &str) -> bool + Sync);

/// The entry point. Stateless: every call is independent and thread-safe.
pub struct Phx;

impl Phx {
    /// Open a file into a scene (no progress, no cancel).
    pub fn open(path: &Path) -> Result<SceneModel, PhxError> {
        Self::open_with(path, &|_, _| true)
    }

    /// Open a file into a scene, reporting progress.
    pub fn open_with(path: &Path, progress: Progress) -> Result<SceneModel, PhxError> {
        crate::device::blend::set_image_source(path);
        blend(path, progress)
    }

    /// Open a file progressively: the base scene now, the modifier work
    /// deferred. Returns the base [`SceneModel`] plus a [`DeferredModifiers`]
    /// (when any object carries a modifier) for the host to finish on a worker
    /// thread and swap in — base meshes reach the viewport without waiting on
    /// subdivision, arrays or armature deform. Blender's depsgraph does the
    /// same: the file opens, then the evaluated meshes arrive.
    pub fn open_progressive(
        path: &Path,
        progress: Progress,
    ) -> Result<(SceneModel, Option<crate::scene::DeferredModifiers>), PhxError> {
        crate::device::blend::set_image_source(path);
        crate::device::blend::with_progressive_open(|| blend_progressive(path, progress))
    }

    /// Open a `.blend` that is already in memory, under the name to record
    /// as its source.
    ///
    /// For a caller with no filesystem to offer: a browser's file picker, an
    /// HTTP body, an asset server. `name` is only what the scene reports as
    /// its origin — it is never opened, and it need not exist.
    ///
    /// The one thing this cannot do is resolve an image the file refers to by
    /// path (`//textures/bark.png`): there is no folder to resolve against,
    /// so those read as missing, which is what Blender shows for an image it
    /// cannot find. Packed images are unaffected.
    pub fn open_bytes(name: &str, data: &[u8]) -> Result<SceneModel, PhxError> {
        Self::open_bytes_with(name, data, &|_, _| true)
    }

    /// [`Phx::open_bytes`], reporting progress.
    ///
    /// Progress starts at the "Indexing blocks" stage rather than "Reading
    /// file": the caller has already done the reading.
    pub fn open_bytes_with(name: &str, data: &[u8], progress: Progress) -> Result<SceneModel, PhxError> {
        crate::device::blend::clear_image_source();
        blend_bytes(name, data, progress)
    }

    /// Install the batch skinner used for Armature deform while opening files
    /// (Phiber installs a GPU one; `None` restores the built-in CPU path).
    pub fn use_skinner(skinner: Option<std::sync::Arc<dyn Skinner>>) {
        crate::device::blend::set_skinner(skinner);
    }

    /// Version, render info and embedded thumbnail — header blocks only.
    pub fn peek(path: &Path) -> crate::Result<Peek> {
        BlendFile::read_quick_info(path)
    }

    /// [`Phx::peek`] from bytes already in memory — header blocks only, so
    /// it reads the thumbnail and frame range without parsing the file.
    pub fn peek_bytes(data: &[u8]) -> crate::Result<Peek> {
        BlendFile::quick_info_bytes(data)
    }

    /// Modifier types the viewport evaluates (Blender RNA ids).
    pub fn modifiers() -> Vec<&'static str> {
        crate::device::blend::supported_modifiers().collect()
    }

    /// Format from magic bytes, else from the extension.
    pub fn detect(path: &Path, head: &[u8]) -> Option<FileFormat> {
        FileFormat::detect(head).or_else(|| {
            let ext = path.extension()?.to_str()?;
            FileFormat::from_extension(ext)
        })
    }
}

use crate::device::blend::open::{blend, blend_bytes, blend_progressive};

/// Test-only hook exposing the private [`classify_native_error`] so the
/// sibling `tests_load` file can assert its category mapping without
/// widening the production surface.
#[cfg(test)]
pub(crate) fn classify_native_error_test_hook(
    err: crate::PointCloudError,
) -> PhxError {
    crate::device::blend::open::classify_native_error(err)
}
