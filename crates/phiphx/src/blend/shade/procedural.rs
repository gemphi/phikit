//! The procedural textures a material's surface reads per pixel — what the
//! colour fold (`eval.rs`) can only stand in for with a mean: Principled
//! `Base Color` ← [Color Ramp ←] Noise / Voronoi Texture, the surface Mix
//! Shader's `Fac` the same way, and Principled `Normal` ← Bump whose
//! `Height` is one. Each lands as a [`BlendProc`]: the texture node —
//! Noise, Voronoi, Checker, Brick, Gradient, Magic or Wave — with its
//! inputs and settings (`texnode.rs`), its coordinates (Generated unless
//! `Vector` is linked, through a Point Mapping's Location and Scale to a
//! Texture Coordinate output) and the ramp, if any, baked to 256 linear
//! RGBA stops (`BKE_colorband_evaluate`) the way the tracer samples it
//! (`phiray`'s `procedural.wgsl`).

use super::eval::BGraph;
use super::links::BLink;
use super::principled::BSocket;
use crate::blend::file::types::*;

pub use super::texnode::{BlendProc, BlendProcCoord, BlendProcKind};

/// A Bump node on the Principled's Normal, its Height a texture.
#[derive(Clone, Debug, PartialEq)]
pub struct BlendBump {
    pub height: BlendProc,
    pub strength: f32,
    pub distance: f32,
}

/// The textures a material's surface reads per pixel.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlendProcedural {
    /// Principled ▸ Base Color.
    pub base: Option<BlendProc>,
    /// The surface Mix Shader ▸ Fac.
    pub mix: Option<BlendProc>,
    /// Principled ▸ Normal ← Bump.
    pub bump: Option<BlendBump>,
}

/// Steps of a ramp bake: the ramp image is `RAMP_STOPS × 1`.
pub const RAMP_STOPS: usize = 256;

impl BlendFile {
    /// The textures `ma`'s surface reads per pixel (`BlendProcedural`).
    pub(crate) fn surface_procedural(&self, ma: &BlendBlock, dna: &DnaType) -> BlendProcedural {
        let nodes = self.shader_nodes(ma, dna);
        let links = self.links(ma, dna);
        let graph = BGraph::new(self, ma, dna);
        let find = BProcFinder { f: self, nodes: &nodes, links: &links, graph: &graph, dna };
        let principled = self.surface_principled(ma, dna);
        BlendProcedural {
            base: principled.and_then(|p| find.at(p, "Base Color")),
            mix: self.surface_mix_node(&nodes, &links, dna).and_then(|m| find.at(m, "Fac")),
            bump: principled.and_then(|p| find.bump(p)),
        }
    }
}

/// The search for a texture upstream of a socket.
pub(super) struct BProcFinder<'a> {
    pub f: &'a BlendFile,
    nodes: &'a [&'a BlendBlock],
    links: &'a [BLink],
    graph: &'a BGraph<'a>,
    pub dna: &'a DnaType,
}

