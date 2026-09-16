//! Project scaffolding — creates a `.phx` project directory from any supported asset.
//!
//! Format-agnostic: delegates parsing to [`import_asset`] and builds the project
//! structure from the unified [`AssetImportResult`]. Doesn't know or care
//! whether the source is `.blend`, `.gltf`, `.usd`, or any other format.

use super::importer::{AssetImportResult, AssetImporter};
use crate::phx::{PointCloudError, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Scaffolds a `.phx` project directory from any supported asset file.
///
/// # Usage
/// ```ignore
/// let scaffold = ProjectScaffold::new(input_path, output_dir);
/// let project_dir = scaffold.build()?;
/// ```
pub struct ProjectScaffold<'a> {
    input: &'a Path,
    #[allow(dead_code)]
    output_base: &'a Path,
    name: String,
    project_dir: PathBuf,
}

impl<'a> ProjectScaffold<'a> {
    /// Create a new scaffold for the given asset file.
    pub fn new(input: &'a Path, output_base: &'a Path) -> Self {
        let name = input
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let project_dir = output_base.join(format!("{}.phx", name));
        Self {
            input,
            output_base,
            name,
            project_dir,
        }
    }

    /// Build the project: parse the asset, create directories, write manifests.
    pub fn build(&self) -> Result<PathBuf> {
        fs::create_dir_all(&self.project_dir)
            .map_err(|e| PointCloudError::ExportError(e.to_string()))?;

        let progress = |_pct: f32, _msg: &str| {};
        let result = AssetImporter::import(self.input, &progress)
            .map_err(|e| PointCloudError::ImportError(e))?;

        self.write_manifest(&result)?;
        self.write_scene(&result)?;

        Ok(self.project_dir.clone())
    }

    /// Write a minimal `project.json` manifest.
    fn write_manifest(&self, result: &AssetImportResult) -> Result<()> {
        let manifest = format!(
            concat!(
                "{{\n",
                "  \"name\": \"{name}\",\n",
                "  \"version\": \"1.0.0\",\n",
                "  \"default_scene\": \"{name}\",\n",
                "  \"source\": \"{source}\",\n",
                "  \"object_count\": {obj_count},\n",
                "  \"vertex_count\": {vert_total}\n",
                "}}\n",
            ),
            name = self.name,
            source = result.mesh_path,
            obj_count = result.obj_count,
            vert_total = result.vert_total,
        );
        fs::write(self.project_dir.join("project.json"), &manifest)
            .map_err(|e| PointCloudError::ExportError(e.to_string()))
    }

    /// Write `Scenes/<name>.scene.json` with entity references.
    fn write_scene(&self, result: &AssetImportResult) -> Result<()> {
        let scenes_dir = self.project_dir.join("Scenes");
        fs::create_dir_all(&scenes_dir).map_err(|e| PointCloudError::ExportError(e.to_string()))?;

        let mut entities = Vec::new();

        let child_ids: Vec<String> = (2..=result.obj_names.len() as u64 + 1)
            .map(|i| i.to_string())
            .collect();
        entities.push(format!(
            "    {{ \"id\": 1, \"name\": \"Scene\", \"parent_id\": null, \"children\": [{}] }}",
            child_ids.join(", ")
        ));

        for (i, obj_name) in result.obj_names.iter().enumerate() {
            let id = i as u64 + 2;
            let safe_name = obj_name.replace('"', "'");
            entities.push(format!(
                "    {{ \"id\": {id}, \"name\": \"{safe_name}\", \"parent_id\": 1, \"mesh_ref\": \"{}?objects={i}\" }}",
                result.mesh_path,
            ));
        }

        let scene_json = format!(
            "{{\n  \"name\": \"{}\",\n  \"entities\": [\n{}\n  ]\n}}\n",
            self.name,
            entities.join(",\n")
        );
        fs::write(
            scenes_dir.join(format!("{}.scene.json", self.name)),
            &scene_json,
        )
        .map_err(|e| PointCloudError::ExportError(e.to_string()))
    }
}
