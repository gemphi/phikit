// SPDX-License-Identifier: Apache-2.0
//! Ported from `tests/python/cycles_render_tests.py` and
//! `modules/render_report.py` (`test_get_name`, `test_get_images`): where a
//! test scene and the image Blender made of it are. A scene is
//! `<root>/<category>/<name>.blend`; its reference is
//! `<root>/<category>/cycles_renders/<name>.png` (`reference_dir` is the
//! engine's name and `_renders`).
use std::path::{Path, PathBuf};

use super::TruthError;

/// The folder of the render test scenes inside Blender's tests, from the
/// root of the workspace.
pub const TEST_ROOT: &str = "ext/phiber-refs/blender/blender/tests/files/render";

/// A test scene and its reference.
#[derive(Debug, Clone, PartialEq)]
pub struct TestScene {
    /// `test_dir_name`: the folder, which sets the allowance.
    pub category: String,
    /// The file's name without `.blend`.
    pub name: String,
    pub blend: PathBuf,
    /// Where the reference is, whether or not it exists.
    pub reference: PathBuf,
}

impl TestScene {
    /// The scene `category/name` under `root`.
    pub fn new(root: &Path, category: &str, name: &str) -> Self {
        let folder = root.join(category);
        Self { category: category.into(), name: name.into(), blend: folder.join(format!("{name}.blend")), reference: folder.join("cycles_renders").join(format!("{name}.png")) }
    }
    /// Both files are there.
    pub fn exists(&self) -> bool {
        self.blend.is_file() && self.reference.is_file()
    }
    /// Every scene of `category` that has a `.blend` file, by name.
    pub fn list(root: &Path, category: &str) -> Vec<TestScene> {
        let Ok(entries) = std::fs::read_dir(root.join(category)) else { return Vec::new() };
        let mut name: Vec<String> = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "blend"))
            .filter_map(|path| path.file_stem().map(|stem| stem.to_string_lossy().into_owned()))
            .collect();
        name.sort();
        name.iter().map(|name| Self::new(root, category, name)).collect()
    }
}

/// The test root: `CYCLES_TEST_FILES` when set, else [`TEST_ROOT`] found in
/// the closest parent of `start` that has it.
pub fn find_root(start: &Path) -> Result<PathBuf, TruthError> {
    if let Some(root) = std::env::var_os("CYCLES_TEST_FILES").map(PathBuf::from) {
        return if root.is_dir() { Ok(root) } else { Err(TruthError::NoRoot(root.display().to_string())) };
    }
    start
        .ancestors()
        .map(|folder| folder.join(TEST_ROOT))
        .find(|root| root.is_dir())
        .ok_or_else(|| TruthError::NoRoot(format!("{TEST_ROOT} above {}", start.display())))
}
