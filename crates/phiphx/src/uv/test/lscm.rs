use crate::uv::uv_test::grid;
use crate::uv::{BUnwrap, BUvPin, unwrap};
use crate::uv::lscm::*;

/// The angle a triangle's UVs make at each corner, against the angle
/// the mesh makes there: a conformal map keeps them.
fn angle_error(mesh: BUvMesh<'_>, uv: &[[f32; 2]]) -> f32 {
    let mut worst = 0.0f32;
    for f in 0..mesh.faces() {
        let run = mesh.face(f);
        for k in 1..run.len().saturating_sub(1) {
            let tri = [0, k, k + 1];
            let at = mesh.starts[f] as usize;
            for j in 0..3 {
                let (a, b, c) = (tri[j], tri[(j + 1) % 3], tri[(j + 2) % 3]);
                let ang3 = angle3(mesh.pos[mesh.corner[at + a] as usize], mesh.pos[mesh.corner[at + b] as usize], mesh.pos[mesh.corner[at + c] as usize]);
                let ang2 = angle2(uv[at + a], uv[at + b], uv[at + c]);
                worst = worst.max((ang3 - ang2).abs());
            }
        }
    }
    worst
}

fn angle3(o: [f32; 3], a: [f32; 3], b: [f32; 3]) -> f32 {
    let u = [a[0] - o[0], a[1] - o[1], a[2] - o[2]];
    let v = [b[0] - o[0], b[1] - o[1], b[2] - o[2]];
    let d = u[0] * v[0] + u[1] * v[1] + u[2] * v[2];
    let lu = (u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt();
    let lv = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    (d / (lu * lv).max(1e-12)).clamp(-1.0, 1.0).acos()
}

fn angle2(o: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let u = [a[0] - o[0], a[1] - o[1]];
    let v = [b[0] - o[0], b[1] - o[1]];
    let d = u[0] * v[0] + u[1] * v[1];
    let lu = (u[0] * u[0] + u[1] * u[1]).sqrt();
    let lv = (v[0] * v[0] + v[1] * v[1]).sqrt();
    (d / (lu * lv).max(1e-12)).clamp(-1.0, 1.0).acos()
}

/// A flat grid is already flat: the conformal map of it is the grid
/// itself, so every angle survives exactly. If this is wrong nothing
/// else can be right.
#[test]
fn a_flat_grid_unwraps_to_itself() {
    let (pos, corner, starts) = grid(4);
    let mesh = BUvMesh { pos: &pos, corner: &corner, starts: &starts };
    let uv = unwrap(mesh, BUnwrap::Conformal, &[], &[]);
    assert_eq!(uv.len(), corner.len());
    assert!(angle_error(mesh, &uv) < 1e-3, "every angle kept: {}", angle_error(mesh, &uv));
}

/// A cylinder cut open along one seam — a real curved surface — comes
/// back with its angles within a couple of degrees, which is what a
/// conformal map promises (and area, which it does not, is free to
/// stretch).
#[test]
fn an_open_cylinder_keeps_its_angles() {
    let (rings, seg) = (6usize, 12usize);
    let mut pos = Vec::new();
    for j in 0..=rings {
        for i in 0..=seg {
            let t = i as f32 / seg as f32 * std::f32::consts::TAU;
            pos.push([t.cos(), t.sin(), j as f32 / rings as f32 * 2.0]);
        }
    }
    let (mut corner, mut starts) = (Vec::new(), vec![0u32]);
    let stride = (seg + 1) as u32;
    for j in 0..rings as u32 {
        for i in 0..seg as u32 {
            let a = j * stride + i;
            corner.extend_from_slice(&[a, a + 1, a + stride + 1, a + stride]);
            starts.push(corner.len() as u32);
        }
    }
    let mesh = BUvMesh { pos: &pos, corner: &corner, starts: &starts };
    let uv = unwrap(mesh, BUnwrap::Conformal, &[], &[]);
    let worst = angle_error(mesh, &uv).to_degrees();
    assert!(worst < 3.0, "the angles survive the unwrap: worst {worst:.2} degrees");
}

/// Pins hold. Two corners pinned where the user put them, and the
/// solved UVs land on them — which is the only thing that makes a pin
/// more than a different colour of dot.
#[test]
fn a_pinned_corner_stays_where_it_was_put() {
    let (pos, corner, starts) = grid(3);
    let mesh = BUvMesh { pos: &pos, corner: &corner, starts: &starts };
    let pins = [BUvPin { corner: 0, uv: [0.25, 0.75] }, BUvPin { corner: 2, uv: [0.9, 0.1] }];
    let islands = super::super::island::split(mesh, &[]);
    let mut uv = vec![[0.0f32; 2]; corner.len()];
    super::solve(mesh, &islands[0], &pins, &mut uv);
    for p in pins {
        assert!((uv[p.corner][0] - p.uv[0]).abs() < 1e-5 && (uv[p.corner][1] - p.uv[1]).abs() < 1e-5, "{:?} vs {:?}", uv[p.corner], p.uv);
    }
    // And the rest moved with them rather than ignoring them: the
    // solved map is the one the two pins pick out of the similarity
    // family, so it is not the identity any more.
    assert!(uv.iter().any(|c| c[0] > 0.9 || c[1] > 0.75), "the island followed the pins: {uv:?}");
}
