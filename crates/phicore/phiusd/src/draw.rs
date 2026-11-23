//! What is drawn of a phiecs world: every thing under a word whose prim
//! names a binary asset (`phiecs:asset`, kept by phiecs's stage), with
//! that asset's meshes placed where the world puts the thing. An asset is
//! read once and kept, however many things name it.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use phiecs::comp::{Matrix, children};
use phiecs::comp::tree::matrix;
use phiecs::prelude::*;
use phiecs::usd::Kind;
use phiecs::usd::stage::ASSET;

use super::{Scene, read};

/// The assets read so far, by file.
#[derive(Debug, Clone, Default)]
pub struct Library {
    kept: HashMap<PathBuf, Scene>,
}

impl Library {
    /// An asset's scene, read now if it was not before.
    pub fn scene(&mut self, file: &Path) -> &Scene {
        self.kept.entry(file.to_path_buf()).or_insert_with(|| read(file))
    }

    /// Keep another library's scenes too (one read away from here).
    pub fn take(&mut self, other: Library) {
        self.kept.extend(other.kept);
    }
}

/// A thing drawn: its word, the matrix from its asset's world to the
/// world, and the asset.
#[derive(Debug, Clone)]
pub struct Placed {
    pub word: Word,
    pub asset: PathBuf,
    pub matrix: Matrix,
}

/// Every thing under a word (itself too) that names an asset, and where
/// its asset's world stands in the world. Where the thing writes its own
/// transform, it stands over the asset's root's: the asset's root is
/// undone first, as a reference's own opinions win in USD.
pub fn placed(world: &World, under: Word, library: &mut Library) -> Vec<Placed> {
    placed_but(world, under, &[], library)
}

/// The same, but for what stands under the words skipped (the robot,
/// walking in the greenhouse it is drawn apart from).
pub fn placed_but(world: &World, under: Word, skip: &[Word], library: &mut Library) -> Vec<Placed> {
    let mut out = Vec::new();
    let mut stack = vec![under];
    while let Some(word) = stack.pop() {
        if skip.contains(&word) {
            continue;
        }
        stack.extend(children(world, word));
        let Some(kind) = world.read::<Kind>(word) else { continue };
        let Some(asset) = kind.get(ASSET).map(|v| PathBuf::from(v.text())) else { continue };
        let own = kind.get("xformOpOrder").is_some();
        let root = library.scene(&asset).root;
        let undo = match own {
            true => root.inverse().unwrap_or_default(),
            false => Matrix::default(),
        };
        out.push(Placed { word, asset, matrix: matrix(world, word).then(&undo) });
    }
    out
}

/// The triangles of every thing under a word, in the world, but for
/// what stands under the words skipped.
pub fn triangles(world: &World, under: Word, skip: &[Word], library: &mut Library) -> Vec<[[f32; 3]; 3]> {
    let placed = placed_but(world, under, skip, library);
    let mut out = Vec::new();
    for p in placed {
        let scene = library.scene(&p.asset);
        for mesh in &scene.mesh {
            let m = p.matrix.then(&mesh.world);
            out.extend(mesh.index.chunks_exact(3).filter(|t| t.iter().all(|&i| (i as usize) < mesh.point.len())).map(|t| [0, 1, 2].map(|k| m.apply(mesh.point[t[k] as usize]))));
        }
    }
    out
}
