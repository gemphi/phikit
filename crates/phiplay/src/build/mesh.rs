//! A triangle mesh as the generators build it: positions, a normal and a
//! UV per vertex, and triangles. Hard edges are split vertices — a box is
//! 24 vertices, not 8 — so every face shades flat where it should and
//! smooth where it should, with no smoothing flags to carry.

use glam::{Mat4, Vec2, Vec3};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

impl BMesh {
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Adds a vertex; its index.
    pub fn vertex(&mut self, p: Vec3, n: Vec3, uv: Vec2) -> u32 {
        self.positions.push(p.into());
        self.normals.push(n.normalize_or_zero().into());
        self.uvs.push(uv.into());
        (self.positions.len() - 1) as u32
    }

    pub fn triangle(&mut self, a: u32, b: u32, c: u32) {
        self.indices.extend([a, b, c]);
    }

    /// A flat quad, `a b c d` counter-clockwise seen from its front, its
    /// UVs the unit square.
    pub fn quad(&mut self, a: Vec3, b: Vec3, c: Vec3, d: Vec3) {
        let n = (b - a).cross(d - a);
        let i = self.vertex(a, n, Vec2::new(0.0, 0.0));
        self.vertex(b, n, Vec2::new(1.0, 0.0));
        self.vertex(c, n, Vec2::new(1.0, 1.0));
        self.vertex(d, n, Vec2::new(0.0, 1.0));
        self.triangle(i, i + 1, i + 2);
        self.triangle(i, i + 2, i + 3);
    }

    /// `other` appended, its vertices moved by `at`.
    pub fn merge(&mut self, other: &BMesh, at: Mat4) {
        let base = self.positions.len() as u32;
        let normal = at.inverse().transpose();
        for ((p, n), uv) in other.positions.iter().zip(&other.normals).zip(&other.uvs) {
            self.positions.push(at.transform_point3(Vec3::from(*p)).into());
            self.normals.push(normal.transform_vector3(Vec3::from(*n)).normalize_or_zero().into());
            self.uvs.push(*uv);
        }
        self.indices.extend(other.indices.iter().map(|i| i + base));
    }

    /// A box of `size` (full extents) centred on the origin, each face its
    /// own four vertices.
    pub fn cuboid(size: Vec3) -> BMesh {
        let h = size / 2.0;
        let mut m = BMesh::default();
        let c = |x: f32, y: f32, z: f32| Vec3::new(x * h.x, y * h.y, z * h.z);
        m.quad(c(-1., -1., 1.), c(1., -1., 1.), c(1., 1., 1.), c(-1., 1., 1.));
        m.quad(c(-1., 1., -1.), c(1., 1., -1.), c(1., -1., -1.), c(-1., -1., -1.));
        m.quad(c(-1., -1., -1.), c(1., -1., -1.), c(1., -1., 1.), c(-1., -1., 1.));
        m.quad(c(1., 1., -1.), c(-1., 1., -1.), c(-1., 1., 1.), c(1., 1., 1.));
        m.quad(c(1., -1., -1.), c(1., 1., -1.), c(1., 1., 1.), c(1., -1., 1.));
        m.quad(c(-1., 1., -1.), c(-1., -1., -1.), c(-1., -1., 1.), c(-1., 1., 1.));
        m
    }

    /// A UV sphere of `radius`, `segments` round and `rings` pole to pole,
    /// smooth; UVs equirectangular with v = 0 at the north pole (+z).
    pub fn sphere(radius: f32, segments: u32, rings: u32) -> BMesh {
        let mut m = BMesh::default();
        for r in 0..=rings {
            let theta = std::f32::consts::PI * r as f32 / rings as f32;
            for s in 0..=segments {
                let phi = std::f32::consts::TAU * s as f32 / segments as f32;
                let n = Vec3::new(theta.sin() * phi.cos(), theta.sin() * phi.sin(), theta.cos());
                m.vertex(n * radius, n, Vec2::new(s as f32 / segments as f32, r as f32 / rings as f32));
            }
        }
        let row = segments + 1;
        for r in 0..rings {
            for s in 0..segments {
                let (a, b) = (r * row + s, (r + 1) * row + s);
                if r > 0 {
                    m.triangle(a, b, a + 1);
                }
                if r + 1 < rings {
                    m.triangle(a + 1, b, b + 1);
                }
            }
        }
        m
    }

