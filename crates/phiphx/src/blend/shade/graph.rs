//! A material's node graph as the Shader editor needs to draw it — every node
//! with its type, name, position and size, and every link between them.
//!
//! `shade/nodes.rs` already walks `bNodeTree.nodes`, but only to find one Image
//! Texture's filepath, and `shade/links.rs` follows the normal-map chain. Both
//! throw the graph away. The Shader editor had nothing to draw from, which is why
//! it drew two hardcoded cards — the same "Principled BSDF" and "Material Output"
//! for every file ever opened.
//!
//! Ported from Blender 5.2.0 in `crates/phiber/refs/blender`,
//! `makesdna/DNA_node_types.h:1620`:
//!
//! | field | what it gives |
//! |---|---|
//! | `idname[64]` | the node's TYPE — `ShaderNodeBsdfPrincipled` and so on |
//! | `name[64]` | its unique name in the tree |
//! | `label[64]` | the user's own label, when they set one |
//! | `location[2]` | where it sits on the canvas |
//! | `width`, `height` | the card's size |
//! | `flag` | `NODE_SELECT` among others |
//!
//! The one trap is `location`. Blender 5.x stores the position in
//! `float location[2]`, but files saved by older versions carry it in
//! `locx_legacy` / `locy_legacy` (and older still, `locx` / `locy`) — the fields
//! sit side by side in the struct precisely because both exist in the wild. A
//! reader that takes only `location` puts every node of an older file at the
//! origin, stacked on top of each other, which looks like a layout bug rather
//! than a parsing one. `position_of` tries each spelling against the file's OWN
//! DNA, so whichever the file was written with is the one read.

use super::nodes::read_cstr;
use super::socket::BlendSocket;
use crate::blend::file::types::*;

/// One node of a material's tree, as the editor draws it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlendNode {
    /// `bNode.idname` — the type, e.g. `ShaderNodeBsdfPrincipled`.
    pub idname: String,
    /// `bNode.name` — unique within the tree, which makes it the id to select by.
    pub name: String,
    /// `bNode.label` — empty unless the user named it; Blender shows the label
    /// when there is one and the type's own name otherwise.
    pub label: String,
    /// `bNode.location` — canvas coordinates, y UP as Blender's are.
    pub x: f32,
    pub y: f32,
    /// `bNode.width` / `height`.
    pub width: f32,
    pub height: f32,
    /// `bNode.flag`, for `NODE_SELECT`.
    pub flag: i32,
    /// The saved address, so links can name their ends.
    pub address: u64,
    /// `bNode.*parent` — the address of the frame this node is parented into,
    /// or `None`. A frame node (`NodeFrame`) is sized to bound its children, so
    /// a frame has to know which nodes name it as their parent; the address is
    /// what a child carries and the frame's own `address` is what it matches.
    pub parent: Option<u64>,
    /// `bNode.custom1` and `custom2` — the node type's own small enums, where
    /// Blender keeps a choice too simple to deserve a `storage` struct: a Mix
    /// node's blend type, an Invert node's channels, an Alpha Over mode. 5.x
    /// is moving these to menu *sockets* one node at a time, so a reader has
    /// to try the socket first and fall back here — which is why both are
    /// carried rather than one being read on demand.
    pub custom1: i32,
    pub custom2: i32,
    /// `bNode.*storage` — the address of the node type's own settings
    /// struct, where a choice too big for `custom1` lives (`NodeShaderMix`'s
    /// `blend_type`, a `CurveMapping`, a `NodeTexSky`). The address rather
    /// than the contents, because what the struct *is* depends on the node
    /// type: the reader that wants it knows which struct to read.
    pub storage: u64,
    /// `bNode.inputs` and `bNode.outputs`, in the file's own order — which is
    /// the order Blender lays the rows out in, so it is not sorted. Where each
    /// one SITS is not in the file (`socket.rs`); phiber lays that out.
    pub inputs: Vec<BlendSocket>,
    pub outputs: Vec<BlendSocket>,
}

