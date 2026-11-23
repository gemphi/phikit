//! Asset Catalog — Asset indexing and discovery
//!
//! Corresponds to: Assets/ directory in Reality Composer Pro bundles

use serde::{Deserialize, Serialize};
use serde_json;
use std::path::{Path, PathBuf};

/// An indexed asset discovered during scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetEntry {
    pub name: String,
    pub relative_path: String,
    pub asset_type: AssetType,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AssetType {
    PhxBundle,
    Mesh,
    Texture,
    Audio,
    Material,
    Scene,
    Other,
}

impl AssetType {
    pub fn from_path(path: &Path) -> Self {
        if path.is_dir() && path.extension().map(|e| e == "phx").unwrap_or(false) {
            return AssetType::PhxBundle;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        match ext.to_lowercase().as_str() {
            "obj" | "gltf" | "glb" | "fbx" | "usdz" | "usda" | "usdc" | "usd" | "ply" | "stl" => {
                AssetType::Mesh
            }
            "png" | "jpg" | "jpeg" | "hdr" | "exr" | "tga" | "bmp" => AssetType::Texture,
            "wav" | "mp3" | "ogg" | "flac" | "aac" => AssetType::Audio,
            "json" if path.to_string_lossy().ends_with(".mat.json") => AssetType::Material,
            "json" if path.to_string_lossy().ends_with(".scene.json") => AssetType::Scene,
            _ => AssetType::Other,
        }
    }
}

/// Asset catalog — indexes all assets in a project
pub struct AssetCatalog {
    pub entries: Vec<AssetEntry>,
    pub root: PathBuf,
}

impl AssetCatalog {
    pub fn new(root: PathBuf) -> Self {
        Self {
            entries: Vec::new(),
            root,
        }
    }

    pub fn scan(&mut self) -> Result<(), String> {
        let assets_dir = self.root.join("Assets");
        if !assets_dir.exists() {
            return Ok(());
        }
        self.entries.clear();
        self.scan_dir(&assets_dir, "Assets")?;
        Ok(())
    }

    fn scan_dir(&mut self, dir: &Path, relative: &str) -> Result<(), String> {
        for entry in std::fs::read_dir(dir).map_err(|e| format!("Scan failed: {}", e))? {
            let entry = entry.map_err(|e| format!("Entry failed: {}", e))?;
            let path = entry.path();
            let rel_path = format!("{}/{}", relative, entry.file_name().to_string_lossy());

            if path.is_dir() {
                self.scan_dir(&path, &rel_path)?;
            } else {
                let metadata = entry
                    .metadata()
                    .map_err(|e| format!("Metadata failed: {}", e))?;
                self.entries.push(AssetEntry {
                    name: entry.file_name().to_string_lossy().to_string(),
                    relative_path: rel_path,
                    asset_type: AssetType::from_path(&path),
                    size_bytes: metadata.len(),
                });
            }
        }
        Ok(())
    }
}