    /// A ring in the x–y plane about the origin: `major` to the tube's
    /// centre, `minor` the tube's radius — a rim.
    pub fn torus(major: f32, minor: f32, segments: u32, sides: u32) -> BMesh {
        let mut m = BMesh::default();
        for s in 0..=segments {
            let a = std::f32::consts::TAU * s as f32 / segments as f32;
            let out = Vec3::new(a.cos(), a.sin(), 0.0);
            for t in 0..=sides {
                let b = std::f32::consts::TAU * t as f32 / sides as f32;
                let n = out * b.cos() + Vec3::Z * b.sin();
                m.vertex(out * major + n * minor, n, Vec2::new(s as f32 / segments as f32, t as f32 / sides as f32));
            }
        }
        let row = sides + 1;
        for s in 0..segments {
            for t in 0..sides {
                let (a, b) = (s * row + t, (s + 1) * row + t);
                m.triangle(a, b, b + 1);
                m.triangle(a, b + 1, a + 1);
            }
        }
        m
    }

    /// A rod from `a` to `b` of `radius`, eight-sided: a post, a crossbar.
    pub fn rod(a: Vec3, b: Vec3, radius: f32) -> BMesh {
        let length = a.distance(b);
        let mut m = BMesh::cylinder(radius, radius, length, 8);
        let mut out = BMesh::default();
        let turn = glam::Quat::from_rotation_arc(Vec3::Z, (b - a).normalize_or(Vec3::Z));
        out.merge(&m, Mat4::from_rotation_translation(turn, a));
        std::mem::swap(&mut m, &mut out);
        m
    }

    /// A closed tube along z from 0 to `height`: `segments` round, radius
    /// `bottom` at the foot and `top` at the head, smooth sides, flat caps.
    pub fn cylinder(bottom: f32, top: f32, height: f32, segments: u32) -> BMesh {
        let ring = |r: f32, z: f32| (0..segments).map(move |s| {
            let a = std::f32::consts::TAU * s as f32 / segments as f32;
            (Vec3::new(r * a.cos(), r * a.sin(), z), Vec3::new(a.cos(), a.sin(), (bottom - top) / height))
        });
        BMesh::prism_from(ring(bottom, 0.0).collect(), ring(top, height).collect(), true)
    }

    /// An outline in the x–z plane (`(x, z)` pairs, counter-clockwise seen
    /// from −y) extruded `depth` along +y: a seat back, a sail. Flat sides
    /// (each face its own normal) when `points` is low, which is what makes
    /// a semi-hexagon read as cut rather than turned.
    pub fn extruded(outline: &[[f32; 2]], depth: f32) -> BMesh {
        let front: Vec<(Vec3, Vec3)> = outline.iter().map(|p| (Vec3::new(p[0], 0.0, p[1]), Vec3::ZERO)).collect();
        let back: Vec<(Vec3, Vec3)> = outline.iter().map(|p| (Vec3::new(p[0], depth, p[1]), Vec3::ZERO)).collect();
        BMesh::prism_from(front, back, false)
    }

