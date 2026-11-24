//! `Phx::convert`: any file phiphx opens to any scene format it writes,
//! the formats picked from the two paths' extensions. A USD file going to
//! USD is copied as it is; everything else is opened into a
//! [`SceneModel`] (posed at its saved frame) and written by
//! [`SceneTarget`]. This is what a server runs on every uploaded file.

use crate::export::target::SceneTarget;
use crate::import::FileFormat;
use crate::phx::{Phx, PhxError, SceneModel};
use std::path::Path;

/// What a conversion did.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Conversion {
    /// The input copied unchanged (USD to USD).
    pub copied: bool,
    /// Objects in the opened scene (meshes, cameras, lights, empties).
    pub object_count: usize,
    /// Objects written as meshes (shown, with faces).
    pub mesh_count: usize,
    pub triangle_count: usize,
    pub byte_count: usize,
}

impl Phx {
    /// Convert `input` to `output`, formats from their extensions.
    pub fn convert(input: &Path, output: &Path) -> Result<Conversion, PhxError> {
        let ext = |p: &Path| p.extension().and_then(|e| e.to_str()).and_then(FileFormat::from_extension);
        let write = |bytes: &[u8]| std::fs::write(output, bytes).map_err(|e| PhxError::Inaccessible(e.to_string()));
        match (ext(input), ext(output)) {
            (Some(from), Some(to)) if passes(from, to) => {
                let bytes = std::fs::read(input).map_err(|e| PhxError::Inaccessible(e.to_string()))?;
                write(&bytes)?;
                let counted = Phx::open(input).map(|s| count(&posed(s))).unwrap_or_default();
                Ok(Conversion { copied: true, byte_count: bytes.len(), ..counted })
            }
            (_, to) => {
                let target = to.and_then(SceneTarget::of).ok_or_else(|| PhxError::Unsupported(format!("cannot write {}", output.display())))?;
                let scene = posed(Phx::open(input)?);
                let bytes = target.write(&scene).map_err(|e| PhxError::Malformed(e.to_string()))?;
                write(&bytes)?;
                Ok(Conversion { byte_count: bytes.len(), ..count(&scene) })
            }
        }
    }
}

/// Whether a file of format `from` may be copied as it is to an output
/// named as `to`: the same USD encoding, or any USD layer to a `.usd`.
fn passes(from: FileFormat, to: FileFormat) -> bool {
    match (from, to) {
        (FileFormat::Usda | FileFormat::Usdc | FileFormat::Usd, FileFormat::Usd) => true,
        (a, b) => a == b && a.is_usd(),
    }
}

/// The scene with every rig-deformed mesh posed at its saved frame, as
/// the viewport shows it.
fn posed(mut scene: SceneModel) -> SceneModel {
    let frame = scene.timeline.current_frame as f32;
    let (mesh, _) = scene.motion.posed_at(frame);
    for (i, m) in mesh {
        if let Some(o) = scene.objects.get_mut(i) {
            o.mesh = std::sync::Arc::new(m);
        }
    }
    scene
}

/// The counts a conversion reports for `scene`.
fn count(scene: &SceneModel) -> Conversion {
    let shown: Vec<_> = (0..scene.objects.len()).filter(|&i| scene.shows(i)).collect();
    Conversion {
        object_count: scene.objects.len() + scene.cameras.len() + scene.lights.len() + scene.empties.len(),
        mesh_count: shown.len(),
        triangle_count: shown.iter().map(|&i| scene.objects[i].mesh.indices.len() / 3).sum(),
        ..Default::default()
    }
}
