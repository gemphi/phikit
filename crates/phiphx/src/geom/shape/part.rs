//! A shape's part and the builder that makes its faces: points welded by
//! position, one normal per face corner.

use std::collections::HashMap;

/// One part of a shape: its name and look from the table, and its mesh —
/// polygons (`count` corners each) over `point`, a normal per corner.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ShapePart {
    pub name: String,
    /// Linear r g b, 0-1.
    pub colour: [f32; 3],
    pub roughness: f32,
    pub metallic: f32,
    pub point: Vec<[f32; 3]>,
    pub count: Vec<u32>,
    pub index: Vec<u32>,
    /// One per face corner (`len == index.len()`).
    pub normal: Vec<[f32; 3]>,
}

impl ShapePart {
    /// Triangles once every polygon is fanned (`n - 2` per face).
    pub fn triangle_count(&self) -> usize {
        self.count.iter().map(|&n| n.saturating_sub(2) as usize).sum()
    }

    /// Each face's corner indices, in order.
    pub fn face(&self) -> impl Iterator<Item = &[u32]> + '_ {
        let mut at = 0usize;
        self.count.iter().map(move |&n| {
            let face = &self.index[at..at + n as usize];
            at += n as usize;
            face
        })
    }

    /// Smallest and largest corner of the points' box.
    pub fn extent(&self) -> [[f32; 3]; 2] {
        self.point.iter().fold([[f32::MAX; 3], [f32::MIN; 3]], |[lo, hi], p| {
            [
                [lo[0].min(p[0]), lo[1].min(p[1]), lo[2].min(p[2])],
                [hi[0].max(p[0]), hi[1].max(p[1]), hi[2].max(p[2])],
            ]
        })
    }
}

/// Builds a part's mesh: faces added corner by corner, points that land
/// on the same spot (to a tenth of a micrometre) shared.
#[derive(Debug, Default)]
pub struct ShapeMesh {
    pub point: Vec<[f32; 3]>,
    pub count: Vec<u32>,
    pub index: Vec<u32>,
    pub normal: Vec<[f32; 3]>,
    seen: HashMap<[i64; 3], u32>,
}

/// Points this close (metres, per axis) are one point.
const WELD: f64 = 1e-7;

impl ShapeMesh {
    /// Add one face: its corners in counter-clockwise order seen from
    /// outside, each a position and its normal.
    pub fn face(&mut self, corners: &[([f32; 3], [f32; 3])]) {
        self.count.push(corners.len() as u32);
        for &(p, n) in corners {
            let index = self.point(p);
            self.index.push(index);
            self.normal.push(unit(n));
        }
    }

    fn point(&mut self, p: [f32; 3]) -> u32 {
        let key = p.map(|v| (v as f64 / WELD).round() as i64);
        let next = self.point.len() as u32;
        let index = *self.seen.entry(key).or_insert(next);
        if index == next {
            self.point.push(p);
        }
        index
    }

    /// The finished part under its table name and look.
    pub fn part(self, look: &super::ShapeLook) -> ShapePart {
        ShapePart {
            name: look.name.clone(),
            colour: look.colour,
            roughness: look.roughness,
            metallic: look.metallic,
            point: self.point,
            count: self.count,
            index: self.index,
            normal: self.normal,
        }
    }
}

/// `v` scaled to length 1 (zero stays zero).
pub fn unit(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    match l > 0.0 {
        true => v.map(|c| c / l),
        false => v,
    }
}

/// A place and turn a part is built in: local x, y, z go to `x`, `y`, `z`
/// (right-handed, unit) about `origin`.
#[derive(Clone, Copy, Debug)]
pub struct Frame {
    pub origin: [f32; 3],
    pub x: [f32; 3],
    pub y: [f32; 3],
    pub z: [f32; 3],
}

impl Frame {
    /// The shape's own space.
    pub const IDENTITY: Frame = Frame { origin: [0.0; 3], x: [1.0, 0.0, 0.0], y: [0.0, 1.0, 0.0], z: [0.0, 0.0, 1.0] };

    /// A local point in shape space.
    pub fn point(&self, p: [f32; 3]) -> [f32; 3] {
        let d = self.turn(p);
        [self.origin[0] + d[0], self.origin[1] + d[1], self.origin[2] + d[2]]
    }

    /// A local direction in shape space.
    pub fn turn(&self, v: [f32; 3]) -> [f32; 3] {
        [0, 1, 2].map(|i| self.x[i] * v[0] + self.y[i] * v[1] + self.z[i] * v[2])
    }
}
