//! Project Manager — Main project interface
//!
//! Mirrors: NSDocument / NSBundle for Reality Composer Pro projects

use super::asset_catalog::{AssetCatalog, AssetEntry};
use super::manifest::ProjectManifest;
use super::scene_data::SceneData;
use serde_json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A loaded project — used by both editor and runtime
pub struct ProjectManager {
    pub manifest: ProjectManifest,
    pub root: PathBuf,
    pub scenes: HashMap<String, SceneData>,
    pub asset_catalog: AssetCatalog,
}

impl ProjectManager {
    /// Create a new empty project
    pub fn new(name: &str, root: PathBuf) -> Self {
        std::fs::create_dir_all(&root).ok();
        std::fs::create_dir_all(root.join("Scenes")).ok();
        std::fs::create_dir_all(root.join("Assets")).ok();
        std::fs::create_dir_all(root.join("Settings")).ok();

        let mut scenes = HashMap::new();
        let default_scene = SceneData {
            name: "Main".to_string(),
            entities: Vec::new(),
            environment: Default::default(),
        };
        scenes.insert("Main".to_string(), default_scene);

        Self {
            manifest: ProjectManifest::new(name),
            root,
            scenes,
            asset_catalog: AssetCatalog::new(PathBuf::new()),
        }
    }

    /// Load an existing project from disk
    pub fn load(root: &Path) -> Result<Self, String> {
        let manifest_path = root.join("project.json");
        let manifest_json = std::fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Failed to read manifest: {}", e))?;
        let manifest: ProjectManifest = serde_json::from_str(&manifest_json)
            .map_err(|e| format!("Failed to parse manifest: {}", e))?;

        let mut scenes = HashMap::new();
        let scenes_dir = root.join("Scenes");
        if scenes_dir.exists() {
            for entry in
                std::fs::read_dir(&scenes_dir).map_err(|e| format!("Read scenes dir: {}", e))?
            {
                let entry = entry.map_err(|e| format!("Scene entry: {}", e))?;
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    let json =
                        std::fs::read_to_string(&path).map_err(|e| format!("Read scene: {}", e))?;
                    let scene: SceneData =
                        serde_json::from_str(&json).map_err(|e| format!("Parse scene: {}", e))?;
                    scenes.insert(scene.name.clone(), scene);
                }
            }
        }

        let mut asset_catalog = AssetCatalog::new(root.to_path_buf());
        asset_catalog.scan().ok();

        Ok(Self {
            manifest,
            root: root.to_path_buf(),
            scenes,
            asset_catalog,
        })
    }

    /// Save project to disk
    pub fn save(&self) -> Result<(), String> {
        let manifest_json = serde_json::to_string_pretty(&self.manifest)
            .map_err(|e| format!("Serialize manifest: {}", e))?;
        std::fs::write(self.root.join("project.json"), manifest_json)
            .map_err(|e| format!("Write manifest: {}", e))?;

        let scenes_dir = self.root.join("Scenes");
        std::fs::create_dir_all(&scenes_dir).ok();
        for (name, scene) in &self.scenes {
            let json = serde_json::to_string_pretty(scene)
                .map_err(|e| format!("Serialize scene: {}", e))?;
            std::fs::write(scenes_dir.join(format!("{}.scene.json", name)), json)
                .map_err(|e| format!("Write scene: {}", e))?;
        }

        Ok(())
    }

    pub fn get_scene(&self, name: &str) -> Option<&SceneData> {
        self.scenes.get(name)
    }

    pub fn get_scene_mut(&mut self, name: &str) -> Option<&mut SceneData> {
        self.scenes.get_mut(name)
    }

    pub fn add_scene(&mut self, scene: SceneData) {
        self.scenes.insert(scene.name.clone(), scene);
    }
}
