//! The surface shader a material's Material Output is linked to
//! (`ntreeShaderOutputNode`: the output with `NODE_DO_OUTPUT`, else the
//! first), folded to the one [`BlendBsdf`] the tracer takes. Cycles and
//! EEVEE evaluate the closure tree itself; here a Mix Shader lerps the
//! two shaders' parameters by its factor, an Add Shader averages them and
//! sums their emission, and each closure node lands as the Principled
//! parameters that draw most like it (`node_shader_bsdf_*.cc`'s sockets):
//! Glass and Refraction as full transmission at their IOR, Transparent
//! and Holdout as alpha 0, Emission as its radiance over black, Diffuse
//! and Translucent as rough dielectrics, Glossy as a metal, Sheen as the
//! sheen lobe, Subsurface Scattering as the subsurface lobe. That fold is
//! `surface_bsdf`, what anything wanting one set of numbers reads; the
//! tracer takes the Mix Shader on the output as two lobes instead
//! (`surface_mix`, [`BlendMix`]), each side folded, and shades both — so a
//! Mix of unlike closures renders as both, not as one in-between lobe.

use super::eval::BGraph;
use super::links::BLink;
use super::nodes::read_cstr;
use crate::blend::file::types::*;
use crate::blend::scene::types::BlendBsdf;

/// The Mix Shader a material's surface is: its two closures, each folded,
/// `fac` the weight of the second (0 all the first, 1 all the second), and
/// which of the two holds the Principled the surface reaches — the one
/// whose textures the material carries — if either.
#[derive(Clone, Debug, PartialEq)]
pub struct BlendMix {
    pub first: BlendBsdf,
    pub second: BlendBsdf,
    pub fac: f32,
    pub principled: Option<u8>,
}

/// `NODE_DO_OUTPUT` (`DNA_node_types.h`): the output node in use.
const NODE_DO_OUTPUT: i32 = 1 << 6;
/// Shader nodes a surface search passes through to the closures.
const THROUGH: [&str; 3] = ["MixShader", "AddShader", "NodeReroute"];

impl BlendFile {
    pub(crate) fn kind(&self, node: &BlendBlock, dna: &DnaType) -> String {
        self.off("bNode", "idname", dna).map(|o| read_cstr(&node.data, o, 64)).unwrap_or_default()
    }

