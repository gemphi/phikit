//! Lattice against Blender: the Female-01 garment
//! `Female-Mannequin-Body-01.003` with every modifier but the lattice
//! disabled, vertices dumped by `lattice_truth.py`. Skipped when the file is
//! not on this machine (it is a 178 MB production asset, not a fixture).

use crate::blend::file::types::BlendFile;
use crate::blend::modifier::{BlendEvalMesh, EvalCtx};
use glam::Mat4;
use std::collections::HashMap;

const FILE: &str = "/mnt/user-data/uploads/phiace/crates/phiber/assets/CW-Fashion/Epic-Pods-Female-01/Female-01-Chatty-Whims-Rigged-Lace.blend";
const TRUTH: &str = "/tmp/claude-0/-home-claude/5489a85c-1628-523f-90af-14083601666b/scratchpad/lattice_truth.bin";
const OBJECT: &str = "Female-Mannequin-Body-01.003";

fn truth() -> Option<Vec<[f32; 3]>> {
    let b = std::fs::read(TRUTH).ok()?;
    let n = u32::from_le_bytes(b[0..4].try_into().ok()?) as usize;
    Some((0..n).map(|i| {
        let o = 4 + i * 12;
        [0, 4, 8].map(|k| f32::from_le_bytes(b[o + k..o + k + 4].try_into().unwrap()))
    }).collect())
}

#[test]
fn the_lattice_moves_the_garment_where_blender_does() {
    let (Ok(bytes), Some(want)) = (std::fs::read(FILE), truth()) else {
        eprintln!("Female-01 not staged; skipped");
        return;
    };
    let file = BlendFile::parse_bytes(&bytes).expect("parse");
    let dna = file.dna.as_ref().expect("dna");
    let scene = file.extract_scene().expect("scene");
    let o = scene.objects.iter().find(|o| o.name == OBJECT).expect("garment");
    let ob = file.block_at_code(o.block_address, "OB").expect("OB");
    let stack = file.modifier_stack(ob, dna);
    let lattice = stack.iter().find(|m| m.is_lattice()).expect("the garment has a lattice modifier");
    let worlds: HashMap<u64, Mat4> = scene.objects.iter().map(|o| (o.block_address, Mat4::from_cols_array_2d(&o.matrix_world))).collect();
    let mut mesh = BlendEvalMesh {
        vertices: o.vertices.clone(),
        faces: o.faces.clone(),
        loop_uvs: o.loop_uvs.clone(),
        face_materials: o.face_materials.clone(),
        face_smooth: o.face_smooth.clone(),
        dverts: Vec::new(),
        ..Default::default()
    };
    assert_eq!(mesh.vertices.len(), want.len(), "same base mesh");
    let before = mesh.vertices.clone();
    lattice.apply(&mut mesh, &EvalCtx { world: Mat4::from_cols_array_2d(&o.matrix_world), worlds: &worlds });
    let moved = before.iter().zip(&mesh.vertices).map(|(a, b)| dist(*a, *b)).fold(0.0f32, f32::max);
    assert!(moved > 0.01, "the lattice did nothing (max move {moved})");
    let worst = want.iter().zip(&mesh.vertices).map(|(a, b)| dist(*a, *b)).fold(0.0f32, f32::max);
    assert!(worst < 1e-3, "worst vertex {worst} m from Blender's (lattice moves it by up to {moved} m)");
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}