/// `NODE_SELECT` (`DNA_node_types.h:204`): the node is selected in the editor.
pub const NODE_SELECT: i32 = 1 << 0;
/// `NODE_COLLAPSED` (`DNA_node_types.h:207`): the node is folded to a pill —
/// H in the editor. Blender lays it out and draws it a different way entirely
/// (`node_update_collapsed` / `node_draw_collapsed`), so a reader that ignores
/// this flag draws a saved file wrong rather than merely plainly.
pub const NODE_COLLAPSED: i32 = 1 << 3;
/// `NODE_MUTED` (`DNA_node_types.h:215`): the node passes its input through
/// untouched, drawn faded with a wire across it (M in the editor).
pub const NODE_MUTED: i32 = 1 << 9;
/// `NODE_ACTIVE` (`DNA_node_types.h:208`): the node that was clicked last.
///
/// Separate from selection and not a subset of it in spelling, though in
/// practice the active node is selected. The distinction earns its keep in two
/// places: `compare_node_depth` ranks **active above selected above nothing**,
/// and `node_mouse_select`'s toggle checks `NODE_SELECT && NODE_ACTIVE` so that
/// shift-clicking a selected-but-inactive node makes it active instead of
/// deselecting it — *"Check active so clicking on an inactive node activates
/// it."*
pub const NODE_ACTIVE: i32 = 1 << 4;
/// `NODE_DO_OUTPUT` (`DNA_node_types.h:211`): this is the output in use.
pub const NODE_DO_OUTPUT: i32 = 1 << 6;

impl BlendNode {
    /// What the header prints: the user's label when they set one, else the name.
    ///
    /// Blender's `nodeLabel` falls back through the type's own UI name; without a
    /// type registry the node's `name` is the closest honest stand-in, and for an
    /// unlabelled node Blender's name is already the type's name with a numeric
    /// suffix (`Principled BSDF.001`).
    pub fn title(&self) -> &str {
        if self.label.is_empty() { &self.name } else { &self.label }
    }

    pub fn selected(&self) -> bool {
        self.flag & NODE_SELECT != 0
    }

    /// Whether the file marks this as the active node.
    pub fn active(&self) -> bool {
        self.flag & NODE_ACTIVE != 0
    }

    /// Whether the file has this node folded to a pill (`NODE_COLLAPSED`).
    pub fn collapsed(&self) -> bool {
        self.flag & NODE_COLLAPSED != 0
    }

    /// Whether the node is muted (`NODE_MUTED`): M in the editor. Blender draws
    /// it `BUT_INACTIVE` — its contents faded and its outline blended toward the
    /// background — and runs a pass-through wire across it. A reader that
    /// ignores the flag draws a muted node as a live one.
    pub fn muted(&self) -> bool {
        self.flag & NODE_MUTED != 0
    }

    /// Whether this is the output actually being rendered (`NODE_DO_OUTPUT`),
    /// which is what decides whether an output node gets the output colour.
    pub fn active_output(&self) -> bool {
        self.flag & NODE_DO_OUTPUT != 0
    }

    /// Whether this is a reroute (`NODE_REROUTE`): the little dot that tidies a
    /// graph's wiring without changing it. Blender keys it off the TYPE, not a
    /// flag — `node.is_reroute()` is `type == NODE_REROUTE`, and the type's
    /// `idname` is `"NodeReroute"` (`node_common.cc:651`).
    ///
    /// It is drawn unlike any other node (`reroute_node_prepare_for_draw`): no
    /// card, header, title or body — just one socket dot, with its single input
    /// and single output both at the node's own location. A reader that ignores
    /// the type draws a reroute as a full card with a title bar, which is the
    /// shape phiber had before the type was recognised.
    pub fn reroute(&self) -> bool {
        self.idname == "NodeReroute"
    }

