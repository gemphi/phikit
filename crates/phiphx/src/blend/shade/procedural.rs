//! The procedural textures a material's surface reads per pixel — what the
//! colour fold (`eval.rs`) can only stand in for with a mean: Principled
//! `Base Color` ← [Color Ramp ←] Noise / Voronoi Texture, the surface Mix
//! Shader's `Fac` the same way, and Principled `Normal` ← Bump whose
//! `Height` is one — or, with no Bump, the Material Output's
//! `Displacement` ← Displacement node, which Cycles turns into a Bump of
//! Distance 1 over `(height − midlevel) · scale`
//! (`ShaderGraph::bump_from_displacement`, `svm_node_displacement`): the
//! midlevel vanishes in the derivative, so it is the Height at Distance
//! `scale`. Each lands as a [`BlendProc`]: the texture node — Noise,
//! Voronoi, Checker, Brick, Gradient, Magic, Wave, White Noise, Gabor —
//! or a Fresnel / Layer Weight (`texnode.rs`), its coordinates (Generated
//! unless `Vector` is linked, through a Point Mapping's Location and
//! Scale to a Texture Coordinate output) and the ramp, if any, baked to
//! 256 linear RGBA stops (`BKE_colorband_evaluate`) the way the tracer
//! samples it (`phiray`'s `procedural.wgsl`).
//!
//! The scalar nodes on the way — Math, Map Range — and the ramp's bake
//! are `procscalar.rs`.

use super::eval::BGraph;
use super::links::BLink;
use super::principled::BSocket;
use super::scalar::BScalarOp;
use crate::blend::file::types::*;

/// `eNodeSocketDatatype::SOCK_RGBA`, a Mix node over colours.
const SOCK_RGBA: u8 = 2;

pub use super::texnode::{BlendMapping, BlendMath, BlendProc, BlendProcCoord, BlendProcKind};

/// The second texture of a Base Color Mix node, with the mode and factor
/// it is mixed over the first by (`svm_mix`, `ramp_blend`): a floor whose
/// planks are a Wave through a ramp and whose grain is a Noise is one of
/// these, and folding it to one texture lost the grain (or the planks).
#[derive(Clone, Debug, PartialEq)]
pub struct BlendProcMix {
    pub texture: BlendProc,
    /// The Mix node's Factor, folded and clamped as the node clamps it.
    pub fac: f32,
    /// `MA_RAMP_*` (`NodeShaderMix.blend_type`, legacy `bNode.custom1`).
    pub blend: u8,
}

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
    /// The second texture, when Base Color is a Mix of two.
    pub base_mix: Option<BlendProcMix>,
    /// The surface Mix Shader ▸ Fac.
    pub mix: Option<BlendProc>,
    /// Principled ▸ Normal ← Bump, or the output's Displacement as one.
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
        let output = self.material_output(&nodes, dna);
        let (base, base_mix) = principled.map_or((None, None), |p| find.at_mix(p, "Base Color"));
        BlendProcedural {
            base,
            base_mix,
            mix: self.surface_mix_node(&nodes, &links, dna).and_then(|m| find.at(m, "Fac")),
            bump: principled.and_then(|p| find.bump(p)).or_else(|| output.and_then(|o| find.displacement(o))),
        }
    }
}

/// The search for a texture upstream of a socket.
pub(super) struct BProcFinder<'a> {
    pub f: &'a BlendFile,
    nodes: &'a [&'a BlendBlock],
    links: &'a [BLink],
    pub(super) graph: &'a BGraph<'a>,
    pub dna: &'a DnaType,
}

