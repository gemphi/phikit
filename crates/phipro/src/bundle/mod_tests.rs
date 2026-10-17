//! Where a bundle's parts go, and that nothing but `path` decides.

use super::*;

fn bundle() -> BBundle {
    BBundle::new(Path::new("/assets/Rig Mk2.blend"), Path::new("/out")).expect("a usable name")
}

#[test]
fn a_bundle_is_the_assets_own_name_with_the_extension() {
    let made = bundle();
    assert_eq!(made.name, "Rig Mk2");
    assert_eq!(made.root, PathBuf::from("/out").join("Rig Mk2.phx"));
    // The extension is the app's, not a literal typed at each use.
    assert!(made.root.to_string_lossy().ends_with(BBundle::EXTENSION));
}

#[test]
fn every_part_sits_under_the_root() {
    let made = bundle();
    for part in BBundlePart::ALL {
        assert!(made.path(*part).starts_with(&made.root), "{part:?} is inside the bundle");
    }
}

#[test]
fn the_scene_file_is_named_after_the_project() {
    let made = bundle();
    assert_eq!(made.path(BBundlePart::Scene), made.root.join("Scenes").join("Rig Mk2.scene.json"));
    assert_eq!(made.path(BBundlePart::Manifest), made.root.join("project.json"));
    assert_eq!(made.path(BBundlePart::SceneDir), made.root.join("Scenes"));
}

#[test]
fn the_files_are_files_and_the_directories_are_not() {
    assert!(BBundlePart::Manifest.is_file());
    assert!(BBundlePart::Scene.is_file());
    assert!(!BBundlePart::Root.is_file());
    assert!(!BBundlePart::SceneDir.is_file());
}

#[test]
fn making_a_bundle_makes_the_scene_directory_too() {
    // One `create_dir_all` has to leave both directories there — the root
    // alone would make `path(Scene)` unwritable.
    let base = std::env::temp_dir().join(format!("phipro-make-{}", std::process::id()));
    let made = BBundle::new(Path::new("cube.blend"), &base).expect("a usable name");
    made.make().expect("the directories are made");
    assert!(made.path(BBundlePart::Root).is_dir());
    assert!(made.path(BBundlePart::SceneDir).is_dir());
    let _ = std::fs::remove_dir_all(&base);
}
