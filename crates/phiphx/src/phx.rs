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

    /// Install the batch skinner used for Armature deform while opening files
    /// (Phiber installs a GPU one; `None` restores the built-in CPU path).
    pub fn use_skinner(skinner: Option<std::sync::Arc<dyn Skinner>>) {
        crate::blend::set_skinner(skinner);
    }

    /// Version, render info and embedded thumbnail — header blocks only.
    pub fn peek(path: &Path) -> crate::Result<Peek> {
        BlendFile::read_quick_info(path)
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
    let source_path = path.to_string_lossy().into_owned();
    let started = std::time::Instant::now();

    // 1. Accessibility gate.
    if let Err(e) = std::fs::metadata(path) {
        return Err(SceneLoadError::Inaccessible(e.to_string()));
    }

    // 2. Parse under catch_unwind so a parser panic maps to Malformed.
    let parsed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        BlendImporter::import_blend_native_with_progress(path, progress)
    }));

    let scene = match parsed {
        Ok(Ok(scene)) => scene,
        Ok(Err(err)) => return Err(classify_native_error(err)),
        Err(_panic) => {
            return Err(SceneLoadError::Malformed(
                "parser panicked while reading blend structure".to_string(),
            ));
        }
    };

    // 3. Map to a SceneModel. A zero-object result is still a success.
    if !progress(0.97, "Building scene") {
        return Err(SceneLoadError::Cancelled);
    }
    let mut model = SceneModel::from_blend_owned(scene, &source_path);
    model.report.duration_ms = started.elapsed().as_millis() as u64;
    model.report.memory_bytes = scene_heap_bytes(&model);
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
    let started = std::time::Instant::now();

    if let Err(e) = std::fs::metadata(path) {
        return Err(SceneLoadError::Inaccessible(e.to_string()));
    }

    let parsed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        BlendImporter::import_blend_native_with_progress(path, progress)
    }));
    let scene = match parsed {
        Ok(Ok(scene)) => scene,
        Ok(Err(err)) => return Err(classify_native_error(err)),
        Err(_panic) => {
            return Err(SceneLoadError::Malformed(
                "parser panicked while reading blend structure".to_string(),
            ));
        }
    };

    if !progress(0.97, "Building scene") {
        return Err(SceneLoadError::Cancelled);
    }
    let (mut model, deferred) = SceneModel::from_blend_progressive(scene, &source_path);
    model.report.duration_ms = started.elapsed().as_millis() as u64;
    model.report.memory_bytes = scene_heap_bytes(&model);
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

/// Approximate heap bytes held by the scene's mesh buffers (Blender's
/// `MEM_get_memory_in_use` equivalent for the data we actually keep).
fn scene_heap_bytes(model: &SceneModel) -> u64 {
    let mut bytes: u64 = 0;
    for obj in &model.objects {
        let m = &obj.mesh;
        bytes += (m.vertices.len() * std::mem::size_of::<[f32; 3]>()) as u64;
        bytes += (m.indices.len() * std::mem::size_of::<u32>()) as u64;
        bytes += match &m.normals {
            crate::scene::SceneNormals::PerVertex(v) | crate::scene::SceneNormals::PerFace(v) => {
                (v.len() * std::mem::size_of::<[f32; 3]>()) as u64
            }
        };
        bytes += (m.corners.ids.len() * std::mem::size_of::<u32>()) as u64;
        bytes += (m.corners.normals.len() * std::mem::size_of::<[f32; 3]>()) as u64;
        bytes += (m.corners.colors.len() * std::mem::size_of::<[f32; 3]>()) as u64;
        // UVs and material indices live on SceneObject, not SceneMesh.
        bytes += (obj.uvs.len() * std::mem::size_of::<[f32; 2]>()) as u64;
        bytes += (obj.material_indices.len() * std::mem::size_of::<u16>()) as u64;
    }
    bytes
}