    /// The active Material Output of `nodes`.
    pub(crate) fn material_output<'a>(&self, nodes: &[&'a BlendBlock], dna: &DnaType) -> Option<&'a BlendBlock> {
        let outputs: Vec<&BlendBlock> = nodes.iter().copied().filter(|n| self.kind(n, dna).contains("OutputMaterial")).collect();
        let flag = self.off("bNode", "flag", dna);
        let active = outputs.iter().copied().find(|n| flag.and_then(|o| n.data.get(o..o + 4)).is_some_and(|b| self.read_i32(b) & NODE_DO_OUTPUT != 0));
        active.or(outputs.first().copied())
    }

    /// The node linked into `node`'s input socket `index`.
    fn linked_node<'a>(&self, nodes: &[&'a BlendBlock], links: &[BLink], node: &BlendBlock, index: usize, dna: &DnaType) -> Option<&'a BlendBlock> {
        let sock = *self.input_sockets(node, dna).get(index)?;
        let link = links.iter().find(|l| l.to_sock == sock)?;
        nodes.iter().copied().find(|n| n.old_address == link.from)
    }

    /// The node linked into `node`'s input named `name`.
    fn linked_named<'a>(&self, nodes: &[&'a BlendBlock], links: &[BLink], node: &BlendBlock, name: &str, dna: &DnaType) -> Option<&'a BlendBlock> {
        let sock = self.socket_named(node, name, dna)?;
        let link = links.iter().find(|l| l.to_sock == sock)?;
        nodes.iter().copied().find(|n| n.old_address == link.from)
    }

    /// The node the active output's Surface is linked to.
    pub(crate) fn surface_node<'a>(&self, nodes: &[&'a BlendBlock], links: &[BLink], dna: &DnaType) -> Option<&'a BlendBlock> {
        let out = self.material_output(nodes, dna)?;
        self.linked_named(nodes, links, out, "Surface", dna)
    }

    /// The Principled BSDF the surface reaches — through Mix and Add
    /// Shaders and reroutes, the first input first — else the first in the
    /// tree, as before, for a material whose output links elsewhere or
    /// nowhere.
    pub(crate) fn surface_principled<'a>(&'a self, ma: &BlendBlock, dna: &DnaType) -> Option<&'a BlendBlock> {
        let nodes = self.shader_nodes(ma, dna);
        let links = self.links(ma, dna);
        let reached = self.surface_node(&nodes, &links, dna).and_then(|n| self.first_principled(&nodes, &links, n, dna, 0));
        reached.or_else(|| nodes.iter().copied().find(|n| self.is_principled(n, dna)))
    }

    fn first_principled<'a>(&self, nodes: &[&'a BlendBlock], links: &[BLink], node: &'a BlendBlock, dna: &DnaType, depth: u32) -> Option<&'a BlendBlock> {
        if self.is_principled(node, dna) {
            return Some(node);
        }
        let kind = self.kind(node, dna);
        if depth > 16 || !THROUGH.iter().any(|t| kind.contains(t)) {
            return None;
        }
        let shaders: Vec<usize> = if kind.contains("MixShader") { vec![1, 2] } else if kind.contains("AddShader") { vec![0, 1] } else { vec![0] };
        shaders.into_iter().find_map(|i| self.linked_node(nodes, links, node, i, dna).and_then(|n| self.first_principled(nodes, links, n, dna, depth + 1)))
    }

    /// The surface folded to one set of Principled parameters, `None` when
    /// the output links to nothing the fold knows (a group, a volume).
    pub(crate) fn surface_bsdf(&self, ma: &BlendBlock, dna: &DnaType) -> Option<BlendBsdf> {
        let nodes = self.shader_nodes(ma, dna);
        let links = self.links(ma, dna);
        let node = self.surface_node(&nodes, &links, dna)?;
        let graph = BGraph::new(self, ma, dna);
        self.closure(ma, &nodes, &links, &graph, node, dna, 0)
    }

    /// The Mix Shader the surface is, through reroutes, if it is one.
    pub(crate) fn surface_mix_node<'a>(&self, nodes: &[&'a BlendBlock], links: &[BLink], dna: &DnaType) -> Option<&'a BlendBlock> {
        let mut node = self.surface_node(nodes, links, dna)?;
        for _ in 0..16 {
            if !self.kind(node, dna).contains("NodeReroute") {
                break;
            }
            node = self.linked_node(nodes, links, node, 0, dna)?;
        }
        self.kind(node, dna).contains("MixShader").then_some(node)
    }

    /// The surface as two lobes, when it is a Mix Shader (through reroutes)
    /// whose two inputs both fold; `None` for one closure, or a mix the
    /// fold does not know a side of.
    pub(crate) fn surface_mix(&self, ma: &BlendBlock, dna: &DnaType) -> Option<BlendMix> {
        let nodes = self.shader_nodes(ma, dna);
        let links = self.links(ma, dna);
        let node = self.surface_mix_node(&nodes, &links, dna)?;
        let graph = BGraph::new(self, ma, dna);
        let side = |i: usize| self.linked_node(&nodes, &links, node, i, dna);
        let (a, b) = (side(1)?, side(2)?);
        let first = self.closure(ma, &nodes, &links, &graph, a, dna, 1)?;
        let second = self.closure(ma, &nodes, &links, &graph, b, dna, 1)?;
        let reached = self.surface_principled(ma, dna).map(|n| n.old_address);
        let holds = |n: &BlendBlock| self.first_principled(&nodes, &links, n, dna, 1).map(|n| n.old_address) == reached && reached.is_some();
        let principled = if holds(a) { Some(0) } else if holds(b) { Some(1) } else { None };
        Some(BlendMix { first, second, fac: self.arg(&graph, node, "Fac", dna, 0.5).clamp(0.0, 1.0), principled })
    }

    /// `name`'s value at `node`, folded (`eval.rs`), or `default`.
    fn arg(&self, graph: &BGraph, node: &BlendBlock, name: &str, dna: &DnaType, default: f32) -> f32 {
        self.socket_named(node, name, dna).and_then(|s| graph.input_or_default(s, 0)).map_or(default, |v| v.value())
    }

    fn color(&self, graph: &BGraph, node: &BlendBlock, name: &str, dna: &DnaType, default: [f32; 4]) -> [f32; 4] {
        self.socket_named(node, name, dna).and_then(|s| graph.input_or_default(s, 0)).map_or(default, |v| v.rgba)
    }

    fn closure(&self, ma: &BlendBlock, nodes: &[&BlendBlock], links: &[BLink], graph: &BGraph, node: &BlendBlock, dna: &DnaType, depth: u32) -> Option<BlendBsdf> {
        if depth > 16 {
            return None;
        }
        if self.is_principled(node, dna) {
            return self.principled_of(ma, node, dna);
        }
        let kind = self.kind(node, dna);
        let kind = kind.strip_prefix("ShaderNode").unwrap_or(&kind);
        let shader = |i: usize| self.linked_node(nodes, links, node, i, dna).and_then(|n| self.closure(ma, nodes, links, graph, n, dna, depth + 1));
        let white = [1.0, 1.0, 1.0, 1.0];
        let grey = [0.8, 0.8, 0.8, 1.0];
        let plain = |base: [f32; 4], roughness: f32, metallic: f32| BlendBsdf { base, roughness, metallic, ..Default::default() };
        Some(match kind {
            "MixShader" => {
                let fac = self.arg(graph, node, "Fac", dna, 0.5).clamp(0.0, 1.0);
                match (shader(1), shader(2)) {
                    (Some(a), Some(b)) => a.mixed(&b, fac),
                    (a, b) => a.or(b)?,
                }
            }
            "AddShader" => match (shader(0), shader(1)) {
                (Some(a), Some(b)) => a.added(&b),
                (a, b) => a.or(b)?,
            },
            "NodeReroute" => shader(0)?,
            "Emission" => {
                let color = self.color(graph, node, "Color", dna, white);
                BlendBsdf { base: [0.0, 0.0, 0.0, 1.0], roughness: 1.0, emission: [color[0], color[1], color[2]], emission_strength: self.arg(graph, node, "Strength", dna, 1.0), ..Default::default() }
            }
            "BsdfTransparent" | "Holdout" => BlendBsdf { base: self.color(graph, node, "Color", dna, white), alpha: 0.0, ..Default::default() },
            "BsdfGlass" | "BsdfRefraction" => BlendBsdf {
                base: self.color(graph, node, "Color", dna, white),
                roughness: self.arg(graph, node, "Roughness", dna, 0.0),
                ior: self.arg(graph, node, "IOR", dna, 1.5),
                transmission: 1.0,
                ..Default::default()
            },
            "BsdfDiffuse" | "BsdfTranslucent" | "BsdfToon" => plain(self.color(graph, node, "Color", dna, grey), 1.0, 0.0),
            "BsdfGlossy" | "BsdfAnisotropic" | "BsdfMetallic" => BlendBsdf {
                anisotropic: self.arg(graph, node, "Anisotropy", dna, 0.0),
                anisotropic_rotation: self.arg(graph, node, "Rotation", dna, 0.0),
                ..plain(self.color(graph, node, "Color", dna, grey), self.arg(graph, node, "Roughness", dna, 0.5), 1.0)
            },
            "BsdfSheen" | "BsdfVelvet" => BlendBsdf { sheen: 1.0, sheen_roughness: self.arg(graph, node, "Roughness", dna, 0.5), ..plain(self.color(graph, node, "Color", dna, grey), 1.0, 0.0) },
            "SubsurfaceScattering" => BlendBsdf { subsurface: 1.0, subsurface_scale: self.arg(graph, node, "Scale", dna, 0.05), ..plain(self.color(graph, node, "Color", dna, grey), 1.0, 0.0) },
            _ => return None,
        })
    }
}

