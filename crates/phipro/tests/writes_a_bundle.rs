//! A real asset in, a real bundle out.
//!
//! The unit tests cover the names, the JSON and the progress without
//! touching a disk. This is the one that reads a `.blend` through `phiphx`
//! and checks what lands: the directories, the two files, and that both
//! parse and agree with each other.

use phipro::bundle::{BBundle, BBundlePart};
use phipro::{BProjectWrite, BWriteError};
use std::path::{Path, PathBuf};

/// A fixture `phiphx` can read. Its own tests keep it there.
const ASSET: &str = "../phiphx/tests/fixtures/LIGHTS.blend";

fn asset() -> Option<PathBuf> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(ASSET);
    path.exists().then_some(path)
}

/// A directory of this test's own, removed afterwards.
struct Scratch(PathBuf);

impl Scratch {
    fn new(what: &str) -> Self {
        let path = std::env::temp_dir().join(format!("phipro-{what}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("a scratch directory");
        Self(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn read(path: &Path) -> serde_json::Value {
    let text = std::fs::read_to_string(path).unwrap_or_else(|why| panic!("{} is readable: {why}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|why| panic!("{} is JSON: {why}", path.display()))
}

#[test]
fn a_blend_becomes_a_bundle_whose_every_part_is_there() {
    let Some(asset) = asset() else { return };
    let scratch = Scratch::new("bundle");
    let made = BProjectWrite::new(&asset, &scratch.0).run(&|_, _| {}).expect("it imports");

    assert_eq!(made.root, scratch.0.join("LIGHTS.phx"));
    let bundle = BBundle::new(&asset, &scratch.0).expect("the same name");
    for part in BBundlePart::ALL {
        let path = bundle.path(*part);
        assert_eq!(path.is_file(), part.is_file(), "{part:?} at {} is the right kind", path.display());
        assert!(path.exists(), "{part:?} is written");
    }
}

#[test]
fn the_manifest_and_the_scene_agree_about_the_project() {
    let Some(asset) = asset() else { return };
    let scratch = Scratch::new("agree");
    let made = BProjectWrite::new(&asset, &scratch.0).run(&|_, _| {}).expect("it imports");
    let bundle = BBundle::new(&asset, &scratch.0).expect("the same name");

    let manifest = read(&bundle.path(BBundlePart::Manifest));
    let scene = read(&bundle.path(BBundlePart::Scene));
    assert_eq!(manifest["name"], "LIGHTS");
    assert_eq!(manifest["default_scene"], scene["name"], "the manifest points at the scene that exists");
    // One root plus one entity per object, which is what the manifest counts.
    let entities = scene["entity"].as_array().expect("an entity list").len();
    assert_eq!(manifest["object_count"].as_u64(), Some(entities as u64 - 1));
    assert_eq!(manifest["object_count"].as_u64(), Some(made.object_count as u64));
    assert_eq!(manifest["vertex_count"].as_u64(), Some(made.vertex_count as u64));
}

#[test]
fn the_progress_runs_from_the_start_to_the_end_without_going_backwards() {
    let Some(asset) = asset() else { return };
    let scratch = Scratch::new("progress");
    let seen = std::cell::RefCell::new(Vec::new());
    BProjectWrite::new(&asset, &scratch.0)
        .run(&|part, note| seen.borrow_mut().push((part, note.to_string())))
        .expect("it imports");

    let seen = seen.into_inner();
    assert!(seen.len() >= 5, "every step reports: {seen:?}");
    let mut last = 0.0;
    for (part, note) in &seen {
        assert!(*part >= last, "{part} comes after {last} ({note})");
        assert!(!note.is_empty(), "every step says something");
        last = *part;
    }
    assert_eq!(seen.last().map(|(part, _)| *part), Some(1.0), "and the bar finishes");
}

#[test]
fn importing_twice_into_the_same_place_rewrites_rather_than_failing() {
    // Re-importing an asset is how a user refreshes a project, so the
    // second run has to overwrite the first's files, not trip over them.
    let Some(asset) = asset() else { return };
    let scratch = Scratch::new("twice");
    let write = BProjectWrite::new(&asset, &scratch.0);
    let first = write.run(&|_, _| {}).expect("the first import");
    let second = write.run(&|_, _| {}).expect("the second import");
    assert_eq!(first.root, second.root);
    assert_eq!(first.object_count, second.object_count);
}

#[test]
fn a_base_that_cannot_be_written_is_reported_as_a_write_and_not_a_panic() {
    let Some(asset) = asset() else { return };
    // A file where the directory would go: `create_dir_all` fails on it.
    let scratch = Scratch::new("blocked");
    let blocked = scratch.0.join("LIGHTS.phx");
    std::fs::write(&blocked, b"not a directory").expect("the blocker is written");
    let why = BProjectWrite::new(&asset, &scratch.0).run(&|_, _| {}).expect_err("the path is taken");
    assert!(matches!(why, BWriteError::Write(BBundlePart::SceneDir, _)), "{why:?}");
    assert!(!why.message().is_empty());
}
