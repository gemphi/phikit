//! `load_blend_scene` — open + parse a `.blend` file and settle any failure
//! into a categorised [`SceneLoadError`]. Split out of `scene/mod.rs` so the
//! parent stays under the 200-line cap.

use super::{SceneLoadError, SceneModel, scene_from_blend};
use crate::importer::blend::BlendImporter;
use std::path::Path;

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
    load_blend_scene_with_progress(path, &|_, _| true)
}

/// [`load_blend_scene`] with a progress sink. `progress(fraction, stage)`
/// is called from the reading thread; return `false` to cancel, which
/// settles as [`SceneLoadError::Cancelled`].
///
/// Opening a file never writes beside it (Blender parity: `WM_file_read`
/// has no on-disk side effects). The former `.phx` sidecar fast-path was a
/// workaround for the O(n²) pointer remap, which is now a hash lookup.
pub fn load_blend_scene_with_progress(
    path: &Path,
    progress: &(dyn Fn(f32, &str) -> bool + Sync),
) -> Result<SceneModel, SceneLoadError> {
    let source_path = path.to_string_lossy().into_owned();

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
    let model = scene_from_blend(&scene, &source_path);
    progress(1.0, "Done");
    Ok(model)
}

/// Test-only hook exposing the private [`classify_native_error`] so the
/// sibling `tests_load` file can assert its category mapping without
/// widening the production surface.
#[cfg(test)]
pub(super) fn classify_native_error_test_hook(
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