impl BlendBsdf {
    /// A Mix Shader's two inputs at `fac` (0 the first, 1 the second):
    /// every parameter lerped, emission as radiance.
    pub fn mixed(&self, other: &Self, fac: f32) -> Self {
        let l = |a: f32, b: f32| a + (b - a) * fac;
        let l3 = |a: [f32; 3], b: [f32; 3]| [l(a[0], b[0]), l(a[1], b[1]), l(a[2], b[2])];
        let (ea, eb) = (self.emitted(), other.emitted());
        Self {
            base: [l(self.base[0], other.base[0]), l(self.base[1], other.base[1]), l(self.base[2], other.base[2]), l(self.base[3], other.base[3])],
            roughness: l(self.roughness, other.roughness),
            metallic: l(self.metallic, other.metallic),
            alpha: l(self.alpha, other.alpha),
            ior: l(self.ior, other.ior),
            transmission: l(self.transmission, other.transmission),
            emission: l3(ea, eb),
            emission_strength: 1.0,
            subsurface: l(self.subsurface, other.subsurface),
            subsurface_radius: l3(self.subsurface_radius, other.subsurface_radius),
            subsurface_scale: l(self.subsurface_scale, other.subsurface_scale),
            sheen: l(self.sheen, other.sheen),
            sheen_roughness: l(self.sheen_roughness, other.sheen_roughness),
            sheen_tint: l3(self.sheen_tint, other.sheen_tint),
            coat: l(self.coat, other.coat),
            coat_roughness: l(self.coat_roughness, other.coat_roughness),
            anisotropic: l(self.anisotropic, other.anisotropic),
            anisotropic_rotation: l(self.anisotropic_rotation, other.anisotropic_rotation),
        }
    }

