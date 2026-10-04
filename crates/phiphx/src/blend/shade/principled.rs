//! Principled BSDF inputs (`node_shader_bsdf_principled.cc`): the socket
//! values of the Principled node the material's surface reaches
//! (`surface.rs`; the first in the tree when none does). Socket names
//! follow Blender 4.x, with the 2.8–3.x spellings as aliases.

use super::nodes::read_cstr;
use crate::blend::file::types::*;
use crate::blend::scene::types::BlendBsdf;

/// `eNodeSocketDatatype`.
const SOCK_FLOAT: u16 = 0;
const SOCK_VECTOR: u16 = 1;
const SOCK_RGBA: u16 = 2;
/// `SH_NODE_BSDF_PRINCIPLED`.
const NODE_PRINCIPLED: u16 = 194;

/// One socket value.
#[derive(Clone, Copy, Debug)]
pub(crate) enum BSocket {
    Float(f32),
    Vec3([f32; 3]),
    Rgba([f32; 4]),
}

/// Socket name(s) → how the value lands in [`BlendBsdf`].
type Apply = fn(&mut BlendBsdf, BSocket);
const INPUTS: &[(&[&str], Apply)] = &[
    (&["Base Color"], |b, v| if let BSocket::Rgba(c) = v { b.base = c }),
    (&["Roughness"], |b, v| if let BSocket::Float(f) = v { b.roughness = f }),
    (&["Metallic"], |b, v| if let BSocket::Float(f) = v { b.metallic = f }),
    (&["Alpha"], |b, v| if let BSocket::Float(f) = v { b.alpha = f }),
    (&["IOR"], |b, v| if let BSocket::Float(f) = v { b.ior = f }),
    (&["Transmission Weight", "Transmission"], |b, v| if let BSocket::Float(f) = v { b.transmission = f }),
    (&["Emission Color", "Emission"], |b, v| if let BSocket::Rgba(c) = v { b.emission = [c[0], c[1], c[2]] }),
    (&["Emission Strength"], |b, v| if let BSocket::Float(f) = v { b.emission_strength = f }),
    (&["Subsurface Weight", "Subsurface"], |b, v| if let BSocket::Float(f) = v { b.subsurface = f }),
    (&["Subsurface Scale"], |b, v| if let BSocket::Float(f) = v { b.subsurface_scale = f }),
    (&["Sheen Weight", "Sheen"], |b, v| if let BSocket::Float(f) = v { b.sheen = f }),
    (&["Sheen Roughness"], |b, v| if let BSocket::Float(f) = v { b.sheen_roughness = f }),
    // 3.x spells Sheen Tint as a float (white → base colour); only the 4.x
    // colour socket lands here, so a 3.x file keeps the white default.
    (&["Sheen Tint"], |b, v| if let BSocket::Rgba(c) = v { b.sheen_tint = [c[0], c[1], c[2]] }),
    (&["Coat Weight", "Clearcoat"], |b, v| if let BSocket::Float(f) = v { b.coat = f }),
    (&["Coat Roughness", "Clearcoat Roughness"], |b, v| if let BSocket::Float(f) = v { b.coat_roughness = f }),
    (&["Anisotropic"], |b, v| if let BSocket::Float(f) = v { b.anisotropic = f }),
    (&["Anisotropic Rotation"], |b, v| if let BSocket::Float(f) = v { b.anisotropic_rotation = f }),
];

/// Inputs that take a representative value even when textures feed them.
const FOLDED: [&str; 3] = ["Base Color", "Roughness", "Metallic"];

impl BlendFile {
    /// The material's Principled BSDF — the one its surface reaches
    /// (`surface_principled`) — `None` without a node tree or node.
    pub(crate) fn principled(&self, ma: &BlendBlock, dna: &DnaType) -> Option<BlendBsdf> {
        let node = self.surface_principled(ma, dna)?;
        self.principled_of(ma, node, dna)
    }

