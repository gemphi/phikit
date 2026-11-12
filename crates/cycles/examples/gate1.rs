// SPDX-License-Identifier: Apache-2.0
//! Gate 1 on Blender's own test scenes.
//!
//! ```text
//! cargo run -p cycles --release --example gate1 -- scan
//! cargo run -p cycles --release --example gate1 -- run bsdf/diffuse shader/emission [--samples N]
//! cargo run -p cycles --release --example gate1 -- run-scan [--samples N]
//! ```
//!
//! `scan` lists, for every scene with a reference, what it uses and whether
//! that is all gate 1 has. `run` renders the scenes named and judges each
//! against Blender's image; `run-scan` does so for every scene the scan
//! finds. Images go to `target/gate1/`.
use std::path::{Path, PathBuf};

use cycles::blender::{MaterialSource, NodeSurface, SceneSource, read_blend};
use cycles::truth::{Outcome, SceneRenderer, TestScene, find_root, judge};

/// Node kinds a gate 1 material graph may hold.
const NODE_OK: [&str; 3] = ["ShaderNodeOutputMaterial", "ShaderNodeBsdfDiffuse", "ShaderNodeEmission"];

/// Why a scene is not all gate 1, or none if it is.
fn why_not(source: &SceneSource) -> Vec<String> {
    let mut why = Vec::new();
    if source.light_count > 0 {
        why.push(format!("{} lights", source.light_count));
    }
    if source.world_texture.is_some() {
        why.push("world image".into());
    }
    if source.camera.is_none() {
        why.push("no camera".into());
    }
    if source.object.is_empty() {
        why.push("no objects".into());
    }
    let material = source.mesh.iter().flat_map(|mesh| mesh.material.iter());
    for material in material {
        why.extend(material_why(material));
    }
    if source.mesh.iter().any(|mesh| mesh.material.is_empty()) {
        why.push("a mesh with no material".into());
    }
    why.sort();
    why.dedup();
    why
}

fn material_why(material: &MaterialSource) -> Vec<String> {
    let mut why: Vec<String> = material.node_kind.iter().filter(|kind| !NODE_OK.contains(&kind.as_str())).map(|kind| format!("node {kind}")).collect();
    if material.node == NodeSurface::Unknown {
        why.push("surface is not a lone diffuse or emission".into());
    }
    if let NodeSurface::Diffuse { roughness, .. } = material.node {
        if roughness != 0.0 {
            why.push("diffuse roughness".into());
        }
    }
    why
}

