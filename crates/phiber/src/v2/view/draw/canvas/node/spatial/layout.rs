//! The shared layout pass: where every node and every link is, in view units.
//!
//! The flat editor answers these questions in `plan.rs`, in canvas pixels, from
//! `view::node_bounds`, `BCardSize::height_of`, `socket::rows` and the drag
//! displacements. This reads the SAME functions and stops before the pixels,
//! so the two modes cannot disagree about where a socket is: a point here
//! projected straight down (`camera.rs`, pitch 90) is the point `plan` paints.
//! That agreement is what makes the second mode a renderer over the layout
//! rather than a second implementation of it, and `tests.rs` holds it.
//!
//! What is added, and is the spatial view's own: the class of each node and
//! the solid the class is drawn as (`set.rs`), and the height each link end
//! sits at — the rim of its node — which the flat view has no use for.

use super::super::plan::BCardSize;
use super::super::view::node_bounds;
use super::super::{frame, link, select, socket};
use super::set::BSolidSpec;
use crate::phiphx::blend::shade::graph::{BlendNode, BlendNodeGraph, BlendNodeLink};
use crate::v2::theme::{BNodeClass, class_of};

/// One node's footprint and height.
#[derive(Clone, Debug, PartialEq)]
pub struct BSolid {
    pub name: String,
    /// What the top face prints — the label when the user set one, else the
    /// name (`BlendNode::title`).
    pub title: String,
    /// The footprint in view units, Y up: left, bottom, right, top.
    pub rect: (f32, f32, f32, f32),
    pub class: BNodeClass,
    pub spec: BSolidSpec,
    /// How tall it stands above the floor, in view units.
    pub height: f32,
    /// `select::rank`: 0 plain, 1 selected, 2 active.
    pub rank: u8,
    /// A muted node reads as off, in both modes.
    pub muted: bool,
}

impl BSolid {
    pub fn width(&self) -> f32 {
        self.rect.2 - self.rect.0
    }

    pub fn depth(&self) -> f32 {
        self.rect.3 - self.rect.1
    }

    /// The footprint's centre on the floor.
    pub fn centre(&self) -> (f32, f32) {
        ((self.rect.0 + self.rect.2) / 2.0, (self.rect.1 + self.rect.3) / 2.0)
    }
}

/// One link, as two points in graph space — each on the rim of its node.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BTubeEnd {
    pub from: [f32; 3],
    pub to: [f32; 3],
}

/// The whole graph, laid out.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BSpatialLayout {
    pub solids: Vec<BSolid>,
    /// Frames, as floor plates — drawn first, under everything.
    pub plates: Vec<BSolid>,
    pub tubes: Vec<BTubeEnd>,
}

/// The per-node drag displacements, as `plan::positions` gathers them.
pub type Offsets<'a> = &'a [(String, (f32, f32))];

impl BSpatialLayout {
    pub fn of(
        graph: &BlendNodeGraph,
        wired: &[BlendNodeLink],
        offsets: Offsets,
        size: BCardSize,
        selection: &select::BNodeSelection,
    ) -> Self {
        let mut out = Self::default();
        for node in &graph.nodes {
            let class = class_of(&node.idname, node.active_output());
            let spec = BSolidSpec::of(class);
            let rank = select::rank(node, selection);
            if node.frame() {
                let Some(rect) = frame::bounds_of(node, &graph.nodes, size, offsets, 0) else { continue };
                out.plates.push(BSolid { name: node.name.clone(), title: node.title().to_string(), rect, class, spec, height: spec.height(size.header), rank, muted: node.muted() });
                continue;
            }
            let rect = Self::rect_of(node, wired, offsets, size);
            out.solids.push(BSolid { name: node.name.clone(), title: node.title().to_string(), rect, class, spec, height: spec.height(size.header), rank, muted: node.muted() });
        }
        for wire in wired {
            let Some(from) = Self::socket_point(graph, wired, offsets, size, &out, &wire.from, &wire.from_socket, socket::BSocketSide::Output) else { continue };
            let Some(to) = Self::socket_point(graph, wired, offsets, size, &out, &wire.to, &wire.to_socket, socket::BSocketSide::Input) else { continue };
            out.tubes.push(BTubeEnd { from, to });
        }
        out
    }

    /// A node's drawn footprint: its file rect, sized by its real,
    /// link-dependent height, shifted by its drag displacement.
    fn rect_of(node: &BlendNode, wired: &[BlendNodeLink], offsets: Offsets, size: BCardSize) -> (f32, f32, f32, f32) {
        let linked = link::linked_inputs(wired, &node.name);
        let (x0, y0, x1, y1) = node_bounds(node, size.height_of(node, &linked), size.width);
        let (ox, oy) = Self::shift(offsets, &node.name);
        (x0 + ox, y0 + oy, x1 + ox, y1 + oy)
    }

    fn shift(offsets: Offsets, name: &str) -> (f32, f32) {
        offsets.iter().find(|(n, _)| n == name).map(|(_, o)| *o).unwrap_or((0.0, 0.0))
    }

    /// Where a link end sits: the socket's place on the card's edge
    /// (`BNodeCard::socket_at`, in view units and Y up), lifted to the rim of
    /// its solid. A reroute's two sockets coincide at its location.
    #[allow(clippy::too_many_arguments)]
    fn socket_point(
        graph: &BlendNodeGraph,
        wired: &[BlendNodeLink],
        offsets: Offsets,
        size: BCardSize,
        laid: &Self,
        node_name: &str,
        identifier: &str,
        side: socket::BSocketSide,
    ) -> Option<[f32; 3]> {
        let node = graph.nodes.iter().find(|n| n.name == node_name)?;
        let solid = laid.solids.iter().find(|s| s.name == node_name)?;
        let (ox, oy) = Self::shift(offsets, node_name);
        if node.reroute() {
            return Some([node.x + ox, node.y + oy, solid.height]);
        }
        let linked = link::linked_inputs(wired, node_name);
        let row = socket::row_of(node, identifier, &linked)
            .or_else(|| socket::rows(node, &linked).into_iter().find(|r| r.side == side))?;
        let x = match row.side {
            socket::BSocketSide::Input => solid.rect.0,
            socket::BSocketSide::Output => solid.rect.2,
        };
        // Rows are measured down from the header's bottom, and the header
        // hangs from the node's LOCATION — `plan` places the card at the
        // location, not at `node_bounds`'s top, which a collapsed pill lifts
        // above it (`socket::collapsed_top`). A collapsed node has no header
        // band at all (`BCardSize::height_of`).
        let header = if node.collapsed() { 0.0 } else { size.header };
        let y = node.y + oy - header - row.y * size.unit;
        Some([x, y, solid.height])
    }

    /// The floor the graph covers: the union of every footprint, or `None`
    /// for an empty graph.
    pub fn floor(&self) -> Option<(f32, f32, f32, f32)> {
        self.solids.iter().chain(self.plates.iter()).map(|s| s.rect).reduce(|a, b| {
            (a.0.min(b.0), a.1.min(b.1), a.2.max(b.2), a.3.max(b.3))
        })
    }
}
