//! `load_blend_scene` + error-classification tests. Split out of the parent
//! `mod tests` so this file stays under the 200-line cap.

use super::{SceneLoadError, load::classify_native_error_test_hook, load_blend_scene};
use std::path::PathBuf;

#[test]
fn load_blend_scene_missing_file_is_inaccessible() {
    // A path that cannot exist on any platform: nonexistent file under a
    // nonexistent directory. metadata() fails, so we get Inaccessible and
    // never touch the parser.
    let mut path = std::env::temp_dir();
    path.push("blend_file_mesh_loading_does_not_exist");
    path.push("nope_12345.blend");

    match load_blend_scene(&path) {
        Err(SceneLoadError::Inaccessible(_)) => {}
        other => panic!("expected Err(Inaccessible), got {other:?}"),
    }
}

#[test]
fn load_blend_scene_directory_is_inaccessible_or_malformed_never_panics() {
    // Handing a directory to the loader must not panic; the temp dir exists
    // so metadata() succeeds, and the subsequent read fails as an IO error
    // (Inaccessible) — either way it is an Err, never a panic and never a
    // process termination.
    let dir: PathBuf = std::env::temp_dir();
    match load_blend_scene(&dir) {
        Err(_) => {}
        Ok(model) => panic!("expected Err for a directory, got Ok({model:?})"),
    }
}

#[test]
fn classify_native_error_io_is_inaccessible() {
    let io = crate::phiphx::PointCloudError::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "missing",
    ));
    assert!(matches!(
        classify_native_error_test_hook(io),
        SceneLoadError::Inaccessible(_)
    ));
}

#[test]
fn classify_native_error_unsupported_by_message() {
    let e = crate::phiphx::PointCloudError::ImportError("unsupported blend version".to_string());
    assert!(matches!(
        classify_native_error_test_hook(e),
        SceneLoadError::Unsupported(_)
    ));
}

#[test]
fn classify_native_error_defaults_to_malformed() {
    let e = crate::phiphx::PointCloudError::InvalidFormat("bad SDNA block".to_string());
    assert!(matches!(
        classify_native_error_test_hook(e),
        SceneLoadError::Malformed(_)
    ));
}