fn root() -> PathBuf {
    find_root(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("the test scenes")
}

fn categories(root: &Path) -> Vec<String> {
    let mut list: Vec<String> = std::fs::read_dir(root).unwrap().filter_map(Result::ok).filter(|entry| entry.path().is_dir()).map(|entry| entry.file_name().to_string_lossy().into_owned()).collect();
    list.sort();
    list
}

/// Every scene with a reference whose source reads.
fn scan(root: &Path) -> Vec<(TestScene, Result<SceneSource, String>)> {
    let mut out = Vec::new();
    for category in categories(root) {
        for scene in TestScene::list(root, &category) {
            if scene.exists() {
                let source = read_blend(&scene.blend);
                out.push((scene, source));
            }
        }
    }
    out
}

fn run(root: &Path, names: &[String], samples: Option<i32>) {
    let out = PathBuf::from("target/gate1");
    std::fs::create_dir_all(&out).unwrap();
    println!("{:<34} {:<8} {:>9} {:>10} {:>8} {:>7} {:>8}  notes", "scene", "verdict", "failed", "max error", "samples", "of", "seconds");
    for name in names {
        let (category, scene) = name.split_once('/').expect("category/name");
        let test = TestScene::new(root, category, scene);
        let mut renderer = samples.map_or_else(SceneRenderer::new, SceneRenderer::with_samples);
        let verdict = judge(&test, &mut renderer, &out);
        let report = renderer.last.clone();
        if let Some(image) = &renderer.image {
            let _ = image.write_png(&out.join(format!("{category}_{scene}.render.png")));
        }
        let (failed, max_error) = verdict.comparison.as_ref().map_or((f64::NAN, f32::NAN), |c| (c.failed_percent, c.max_error));
        let (used, of, seconds, notes) = report.map_or((0, 0, 0.0, String::new()), |r| {
            let mut notes = r.missing.clone();
            if r.view.approximated() {
                notes.push(format!("view {:?}", r.view.transform));
            }
            (r.samples, r.scene_samples, r.seconds, notes.join(", "))
        });
        let word = match &verdict.outcome {
            Outcome::Pass => "PASS".to_string(),
            Outcome::Fail => "FAIL".to_string(),
            Outcome::NoReference => "NOREF".to_string(),
            Outcome::RenderFailed(message) => format!("ERR {message}"),
        };
        println!("{:<34} {:<8} {:>8.3}% {:>10.4} {:>8} {:>7} {:>8.2}  {}", name, word, failed, max_error, used, of, seconds, notes);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let root = root();
    let samples = args.iter().position(|arg| arg == "--samples").and_then(|at| args.get(at + 1)).and_then(|value| value.parse().ok());
    match args.first().map(String::as_str) {
        Some("scan") => {
            for (scene, source) in scan(&root) {
                match source {
                    Ok(source) => {
                        let why = why_not(&source);
                        let samples = cycles::blender::sample_count(&source);
                        println!("{}/{}\t{}x{}\t{} samples\t{}", scene.category, scene.name, source.width, source.height, samples, if why.is_empty() { "GATE1".to_string() } else { why.join("; ") });
                    }
                    Err(error) => println!("{}/{}\tunreadable: {error}", scene.category, scene.name),
                }
            }
        }
        Some("info") => {
            for name in &args[1..] {
                let (category, scene) = name.split_once('/').expect("category/name");
                let test = TestScene::new(&root, category, scene);
                let source = read_blend(&test.blend).expect("reads");
                println!("{name}: {}x{} lights {} view {} look {} exposure {} gamma {} transparent {}", source.width, source.height, source.light_count, source.view_transform, source.look, source.exposure, source.gamma, source.film_transparent);
                println!("  cycles settings: {:?}", source.cycles);
                println!("  world {:?} x {} texture {:?}", source.world_color, source.world_strength, source.world_texture);
                let mut synced = cycles::blender::sync_scene(&source);
                let dscene = synced.device_update();
                let data = &dscene.data;
                let int = |field: &str| data.int("integrator", field);
                println!("  integrator: pattern {} seed {} blue_noise_length {} index_mask {:#x} max_bounce {} min_bounce {} diffuse {} clamp {} {}", int("sampling_pattern"), int("seed") as u32, int("blue_noise_sequence_length"), int("sobol_index_mask") as u32, int("max_bounce"), int("min_bounce"), int("max_diffuse_bounce"), data.float("integrator", "sample_clamp_direct"), data.float("integrator", "sample_clamp_indirect"));
                println!("  film: stride {} exposure {} filter offset {}", data.int("film", "pass_stride"), data.float("film", "exposure"), data.int("tables", "filter_table_offset"));
                println!("  camera: type {} fov-ish width {} height {}", data.int("cam", "type"), data.float("cam", "width"), data.float("cam", "height"));
                for (index, surface) in synced.surface.iter().enumerate() {
                    println!("  shader {index}: {surface:?}");
                }
            }
        }
        Some("run") => {
            let names: Vec<String> = args[1..].iter().take_while(|arg| !arg.starts_with("--")).cloned().collect();
            run(&root, &names, samples);
        }
        Some("run-scan") => {
            let names: Vec<String> = scan(&root).into_iter().filter(|(_, source)| source.as_ref().is_ok_and(|source| why_not(source).is_empty())).map(|(scene, _)| format!("{}/{}", scene.category, scene.name)).collect();
            run(&root, &names, samples);
        }
        _ => eprintln!("usage: gate1 scan | run <category/name>... [--samples N] | run-scan [--samples N]"),
    }
}
