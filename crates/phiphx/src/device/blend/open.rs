//! Opening a `.blend`: the readers behind [`super::BlendDevice`] and `Phx::open`.
//! A read is guarded (a parser panic settles as `Malformed`), then the parsed
//! scene is settled into a `SceneModel`.
use crate::scene::{SceneLoadError, SceneModel};
use super::BlendImporter;
use crate::phx::Progress;
use std::path::Path;

/// Read `path` into a [`SceneModel`]. `progress` is called from the
/// reading thread; returning `false` settles as [`SceneLoadError::Cancelled`].
/// Accessibility is checked first; parser failures map to `Malformed` /
/// `Unsupported` / `Inaccessible`; a parser panic is caught (→ `Malformed`);
/// a file without meshes is still `Ok` with an empty scene.
///
/// Opening a file never writes beside it (Blender parity: `WM_file_read`
/// has no on-disk side effects). The former `.phx` sidecar fast-path was a
/// workaround for the O(n²) pointer remap, which is now a hash lookup.
pub(crate) fn blend(path: &Path, progress: Progress) -> Result<SceneModel, SceneLoadError> {
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
pub(crate) fn blend_bytes(
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
    read: impl FnOnce() -> crate::Result<super::BlendScene>,
) -> Result<super::BlendScene, SceneLoadError> {
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
    scene: super::BlendScene,
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
/// returns the base [`SceneModel`] and the [`DeferredModifierList`] to finish
/// later. Must run inside `with_progressive_open` (so the reader defers);
/// [`Phx::open_progressive`] arranges that.
pub(crate) fn blend_progressive(
    path: &Path,
    progress: Progress,
) -> Result<(SceneModel, Option<crate::scene::DeferredModifierList>), SceneLoadError> {
    let source_path = path.to_string_lossy().into_owned();
    let started = web_time::Instant::now();

    if let Err(e) = std::fs::metadata(path) {
        return Err(SceneLoadError::Inaccessible(e.to_string()));
    }

    let scene = guarded(|| BlendImporter::import_blend_native_with_progress(path, progress))?;

    settle_progressive(scene, &source_path, started, progress)
}
/// [`blend_progressive`] from bytes already in memory. Same rules: run it
/// inside `with_progressive_open`.
pub(crate) fn blend_progressive_bytes(
    name: &str,
    data: &[u8],
    progress: Progress,
) -> Result<(SceneModel, Option<crate::scene::DeferredModifierList>), SceneLoadError> {
    let started = web_time::Instant::now();
    let scene = guarded(|| BlendImporter::import_blend_bytes_with_progress(data, progress))?;
    settle_progressive(scene, name, started, progress)
}
/// What follows the read on the progressive path: the base scene and the
/// modifier work left for the host.
fn settle_progressive(
    scene: super::BlendScene,
    source: &str,
    started: web_time::Instant,
    progress: Progress,
) -> Result<(SceneModel, Option<crate::scene::DeferredModifierList>), SceneLoadError> {
    if !progress(0.97, "Building scene") {
        return Err(SceneLoadError::Cancelled);
    }
    let (mut model, deferred) = SceneModel::from_blend_progressive(scene, source);
    model.report.duration_ms = started.elapsed().as_millis() as u64;
    model.report.memory_bytes = model.heap_bytes();
    progress(1.0, "Done");
    Ok((model, deferred))
}/// Classify a native importer error into a [`SceneLoadError`] category.
///
/// An IO failure that slipped past the accessibility gate (e.g. the file was
/// removed between the `metadata` check and the read) still maps to
/// [`SceneLoadError::Inaccessible`]. A message that clearly names an
/// unrecognized/unsupported format maps to [`SceneLoadError::Unsupported`];
/// everything else defaults to [`SceneLoadError::Malformed`].
pub(crate) fn classify_native_error(err: crate::PointCloudError) -> SceneLoadError {
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
