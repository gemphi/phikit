//! What a bundle holds, and under which names.
//!
//! The names are here and nowhere else; [`super::BBundle::path`] is the
//! only thing that reads them.

/// The directory the scene files live in.
pub const SCENE_DIR: &str = "Scenes";

/// The manifest at the bundle's root.
pub const MANIFEST: &str = "project.json";

/// What a scene file's name ends with, after the project's own name.
pub const SCENE_SUFFIX: &str = ".scene.json";

/// One part of a bundle, named rather than spelt at each use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BBundlePart {
    /// The bundle's directory.
    Root,
    /// `Scenes/`.
    SceneDir,
    /// `project.json`.
    Manifest,
    /// `Scenes/<name>.scene.json`.
    Scene,
}

impl BBundlePart {
    /// Every part, for a caller that wants to check a bundle is whole.
    pub const ALL: &'static [Self] = &[Self::Root, Self::SceneDir, Self::Manifest, Self::Scene];

    /// Whether this part is a file rather than a directory.
    pub fn is_file(self) -> bool {
        matches!(self, Self::Manifest | Self::Scene)
    }
}
