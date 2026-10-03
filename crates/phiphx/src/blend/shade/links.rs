//! Node links (`bNodeTree.links` → `bNodeLink {fromnode, tonode, fromsock,
//! tosock}`): follows Principled `Normal` ← Normal Map ← Image Texture, the
//! chain Blender builds for tangent-space normal maps.

use super::nodes::read_cstr;
use super::principled::BSocket;
use crate::blend::file::types::*;

/// Nodes a colour search does not pass through (they carry normals).
const SKIP: [&str; 2] = ["Bump", "NormalMap"];

/// One link as saved addresses.
#[derive(Clone, Copy, Debug)]
pub(crate) struct BLink {
    pub from: u64,
    pub to: u64,
    pub from_sock: u64,
    pub to_sock: u64,
}

/// A material's tangent-space normal map.
#[derive(Clone, Debug, PartialEq)]
pub struct BlendNormalMap {
    /// Image key (see `image.rs`).
    pub image: String,
    /// Normal Map node `Strength`.
    pub strength: f32,
}

impl BlendFile {
    pub(crate) fn links(&self, ma: &BlendBlock, dna: &DnaType) -> Vec<BLink> {
        self.tree_links(ma, "Material", dna)
    }

    /// The links of `id`'s node tree (`owner` names the datablock type).
    pub(crate) fn tree_links(&self, id: &BlendBlock, owner: &str, dna: &DnaType) -> Vec<BLink> {
        let off = |s: &str, f: &str| self.off(s, f, dna);
        let Some(nt) = self.block_at(self.ptr(id, off(owner, "*nodetree"))) else { return Vec::new() };
        let (from, to) = (off("bNodeLink", "*fromnode"), off("bNodeLink", "*tonode"));
        let (fsock, sock) = (off("bNodeLink", "*fromsock"), off("bNodeLink", "*tosock"));
        self.list(self.ptr(nt, off("bNodeTree", "links")), off("bNodeLink", "*next"))
            .into_iter()
            .map(|l| BLink { from: self.ptr(l, from), to: self.ptr(l, to), from_sock: self.ptr(l, fsock), to_sock: self.ptr(l, sock) })
            .collect()
    }

    fn node_named<'a>(&'a self, nodes: &[&'a BlendBlock], addr: u64, dna: &DnaType, idname: &str) -> Option<&'a BlendBlock> {
        let o = self.off("bNode", "idname", dna)?;
        nodes.iter().copied().find(|n| n.old_address == addr && read_cstr(&n.data, o, 64).contains(idname))
    }

    pub(crate) fn socket_name(&self, addr: u64, dna: &DnaType) -> String {
        match (self.block_at(addr), self.off("bNodeSocket", "name", dna)) {
            (Some(s), Some(o)) => read_cstr(&s.data, o, 64),
            _ => String::new(),
        }
    }

    /// The node feeding `node`'s input socket `name`.
    fn feeding(&self, links: &[BLink], node: u64, name: &str, dna: &DnaType) -> Option<u64> {
        links.iter().find(|l| l.to == node && self.socket_name(l.to_sock, dna) == name).map(|l| l.from)
    }

    /// The first Image Texture upstream of Principled `Base Color`.
    pub(crate) fn base_color_image(&self, ma: &BlendBlock, dna: &DnaType) -> Option<Option<String>> {
        self.socket_image(ma, dna, "Base Color")
    }

    /// The first Image Texture upstream of the Principled input `socket`
    /// (breadth-first, sockets in order, not through Bump / Normal Map):
    /// `Some(None)` when there is none, `None` without a Principled node
    /// (the caller then falls back to the first Image Texture).
    pub(crate) fn socket_image(&self, ma: &BlendBlock, dna: &DnaType, socket: &str) -> Option<Option<String>> {
        let nodes = self.shader_nodes(ma, dna);
        let bsdf = nodes.iter().find(|n| self.is_principled(n, dna))?;
        let links = self.links(ma, dna);
        let idname = self.off("bNode", "idname", dna);
        let kind = |n: &BlendBlock| idname.map(|o| read_cstr(&n.data, o, 64)).unwrap_or_default();
        let base = self.input_sockets(bsdf, dna).into_iter().find(|&s| self.socket_name(s, dna) == socket)?;
        let mut queue = std::collections::VecDeque::from([base]);
        let mut seen = Vec::new();
        while let Some(sock) = queue.pop_front() {
            let Some(link) = links.iter().find(|l| l.to_sock == sock) else { continue };
            let Some(node) = nodes.iter().copied().find(|n| n.old_address == link.from) else { continue };
            let k = kind(node);
            if seen.contains(&link.from) || SKIP.iter().any(|s| k.contains(s)) {
                continue;
            }
            seen.push(link.from);
            if k.contains("TexImage") {
                let im = self.block_at(self.ptr(node, self.off("bNode", "*id", dna)));
                if let Some(key) = im.map(|im| self.image_key(im, dna)).filter(|k| !k.is_empty()) {
                    return Some(Some(key));
                }
                continue;
            }
            queue.extend(self.input_sockets(node, dna));
        }
        Some(None)
    }

    pub(crate) fn input_sockets(&self, node: &BlendBlock, dna: &DnaType) -> Vec<u64> {
        let first = self.ptr(node, self.off("bNode", "inputs", dna));
        self.list(first, self.off("bNodeSocket", "*next", dna)).into_iter().map(|s| s.old_address).collect()
    }

    /// Principled `Normal` ← Normal Map ← Image Texture, if the material has it.
    pub(crate) fn normal_map(&self, ma: &BlendBlock, dna: &DnaType) -> Option<BlendNormalMap> {
        let nodes = self.shader_nodes(ma, dna);
        let bsdf = nodes.iter().find(|n| self.is_principled(n, dna))?;
        let links = self.links(ma, dna);
        let nm = self.feeding(&links, bsdf.old_address, "Normal", dna)?;
        let nm = self.node_named(&nodes, nm, dna, "NormalMap")?;
        let strength = self
            .socket_values(nm, dna)
            .into_iter()
            .find_map(|(n, v)| match (n.as_str(), v) {
                ("Strength", BSocket::Float(f)) => Some(f),
                _ => None,
            })
            .unwrap_or(1.0);
        let tex = self.feeding(&links, nm.old_address, "Color", dna)?;
        let tex = self.node_named(&nodes, tex, dna, "TexImage")?;
        let im = self.block_at(self.ptr(tex, self.off("bNode", "*id", dna)))?;
        let image = self.image_key(im, dna);
        (!image.is_empty()).then_some(BlendNormalMap { image, strength })
    }
}