    /// Whether this is a frame (`NODE_FRAME`): the labelled box drawn BEHIND a
    /// group of nodes to show they belong together. Blender keys it off the type
    /// (`node.is_frame()` is `type == NODE_FRAME`), and the type's `idname` is
    /// `"NodeFrame"` (`node_common.cc`).
    ///
    /// A frame is not a card: it has no sockets, no header and no title bar, and
    /// it is sized to bound the nodes parented into it rather than to its own
    /// content (`calc_node_frame_dimensions`). A reader that ignores the type
    /// draws a frame as a tiny titled card floating among the nodes it is meant
    /// to contain, which is the shape phiber had before the type was recognised.
    pub fn frame(&self) -> bool {
        self.idname == "NodeFrame"
    }
}

/// One link, by the names of the nodes it joins and the sockets on each.
///
/// Blender's `bNodeLink` carries four pointers — `fromnode`, `fromsock`,
/// `tonode`, `tosock` — because a link joins SOCKETS, not nodes: two links
/// between the same pair of nodes are an ordinary thing and are drawn as two
/// separate curves. Resolving the ends to names and identifiers rather than
/// keeping the addresses means a link survives being handed out of the reader.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlendNodeLink {
    pub from: String,
    pub to: String,
    /// `bNodeSocket.identifier` of each end, which is what picks the row the
    /// curve starts and finishes at.
    pub from_socket: String,
    pub to_socket: String,
}

/// A material's whole graph.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlendNodeGraph {
    pub nodes: Vec<BlendNode>,
    pub links: Vec<BlendNodeLink>,
}

impl BlendNodeGraph {
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

impl BlendFile {
    /// A material's node graph — `Material.*nodetree`, the same pointer
    /// `image_texture_path` follows to hunt for a texture.
    pub(crate) fn material_graph(&self, ma: &BlendBlock, dna: &DnaType) -> Option<BlendNodeGraph> {
        let ps = self.header.pointer_size as usize;
        let nt_off = Self::struct_field_offset_exact("Material", "*nodetree", dna, ps)?;
        let nt_addr = Self::read_ptr(&ma.data, nt_off, ps, self.header.little_endian);
        (nt_addr != 0).then(|| self.node_graph_at(nt_addr, dna)).flatten()
    }

    /// The node graph of the tree at `nt_addr`, or `None` when there is no tree.
    pub(crate) fn node_graph_at(&self, nt_addr: u64, dna: &DnaType) -> Option<BlendNodeGraph> {
        let ps = self.header.pointer_size as usize;
        let le = self.header.little_endian;
        let nt = self.block_at(nt_addr)?;

        let nodes_off = Self::struct_field_offset_exact("bNodeTree", "nodes", dna, ps)?;
        let next_off = Self::struct_field_offset_exact("bNode", "*next", dna, ps)?;
        let idname_off = Self::struct_field_offset_exact("bNode", "idname", dna, ps);
        let name_off = Self::struct_field_offset_exact("bNode", "name", dna, ps);
        let label_off = Self::struct_field_offset_exact("bNode", "label", dna, ps);
        let width_off = Self::struct_field_offset_exact("bNode", "width", dna, ps);
        let height_off = Self::struct_field_offset_exact("bNode", "height", dna, ps);
        let flag_off = Self::struct_field_offset_exact("bNode", "flag", dna, ps);
        let parent_off = Self::struct_field_offset_exact("bNode", "*parent", dna, ps);
        // `custom1` / `custom2` are `short` in every version that has them.
        let custom1_off = Self::struct_field_offset_exact("bNode", "custom1", dna, ps);
        let custom2_off = Self::struct_field_offset_exact("bNode", "custom2", dna, ps);
        let storage_off = Self::struct_field_offset_exact("bNode", "*storage", dna, ps);

        let mut nodes = Vec::new();
        let mut addr = Self::read_ptr(&nt.data, nodes_off, ps, le);
        let mut guard = 0;
        while addr != 0 && guard < 8192 {
            guard += 1;
            let Some(node) = self.block_at(addr) else { break };
            let text = |off: Option<usize>| off.map_or_else(String::new, |o| read_cstr(&node.data, o, 64));
            let f32_at = |off: Option<usize>| {
                off.and_then(|o| node.data.get(o..o + 4)).map_or(0.0, |b| self.read_f32(b))
            };
            let (x, y) = self.node_position(&node.data, dna, ps);
            nodes.push(BlendNode {
                idname: text(idname_off),
                name: text(name_off),
                label: text(label_off),
                x,
                y,
                width: f32_at(width_off),
                height: f32_at(height_off),
                flag: flag_off.and_then(|o| node.data.get(o..o + 4)).map_or(0, |b| self.read_i32(b)),
                address: addr,
                custom1: custom1_off.and_then(|o| node.data.get(o..o + 2)).map_or(0, |b| i32::from(self.read_u16(b) as i16)),
                custom2: custom2_off.and_then(|o| node.data.get(o..o + 2)).map_or(0, |b| i32::from(self.read_u16(b) as i16)),
                storage: storage_off.map_or(0, |o| Self::read_ptr(&node.data, o, ps, le)),
                parent: parent_off
                    .map(|o| Self::read_ptr(&node.data, o, ps, le))
                    .filter(|&p| p != 0),
                inputs: self.node_sockets(node, "inputs", dna),
                outputs: self.node_sockets(node, "outputs", dna),
            });
            addr = Self::read_ptr(&node.data, next_off, ps, le);
        }

        let links = self.node_links(nt, &nodes, dna, ps, le);
        Some(BlendNodeGraph { nodes, links })
    }

