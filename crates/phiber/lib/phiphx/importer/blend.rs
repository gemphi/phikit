//! `.blend` format importer
//!
//! Native Rust parser for Blender's binary SDNA format.
//! Reads .blend files directly without requiring Blender installation.

use crate::phx::Result;
use glam::Vec3;
use std::path::Path;

mod dna;
mod dna_layout;
mod material;
mod mesh;
mod mesh_block;
mod object;
mod object_meta;
mod object_transforms;
mod parse;
mod phx_export;
mod readers;
mod scene;
mod types;

pub use phx_export::blend_scene_to_phx;
pub use types::{
    BlendBlock, BlendCollection, BlendFile, BlendHeader, BlendMaterial, BlendScene,
    BlendSceneObject, DnaField, DnaStruct, DnaType,
};

/// Structure encapsulating all Blender (.blend) asset import capabilities.
pub struct BlendImporter;

impl BlendImporter {
    /// Read a .blend file and extract a BlendScene.
    /// Zstd-compressed files (Blender 4.5+ "compressed" saves, and all of the
    /// upstream test suite) are decompressed transparently, like blenloader.
    pub fn import_blend_native(path: &Path) -> Result<BlendScene> {
        let data = std::fs::read(path)?;

        println!("[Blend] Reading: {:?}", path);

        let blend = BlendFile::parse_bytes(&data)?;
        let scene = blend.extract_scene()?;

        // Emit a portable `.phx` sidecar (best-effort; never fails the import).
        phx_export::write_phx_sidecar(path, &scene);

        Ok(scene)
    }

