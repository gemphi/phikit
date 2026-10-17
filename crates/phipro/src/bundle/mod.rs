//! The bundle on disk: where it sits, what it is called, and which parts
//! it has.
//!
//! One type owns the layout. Nothing else in the crate joins a path or
//! decides a file name, so a bundle cannot grow a directory in one place
//! and look for it in another — which is how the old module read: the
//! scaffold chose `"{stem}.phx"` and the importer that called it chose
//! nothing, then both wrote `project.json` with different fields in it.

pub mod manifest;
pub mod name;
pub mod scene;
pub mod set;

pub use name::{BNameError, BProjectName};
pub use set::BBundlePart;

use std::path::{Path, PathBuf};

/// A project bundle: the directory, and the name everything inside it goes
/// by.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BBundle {
    /// The directory itself — `<base>/<name>.phx`.
    pub root: PathBuf,
    /// The project's name, which is also its default scene's
    /// ([`BProjectName`]).
    pub name: String,
}

impl BBundle {
    /// Where a bundle for `asset` goes under `base`.
    ///
    /// Nothing is created: a bundle is decided before it is written, so
    /// that the name rules can be tested without a filesystem.
    pub fn new(asset: &Path, base: &Path) -> Result<Self, BNameError> {
        let BProjectName(name) = BProjectName::of(asset)?;
        Ok(Self { root: base.join(format!("{name}.{}", Self::EXTENSION)), name })
    }

    /// The directory suffix a project carries, as the app looks for it.
    pub const EXTENSION: &'static str = "phx";

    /// The path of one part of the bundle.
    pub fn path(&self, part: BBundlePart) -> PathBuf {
        match part {
            BBundlePart::Root => self.root.clone(),
            BBundlePart::SceneDir => self.root.join(set::SCENE_DIR),
            BBundlePart::Manifest => self.root.join(set::MANIFEST),
            BBundlePart::Scene => self.root.join(set::SCENE_DIR).join(format!("{}{}", self.name, set::SCENE_SUFFIX)),
        }
    }

    /// Create the directories a bundle needs, and only those.
    ///
    /// `create_dir_all` on the scene directory makes the root as well, so
    /// the two are one call; listing both would be a second answer to the
    /// same question.
    pub fn make(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(self.path(BBundlePart::SceneDir))
    }
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod mod_tests;