    /// An Add Shader's two inputs: the lobes averaged, the emission summed,
    /// the coverage the union of the two.
    pub fn added(&self, other: &Self) -> Self {
        let (ea, eb) = (self.emitted(), other.emitted());
        Self {
            emission: [ea[0] + eb[0], ea[1] + eb[1], ea[2] + eb[2]],
            emission_strength: 1.0,
            alpha: 1.0 - (1.0 - self.alpha) * (1.0 - other.alpha),
            ..self.mixed(other, 0.5)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixing_lerps_and_adding_sums() {
        let glass = BlendBsdf { base: [1.0, 1.0, 1.0, 1.0], roughness: 0.0, ior: 1.45, transmission: 1.0, ..Default::default() };
        let clear = BlendBsdf { alpha: 0.0, ..Default::default() };
        let half = clear.mixed(&glass, 0.5);
        assert_eq!((half.alpha, half.transmission, half.roughness), (0.5, 0.5, 0.25));
        let whole = clear.mixed(&glass, 1.0);
        assert_eq!((whole.transmission, whole.ior, whole.alpha, whole.emitted()), (1.0, 1.45, 1.0, [0.0; 3]), "factor 1 is the second shader (its emission as radiance)");
        let lamp = BlendBsdf { base: [0.0, 0.0, 0.0, 1.0], emission: [1.0, 0.5, 0.0], emission_strength: 4.0, ..Default::default() };
        let lit = glass.added(&lamp);
        assert_eq!(lit.emission, [4.0, 2.0, 0.0], "emission as radiance, summed");
        assert_eq!(lit.emission_strength, 1.0);
        assert_eq!(lit.alpha, 1.0);
        assert_eq!(lit.transmission, 0.5);
        assert_eq!(clear.added(&clear).alpha, 0.0, "two transparents are transparent");
    }
}
