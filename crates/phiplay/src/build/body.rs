//! A player's body as the kit is cut from it: her mesh with its bone
//! weights and her skeleton's bones, handed over by Phiber from the rigged
//! figure the players are made of. What is cut from it — a vest, socks, a
//! shoe, a hairstyle's shell — keeps the weights of the skin beneath, so it
//! moves with her without a rig of its own.
//!
//! The cut is make_demo's: the body is first split clean along the planes
//! a garment's edges run on (`bisect`), so a hem is a straight line rather
//! than the ragged edge of whole faces; then the faces a garment wants are
//! kept and stood off the skin along its normals.

use glam::Vec3;
use std::collections::HashMap;

/// A bone of the skeleton, in the body's rest space.
#[derive(Clone, Debug, PartialEq)]
pub struct BBone {
    pub name: String,
    pub head: [f32; 3],
    pub tail: [f32; 3],
}

/// The body, as cut from.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BBody {
    pub positions: Vec<[f32; 3]>,
    pub faces: Vec<Vec<u32>>,
    /// The vertex groups `weights` index — the deforming bones' names.
    pub groups: Vec<String>,
    /// Per vertex: `(group, weight)`.
    pub weights: Vec<Vec<(u32, f32)>>,
    pub bones: Vec<BBone>,
}

/// A piece cut from the body: its own vertices (stood off the skin),
/// polygons, a UV per vertex, the body's weights, and a material slot per
/// face.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BPiece {
    pub positions: Vec<[f32; 3]>,
    pub faces: Vec<Vec<u32>>,
    pub uvs: Vec<[f32; 2]>,
    pub weights: Vec<Vec<(u32, f32)>>,
    pub face_materials: Vec<u16>,
}

impl BBody {
    /// A bone's head, by name.
    pub fn head(&self, name: &str) -> Option<Vec3> {
        self.bones.iter().find(|b| b.name == name).map(|b| Vec3::from(b.head))
    }

    /// A bone's tail, by name.
    pub fn tail(&self, name: &str) -> Option<Vec3> {
        self.bones.iter().find(|b| b.name == name).map(|b| Vec3::from(b.tail))
    }

    /// Each vertex's normal: the area-weighted mean of its faces'.
    pub fn normals(&self) -> Vec<Vec3> {
        let mut n = vec![Vec3::ZERO; self.positions.len()];
        for f in &self.faces {
            let p = |i: usize| Vec3::from(self.positions[f[i] as usize]);
            let mut area = Vec3::ZERO;
            for k in 1..f.len() - 1 {
                area += (p(k) - p(0)).cross(p(k + 1) - p(0));
            }
            for &v in f {
                n[v as usize] += area;
            }
        }
        n.into_iter().map(|v| v.normalize_or_zero()).collect()
    }

    /// How much of a face's weight is in groups whose names start with any
    /// of `prefixes` — an arm's share, a head's.
    pub fn share(&self, face: &[u32], prefixes: &[&str]) -> f32 {
        let (mut total, mut part) = (0.0, 0.0);
        for &v in face {
            for &(g, w) in &self.weights[v as usize] {
                total += w;
                if self.groups.get(g as usize).is_some_and(|name| prefixes.iter().any(|p| name.starts_with(p))) {
                    part += w;
                }
            }
        }
        part / total.max(1e-6)
    }

    /// The body split along the plane where coordinate `axis` (0 x, 1 y,
    /// 2 z) is `at`: every face crossing it cut in two, the new vertices'
    /// positions and weights interpolated along the edges they sit on, one
    /// new vertex per edge however many faces share it.
    pub fn bisect(&self, axis: usize, at: f32) -> BBody {
        let mut out = self.clone();
        out.faces.clear();
        let mut on_edge: HashMap<(u32, u32), u32> = HashMap::new();
        let side = |v: u32| self.positions[v as usize][axis] - at;
        for f in &self.faces {
            let crosses = f.iter().any(|&v| side(v) < -1e-6) && f.iter().any(|&v| side(v) > 1e-6);
            if !crosses {
                out.faces.push(f.clone());
                continue;
            }
            let (mut below, mut above) = (Vec::new(), Vec::new());
            for k in 0..f.len() {
                let (a, b) = (f[k], f[(k + 1) % f.len()]);
                let (da, db) = (side(a), side(b));
                if da <= 1e-6 {
                    below.push(a);
                }
                if da >= -1e-6 {
                    above.push(a);
                }
                if (da < -1e-6 && db > 1e-6) || (da > 1e-6 && db < -1e-6) {
                    let key = (a.min(b), a.max(b));
                    let v = *on_edge.entry(key).or_insert_with(|| {
                        let t = da / (da - db);
                        let (pa, pb) = (Vec3::from(self.positions[a as usize]), Vec3::from(self.positions[b as usize]));
                        out.positions.push(pa.lerp(pb, t).into());
                        out.weights.push(mix_weights(&self.weights[a as usize], &self.weights[b as usize], t));
                        (out.positions.len() - 1) as u32
                    });
                    below.push(v);
                    above.push(v);
                }
            }
            for part in [below, above] {
                if part.len() >= 3 {
                    out.faces.push(part);
                }
            }
        }
        out
    }