    /// `bNode`'s position, whichever spelling this file was written with.
    ///
    /// 5.x writes `float location[2]`; older files carry `locx_legacy` /
    /// `locy_legacy`, and older still `locx` / `locy`. Each is tried against the
    /// file's own DNA, so an old file does not come back with every node at the
    /// origin.
    fn node_position(&self, data: &[u8], dna: &DnaType, ps: usize) -> (f32, f32) {
        let at = |name: &str| {
            Self::struct_field_offset_exact("bNode", name, dna, ps)
                .and_then(|o| data.get(o..o + 4))
                .map(|b| self.read_f32(b))
        };
        if let Some(o) = Self::struct_field_offset_exact("bNode", "location[2]", dna, ps)
            .or_else(|| Self::struct_field_offset_exact("bNode", "location", dna, ps))
        {
            let x = data.get(o..o + 4).map(|b| self.read_f32(b));
            let y = data.get(o + 4..o + 8).map(|b| self.read_f32(b));
            if let (Some(x), Some(y)) = (x, y) {
                return (x, y);
            }
        }
        let x = at("locx_legacy").or_else(|| at("locx")).unwrap_or(0.0);
        let y = at("locy_legacy").or_else(|| at("locy")).unwrap_or(0.0);
        (x, y)
    }

    /// `bNodeTree.links` → `bNodeLink {*fromnode, *tonode}`, resolved to the node
    /// names the graph carries, so a link survives without the addresses.
    fn node_links(
        &self,
        nt: &BlendBlock,
        nodes: &[BlendNode],
        dna: &DnaType,
        ps: usize,
        le: bool,
    ) -> Vec<BlendNodeLink> {
        let Some(links_off) = Self::struct_field_offset_exact("bNodeTree", "links", dna, ps) else {
            return Vec::new();
        };
        let (Some(next), Some(from), Some(to)) = (
            Self::struct_field_offset_exact("bNodeLink", "*next", dna, ps),
            Self::struct_field_offset_exact("bNodeLink", "*fromnode", dna, ps),
            Self::struct_field_offset_exact("bNodeLink", "*tonode", dna, ps),
        ) else {
            return Vec::new();
        };
        let from_sock = Self::struct_field_offset_exact("bNodeLink", "*fromsock", dna, ps);
        let to_sock = Self::struct_field_offset_exact("bNodeLink", "*tosock", dna, ps);
        let named = |addr: u64| {
            nodes.iter().find(|n| n.address == addr).map(|n| n.name.clone()).unwrap_or_default()
        };
        // A socket end is an ADDRESS in the file; the identifier it corresponds
        // to is only knowable by looking it up among the sockets already read.
        let socket = |addr: u64| {
            nodes
                .iter()
                .flat_map(|n| n.inputs.iter().chain(n.outputs.iter()))
                .find(|s| s.address == addr)
                .map(|s| s.identifier.clone())
                .unwrap_or_default()
        };
        let mut out = Vec::new();
        let mut addr = Self::read_ptr(&nt.data, links_off, ps, le);
        let mut guard = 0;
        while addr != 0 && guard < 8192 {
            guard += 1;
            let Some(link) = self.block_at(addr) else { break };
            let (a, b) = (
                named(Self::read_ptr(&link.data, from, ps, le)),
                named(Self::read_ptr(&link.data, to, ps, le)),
            );
            // A link whose ends are not both in the tree is not drawable.
            if !a.is_empty() && !b.is_empty() {
                let at = |off: Option<usize>| {
                    off.map_or_else(String::new, |o| socket(Self::read_ptr(&link.data, o, ps, le)))
                };
                out.push(BlendNodeLink {
                    from: a,
                    to: b,
                    from_socket: at(from_sock),
                    to_socket: at(to_sock),
                });
            }
            addr = Self::read_ptr(&link.data, next, ps, le);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_node_shows_its_label_when_it_has_one_and_its_name_otherwise() {
        // `nodeLabel`: the user's label wins; without one the node's own name is
        // what Blender has already made from the type (`Principled BSDF.001`).
        let mut node = BlendNode { name: "Principled BSDF".into(), ..Default::default() };
        assert_eq!(node.title(), "Principled BSDF");
        node.label = "Skin".into();
        assert_eq!(node.title(), "Skin", "a labelled node is called by its label");
    }

    #[test]
    fn selection_reads_the_flag_bit_blender_sets() {
        let mut node = BlendNode::default();
        assert!(!node.selected());
        node.flag = NODE_SELECT;
        assert!(node.selected());
        // Other bits in `flag` are not selection.
        node.flag = 1 << 4;
        assert!(!node.selected(), "only NODE_SELECT means selected");
    }

    #[test]
    fn a_reroute_is_known_by_its_type_name() {
        // `node.is_reroute()` is a TYPE test in Blender, not a flag — the type's
        // idname is `"NodeReroute"`. Every other node is drawn as a card; a
        // reroute is drawn as a bare dot, so getting this wrong draws the dot as
        // a card.
        let mut node = BlendNode::default();
        assert!(!node.reroute(), "an untyped node is not a reroute");
        node.idname = "NodeReroute".into();
        assert!(node.reroute());
        node.idname = "ShaderNodeBsdfPrincipled".into();
        assert!(!node.reroute(), "a real shader node is not a reroute");
    }

    #[test]
    fn a_frame_is_known_by_its_type_name_and_carries_its_parent() {
        // Like `reroute`, `is_frame()` is a TYPE test — the idname is
        // `"NodeFrame"`. A node not inside any frame carries no parent.
        let mut node = BlendNode::default();
        assert!(!node.frame(), "an untyped node is not a frame");
        assert_eq!(node.parent, None, "and it is parented into nothing");
        node.idname = "NodeFrame".into();
        assert!(node.frame());
        node.idname = "ShaderNodeMath".into();
        assert!(!node.frame());
        // A child names its frame by the frame's own address.
        node.parent = Some(0xBEEF);
        assert_eq!(node.parent, Some(0xBEEF));
    }

    #[test]
    fn an_empty_graph_says_so() {
        assert!(BlendNodeGraph::default().is_empty());
        let one = BlendNodeGraph {
            nodes: vec![BlendNode { name: "A".into(), ..Default::default() }],
            links: Vec::new(),
        };
        assert!(!one.is_empty());
    }
}
