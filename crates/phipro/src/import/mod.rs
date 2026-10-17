//! Reading an asset and writing a bundle from it, with progress.
//!
//! The one call an app makes. It is here rather than in `phiphx` because
//! the parse is `phiphx`'s and the files are this crate's, and the service
//! that joins them belongs with the half that writes.
//!
//! Progress is reported through [`step::BImportStep`] rather than as bare
//! numbers at the call sites, which is what the two versions this replaces
//! did — `progress(0.05, …)`, `progress(0.72, …)`, `progress(0.80, …)`,
//! with nothing saying what the fractions added up to or why one of them
//! stopped at 0.80.

pub mod step;

use crate::bundle::{BBundle, BBundlePart, BNameError, manifest::BManifest, scene::BScene};
use glam::Vec3;
use std::path::{Path, PathBuf};
use step::{BImportStep, BProgress};

/// What a finished import produced.
#[derive(Clone, Debug, PartialEq)]
pub struct BProjectMade {
    /// The bundle's directory.
    pub root: PathBuf,
    pub object_count: usize,
    pub vertex_count: usize,
    pub object_name: Vec<String>,
    /// The bounding sphere, for the camera to frame the import.
    pub bounds: Option<(Vec3, f32)>,
}

/// Why an import did not finish.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BWriteError {
    /// The asset path gives no project name.
    Name(BNameError),
    /// `phiphx` could not read the asset.
    Read(String),
    /// A directory or file could not be written. The part says which.
    Write(BBundlePart, String),
    /// The manifest or the scene would not serialise.
    Encode(String),
}

impl BWriteError {
    pub fn message(&self) -> String {
        match self {
            Self::Name(why) => why.message(),
            Self::Read(why) => format!("cannot read the asset: {why}"),
            Self::Write(part, why) => format!("cannot write the project's {part:?}: {why}"),
            Self::Encode(why) => format!("cannot encode the project: {why}"),
        }
    }
}

/// An import waiting to be run: the asset, and where the bundle goes.
pub struct BProjectWrite<'a> {
    asset: &'a Path,
    base: &'a Path,
}

impl<'a> BProjectWrite<'a> {
    pub fn new(asset: &'a Path, base: &'a Path) -> Self {
        Self { asset, base }
    }

    /// Read the asset and write the bundle, reporting each step.
    ///
    /// The bundle is decided first and read second, so a name that cannot
    /// become a directory is refused before the asset is parsed — the old
    /// order parsed a `.blend` through and then failed on the name.
    pub fn run(&self, note: &dyn Fn(f32, &str)) -> Result<BProjectMade, BWriteError> {
        let bundle = BBundle::new(self.asset, self.base).map_err(BWriteError::Name)?;
        // The bar only ever moves forward, which the reader's own progress
        // does not ([`BProgress`]).
        let say = BProgress::new(note);

        say.at(BImportStep::Read, &self.asset.display().to_string());
        let read = phiphx::AssetImporter::import(self.asset, &|part, what| {
            say.within(BImportStep::Read, part, what);
        })
        .map_err(BWriteError::Read)?;

        say.at(BImportStep::Make, &bundle.root.display().to_string());
        bundle.make().map_err(|why| BWriteError::Write(BBundlePart::SceneDir, why.to_string()))?;

        say.at(BImportStep::Manifest, &bundle.name);
        let manifest = BManifest::new(&bundle.name, &read.mesh_path, read.obj_count, read.vert_total);
        self.put(&bundle, BBundlePart::Manifest, manifest.text())?;

        say.at(BImportStep::Scene, &bundle.name);
        let scene = BScene::new(&bundle.name, &read.obj_names, &read.mesh_path, read.bounds);
        self.put(&bundle, BBundlePart::Scene, scene.text())?;

        say.at(BImportStep::Done, &bundle.name);
        Ok(BProjectMade {
            root: bundle.root,
            object_count: read.obj_count,
            vertex_count: read.vert_total,
            object_name: read.obj_names,
            bounds: read.bounds,
        })
    }

    /// Write one part's text, naming the part in anything that goes wrong.
    fn put(&self, bundle: &BBundle, part: BBundlePart, text: Result<String, serde_json::Error>) -> Result<(), BWriteError> {
        let text = text.map_err(|why| BWriteError::Encode(why.to_string()))?;
        std::fs::write(bundle.path(part), text).map_err(|why| BWriteError::Write(part, why.to_string()))
    }
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod mod_tests;