    /// The faces `keep` picks (by face, given its centre), stood off the
    /// skin by `offset(rest position)` along the vertex normals, compacted
    /// to the vertices they use; `uv` gives each kept vertex its UV, and
    /// `slot` each face its material.
    pub fn piece(&self, keep: impl Fn(&[u32], Vec3) -> bool, offset: impl Fn(Vec3) -> f32, uv: impl Fn(Vec3) -> [f32; 2], slot: impl Fn(Vec3) -> u16) -> BPiece {
        let normals = self.normals();
        let mut map: HashMap<u32, u32> = HashMap::new();
        let mut piece = BPiece::default();
        for f in &self.faces {
            let centre = f.iter().map(|&v| Vec3::from(self.positions[v as usize])).sum::<Vec3>() / f.len() as f32;
            if !keep(f, centre) {
                continue;
            }
            let face: Vec<u32> = f
                .iter()
                .map(|&v| {
                    *map.entry(v).or_insert_with(|| {
                        let p = Vec3::from(self.positions[v as usize]);
                        piece.positions.push((p + normals[v as usize] * offset(p)).into());
                        piece.uvs.push(uv(p));
                        piece.weights.push(self.weights[v as usize].clone());
                        (piece.positions.len() - 1) as u32
                    })
                })
                .collect();
            piece.face_materials.push(slot(centre));
            piece.faces.push(face);
        }
        piece
    }
}

/// Two vertices' weights blended `t` of the way from `a` to `b`.
fn mix_weights(a: &[(u32, f32)], b: &[(u32, f32)], t: f32) -> Vec<(u32, f32)> {
    let mut out: Vec<(u32, f32)> = a.iter().map(|&(g, w)| (g, w * (1.0 - t))).collect();
    for &(g, w) in b {
        match out.iter_mut().find(|(h, _)| *h == g) {
            Some(slot) => slot.1 += w * t,
            None => out.push((g, w * t)),
        }
    }
    out.retain(|(_, w)| *w > 1e-4);
    out
}

#[cfg(test)]
pub mod tests {
    use super::*;

    /// A column 2 m tall in four rings of four, weighted "root" below 1 m
    /// and "tip" above — the build tests' figure.
    pub fn column() -> BBody {
        let mut b = BBody { groups: vec!["root".into(), "tip".into()], ..Default::default() };
        for z in [0.0f32, 0.5, 1.0, 1.5, 2.0] {
            for (x, y) in [(-0.1, -0.1), (0.1, -0.1), (0.1, 0.1), (-0.1, 0.1)] {
                b.positions.push([x, y, z]);
                b.weights.push(vec![(u32::from(z >= 1.0), 1.0)]);
            }
        }
        for r in 0..4u32 {
            for k in 0..4u32 {
                let (a, c) = (r * 4 + k, r * 4 + (k + 1) % 4);
                b.faces.push(vec![a, c, c + 4, a + 4]);
            }
        }
        b.bones = vec![BBone { name: "root".into(), head: [0.0; 3], tail: [0.0, 0.0, 1.0] }, BBone { name: "tip".into(), head: [0.0, 0.0, 1.0], tail: [0.0, 0.0, 2.0] }];
        b
    }

    #[test]
    fn a_bisect_cuts_clean_and_carries_the_weights_and_a_piece_stands_off_the_skin() {
        let body = column().bisect(2, 0.75);
        // One new ring at 0.75, its weights half root and half tip.
        let new: Vec<usize> = (20..body.positions.len()).collect();
        assert_eq!(new.len(), 4);
        for &v in &new {
            assert!((body.positions[v][2] - 0.75).abs() < 1e-6);
            let w = &body.weights[v];
            assert!(w.iter().any(|&(g, x)| g == 0 && (x - 0.5).abs() < 1e-5) && w.iter().any(|&(g, x)| g == 1 && (x - 0.5).abs() < 1e-5), "{w:?}");
        }
        assert_eq!(body.faces.len(), 20, "the four faces across the cut split in two");
        // A band from 0.75 to 1.5 m, stood 2 cm off: its hem is the cut.
        let band = body.piece(|_, c| c.z > 0.75 && c.z < 1.5, |_| 0.02, |p| [0.0, p.z], |_| 0);
        let low = band.positions.iter().map(|p| p[2]).fold(f32::MAX, f32::min);
        assert!((low - 0.75).abs() < 1e-5, "a straight hem at the cut: {low}");
        let out = band.positions.iter().map(|p| p[0].hypot(p[1])).fold(f32::MAX, f32::min);
        assert!(out > 0.1414 + 0.015, "stood off the skin: {out}");
        assert_eq!(band.weights.len(), band.positions.len());
    }
}
