//! The open side's one `match`: a file's format picks the reader, and
//! every reader ends in the same [`SceneModel`]. `.blend` goes through
//! `Phx::open` (its own reader, posed and evaluated); USD, glTF, OBJ, PLY
//! and PHX come here.

use super::FileFormat;
use crate::export::scene::PhxScene;
use crate::scene::{SceneLoadError, SceneModel};
use crate::{PointCloud, PointCloudFormat};
use std::path::Path;

impl FileFormat {
    /// The format of the file at `path`: its first bytes, else its extension.
    pub fn of_path(path: &Path) -> Option<FileFormat> {
        use std::io::Read;
        let mut head = [0u8; 64];
        let read = std::fs::File::open(path).and_then(|mut f| f.read(&mut head)).unwrap_or(0);
        crate::Phx::detect(path, &head[..read])
    }

    /// Whether it is one of USD's (text, binary, package or either).
    pub fn is_usd(self) -> bool {
        matches!(self, Self::Usd | Self::Usda | Self::Usdc | Self::Usdz)
    }

    /// Read the file at `path`, in this format, into a scene.
    pub fn open(self, path: &Path) -> Result<SceneModel, SceneLoadError> {
        if let Err(e) = std::fs::metadata(path) {
            return Err(SceneLoadError::Inaccessible(e.to_string()));
        }
        let source = path.to_string_lossy().into_owned();
        let fail = |e: crate::PointCloudError| SceneLoadError::Malformed(e.to_string());
        match self {
            Self::Blend => crate::Phx::open(path),
            Self::Usd | Self::Usda | Self::Usdc | Self::Usdz => {
                let mesh = crate::import::usd::UsdImporter::import_file_as_mesh(&source).map_err(fail)?;
                Ok(SceneModel::from_meshes(mesh, &source, !z_up(path)))
            }
            #[cfg(feature = "gltf-import")]
            Self::Gltf | Self::Glb => {
                let mesh = crate::import::gltf::GltfImporter::import_meshes(&source).map_err(fail)?;
                Ok(SceneModel::from_meshes(mesh, &source, true))
            }
            Self::Obj => {
                let mesh = super::ObjImporter::import_file_as_mesh(&source).map_err(fail)?;
                Ok(SceneModel::from_meshes(mesh, &source, true))
            }
            Self::Ply => {
                let text = std::fs::read_to_string(path).map_err(|e| SceneLoadError::Inaccessible(e.to_string()))?;
                let format = PointCloudFormat::Ply { binary: false, include_normals: true, include_colors: true };
                let cloud = PointCloud::import(&text, &format).map_err(fail)?;
                Ok(SceneModel::from_meshes(vec![points(&cloud, path)], &source, false))
            }
            Self::Phx => {
                let data = std::fs::read(path).map_err(|e| SceneLoadError::Inaccessible(e.to_string()))?;
                let scene = PhxScene::from_bytes(&data).map_err(fail)?;
                Ok(SceneModel::from_phx(&scene, &source))
            }
            other => Err(SceneLoadError::Unsupported(format!("no scene reader for {other:?}"))),
        }
    }
}

/// Whether a text USD layer says it stands Z up (USD's default is Y).
fn z_up(path: &Path) -> bool {
    use std::io::Read;
    let mut head = Vec::new();
    let _ = std::fs::File::open(path).map(|f| f.take(4096).read_to_end(&mut head));
    let text = String::from_utf8_lossy(&head);
    text.lines().any(|l| l.trim().replace(' ', "") == "upAxis=\"Z\"")
}

/// A point cloud as a mesh with points and no faces.
fn points(cloud: &PointCloud, path: &Path) -> crate::Mesh {
    let vertex = cloud.vertices();
    let normal: Vec<[f32; 3]> = vertex.iter().filter_map(|v| v.normal).collect();
    crate::Mesh {
        name: path.file_stem().and_then(|s| s.to_str()).unwrap_or_default().to_string(),
        positions: vertex.iter().map(|v| v.position).collect(),
        normals: Some(normal).filter(|n| n.len() == vertex.len()),
        ..Default::default()
    }
}
