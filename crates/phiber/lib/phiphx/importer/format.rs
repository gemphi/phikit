//! Known 3D formats and basic format detection / parsing.

use crate::phx::Mesh;
use crate::phx::Result;

/// Known 3D file formats
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileFormat {
    Gltf,
    Glb,
    Usd,
    Usdz,
    Usda,
    Usdc,
    Obj,
    Stl,
    Ply,
    Fbx,
    Blend,
    Phx,
}

impl FileFormat {
    /// Get file extensions for this format
    pub fn extensions(&self) -> &[&'static str] {
        match self {
            FileFormat::Gltf => &["gltf"],
            FileFormat::Glb => &["glb"],
            FileFormat::Usd => &["usd"],
            FileFormat::Usdz => &["usdz"],
            FileFormat::Usda => &["usda"],
            FileFormat::Usdc => &["usdc"],
            FileFormat::Obj => &["obj"],
            FileFormat::Stl => &["stl"],
            FileFormat::Ply => &["ply"],
            FileFormat::Fbx => &["fbx"],
            FileFormat::Blend => &["blend"],
            FileFormat::Phx => &["phx"],
        }
    }

    /// Get from extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        let ext = ext.to_lowercase();
        match ext.as_str() {
            "gltf" => Some(FileFormat::Gltf),
            "glb" => Some(FileFormat::Glb),
            "usd" => Some(FileFormat::Usd),
            "usdz" => Some(FileFormat::Usdz),
            "usda" => Some(FileFormat::Usda),
            "usdc" => Some(FileFormat::Usdc),
            "obj" => Some(FileFormat::Obj),
            "stl" => Some(FileFormat::Stl),
            "ply" => Some(FileFormat::Ply),
            "fbx" => Some(FileFormat::Fbx),
            "blend" => Some(FileFormat::Blend),
            "phx" => Some(FileFormat::Phx),
            _ => None,
        }
    }
    /// Detect file format from magic bytes
    pub fn detect(data: &[u8]) -> Option<Self> {
        #[cfg(feature = "gltf-import")]
        if crate::phiphx::importer::gltf::GltfImporter::can_import(data) {
            return Some(FileFormat::Gltf);
        }
        if crate::phiphx::importer::usd::UsdImporter::can_import(data) {
            return Some(FileFormat::Usd);
        }
        if data.starts_with(b"# OBJ") || data.starts_with(b"o ") || data.starts_with(b"v ") {
            Some(FileFormat::Obj)
        } else {
            None
        }
    }
}

/// Simple OBJ mesh importer
pub struct ObjImporter;

impl ObjImporter {
    pub fn import_file_as_mesh(path: &str) -> Result<Vec<Mesh>> {
        let text = std::fs::read_to_string(path).map_err(|e| {
            crate::PointCloudError::ImportError(format!("Failed to read OBJ: {}", e))
        })?;

        let mut meshes = Vec::new();
        let mut current_mesh = Mesh::new();
        current_mesh.name = "Default".to_string();

        for line in text.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "o" | "g" => {
                    if current_mesh.vertex_count() > 0 {
                        meshes.push(current_mesh);
                        current_mesh = Mesh::new();
                    }
                    if parts.len() > 1 {
                        current_mesh.name = parts[1].to_string();
                    }
                }
                "v" => {
                    if parts.len() >= 4 {
                        let x = parts[1].parse::<f32>().unwrap_or(0.0);
                        let y = parts[2].parse::<f32>().unwrap_or(0.0);
                        let z = parts[3].parse::<f32>().unwrap_or(0.0);
                        current_mesh.positions.push([x, y, z]);
                    }
                }
                "f" => {
                    if parts.len() >= 4 {
                        let mut face_indices = Vec::new();
                        for p in &parts[1..] {
                            let idx_str = p.split('/').next().unwrap_or("0");
                            if let Ok(idx) = idx_str.parse::<u32>() {
                                if idx > 0 {
                                    face_indices.push(idx - 1);
                                }
                            }
                        }
                        // Triangulate
                        for i in 1..(face_indices.len() - 1) {
                            current_mesh.indices.push([
                                face_indices[0],
                                face_indices[i],
                                face_indices[i + 1],
                            ]);
                        }
                    }
                }
                _ => {}
            }
        }

        if current_mesh.vertex_count() > 0 {
            meshes.push(current_mesh);
        }

        Ok(meshes)
    }
}