    /// `node`'s Principled inputs. Linked inputs are folded through the
    /// colour graph (`eval.rs`); alpha, emission, IOR and transmission only
    /// when the fold is exact.
    pub(crate) fn principled_of(&self, ma: &BlendBlock, node: &BlendBlock, dna: &DnaType) -> Option<BlendBsdf> {
        let graph = super::eval::BGraph::new(self, ma, dna);
        let type_o = self.off("bNodeSocket", "type", dna)?;
        let mut bsdf = BlendBsdf::default();
        // Blender < 4.0 has no Emission Strength socket: emission is used as is.
        bsdf.emission_strength = 1.0;
        for sock in self.input_sockets(node, dna) {
            let name = self.socket_name(sock, dna);
            let Some((names, apply)) = INPUTS.iter().find(|(names, _)| names.contains(&name.as_str())) else { continue };
            // A link from a node the fold does not know leaves the socket
            // on its own default, as Blender does for an undefined node.
            let Some(v) = graph.input_or_default(sock, 0) else { continue };
            if graph.linked(sock) && !v.exact && !names.iter().any(|n| FOLDED.contains(n)) {
                continue;
            }
            let ty = self.block_at(sock).and_then(|b| b.data.get(type_o..type_o + 2)).map(|b| self.read_u16(b));
            apply(&mut bsdf, if ty == Some(SOCK_RGBA) { BSocket::Rgba(v.rgba) } else { BSocket::Float(v.value()) });
        }
        // Subsurface Radius is a vector socket; the colour fold reduces vectors
        // to their mean, which would throw away the red bleed skin depends on.
        if let Some(BSocket::Vec3(r)) = self.socket_values(node, dna).into_iter().find(|(n, _)| n == "Subsurface Radius").map(|(_, v)| v) {
            bsdf.subsurface_radius = r;
        }
        Some(bsdf)
    }

    pub(crate) fn shader_nodes(&self, ma: &BlendBlock, dna: &DnaType) -> Vec<&BlendBlock> {
        self.tree_nodes(ma, "Material", dna)
    }

    /// The shader nodes of `id`'s node tree; `owner` is the datablock type
    /// holding the `*nodetree` pointer (`Material`, `World`, `Light`).
    pub(crate) fn tree_nodes(&self, id: &BlendBlock, owner: &str, dna: &DnaType) -> Vec<&BlendBlock> {
        let ma = id;
        let ps = self.header.pointer_size as usize;
        let le = self.header.little_endian;
        let (Some(nt_off), Some(nodes_off)) = (
            Self::struct_field_offset_exact(owner, "*nodetree", dna, ps),
            Self::struct_field_offset_exact("bNodeTree", "nodes", dna, ps),
        ) else {
            return Vec::new();
        };
        let Some(nt) = self.block_at(Self::read_ptr(&ma.data, nt_off, ps, le)) else { return Vec::new() };
        let next = Self::struct_field_offset_exact("bNode", "*next", dna, ps);
        self.list(Self::read_ptr(&nt.data, nodes_off, ps, le), next)
    }

    pub(crate) fn is_principled(&self, node: &BlendBlock, dna: &DnaType) -> bool {
        let ps = self.header.pointer_size as usize;
        let by_name = Self::struct_field_offset_exact("bNode", "idname", dna, ps)
            .is_some_and(|o| read_cstr(&node.data, o, 64).contains("Principled"));
        let by_type = Self::struct_field_offset_exact("bNode", "type", dna, ps)
            .and_then(|o| node.data.get(o..o + 2))
            .is_some_and(|b| self.read_u16(b) == NODE_PRINCIPLED);
        by_name || by_type
    }

    /// `(name, value)` of every float / colour input socket of `node`.
    pub(crate) fn socket_values(&self, node: &BlendBlock, dna: &DnaType) -> Vec<(String, BSocket)> {
        let ps = self.header.pointer_size as usize;
        let le = self.header.little_endian;
        let off = |s: &str, f: &str| Self::struct_field_offset_exact(s, f, dna, ps);
        let (Some(inputs), Some(name_o), Some(val_o), Some(type_o)) =
            (off("bNode", "inputs"), off("bNodeSocket", "name"), off("bNodeSocket", "*default_value"), off("bNodeSocket", "type"))
        else {
            return Vec::new();
        };
        let socks = self.list(Self::read_ptr(&node.data, inputs, ps, le), off("bNodeSocket", "*next"));
        socks
            .into_iter()
            .filter_map(|s| {
                let ty = self.read_u16(s.data.get(type_o..type_o + 2)?);
                let val = self.block_at(Self::read_ptr(&s.data, val_o, ps, le))?;
                let f = |i: usize| val.data.get(i..i + 4).map(|b| self.read_f32(b));
                let value = match ty {
                    SOCK_FLOAT => BSocket::Float(f(4)?),
                    SOCK_VECTOR => BSocket::Vec3([f(4)?, f(8)?, f(12)?]),
                    SOCK_RGBA => BSocket::Rgba([f(0)?, f(4)?, f(8)?, f(12)?]),
                    _ => return None,
                };
                Some((read_cstr(&s.data, name_o, 64), value))
            })
            .collect()
    }
}