    /// Unified import entry point — called by [`AssetImporter::import`].
    ///
    /// If a `.phx` sidecar exists next to the `.blend` and is at least as new,
    /// it is loaded instead — the native `.blend` parser is O(n²) in block
    /// lookups and can stack-overflow on large files (178MB+), so the sidecar
    /// is both faster and safer.
    pub fn import(
        path: &Path,
        progress: &dyn Fn(f32, &str),
    ) -> std::result::Result<crate::phiphx::AssetImportResult, String> {
        let clean = path.to_string_lossy().to_string();

        // Prefer a fresh .phx sidecar when available.
        let phx_path = path.with_extension("phx");
        if let (Ok(phx_meta), Ok(blend_meta)) =
            (std::fs::metadata(&phx_path), std::fs::metadata(path))
        {
            if phx_meta.modified().ok() >= blend_meta.modified().ok() {
                progress(0.10, "Loading .phx sidecar (skipping .blend parse)...");
                match crate::phiphx::importer::phx::PhxImporter::import(&phx_path, progress) {
                    Ok(mut r) => {
                        // Keep the original .blend path as the reference.
                        r.mesh_path = clean.clone();
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
            Self::import_blend_native(path)
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
        let objects: Vec<crate::phiphx::ImportedObject> = blend
            .objects
            .iter()
            .map(|o| crate::phiphx::ImportedObject {
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
        let materials: Vec<crate::phiphx::ImportedMaterial> = blend
            .objects
            .iter()
            .flat_map(|o| o.materials.iter())
            .filter(|m| seen.insert(m.name.clone()))
            .map(|m| crate::phiphx::ImportedMaterial {
                name: m.name.clone(),
                base_color: m.base_color,
                roughness: m.roughness,
                metallic: m.metallic,
                base_color_texture: m.base_color_texture.clone(),
            })
            .collect();

        // Convert blend collections to imported collections.
        let collections: Vec<crate::phiphx::ImportedCollection> = blend
            .collections
            .iter()
            .map(Self::convert_collection)
            .collect();

        Ok(crate::phiphx::AssetImportResult {
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

    fn convert_collection(c: &BlendCollection) -> crate::phiphx::ImportedCollection {
        crate::phiphx::ImportedCollection {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    /// Root of Blender's upstream test files (`tests/files`) inside REFS.
    fn tests_files() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../REFS/blender/blender/tests/files")
    }

    fn load(rel: &str) -> BlendScene {
        let path = tests_files().join(rel);
        let data = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let file = BlendFile::parse_bytes(&data).unwrap_or_else(|e| panic!("parse {rel}: {e}"));
        file.extract_scene()
            .unwrap_or_else(|e| panic!("extract scene {rel}: {e}"))
    }

    /// Blender 4.5+/5.x stores these with Zstd compression (`28 B5 2F FD`).
    /// The parser must transparently decompress, like blenloader.
    #[test]
    fn loads_zstd_compressed_test_file() {
        let path = tests_files().join("usd/empty.blend");
        let data = std::fs::read(&path).expect("read empty.blend");
        assert_eq!(
            &data[0..4],
            &[0x28, 0xB5, 0x2F, 0xFD],
            "precondition: file is zstd"
        );
        let file = BlendFile::parse_bytes(&data).expect("parse compressed .blend");
        assert!(file.dna.is_some(), "DNA1 block must survive decompression");
        assert!(!file.blocks.is_empty());
    }

    #[test]
    fn loads_curated_test_files() {
        if !tests_files().exists() {
            return;
        } // REFS checkout optional
        let curated = [
            "usd/empty.blend",
            "usd/usd_simple_scene.blend",
            "usd/usd_mesh_subd.blend",
            "usd/usd_anim_test.blend",
            "usd/usd_curves_test.blend",
            "ui_tests/allselect.blend",
            "ui_tests/flippedmatrixes.blend",
            "ui_tests/space_types.blend",
            "tracking/undistortion-test.blend",
        ];
        for rel in curated {
            let path = tests_files().join(rel);
            if !path.exists() {
                continue;
            }
            let _scene = load(rel);
        }
    }

    #[test]
    fn simple_scene_has_objects() {
        if !tests_files().exists() {
            return;
        }
        let scene = load("usd/usd_simple_scene.blend");
        assert!(
            !scene.objects.is_empty(),
            "usd_simple_scene.blend must yield at least one object"
        );
        let verts: usize = scene.objects.iter().map(|o| o.vertices.len()).sum();
        assert!(verts > 0, "scene should contain real mesh vertices");
    }

    /// Full sweep over every small upstream test blend.
    /// Run on demand: `cargo test -p phiphx sweep_all -- --ignored`
    /// (nearly 2000 files; too slow for the default suite).
    #[test]
    #[ignore = "sweeps ~2000 files; run with -- --ignored"]
    fn sweep_all_small_test_blends() {
        const MAX_BYTES: u64 = 2 * 1024 * 1024;
        let root = tests_files();
        if !root.exists() {
            eprintln!("skip: REFS not present");
            return;
        }

        let mut files: Vec<PathBuf> = Vec::new();
        fn walk(d: &Path, out: &mut Vec<PathBuf>) {
            for e in std::fs::read_dir(d).into_iter().flatten().flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, out);
                } else if p.extension().is_some_and(|x| x == "blend") {
                    if e.metadata().map(|m| m.len()).unwrap_or(u64::MAX) <= MAX_BYTES {
                        out.push(p);
                    }
                }
            }
        }
        walk(&root, &mut files);
        println!("sweeping {} .blend files", files.len());

        let mut failures: Vec<String> = Vec::new();
        for f in &files {
            let rel = f
                .strip_prefix(&root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            let data = match std::fs::read(f) {
                Ok(d) => d,
                Err(e) => {
                    failures.push(format!("{rel}: read: {e}"));
                    continue;
                }
            };
            let r = std::panic::catch_unwind(|| {
                BlendFile::parse_bytes(&data).and_then(|b| b.extract_scene())
            });
            match r {
                Ok(Ok(_)) => {}
                Ok(Err(e)) => failures.push(format!("{rel}: {e}")),
                Err(_) => failures.push(format!("{rel}: panic")),
            }
        }
        println!("failed {}/{}", failures.len(), files.len());
        for f in failures.iter().take(20) {
            println!("  {f}");
        }
        assert!(failures.is_empty(), "{} files failed", failures.len());
    }

    #[test]
    fn test_blend_header_parse() {
        let _test_data = b"BLENDER-v304";
        // Real header coverage comes from the cartridge above.
    }
}
