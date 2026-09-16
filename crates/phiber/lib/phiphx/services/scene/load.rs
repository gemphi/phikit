//! `load_blend_scene` — open + parse a `.blend` file and settle any failure
//! into a categorised [`SceneLoadError`]. Split out of `scene/mod.rs` so the
//! parent stays under the 200-line cap.

use super::{SceneLoadError, SceneModel, scene_from_blend};
use crate::phiphx::importer::blend::BlendImporter;
use crate::phx::exporter::scene::import_scene_from_bytes;
use std::path::Path;

/// If a fresh `.phx` sidecar sits next to `path`, load the scene from it
/// (skipping the O(n²) SDNA block walk + fan-triangulation the native
/// `.blend` parser has to do). Returns `None` when no sidecar exists, the
/// sidecar is older than the source, or the sidecar decode fails — the
/// caller then falls back to the native parser.
///
/// This is the file-load equivalent of the `.phx`-first strategy the
/// `BlendImporter::import` progress path already uses; wiring it here means
/// every runtime scene load benefits, not just the asset-import flow.
fn try_phx_sidecar(path: &Path) -> Option<SceneModel> {
    let sidecar = path.with_extension("phx");
    let (phx_meta, blend_meta) =
        (std::fs::metadata(&sidecar).ok()?, std::fs::metadata(path).ok()?);
    if phx_meta.modified().ok()? < blend_meta.modified().ok()? {
        return None; // sidecar stale
    }
    let bytes = std::fs::read(&sidecar).ok()?;
    let phx = import_scene_from_bytes(&bytes).ok()?;
    let source_path = path.to_string_lossy().into_owned();
    Some(super::import::scene_from_phx(&phx, &source_path))
}

/// Import a `.blend` file at `path` and map it into a renderable
/// [`SceneModel`].
///
/// This wraps [`BlendImporter::import_blend_native`] (which opens and parses
/// the file) plus [`scene_from_blend`], translating every failure mode into a
/// categorized [`SceneLoadError`] and guaranteeing the host process is never
/// terminated (Requirement 3.1, 3.2, 3.5, 7.1):
///
/// - The file's accessibility is checked first: if it cannot be opened/read
///   (e.g. it does not exist or permission is denied) the function returns
///   [`SceneLoadError::Inaccessible`] carrying the underlying IO message.
/// - Parse-structure failures map to [`SceneLoadError::Malformed`]; a native
///   importer error is classified into `Malformed` / `Unsupported` /
///   `Inaccessible` by [`classify_native_error`].
/// - A parser panic is caught and mapped to `Malformed` — the process never
///   terminates.
/// - When the file parses but contains no mesh objects, the function returns
///   `Ok` with a zero-object [`SceneModel`] whose `source_path` is set
///   (Requirement 3.5, 7.2).
///
/// Requirements: 2.4, 3.1, 3.2, 3.5, 7.1, 7.2
pub fn load_blend_scene(path: &Path) -> Result<SceneModel, SceneLoadError> {
    let source_path = path.to_string_lossy().into_owned();

    // 1. Accessibility gate.
    if let Err(e) = std::fs::metadata(path) {
        return Err(SceneLoadError::Inaccessible(e.to_string()));
    }

    // 2. Fast-path: if a fresh `.phx` sidecar exists, load from it. This
    //    skips the native SDNA walk entirely (documented O(n²) block
    //    lookups, dominant cost on any non-trivial `.blend`). The sidecar
    //    stores already-triangulated indices so fan-triangulation is
    //    skipped too. Every successful native import writes the sidecar
    //    (see `BlendImporter::import_blend_native`), so subsequent loads
    //    of the same file are near-instant.
    if let Some(model) = try_phx_sidecar(path) {
        return Ok(model);
    }

    // 3. Parse under catch_unwind so a parser panic maps to Malformed.
    let parsed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        BlendImporter::import_blend_native(path)
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
    Ok(scene_from_blend(&scene, &source_path))
}

/// Test-only hook exposing the private [`classify_native_error`] so the
/// sibling `tests_load` file can assert its category mapping without
/// widening the production surface.
#[cfg(test)]
pub(super) fn classify_native_error_test_hook(
    err: crate::phiphx::PointCloudError,
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
fn classify_native_error(err: crate::phiphx::PointCloudError) -> SceneLoadError {
    if let crate::phiphx::PointCloudError::Io(_) = err {
        return SceneLoadError::Inaccessible(err.to_string());
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
