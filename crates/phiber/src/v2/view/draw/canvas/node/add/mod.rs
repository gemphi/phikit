//! Adding nodes: the overlay of nodes the user has created, merged over the
//! file's own tree.
//!
//! The file is read-only — phiber reads the node tree out of the `.blend` on
//! every frame (`link/mod.rs` says why) — so a NEW node is an overlay, exactly
//! as a made link or a drag displacement is. `apply` merges the added nodes into
//! the file's list; nothing is written back until `phiphx` grows a node-tree
//! writer, at which point this layer feeds it and becomes the undo buffer.
//!
//! What a node of a given type IS — its sockets — comes from the [`catalog`],
//! not from the store: the overlay records only the type, the name and the
//! place, and the catalog supplies the shape. That keeps the stored value small
//! and keeps one source for a type's sockets, the same way the file is the one
//! source for an existing node's.
//!
//! This is the foundation of the Add menu (`node_add.cc`): the menu, when it
//! lands, writes `NodeKey::Added` and the link-drag search creates-and-links
//! through this same overlay. Nothing writes it yet, so an empty overlay merges
//! to the file unchanged.

pub mod catalog;
pub mod store;

use crate::phiphx::blend::shade::graph::BlendNode;
pub use catalog::{BAddCategory, BAddSocket, BNodeDecl, BShaderNode, CATALOG, find, search};
pub use store::{additions_of, decode, encode};

/// One node the user has added: its type, its unique name, and where it sits
/// (view units, the same space `bNode.location` is in).
#[derive(Clone, Debug, PartialEq)]
pub struct BNodeAddition {
    pub idname: String,
    pub name: String,
    pub at: (f32, f32),
}

/// The nodes added on top of the file's own, for one host.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BNodeAdditions {
    pub added: Vec<BNodeAddition>,
}

impl BNodeAdditions {
    /// The file's nodes with the added ones merged in — what gets drawn.
    ///
    /// An addition whose type the catalog does not carry is dropped rather than
    /// merged as a typeless, socketless box: a node phiber cannot give the right
    /// sockets is one it should not pretend to have added. The file is cloned,
    /// never mutated — the whole point of an overlay.
    pub fn apply(&self, file: &[BlendNode]) -> Vec<BlendNode> {
        let mut out = file.to_vec();
        for a in &self.added {
            if let Some(ty) = catalog::find(&a.idname) {
                out.push(instantiate(ty, &a.name, a.at));
            }
        }
        out
    }
}

/// A fresh file-shaped node of `ty`, named `name`, at `at` — Blender's
/// `nodeAddNode`: the type's declared sockets, selected off, parented into
/// nothing, and no saved address (an added node is resolved by name like every
/// node).
pub fn instantiate(ty: &dyn BShaderNode, name: &str, at: (f32, f32)) -> BlendNode {
    let decl = ty.sockets();
    BlendNode {
        idname: ty.idname().into(),
        name: name.into(),
        x: at.0,
        y: at.1,
        inputs: decl.inputs.iter().map(|s| s.to_socket()).collect(),
        outputs: decl.outputs.iter().map(|s| s.to_socket()).collect(),
        ..Default::default()
    }
}

/// A name for a new node that no existing node already has — Blender's
/// `nodeUniqueName` (`BLI_uniquename`): the base if it is free, else the base
/// with a `.001`, `.002`, … suffix, three digits, counting up to the first gap.
///
/// `base` is the type's label, which is what Blender names a fresh node after
/// (`Emission`, then `Emission.001`). `taken` is every name already in the
/// graph — the file's nodes and the ones already added — because a duplicate
/// name would make two nodes answer to one id, and selection and links are by
/// name.
pub fn unique_name(base: &str, taken: &[&str]) -> String {
    if !taken.iter().any(|n| *n == base) {
        return base.to_string();
    }
    for i in 1..10000 {
        let candidate = format!("{base}.{i:03}");
        if !taken.iter().any(|n| *n == candidate) {
            return candidate;
        }
    }
    base.to_string()
}

/// `current` with one more node of `ty` added at `at`, uniquely named against
/// `taken`. The caller encodes the result and writes `NodeKey::Added` — this is
/// the pure half, so the naming and the append can be checked without a store.
pub fn place(current: &BNodeAdditions, taken: &[&str], ty: &dyn BShaderNode, at: (f32, f32)) -> BNodeAdditions {
    let mut next = current.clone();
    next.added.push(BNodeAddition {
        idname: ty.idname().into(),
        name: unique_name(ty.label(), taken),
        at,
    });
    next
}

/// The Add menu's rows, one per searched type — `canvas/node/add/menu.rs`.
pub mod menu;

#[cfg(test)]
mod tests;
