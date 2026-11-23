//! OpenUSD's binary layers for phiecs: meshes and skeletons read from
//! `.usdc` and `.usdz` (and text) with the `openusd` crate, each where
//! its transforms put it. phiecs reads a scene's text layers into the
//! world (`phiecs::usd`); what those layers reference in binary, the
//! meshes and their rigs, is read here, to be drawn live.

pub mod draw;
mod read;
mod xform;

use std::path::Path;

use openusd::Stage;
use phiecs::comp::Matrix;

use read::{ints, points, text, tokens};
pub use xform::world;

/// A mesh: its prim's path, where it stands in the layer's world, its
/// points and its triangles (three indices each into the points).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Mesh {
    pub path: String,
    pub world: Matrix,
    pub point: Vec<[f32; 3]>,
    pub index: Vec<u32>,
}

impl Mesh {
    /// Its triangles, where its world puts them.
    pub fn triangle(&self) -> impl Iterator<Item = [[f32; 3]; 3]> + '_ {
        self.index.chunks_exact(3).filter_map(|t| Some([self.world.apply(*self.point.get(t[0] as usize)?), self.world.apply(*self.point.get(t[1] as usize)?), self.world.apply(*self.point.get(t[2] as usize)?)]))
    }
}

/// A skeleton: its joints' names (the last step of each joint's path)
/// and where each stands in the layer's world at rest.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Skeleton {
    pub path: String,
    pub joint: Vec<String>,
    pub world: Vec<Matrix>,
}

impl Skeleton {
    /// Where a joint of that name stands, at rest.
    pub fn find(&self, name: &str) -> Option<Matrix> {
        self.joint.iter().position(|j| j == name).and_then(|i| self.world.get(i).copied())
    }
}

/// What a layer holds that can be drawn.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Scene {
    pub mesh: Vec<Mesh>,
    pub skeleton: Vec<Skeleton>,
    /// Its default prim's own transform: what a prim that references it
    /// and writes its own transform stands over.
    pub root: Matrix,
}

impl Scene {
    /// How many triangles it has.
    pub fn count(&self) -> usize {
        self.mesh.iter().map(|m| m.index.len() / 3).sum()
    }
}

/// Read a layer's meshes and skeletons. Nothing where it cannot be
/// opened; a prim that is not active, invisible, or only a guide or a
/// proxy is left out.
pub fn read(file: &Path) -> Scene {
    let Ok(stage) = Stage::builder().on_error(|_| Ok(())).open(&file.to_string_lossy()) else { return Scene::default() };
    let mut path = Vec::new();
    let _ = stage.traverse(|p| path.push(p.to_string()));
    let root = stage.default_prim().map(|name| xform::local(&stage, &format!("/{}", name.trim_start_matches('/')))).unwrap_or_default();
    let mut scene = Scene { root, ..Scene::default() };
    for prim in path {
        let hidden = text(&stage, &prim, "visibility") == "invisible" || matches!(text(&stage, &prim, "purpose").as_str(), "guide" | "proxy");
        let gone = path_hidden(&stage, &prim);
        if hidden || gone {
            continue;
        }
        let point = points(&stage, &prim, "points");
        match point.is_empty() {
            false => {
                let index = triangulate(&ints(&stage, &prim, "faceVertexCounts"), &ints(&stage, &prim, "faceVertexIndices"));
                scene.mesh.push(Mesh { world: world(&stage, &prim), path: prim, point, index });
            }
            true => {
                let joint = tokens(&stage, &prim, "joints");
                let rest = read::matrices(&stage, &prim, "restTransforms");
                if !joint.is_empty() && joint.len() == rest.len() {
                    scene.skeleton.push(skeleton(&stage, prim, joint, rest));
                }
            }
        }
    }
    scene
}

/// Whether a prim or one above it is not active.
fn path_hidden(stage: &Stage, prim: &str) -> bool {
    let mut at = prim.to_string();
    while !at.is_empty() && at != "/" {
        if stage.field::<bool>(at.as_str(), "active").ok().flatten() == Some(false) {
            return true;
        }
        at = at.rsplit_once('/').map(|(above, _)| above.to_string()).unwrap_or_default();
    }
    false
}

/// A skeleton's joints in the layer's world: each joint's rest is
/// relative to the joint its path names before it.
fn skeleton(stage: &Stage, path: String, joint: Vec<String>, rest: Vec<Matrix>) -> Skeleton {
    let root = world(stage, &path);
    let mut placed: Vec<Matrix> = Vec::with_capacity(joint.len());
    for (i, name) in joint.iter().enumerate() {
        let parent = name.rsplit_once('/').and_then(|(above, _)| joint.iter().position(|j| j == above)).and_then(|p| placed.get(p).copied()).unwrap_or(root);
        placed.push(parent.then(&rest[i]));
    }
    let short = joint.iter().map(|j| j.rsplit('/').next().unwrap_or(j).to_string()).collect();
    Skeleton { path, joint: short, world: placed }
}

/// USD's faces (each a count of corners) as triangles, fanned from
/// each face's first corner; a face of fewer than three is left out.
pub fn triangulate(count: &[i32], index: &[i32]) -> Vec<u32> {
    let mut out = Vec::with_capacity(index.len() * 2);
    let mut at = 0usize;
    for &n in count {
        let n = n.max(0) as usize;
        let face = index.get(at..at + n).unwrap_or_default();
        for k in 1..n.saturating_sub(1) {
            out.extend([face[0], face[k], face[k + 1]].map(|i| i.max(0) as u32));
        }
        at += n;
    }
    out
}
