//! **The phiphx front door.** Everything a host does with a file goes
//! through [`Phx`]: open it into a [`SceneModel`], peek at its preview
//! without parsing, or detect its format. Parts live in `crate::blend`,
//! `crate::import`, `crate::scene`; callers never reach into them.
//! Status and next steps: `phx/jobs.md` moved to `src/jobs.md`.

use crate::blend::BlendFile;
use crate::import::FileFormat;
use std::path::Path;

pub use crate::blend::BlendQuickInfo as Peek;
pub use crate::blend::{CpuSkin, SKIN_MIN, SkinJob, SkinPair, Skinner};
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
        crate::blend::set_image_source(path);
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
        crate::blend::set_image_source(path);
        crate::blend::with_progressive_open(|| blend_progressive(path, progress))
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
        crate::blend::clear_image_source();
        blend_bytes(name, data, progress)
    }

    /// Install the batch skinner used for Armature deform while opening files
    /// (Phiber installs a GPU one; `None` restores the built-in CPU path).
    pub fn use_skinner(skinner: Option<std::sync::Arc<dyn Skinner>>) {
        crate::blend::set_skinner(skinner);
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
        crate::blend::supported_modifiers().collect()
    }

    /// Format from magic bytes, else from the extension.
    pub fn detect(path: &Path, head: &[u8]) -> Option<FileFormat> {
        FileFormat::detect(head).or_else(|| {
            let ext = path.extension()?.to_str()?;
            FileFormat::from_extension(ext)
        })
    }
}

// ── Opening a `.blend` ────────────────────────────────────────────────────
use crate::scene::SceneLoadError;
use crate::blend::BlendImporter;

/// Read `path` into a [`SceneModel`]. `progress` is called from the
/// reading thread; returning `false` settles as [`SceneLoadError::Cancelled`].
/// Accessibility is checked first; parser failures map to `Malformed` /
/// `Unsupported` / `Inaccessible`; a parser panic is caught (→ `Malformed`);
/// a file without meshes is still `Ok` with an empty scene.
///
/// Opening a file never writes beside it (Blender parity: `WM_file_read`
/// has no on-disk side effects). The former `.phx` sidecar fast-path was a
/// workaround for the O(n²) pointer remap, which is now a hash lookup.
pub(super) fn blend(path: &Path, progress: Progress) -> Result<SceneModel, SceneLoadError> {
    // 1. Accessibility gate.
    if let Err(e) = std::fs::metadata(path) {
        return Err(SceneLoadError::Inaccessible(e.to_string()));
    }
    let started = web_time::Instant::now();
    let scene = guarded(|| BlendImporter::import_blend_native_with_progress(path, progress))?;
    settle(scene, &path.to_string_lossy(), started, progress)
}

/// [`blend`] from bytes already in memory ([`Phx::open_bytes`]).
///
/// No accessibility gate, because there is no file to be inaccessible —
/// `name` is a label, not a path. Everything after the read is shared with
/// [`blend`], which is the point: one reader, two ways in.
pub(super) fn blend_bytes(
    name: &str,
    data: &[u8],
    progress: Progress,
) -> Result<SceneModel, SceneLoadError> {
    let started = web_time::Instant::now();
    let scene = guarded(|| BlendImporter::import_blend_bytes_with_progress(data, progress))?;
    settle(scene, name, started, progress)
}

/// Parse under `catch_unwind`, so a parser panic on a malformed file settles
/// as [`SceneLoadError::Malformed`] rather than taking the host down.
fn guarded(
    read: impl FnOnce() -> crate::Result<crate::blend::BlendScene>,
) -> Result<crate::blend::BlendScene, SceneLoadError> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(read)) {
        Ok(Ok(scene)) => Ok(scene),
        Ok(Err(err)) => Err(classify_native_error(err)),
        Err(_panic) => Err(SceneLoadError::Malformed(
            "parser panicked while reading blend structure".to_string(),
        )),
    }
}

/// Map a parsed scene to a [`SceneModel`] and fill its report. A zero-object
/// result is still a success.
fn settle(
    scene: crate::blend::BlendScene,
    source: &str,
    started: web_time::Instant,
    progress: Progress,
) -> Result<SceneModel, SceneLoadError> {
    if !progress(0.97, "Building scene") {
        return Err(SceneLoadError::Cancelled);
    }
    let mut model = SceneModel::from_blend_owned(scene, source);
    model.report.duration_ms = started.elapsed().as_millis() as u64;
    model.report.memory_bytes = model.heap_bytes();
    progress(1.0, "Done");
    Ok(model)
}

/// Like [`blend`], but the modifier work is captured rather than applied:
/// returns the base [`SceneModel`] and the [`DeferredModifiers`] to finish
/// later. Must run inside `with_progressive_open` (so the reader defers);
/// [`Phx::open_progressive`] arranges that.
pub(super) fn blend_progressive(
    path: &Path,
    progress: Progress,
) -> Result<(SceneModel, Option<crate::scene::DeferredModifiers>), SceneLoadError> {
    let source_path = path.to_string_lossy().into_owned();
    let started = web_time::Instant::now();

    if let Err(e) = std::fs::metadata(path) {
        return Err(SceneLoadError::Inaccessible(e.to_string()));
    }

    let scene = guarded(|| BlendImporter::import_blend_native_with_progress(path, progress))?;

    if !progress(0.97, "Building scene") {
        return Err(SceneLoadError::Cancelled);
    }
    let (mut model, deferred) = SceneModel::from_blend_progressive(scene, &source_path);
    model.report.duration_ms = started.elapsed().as_millis() as u64;
    model.report.memory_bytes = model.heap_bytes();
    progress(1.0, "Done");
    Ok((model, deferred))
}

/// Test-only hook exposing the private [`classify_native_error`] so the
/// sibling `tests_load` file can assert its category mapping without
/// widening the production surface.
#[cfg(test)]
pub(crate) fn classify_native_error_test_hook(
    err: crate::PointCloudError,
) -> SceneLoadError {
    classify_native_error(err)
}

/// Classify a native importer error into a [`SceneLoadError`] category.
///
/// An IO failure that slipped past the accessibility gate (e.g. the file was
/// removed between the `metadata` check and the read) still maps to
/// [`SceneLoadError::Inaccessible`]. A message that clearly names an
/// unrecognized/unsupported format maps to [`SceneLoadError::Unsupported`];
/// everything else defaults to [`SceneLoadError::Malformed`].
fn classify_native_error(err: crate::PointCloudError) -> SceneLoadError {
    if let crate::PointCloudError::Io(_) = err {
        return SceneLoadError::Inaccessible(err.to_string());
    }
    if let crate::PointCloudError::Cancelled = err {
        return SceneLoadError::Cancelled;
    }
    let msg = err.to_string();
    let lowered = msg.to_lowercase();
    if lowered.contains("unsupported")
        || lowered.contains("unrecognized")
        || lowered.contains("not supported")
    {
        SceneLoadError::Unsupported(msg)
    } else {
        SceneLoadError::Malformed(msg)
    }
}