impl BProcFinder<'_> {
    pub fn kind(&self, node: &BlendBlock) -> String {
        self.f.kind(node, self.dna)
    }

    /// The link into `node`'s input `name`, with the node it comes from.
    pub(super) fn link_into(&self, node: &BlendBlock, name: &str) -> Option<(&BlendBlock, BLink)> {
        let sock = self.f.socket_named(node, name, self.dna)?;
        let link = *self.links.iter().find(|l| l.to_sock == sock)?;
        Some((self.nodes.iter().copied().find(|n| n.old_address == link.from)?, link))
    }

    /// Through a reroute: the link into its one input.
    pub(super) fn rerouted(&self, node: &BlendBlock) -> Option<(&BlendBlock, BLink)> {
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

    /// The textures feeding `node`'s input `name` when a Mix node is the
    /// first thing on the chain: both its colours, the factor folded and
    /// the blend mode. Anything else is one texture, as before — and a
    /// Mix only one side of which is a texture stays one texture's worth
    /// of nothing, since the other side's colour has nowhere to ride.
    fn at_mix(&self, node: &BlendBlock, name: &str) -> (Option<BlendProc>, Option<BlendProcMix>) {
        let Some(mix) = self.mix_into(node, name) else { return (self.at(node, name), None) };
        let rgba = self.kind(&mix) == "ShaderNodeMixRGB" || self.storage_byte(&mix, "NodeShaderMix", "data_type") == Some(SOCK_RGBA);
        let (a, b) = match rgba {
            true => ("A_Color", "B_Color"),
            false => return (self.at(node, name), None),
        };
        let legacy = self.kind(&mix) == "ShaderNodeMixRGB";
        let (first, second) = match legacy {
            true => (self.at(&mix, "Color1"), self.at(&mix, "Color2")),
            false => (self.at(&mix, a), self.at(&mix, b)),
        };
        let (Some(first), Some(second)) = (first, second) else { return (self.at(node, name), None) };
        let blend = match legacy {
            true => self.custom1(&mix).unwrap_or(0) as u8,
            false => self.storage_byte(&mix, "NodeShaderMix", "blend_type").unwrap_or(0),
        };
        let fac = self.float(&mix, if legacy { "Fac" } else { "Factor_Float" }, 0.5).clamp(0.0, 1.0);
        (Some(first), Some(BlendProcMix { texture: second, fac, blend }))
    }

    /// The Mix node feeding `node`'s input `name` through reroutes, if the
    /// chain starts with one.
    fn mix_into(&self, node: &BlendBlock, name: &str) -> Option<BlendBlock> {
        let (mut from, _) = self.link_into(node, name)?;
        for _ in 0..8 {
            match self.kind(from).as_str() {
                "NodeReroute" => from = self.rerouted(from)?.0,
                "ShaderNodeMix" | "ShaderNodeMixRGB" => return Some(from.clone()),
                _ => return None,
            }
        }
        None
    }

    /// The texture feeding `node`'s input `name`: through reroutes, one
    /// Color Ramp and the scalar nodes (Math, Map Range) around it, to a
    /// texture node `texnode.rs` reads. The scalar nodes before the ramp
    /// are composed into its bake; one after it is the slot's tail.
    fn at(&self, node: &BlendBlock, name: &str) -> Option<BlendProc> {
        let (mut from, mut link) = self.link_into(node, name)?;
        let mut ramp_node = None;
        let mut after: Option<BScalarOp> = None;
        let mut before: Vec<BScalarOp> = Vec::new();
        for _ in 0..16 {
            let kind = self.kind(from);
            let next = match (kind.as_str(), ramp_node.is_some()) {
                ("NodeReroute", _) => self.rerouted(from)?,
                ("ShaderNodeValToRGB", false) => {
                    ramp_node = Some(from);
                    self.link_into(from, "Fac")?
                }
                ("ShaderNodeMath" | "ShaderNodeMapRange", ramped) => {
                    let op = self.scalar_op(from, &kind)?;
                    match (ramped, after.is_some()) {
                        (true, _) => before.push(op),
                        (false, false) => after = Some(op),
                        (false, true) => return None,
                    }
                    self.link_into(from, "Value")?
                }
                _ => break,
            };
            (from, link) = next;
        }
        let math = match after {
            Some(op) => Some(Self::tail(op)?),
            None => None,
        };
        let ramp = match ramp_node {
            Some(r) => Some(self.bake(r, &before)?),
            None => None,
        };
        let output = self.f.socket_name(link.from_sock, self.dna);
        let map = self.coordinates(from);
        Some(BlendProc { ramp, math, ..self.texture(from, &output)?.mapped(map) })
    }

    /// The texture's `Vector`: unlinked is Generated; a Mapping node gives
    /// its Location, Rotation, Scale and Type, and is looked through to
    /// the coordinate.
    fn coordinates(&self, tex: &BlendBlock) -> BlendMapping {
        let mut map = BlendMapping::default();
        let Some((mut from, mut link)) = self.link_into(tex, "Vector") else { return map };
        for _ in 0..8 {
            let kind = self.kind(from);
            let next = match kind.contains("Mapping") {
                true => {
                    map.location = self.vec3(from, "Location", map.location);
                    map.rotation = self.vec3(from, "Rotation", map.rotation);
                    map.scale3 = self.vec3(from, "Scale", map.scale3);
                    map.mapping = self.custom1(from).unwrap_or(0);
                    self.link_into(from, "Vector")
                }
                false if kind.contains("NodeReroute") => self.rerouted(from),
                false => None,
            };
            let Some(n) = next else { break };
            (from, link) = n;
        }
        let kind = self.kind(from);
        map.coords = match (kind.contains("UVMap"), self.f.socket_name(link.from_sock, self.dna).as_str()) {
            (true, _) | (_, "UV") => BlendProcCoord::Uv,
            (_, "Object") => BlendProcCoord::Object,
            _ => BlendProcCoord::Generated,
        };
        map
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

    /// Material Output `Displacement` ← Displacement node, as the Bump
    /// Cycles makes of it (`bump_from_displacement`): the Height texture
    /// at Strength 1 and Distance `Scale`.
    fn displacement(&self, output: &BlendBlock) -> Option<BlendBump> {
        let (node, _) = self.link_into(output, "Displacement")?;
        if self.kind(node) != "ShaderNodeDisplacement" {
            return None;
        }
        Some(BlendBump { height: self.at(node, "Height")?, strength: 1.0, distance: self.float(node, "Scale", 1.0) })
    }
}
