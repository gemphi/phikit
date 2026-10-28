//! Node groups expanded inline — the tree a `ShaderNodeGroup` instances
//! spliced into its parent before the fold runs, the way Cycles'
//! `ShaderGraph` sees it after `ntreeShaderExpandGroups`
//! (`node_shader_tree.cc`): every link into the group's input socket is
//! re-aimed at the sockets its Group Input node fed, every link out of
//! the group's output socket is re-aimed at what fed the active Group
//! Output, and the two interface nodes vanish. 88 of the user's 230 files
//! wrap their materials in groups; without this each group was an
//! unknown node and every socket behind it kept its default.
//!
//! A group input the instance leaves unlinked carries the instance's own
//! socket value: that link is kept pointing at the group node and its
//! input socket, and the fold reads the socket's `default_value` when it
//! meets a link whose source is a `ShaderNodeGroup` (`eval.rs`). A group
//! output whose Group Output input is unlinked does the same with that
//! input's value.
//!
//! One group instanced twice in a tree shares its inner nodes between the
//! two instances: inner socket addresses are the file's, so the second
//! instance's links land on the same sockets and the first instance's
//! win. Groups nest to [`MAX_DEPTH`].

use super::links::BLink;
use crate::blend::file::types::*;

/// Groups within groups within groups: how deep the expansion goes.
const MAX_DEPTH: u32 = 8;

impl BlendFile {
    /// `id`'s node tree with every group expanded; `owner` names the
    /// datablock type holding the `*nodetree` pointer.
    pub(crate) fn flat_nodes(&self, id: &BlendBlock, owner: &str, dna: &DnaType) -> Vec<&BlendBlock> {
        self.flat_tree(id, owner, dna).0
    }

    /// The links of `id`'s node tree with every group expanded.
    pub(crate) fn flat_links(&self, id: &BlendBlock, owner: &str, dna: &DnaType) -> Vec<BLink> {
        self.flat_tree(id, owner, dna).1
    }

    fn flat_tree(&self, id: &BlendBlock, owner: &str, dna: &DnaType) -> (Vec<&BlendBlock>, Vec<BLink>) {
        match self.block_at(self.ptr(id, self.off(owner, "*nodetree", dna))) {
            Some(nt) => self.expand(nt, dna, 0),
            None => (Vec::new(), Vec::new()),
        }
    }

    /// The nodes and links of tree `nt`, each `ShaderNodeGroup` replaced
    /// by its tree.
    fn expand(&self, nt: &BlendBlock, dna: &DnaType, depth: u32) -> (Vec<&BlendBlock>, Vec<BLink>) {
        let mut nodes = self.list(self.ptr(nt, self.off("bNodeTree", "nodes", dna)), self.off("bNode", "*next", dna));
        let mut links = self.links_of(nt, dna);
        if depth >= MAX_DEPTH {
            return (nodes, links);
        }
        let groups: Vec<&BlendBlock> = nodes.iter().copied().filter(|n| self.kind(n, dna) == "ShaderNodeGroup").collect();
        for g in groups {
            // A group whose tree is missing (a library the file lost) stays
            // an unknown node: its outputs fold to nothing, the sockets it
            // fed keep their defaults, as Blender draws it red and dead.
            let Some(sub) = self.block_at(self.ptr(g, self.off("bNode", "*id", dna))) else { continue };
            self.inline(g, sub, &mut nodes, &mut links, dna, depth);
        }
        (nodes, links)
    }

    /// A socket's `identifier` — what a group node's input and its tree's
    /// Group Input output share.
    fn sock_id(&self, addr: u64, dna: &DnaType) -> String {
        match (self.block_at(addr), self.off("bNodeSocket", "identifier", dna)) {
            (Some(s), Some(o)) => super::nodes::read_cstr(&s.data, o, 64),
            _ => String::new(),
        }
    }

    /// Splice group `sub`, instanced by node `g`, into `nodes` and `links`.
    fn inline<'a>(&'a self, g: &'a BlendBlock, sub: &'a BlendBlock, nodes: &mut Vec<&'a BlendBlock>, links: &mut Vec<BLink>, dna: &DnaType, depth: u32) {
        let (inner_nodes, inner_links) = self.expand(sub, dna, depth + 1);
        let ins: Vec<u64> = inner_nodes.iter().filter(|n| self.kind(n, dna) == "NodeGroupInput").map(|n| n.old_address).collect();
        let out = self.active_output(&inner_nodes, "NodeGroupOutput", dna);
        let out_addr = out.map(|n| n.old_address);
        let g_in = |id: &str| self.input_sockets(g, dna).into_iter().find(|&s| self.sock_id(s, dna) == id);
        let go_in = |id: &str| out.and_then(|o| self.input_sockets(o, dna).into_iter().find(|&s| self.sock_id(s, dna) == id));
        let outer_in: Vec<BLink> = links.iter().copied().filter(|l| l.to == g.old_address).collect();
        let outer_out: Vec<BLink> = links.iter().copied().filter(|l| l.from == g.old_address).collect();
        links.retain(|l| l.to != g.old_address && l.from != g.old_address);
        // Inner links from a Group Input: from what fed the instance's
        // socket, else from the instance's socket itself (its value).
        let mut rewired: Vec<BLink> = Vec::with_capacity(inner_links.len());
        for l in &inner_links {
            if !ins.contains(&l.from) {
                rewired.push(*l);
                continue;
            }
            let Some(gs) = g_in(&self.sock_id(l.from_sock, dna)) else { continue };
            rewired.push(match outer_in.iter().find(|o| o.to_sock == gs) {
                Some(o) => BLink { from: o.from, from_sock: o.from_sock, to: l.to, to_sock: l.to_sock },
                None => BLink { from: g.old_address, from_sock: gs, to: l.to, to_sock: l.to_sock },
            });
        }
        // Outer links from the instance: from what feeds the Group
        // Output's matching input (already rewired, so a pass-through
        // input reaches outside the group), else that input's value.
        for o in &outer_out {
            let Some(gos) = go_in(&self.sock_id(o.from_sock, dna)) else { continue };
            links.push(match rewired.iter().find(|l| l.to_sock == gos) {
                Some(l) => BLink { from: l.from, from_sock: l.from_sock, to: o.to, to_sock: o.to_sock },
                None => BLink { from: g.old_address, from_sock: gos, to: o.to, to_sock: o.to_sock },
            });
        }
        links.extend(rewired.into_iter().filter(|l| Some(l.to) != out_addr));
        let interface = |n: &&BlendBlock| ins.contains(&n.old_address) || Some(n.old_address) == out_addr || self.kind(n, dna) == "NodeGroupOutput";
        nodes.extend(inner_nodes.iter().copied().filter(|n| !interface(n)));
    }
}
