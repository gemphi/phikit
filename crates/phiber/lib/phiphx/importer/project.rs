//! Project Importer service — encapsulates asset parsing and Reality Composer Pro project scaffolding.
#![cfg(feature = "appkit")]

use glam::Vec3;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::AssetImporter;
use crate::appkit::project::scaffold;

/// The result of a successful project import.
#[derive(Debug, Clone)]
pub struct ProjectImportResult {
    pub project: PathBuf,
    pub vert_total: usize,
    pub obj_count: usize,
    pub obj_names: Vec<String>,
    pub bounds: Option<(Vec3, f32)>,
}

/// Service to import any supported asset file into a Reality Composer Pro project bundle.
pub struct ProjectImporter<'a> {
    asset_path: &'a Path,
}

impl<'a> ProjectImporter<'a> {
    /// Create a new importer for the given asset file path.
    pub fn new(asset_path: &'a Path) -> Self {
        Self { asset_path }
    }

    /// Run the import process, invoking the progress callback at key checkpoints.
    pub fn import(&self, progress: &dyn Fn(f32, &str)) -> Result<ProjectImportResult, String> {
        progress(0.05, "Parsing asset file...");
        let clean = self.asset_path.to_string_lossy().to_string();

        let result = AssetImporter::import(self.asset_path, progress)?;

        let obj_count = result.obj_count;
        let vert_total = result.vert_total;
        let obj_names = result.obj_names.clone();
        let bounds = result.bounds;

        let proj_name =
            scaffold::project_name_from_asset(self.asset_path).map_err(|e| e.to_string())?;
        let scaffold_res =
            scaffold::scaffold_project(self.asset_path, &proj_name).map_err(|e| e.to_string())?;
        let project = scaffold_res.root;
        progress(0.72, &format!("Created project at {}", project.display()));

        let scene_name = proj_name.clone();

        // project.json — minimal manifest pointing to the default scene.
        let manifest = format!(
            "{{\n  \"name\": \"{proj_name}\",\n  \"version\": \"1.0.0\",\n  \"default_scene\": \"{scene_name}\",\n  \"created\": \"{}\",\n  \"settings\": {{\n    \"physics_enabled\": false,\n    \"render_quality\": \"high\",\n    \"grid_visible\": true\n  }}\n}}\n",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0)
        );
        std::fs::write(project.join("project.json"), &manifest).map_err(|e| e.to_string())?;

        // Scenes/<name>.scene.json — one entity per asset object
        let mut entities_json = Vec::new();
        let child_ids: Vec<String> = (2..=obj_names.len() as u64 + 1)
            .map(|i| i.to_string())
            .collect();
        entities_json.push(format!(
            "    {{ \"id\": 1, \"name\": \"Scene\", \"parent_id\": null, \"transform\": {{ \"position\": [0,0,0], \"rotation\": [0,0,0,1], \"scale\": [1,1,1] }}, \"components\": [], \"children\": [{}] }}",
            child_ids.join(", ")
        ));

        for (i, obj_name) in obj_names.iter().enumerate() {
            let id = i as u64 + 2;
            let mesh_ref = format!("{}?objects={}", clean, i);
            let safe_name = obj_name.replace('"', "'");
            entities_json.push(format!(
                "    {{ \"id\": {id}, \"name\": \"{safe_name}\", \"parent_id\": 1, \"transform\": {{ \"position\": [0,0,0], \"rotation\": [0,0,0,1], \"scale\": [1,1,1] }}, \"components\": [ {{ \"type\": \"model\", \"mesh_ref\": \"{mesh_ref}\", \"visible\": true }} ], \"children\": [] }}"
            ));
        }

        let scene_json = format!(
            "{{\n  \"name\": \"{scene_name}\",\n  \"entities\": [\n{}\n  ]\n}}\n",
            entities_json.join(",\n")
        );
        std::fs::write(
            project
                .join("Scenes")
                .join(format!("{scene_name}.scene.json")),
            &scene_json,
        )
        .map_err(|e| e.to_string())?;

        progress(0.80, "Scaffold written");

        Ok(ProjectImportResult {
            project,
            vert_total,
            obj_count,
            obj_names,
            bounds,
        })
    }
}
