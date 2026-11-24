//! A thing built: its parts placed, each a mesh where it stands in the
//! thing (a part's own place and turn, and its repeats, put into its
//! points), or, for a thing inside it, a group of its own with its
//! place, turn and size as a transform, so the thing keeps its layers.
//! Looks are swapped on the way down: a thing inside a thing wears what
//! the rows above it swap its looks for.

use std::collections::HashMap;

use super::form;
use super::read::{ThingForm, ThingLook, ThingRow, ThingSet};
use crate::geom::shape::{ShapeLook, ShapePart};

/// How finely a placed point is kept (a tenth of a millimetre) and a
/// normal (a thousandth), so a file says no more than can be seen.
const POINT: f32 = 1e4;
const NORMAL: f32 = 1e3;

/// How deep things may stand inside things (a thing that holds itself
/// stops here).
const DEEP: usize = 12;

/// A row-layout transform (USD's: the first three rows are the turned
/// and sized axes, the fourth the place).
pub type Matrix = [[f32; 4]; 4];

/// A built thing's layer: a mesh in a look, or a group of layers.
#[derive(Clone, Debug, PartialEq)]
pub enum ThingNode {
    /// A part's mesh, its points where they stand in the group around
    /// it, the look it wears, and where that look turns from one colour
    /// to another, each point's colour (linear; empty for one colour).
    Mesh { part: ShapePart, look: ThingLook, shade: Vec<[f32; 3]> },
    /// A thing inside: its name, where it stands, its layers, and the
    /// thing it is where it wears that thing's own looks (a file of its
    /// own can be referred to for it).
    Group { name: String, matrix: Matrix, node: Vec<ThingNode>, from: Option<String> },
}

/// A built thing: its name and layers.
#[derive(Clone, Debug, PartialEq)]
pub struct ThingTree {
    pub name: String,
    pub node: Vec<ThingNode>,
}

const IDENTITY: Matrix = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];

impl ThingTree {
    /// Every mesh in it, with the transform from its group to the thing.
    pub fn mesh(&self) -> Vec<(&ShapePart, Matrix)> {
        let mut out = Vec::new();
        walk(&self.node, IDENTITY, &mut out);
        out
    }

    /// How many triangles it has.
    pub fn triangle_count(&self) -> usize {
        self.mesh().iter().map(|(p, _)| p.triangle_count()).sum()
    }

    /// Every look its own meshes wear, once each, in the order first
    /// worn (not those of the things it refers to by their files).
    pub fn looks(&self) -> Vec<ThingLook> {
        let mut out: Vec<ThingLook> = Vec::new();
        let mut stack: Vec<&ThingNode> = self.node.iter().rev().collect();
        while let Some(node) = stack.pop() {
            match node {
                ThingNode::Mesh { look, .. } if out.iter().all(|l| l.name != look.name) => out.push(look.clone()),
                ThingNode::Mesh { .. } => {}
                ThingNode::Group { node, from: None, .. } => stack.extend(node.iter().rev()),
                ThingNode::Group { .. } => {}
            }
        }
        out
    }
}

fn walk<'a>(node: &'a [ThingNode], above: Matrix, out: &mut Vec<(&'a ShapePart, Matrix)>) {
    for n in node {
        match n {
            ThingNode::Mesh { part, .. } => out.push((part, above)),
            ThingNode::Group { matrix, node, .. } => walk(node, times(matrix, &above), out),
        }
    }
}

/// Row-layout transforms one after the other: `a`, then `b`.
fn times(a: &Matrix, b: &Matrix) -> Matrix {
    [0, 1, 2, 3].map(|i| [0, 1, 2, 3].map(|j| (0..4).map(|k| a[i][k] * b[k][j]).sum()))
}

impl ThingSet {
    /// The thing named `name`, built; none where there is no such thing.
    pub fn build(&self, name: &str) -> Option<ThingTree> {
        let thing = self.named(name)?;
        Some(ThingTree { name: thing.name.clone(), node: self.layers(&thing.row, &[], 0) })
    }

    fn layers(&self, row: &[ThingRow], swap: &[HashMap<String, String>], deep: usize) -> Vec<ThingNode> {
        row.iter().flat_map(|r| self.layer(r, swap, deep)).collect()
    }

