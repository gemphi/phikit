//! Asset-pipeline entry (`AssetImporter` → `BlendImporter::import`):
//! reads the file, writes the `.phx` sidecar, and flattens the scene into
//! `AssetImportResult` for the importer UI.

use crate::blend::{BlendCollection, BlendImporter, BlendScene};
use glam::Vec3;
use std::path::Path;

impl BlendImporter {
    /// Unified import entry point — called by [`AssetImporter::import`].
    ///
    /// If a `.phx` sidecar exists next to the `.blend` and is at least as new,
    /// it is loaded instead — the native `.blend` parser is O(n²) in block
    /// lookups and can stack-overflow on large files (178MB+), so the sidecar
    /// is both faster and safer.
    pub fn import(
        path: &Path,
        progress: &dyn Fn(f32, &str),
    ) -> std::result::Result<crate::AssetImportResult, String> {
        let clean = path.to_string_lossy().to_string();

        // Prefer a fresh .phx sidecar when available.
        let phx_path = path.with_extension("phx");
        if let (Ok(phx_meta), Ok(blend_meta)) =
            (std::fs::metadata(&phx_path), std::fs::metadata(path))
        {
            if phx_meta.modified().ok() >= blend_meta.modified().ok() {
                progress(0.10, "Loading .phx sidecar (skipping .blend parse)...");
                match crate::import::phx::PhxImporter::import(&phx_path, progress) {
                    Ok(mut r) => {
                        // Keep the original .blend path as the reference.
                        r.mesh_path = clean;
                        return Ok(r);
                    }
                    Err(e) => {
                        eprintln!(
                            "[Blend] .phx sidecar load failed ({e}), falling back to .blend parse"
                        );
                    }
                }
            }
        }

        progress(0.05, "Reading .blend file...");

        let blend = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            Self::import_blend_native_with_sidecar(path)
        })) {
            Ok(Ok(scene)) => scene,
            Ok(Err(e)) => {
                progress(1.0, &format!("Import error: {e}"));
                return Err(format!("{e}"));
            }
            Err(_panic) => {
                let msg = "Import crashed (stack overflow or parser panic on large .blend). \
                           Use the .phx sidecar instead."
                    .to_string();
                progress(1.0, &msg);
                return Err(msg);
            }
        };

        let obj_count = blend.objects.len();
        let vert_total: usize = blend.objects.iter().map(|o| o.vertices.len()).sum();
        progress(
            0.55,
            &format!("Parsed {obj_count} objects / {vert_total} verts"),
        );

        let obj_names: Vec<String> = blend.objects.iter().map(|o| o.name.clone()).collect();
        let bounds = Self::compute_bounds(&blend);

        // Build detailed object info for UI display.
        let objects: Vec<crate::ImportedObject> = blend
            .objects
            .iter()
            .map(|o| crate::ImportedObject {
                name: o.name.clone(),
                object_type: o.object_type.clone(),
                location: o.location,
                scale: o.scale,
                vertex_count: o.vertices.len(),
                face_count: o.faces.len(),
                material_names: o.materials.iter().map(|m| m.name.clone()).collect(),
                base_color: o.base_color,
                roughness: o.roughness,
                metallic: o.metallic,
                texture_paths: o.texture_paths.clone(),
                is_instance: o.is_instance,
            })
            .collect();

        // Deduplicate materials by name across all objects.
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        let materials: Vec<crate::ImportedMaterial> = blend
            .objects
            .iter()
            .flat_map(|o| o.materials.iter())
            .filter(|m| seen.insert(m.name.clone()))
            .map(|m| crate::ImportedMaterial {
                name: m.name.clone(),
                base_color: m.base_color,
                roughness: m.roughness,
                metallic: m.metallic,
                base_color_texture: m.base_color_texture.clone(),
            })
            .collect();

        // Convert blend collections to imported collections.
        let collections: Vec<crate::ImportedCollection> = blend
            .collections
            .iter()
            .map(Self::convert_collection)
            .collect();

        Ok(crate::AssetImportResult {
            mesh_path: clean,
            vert_total,
            obj_count,
            obj_names,
            bounds,
            objects,
            materials,
            collections,
        })
    }

    fn convert_collection(c: &BlendCollection) -> crate::ImportedCollection {
        crate::ImportedCollection {
            name: c.name.clone(),
            object_indices: c.object_indices.clone(),
            children: c.children.iter().map(Self::convert_collection).collect(),
        }
    }

    fn compute_bounds(blend: &BlendScene) -> Option<(Vec3, f32)> {
        let mut lo = [f32::MAX; 3];
        let mut hi = [f32::MIN; 3];
        for obj in &blend.objects {
            for v in &obj.vertices {
                let w = Self::apply_world(&obj.matrix_world, *v);
                for i in 0..3 {
                    lo[i] = lo[i].min(w[i]);
                    hi[i] = hi[i].max(w[i]);
                }
            }
        }
        if lo[0] <= hi[0] {
            let center = Vec3::new(
                (lo[0] + hi[0]) * 0.5,
                (lo[1] + hi[1]) * 0.5,
                (lo[2] + hi[2]) * 0.5,
            );
            let radius = (Vec3::new(hi[0], hi[1], hi[2]) - center)
                .length()
                .max(0.001);
            Some((center, radius))
        } else {
            None
        }
    }

    fn apply_world(m: &[[f32; 4]; 4], p: [f32; 3]) -> [f32; 3] {
        let finite = m.iter().flatten().all(|v| v.is_finite());
        let nonzero = m.iter().flatten().any(|&v| v != 0.0);
        if !finite || !nonzero {
            return p;
        }
        let (x, y, z) = (p[0], p[1], p[2]);
        [
            m[0][0] * x + m[1][0] * y + m[2][0] * z + m[3][0],
            m[0][1] * x + m[1][1] * y + m[2][1] * z + m[3][1],
            m[0][2] * x + m[1][2] * y + m[2][2] * z + m[3][2],
        ]
    }
}