impl BProcFinder<'_> {
    pub fn kind(&self, node: &BlendBlock) -> String {
        self.f.kind(node, self.dna)
    }

    /// The link into `node`'s input `name`, with the node it comes from.
    fn link_into(&self, node: &BlendBlock, name: &str) -> Option<(&BlendBlock, BLink)> {
        let sock = self.f.socket_named(node, name, self.dna)?;
        let link = *self.links.iter().find(|l| l.to_sock == sock)?;
        Some((self.nodes.iter().copied().find(|n| n.old_address == link.from)?, link))
    }

    /// Through a reroute: the link into its one input.
    fn rerouted(&self, node: &BlendBlock) -> Option<(&BlendBlock, BLink)> {
        let first = self.f.input_sockets(node, self.dna).first().copied()?;
        self.link_into(node, &self.f.socket_name(first, self.dna))
    }

    /// `node`'s float input `name`, folded, or `default`.
    pub fn float(&self, node: &BlendBlock, name: &str, default: f32) -> f32 {
        self.f.socket_named(node, name, self.dna).and_then(|s| self.graph.input_or_default(s, 0)).map_or(default, |v| v.value())
    }

    /// `node`'s colour input `name`, folded, or `default`.
    pub fn color(&self, node: &BlendBlock, name: &str, default: [f32; 3]) -> [f32; 3] {
        let c = self.f.socket_named(node, name, self.dna).and_then(|s| self.graph.input_or_default(s, 0));
        c.map_or(default, |v| [v.rgba[0], v.rgba[1], v.rgba[2]])
    }

    /// `node`'s vector input `name` as stored, or `default`.
    fn vec3(&self, node: &BlendBlock, name: &str, default: [f32; 3]) -> [f32; 3] {
        match self.f.socket_values(node, self.dna).into_iter().find(|(n, _)| n == name) {
            Some((_, BSocket::Vec3(v))) => v,
            _ => default,
        }
    }

    /// The texture feeding `node`'s input `name`: through reroutes and one
    /// Color Ramp, to a texture node `texnode.rs` reads.
    fn at(&self, node: &BlendBlock, name: &str) -> Option<BlendProc> {
        let (mut from, mut link) = self.link_into(node, name)?;
        let mut ramp = None;
        for _ in 0..16 {
            let kind = self.kind(from);
            if kind.contains("NodeReroute") {
                (from, link) = self.rerouted(from)?;
            } else if kind.contains("ValToRGB") && ramp.is_none() {
                ramp = Some(self.bake(from)?);
                (from, link) = self.link_into(from, "Fac")?;
            } else {
                break;
            }
        }
        let output = self.f.socket_name(link.from_sock, self.dna);
        let (coords, location, scale3) = self.coordinates(from);
        Some(BlendProc { coords, location, scale3, ramp, ..self.texture(from, &output)? })
    }

    /// The texture's `Vector`: unlinked is Generated; a Mapping node gives
    /// its Location and Scale and is looked through to the coordinate.
    fn coordinates(&self, tex: &BlendBlock) -> (BlendProcCoord, [f32; 3], [f32; 3]) {
        let (mut location, mut scale3) = ([0.0; 3], [1.0; 3]);
        let Some((mut from, mut link)) = self.link_into(tex, "Vector") else { return (BlendProcCoord::Generated, location, scale3) };
        for _ in 0..8 {
            let kind = self.kind(from);
            let next = if kind.contains("Mapping") {
                location = self.vec3(from, "Location", location);
                scale3 = self.vec3(from, "Scale", scale3);
                self.link_into(from, "Vector")
            } else if kind.contains("NodeReroute") {
                self.rerouted(from)
            } else {
                break;
            };
            let Some(n) = next else { break };
            (from, link) = n;
        }
        let kind = self.kind(from);
        let coords = match (kind.contains("UVMap"), self.f.socket_name(link.from_sock, self.dna).as_str()) {
            (true, _) | (_, "UV") => BlendProcCoord::Uv,
            (_, "Object") => BlendProcCoord::Object,
            _ => BlendProcCoord::Generated,
        };
        (coords, location, scale3)
    }

    /// Principled `Normal` ← Bump: its Strength and Distance, and the
    /// texture on its Height.
    fn bump(&self, principled: &BlendBlock) -> Option<BlendBump> {
        let (node, _) = self.link_into(principled, "Normal")?;
        if !self.kind(node).contains("ShaderNodeBump") {
            return None;
        }
        Some(BlendBump { height: self.at(node, "Height")?, strength: self.float(node, "Strength", 1.0), distance: self.float(node, "Distance", 1.0) })
    }

    /// A Color Ramp baked: its colour at each of `RAMP_STOPS` positions,
    /// linear, clamped to bytes.
    fn bake(&self, ramp: &BlendBlock) -> Option<Vec<u8>> {
        let band = self.f.block_at(self.f.ptr(ramp, self.f.off("bNode", "*storage", self.dna)))?;
        let mut out = Vec::with_capacity(RAMP_STOPS * 4);
        for i in 0..RAMP_STOPS {
            let c = super::blend::ramp(self.f, self.dna, band, i as f32 / (RAMP_STOPS - 1) as f32)?;
            out.extend(c.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8));
        }
        Some(out)
    }
}