    /// One row's layers: one mesh for a part and all its copies, or a
    /// group for each copy of a thing.
    fn layer(&self, row: &ThingRow, swap: &[HashMap<String, String>], deep: usize) -> Vec<ThingNode> {
        let copy = (0..row.repeat).map(|k| {
            let k = k as f32;
            let step = turning(row.step_turn.map(|d| d * k));
            let at = apply(&step, [0, 1, 2].map(|i| row.at[i] + row.step_at[i] * k));
            (mul(&step, &turning(row.turn)), at)
        });
        match row.form {
            ThingForm::Thing if deep < DEEP => {
                let Some(inner) = self.named(&row.from) else { return Vec::new() };
                let mut swaps = vec![swapped(&row.look)];
                swaps.extend(swap.iter().cloned());
                let group = |(turn, at): (Turn, [f32; 3])| {
                    let axis = [0, 1, 2].map(|c| [0, 1, 2].map(|r| turn[r][c] * row.size[c]));
                    let matrix = [[axis[0][0], axis[0][1], axis[0][2], 0.0], [axis[1][0], axis[1][1], axis[1][2], 0.0], [axis[2][0], axis[2][1], axis[2][2], 0.0], [at[0], at[1], at[2], 1.0]];
                    // Its own file shows it as it is here where no look of
                    // it is swapped.
                    let from = Some(inner.name.clone()).filter(|_| self.names(&inner.row, &swaps, deep + 1) == self.names(&inner.row, &[], deep + 1));
                    ThingNode::Group { name: row.name.clone(), matrix, node: self.layers(&inner.row, &swaps, deep + 1), from }
                };
                copy.map(group).collect()
            }
            ThingForm::Thing => Vec::new(),
            _ => {
                let look = self.worn(&row.look, swap);
                let shape = ShapeLook { name: row.name.clone(), colour: look.linear(), roughness: look.roughness, metallic: look.metallic };
                let Some(own) = form::mesh(self, row, &shape) else { return Vec::new() };
                let mut part = ShapePart { point: Vec::new(), count: Vec::new(), index: Vec::new(), normal: Vec::new(), ..own.clone() };
                for (turn, at) in copy {
                    let start = part.point.len() as u32;
                    part.point.extend(own.point.iter().map(|p| near(add(apply(&turn, *p), at), POINT)));
                    part.normal.extend(own.normal.iter().map(|n| near(apply(&turn, *n), NORMAL)));
                    part.count.extend(&own.count);
                    part.index.extend(own.index.iter().map(|i| i + start));
                }
                let shade = shaded(&part, &look);
                vec![ThingNode::Mesh { part, look, shade }]
            }
        }
    }

    /// The names of the looks rows wear, through the swaps given, in
    /// row order, as deep as their things go.
    fn names(&self, row: &[ThingRow], swap: &[HashMap<String, String>], deep: usize) -> Vec<String> {
        let mut out = Vec::new();
        for r in row {
            match (r.form, self.named(&r.from)) {
                (ThingForm::Thing, Some(inner)) if deep < DEEP => {
                    let mut swaps = vec![swapped(&r.look)];
                    swaps.extend(swap.iter().cloned());
                    out.extend(self.names(&inner.row, &swaps, deep + 1));
                }
                (ThingForm::Thing, _) => {}
                _ => out.push(self.worn(&r.look, swap).name),
            }
        }
        out
    }

    /// The look a part wears: its own name taken through every swap,
    /// the nearest first; grey where the collection has no such look.
    fn worn(&self, name: &str, swap: &[HashMap<String, String>]) -> ThingLook {
        let name = swap.iter().fold(name.to_string(), |name, map| map.get(&name).cloned().unwrap_or(name));
        self.look.get(&name).cloned().unwrap_or(ThingLook { name, colour: [0.5; 3], to: None, roughness: 0.5, metallic: 0.0 })
    }
}

/// Each point's colour where a look turns from one colour to another:
/// side to side along the part, its back (-x) in the first, its front
/// (+x) in the second, as the house's grounds turn.
fn shaded(part: &ShapePart, look: &ThingLook) -> Vec<[f32; 3]> {
    let Some(to) = look.linear_to() else { return Vec::new() };
    let from = look.linear();
    let (lo, hi) = part.point.iter().fold((f32::MAX, f32::MIN), |(lo, hi), p| (lo.min(p[0]), hi.max(p[0])));
    let across = (hi - lo).max(f32::EPSILON);
    part.point.iter().map(|p| {
        let t = (p[0] - lo) / across;
        [0, 1, 2].map(|i| from[i] + (to[i] - from[i]) * t)
    }).collect()
}

/// A thing row's swaps: `from=to` words.
fn swapped(look: &str) -> HashMap<String, String> {
    look.split_whitespace().filter_map(|w| w.split_once('=')).map(|(a, b)| (a.to_string(), b.to_string())).collect()
}

type Turn = [[f32; 3]; 3];

/// A turn of so many degrees about x, then y, then z (USD's rotateXYZ).
fn turning(degree: [f32; 3]) -> Turn {
    let about = |axis: usize, d: f32| {
        let (s, c) = d.to_radians().sin_cos();
        let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
        let mut m = [[0.0; 3]; 3];
        m[axis][axis] = 1.0;
        (m[u][u], m[u][v], m[v][u], m[v][v]) = (c, -s, s, c);
        m
    };
    mul(&about(2, degree[2]), &mul(&about(1, degree[1]), &about(0, degree[0])))
}

fn mul(a: &Turn, b: &Turn) -> Turn {
    [0, 1, 2].map(|i| [0, 1, 2].map(|j| (0..3).map(|k| a[i][k] * b[k][j]).sum()))
}

fn apply(m: &Turn, v: [f32; 3]) -> [f32; 3] {
    [0, 1, 2].map(|i| m[i][0] * v[0] + m[i][1] * v[1] + m[i][2] * v[2])
}

/// A point or normal kept to `fine` parts of a unit (never a minus zero).
fn near(v: [f32; 3], fine: f32) -> [f32; 3] {
    v.map(|c| (c * fine).round() / fine + 0.0)
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