    /// Two matching rings joined by a band, capped at both ends. With
    /// `smooth` the band takes the rings' own normals; without, each side
    /// is a flat face.
    fn prism_from(a: Vec<(Vec3, Vec3)>, b: Vec<(Vec3, Vec3)>, smooth: bool) -> BMesh {
        let mut m = BMesh::default();
        let n = a.len();
        // The band faces out when the rings turn counter-clockwise about
        // the axis from `a` to `b`; an outline drawn the other way round
        // walks the band backwards.
        let axis = (b.iter().map(|p| p.0).sum::<Vec3>() - a.iter().map(|p| p.0).sum::<Vec3>()) / n as f32;
        let centre = a.iter().map(|p| p.0).sum::<Vec3>() / n as f32;
        let turn: f32 = (0..n).map(|k| (a[k].0 - centre).cross(a[(k + 1) % n].0 - centre).dot(axis)).sum();
        let order: Vec<usize> = if turn >= 0.0 { (0..n).collect() } else { (0..n).rev().collect() };
        for w in 0..n {
            let (k, j) = (order[w], order[(w + 1) % n]);
            if smooth {
                let i = m.vertex(a[k].0, a[k].1, Vec2::new(w as f32 / n as f32, 0.0));
                m.vertex(a[j].0, a[j].1, Vec2::new((w + 1) as f32 / n as f32, 0.0));
                m.vertex(b[j].0, b[j].1, Vec2::new((w + 1) as f32 / n as f32, 1.0));
                m.vertex(b[k].0, b[k].1, Vec2::new(w as f32 / n as f32, 1.0));
                m.triangle(i, i + 1, i + 2);
                m.triangle(i, i + 2, i + 3);
            } else {
                m.quad(a[k].0, a[j].0, b[j].0, b[k].0);
            }
        }
        // The caps: fans about each ring's centre, facing out of the prism.
        for (ring, out) in [(&a, -axis), (&b, axis)] {
            let centre = ring.iter().map(|p| p.0).sum::<Vec3>() / n as f32;
            let c = m.vertex(centre, out, Vec2::splat(0.5));
            let first = m.positions.len() as u32;
            for p in ring.iter() {
                m.vertex(p.0, out, Vec2::splat(0.5));
            }
            for k in 0..n as u32 {
                let (p, q) = (first + k, first + (k + 1) % n as u32);
                // Wind so the cap faces `out`.
                let facing = (Vec3::from(m.positions[p as usize]) - centre).cross(Vec3::from(m.positions[q as usize]) - centre).dot(out);
                if facing >= 0.0 { m.triangle(c, p, q) } else { m.triangle(c, q, p) }
            }
        }
        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Twice the signed volume of a closed mesh by the divergence theorem:
    /// positive when every face winds outward.
    fn volume(m: &BMesh) -> f32 {
        m.indices
            .chunks(3)
            .map(|t| {
                let [a, b, c] = [t[0], t[1], t[2]].map(|i| Vec3::from(m.positions[i as usize]));
                a.dot(b.cross(c)) / 6.0
            })
            .sum()
    }

    #[test]
    fn closed_shapes_wind_outward_and_hold_their_volume() {
        assert!((volume(&BMesh::cuboid(Vec3::new(2.0, 3.0, 4.0))) - 24.0).abs() < 1e-4);
        let s = volume(&BMesh::sphere(1.0, 48, 24));
        assert!((s - 4.18879).abs() < 0.06, "a fine sphere is nearly 4/3 π: {s}");
        let c = volume(&BMesh::cylinder(1.0, 1.0, 2.0, 64));
        assert!((c - std::f32::consts::TAU).abs() < 0.05, "π r² h: {c}");
        let t = volume(&BMesh::torus(1.0, 0.25, 64, 24));
        assert!((t - 2.0 * std::f32::consts::PI.powi(2) * 0.0625).abs() < 0.02, "2π² R r²: {t}");
        let r = volume(&BMesh::rod(Vec3::ZERO, Vec3::new(0.0, 3.0, 4.0), 0.1));
        assert!(r > 0.0 && (r - 5.0 * 0.1 * 0.1 * 2.0 * 2f32.sqrt()).abs() < 0.02, "an octagonal rod 5 long: {r}");
        let square = [[-0.5, 0.0], [0.5, 0.0], [0.5, 1.0], [-0.5, 1.0]];
        let e = volume(&BMesh::extruded(&square, 0.25));
        assert!((e - 0.25).abs() < 1e-5, "outward whichever way the outline turns: {e}");
        let reversed: Vec<[f32; 2]> = square.iter().rev().copied().collect();
        assert!((volume(&BMesh::extruded(&reversed, 0.25)) - 0.25).abs() < 1e-5);
    }
}
