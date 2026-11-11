//! Modifier evaluator unit tests (pure mesh math).

mod merge;
mod stack;
mod lattice;

use super::subsurf::BSubsurf;
use super::{BModifier, BlendEvalMesh, EvalCtx};
use std::collections::HashMap;

pub(super) fn cube() -> BlendEvalMesh {
    let v = [[-1., -1., -1.], [1., -1., -1.], [1., 1., -1.], [-1., 1., -1.], [-1., -1., 1.], [1., -1., 1.], [1., 1., 1.], [-1., 1., 1.]];
    let f = [[0, 3, 2, 1], [4, 5, 6, 7], [0, 1, 5, 4], [1, 2, 6, 5], [2, 3, 7, 6], [3, 0, 4, 7]];
    BlendEvalMesh {
        vertices: v.to_vec(),
        faces: f.iter().map(|q| q.to_vec()).collect(),
        face_materials: vec![0, 1, 0, 0, 0, 0],
        ..Default::default()
    }
}

fn run(levels: u32, limit: bool) -> BlendEvalMesh {
    let mut m = cube();
    let worlds = HashMap::new();
    let ctx = EvalCtx { world: glam::Mat4::IDENTITY, worlds: &worlds };
    BSubsurf::new(levels, false, limit, false).apply(&mut m, &ctx);
    m
}

#[test]
fn catmull_clark_cube_level1_counts_and_corner() {
    let m = run(1, false);
    assert_eq!((m.vertices.len(), m.faces.len()), (26, 24));
    // Blender (recursive, level 1): a cube corner moves to ±5/9.
    assert!((m.vertices[6][0] - 5.0 / 9.0).abs() < 1e-5, "{:?}", m.vertices[6]);
    assert_eq!(m.face_materials.iter().filter(|&&s| s == 1).count(), 4);
}

#[test]
fn limit_surface_pulls_corners_further_in() {
    let (rec, lim) = (run(2, false), run(2, true));
    let r = |m: &BlendEvalMesh| m.vertices.iter().map(|v| v[0].abs().max(v[1].abs()).max(v[2].abs())).fold(0.0f32, f32::max);
    assert!(r(&lim) < r(&rec));
}

#[test]
fn mirror_names_flip_like_blender() {
    use super::mirror::flip_side;
    assert_eq!(flip_side("hand.L"), "hand.R");
    assert_eq!(flip_side("DEF-arm_r.001"), "DEF-arm_l.001");
    assert_eq!(flip_side("L_eye"), "R_eye");
    assert_eq!(flip_side("LeftFoot"), "RightFoot");
    assert_eq!(flip_side("spine"), "spine");
}


pub(super) fn eval(m: &mut BlendEvalMesh, modifier: &dyn BModifier) {
    let worlds = HashMap::new();
    let ctx = EvalCtx { world: glam::Mat4::IDENTITY, worlds: &worlds };
    modifier.apply(m, &ctx);
}

/// A unit quad in the XY plane, with UVs and one non-default material.
pub(super) fn quad() -> BlendEvalMesh {
    BlendEvalMesh {
        vertices: vec![[0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.]],
        faces: [vec![0u32, 1, 2, 3]].iter().cloned().collect(),
        loop_uvs: vec![[0., 0.], [1., 0.], [1., 1.], [0., 1.]],
        face_materials: vec![2],
        face_smooth: vec![true],
        dverts: vec![vec![(0, 1.0)]; 4],
        ..Default::default()
    }
}

