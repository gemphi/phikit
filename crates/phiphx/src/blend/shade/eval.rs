//! Constant folding of a material's colour graph (Mix, Color Ramp,
//! Hue/Saturation, Bright/Contrast, Gamma, Math, Clamp, Invert, RGB/Value,
//! Reroute) into one representative
//! value per Principled input. Textures have no single value: image colour
//! counts as white (so the result is the tint the image is multiplied by)
//! and procedural outputs as their mean 0.5; such results are marked inexact.
//! Formulas follow Blender's GPU nodes (`gpu_shader_material_mix_color.glsl`,
//! `…_hue_sat_val.glsl`, `BKE_colorband_evaluate`).

use super::links::BLink;
use super::nodes::read_cstr;
use crate::blend::file::types::*;

/// A folded socket value: colour (floats are grey), and whether it is exact.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BVal {
    pub rgba: [f32; 4],
    pub float: bool,
    pub exact: bool,
}

impl BVal {
    fn f(v: f32, exact: bool) -> Self {
        Self { rgba: [v, v, v, 1.0], float: true, exact }
    }
    fn c(rgba: [f32; 4], exact: bool) -> Self {
        Self { rgba, float: false, exact }
    }
    /// As a float socket (colour → Rec.709 luminance, like Blender).
    pub fn value(&self) -> f32 {
        let [r, g, b, _] = self.rgba;
        if self.float { r } else { 0.2126 * r + 0.7152 * g + 0.0722 * b }
    }
}

/// `eNodeSocketDatatype`.
const SOCK_FLOAT: u16 = 0;
const SOCK_VECTOR: u16 = 1;
const SOCK_RGBA: u16 = 2;

pub(crate) struct BGraph<'a> {
    f: &'a BlendFile,
    dna: &'a DnaType,
    nodes: Vec<&'a BlendBlock>,
    links: Vec<BLink>,
}

impl<'a> BGraph<'a> {
    pub fn new(f: &'a BlendFile, ma: &BlendBlock, dna: &'a DnaType) -> Self {
        Self::owned(f, ma, "Material", dna)
    }

    /// The graph of `id`'s node tree; `owner` names the datablock type that
    /// holds the `*nodetree` pointer (`Material`, `World`).
    pub fn owned(f: &'a BlendFile, id: &BlendBlock, owner: &str, dna: &'a DnaType) -> Self {
        Self { f, dna, nodes: f.tree_nodes(id, owner, dna), links: f.tree_links(id, owner, dna) }
    }

    fn off(&self, s: &str, field: &str) -> Option<usize> {
        self.f.off(s, field, self.dna)
    }

    fn text(&self, b: &BlendBlock, s: &str, field: &str) -> String {
        self.off(s, field).map(|o| read_cstr(&b.data, o, 64)).unwrap_or_default()
    }

    pub fn linked(&self, sock: u64) -> bool {
        self.links.iter().any(|l| l.to_sock == sock)
    }

    /// Value arriving at input socket `sock` (its own default when unlinked).
    pub fn input(&self, sock: u64, depth: u32) -> Option<BVal> {
        match self.links.iter().find(|l| l.to_sock == sock) {
            Some(l) if depth < 64 => {
                let node = self.nodes.iter().copied().find(|n| n.old_address == l.from)?;
                let out = self.f.block_at(l.from_sock)?;
                self.output(node, &self.text(out, "bNodeSocket", "identifier"), &self.text(out, "bNodeSocket", "name"), depth + 1)
            }
            Some(_) => None,
            None => self.default(self.f.block_at(sock)?),
        }
    }

    /// The value at `sock`, or — when what feeds it cannot be folded — the
    /// socket's own default. That is what Blender uses when the upstream node
    /// is one it does not know (`Undefined[…]`, saved by a Blender without
    /// that node type): the link is dead and the socket keeps its value.
    pub fn input_or_default(&self, sock: u64, depth: u32) -> Option<BVal> {
        self.input(sock, depth).or_else(|| self.default(self.f.block_at(sock)?))
    }

    /// A socket's stored `default_value`.
    fn default(&self, s: &BlendBlock) -> Option<BVal> {
        let ty = self.f.read_u16(s.data.get(self.off("bNodeSocket", "type")?..)?.get(..2)?);
        let v = self.f.block_at(self.f.ptr(s, self.off("bNodeSocket", "*default_value")))?;
        let fl = |i: usize| v.data.get(i..i + 4).map(|b| self.f.read_f32(b));
        Some(match ty {
            SOCK_FLOAT => BVal::f(fl(4)?, true),
            SOCK_VECTOR => BVal::f((fl(4)? + fl(8)? + fl(12)?) / 3.0, true),
            SOCK_RGBA => BVal::c([fl(0)?, fl(4)?, fl(8)?, fl(12)?], true),
            _ => return None,
        })
    }

    /// Input of `node` by identifier (falls back to name).
    fn arg(&self, node: &BlendBlock, id: &str, depth: u32) -> Option<BVal> {
        let socks = self.f.input_sockets(node, self.dna);
        let find = |field: &str| {
            socks.iter().copied().find(|&a| self.f.block_at(a).is_some_and(|b| self.text(b, "bNodeSocket", field) == id))
        };
        self.input(find("identifier").or_else(|| find("name"))?, depth)
    }

    fn output(&self, node: &BlendBlock, out_id: &str, out_name: &str, depth: u32) -> Option<BVal> {
        let kind = self.text(node, "bNode", "idname");
        let arg = |id: &str| self.arg(node, id, depth);
        let storage = || self.f.block_at(self.f.ptr(node, self.off("bNode", "*storage")));
        match kind.as_str() {
            "ShaderNodeMix" => {
                let st = storage()?;
                let byte = |f: &str| self.off("NodeShaderMix", f).and_then(|o| st.data.get(o)).copied().unwrap_or(0);
                let fac = arg("Factor_Float")?;
                let (a, b) = match byte("data_type") as u16 {
                    SOCK_RGBA => (arg("A_Color")?, arg("B_Color")?),
                    _ => (arg("A_Float")?, arg("B_Float")?),
                };
                let t = if byte("clamp_factor") != 0 { fac.value().clamp(0.0, 1.0) } else { fac.value() };
                let mut rgba = super::blend::mix(byte("blend_type"), t, a.rgba, b.rgba);
                if byte("clamp_result") != 0 {
                    rgba = rgba.map(|c| c.clamp(0.0, 1.0));
                }
                Some(BVal { rgba, float: a.float, exact: fac.exact && a.exact && b.exact })
            }
            "ShaderNodeMixRGB" => {
                let (fac, a, b) = (arg("Fac")?, arg("Color1")?, arg("Color2")?);
                let blend = self.off("bNode", "custom1").and_then(|o| node.data.get(o)).copied().unwrap_or(0);
                Some(BVal::c(super::blend::mix(blend, fac.value().clamp(0.0, 1.0), a.rgba, b.rgba), fac.exact && a.exact && b.exact))
            }
            "ShaderNodeValToRGB" => {
                let fac = arg("Fac")?;
                let c = super::blend::ramp(self.f, self.dna, storage()?, fac.value())?;
                Some(if out_name == "Alpha" { BVal::f(c[3], fac.exact) } else { BVal::c(c, fac.exact) })
            }
            "ShaderNodeHueSaturation" => {
                let (h, s, v, fac, c) = (arg("Hue")?, arg("Saturation")?, arg("Value")?, arg("Fac")?, arg("Color")?);
                let out = super::blend::hue_sat(c.rgba, h.value(), s.value(), v.value(), fac.value());
                Some(BVal::c(out, [h, s, v, fac, c].iter().all(|x| x.exact)))
            }
            "ShaderNodeBrightContrast" => {
                let (c, bright, contrast) = (arg("Color")?, arg("Bright")?, arg("Contrast")?);
                Some(BVal::c(super::scalar::bright_contrast(c.rgba, bright.value(), contrast.value()), c.exact && bright.exact && contrast.exact))
            }
            "ShaderNodeGamma" => {
                let (c, g) = (arg("Color")?, arg("Gamma")?);
                Some(BVal::c(super::scalar::gamma(c.rgba, g.value()), c.exact && g.exact))
            }
            "ShaderNodeMath" => {
                let short = |field: &str| self.off("bNode", field).and_then(|o| node.data.get(o..o + 2)).map_or(0, |b| self.f.read_u16(b));
                let (a, b, c) = (arg("Value")?, arg("Value_001"), arg("Value_002"));
                let value = |v: &Option<BVal>| v.as_ref().map_or(0.0, BVal::value);
                let mut out = super::scalar::math(short("custom1"), a.value(), value(&b), value(&c))?;
                if short("custom2") & super::scalar::MATH_CLAMP != 0 {
                    out = out.clamp(0.0, 1.0);
                }
                Some(BVal::f(out, a.exact && b.is_none_or(|v| v.exact) && c.is_none_or(|v| v.exact)))
            }
            "ShaderNodeClamp" => {
                let range = self.off("bNode", "custom1").and_then(|o| node.data.get(o..o + 2)).map_or(0, |b| self.f.read_u16(b));
                let (v, min, max) = (arg("Value")?, arg("Min")?, arg("Max")?);
                Some(BVal::f(super::scalar::clamp(range, v.value(), min.value(), max.value()), v.exact && min.exact && max.exact))
            }
            "ShaderNodeInvert" => {
                let (fac, c) = (arg("Fac")?, arg("Color")?);
                let t = fac.value();
                let rgb = [0, 1, 2].map(|k| c.rgba[k] + t * (1.0 - 2.0 * c.rgba[k]));
                Some(BVal::c([rgb[0], rgb[1], rgb[2], c.rgba[3]], fac.exact && c.exact))
            }
            "ShaderNodeRGB" | "ShaderNodeValue" => {
                let outs = self.f.list(self.f.ptr(node, self.off("bNode", "outputs")), self.off("bNodeSocket", "*next"));
                self.default(outs.first()?)
            }
            "NodeReroute" => self.input(*self.f.input_sockets(node, self.dna).first()?, depth),
            "ShaderNodeRGBCurve" => arg("Color").map(|c| BVal { exact: false, ..c }),
            // Occlusion needs the scene; an open surface is unoccluded, so the
            // colour passes through and the factor is 1.
            "ShaderNodeAmbientOcclusion" => {
                Some(if out_name == "AO" { BVal::f(1.0, false) } else { BVal { exact: false, ..arg("Color")? } })
            }
            // Generated / UV / Object coordinates run 0..1 over the surface:
            // their mean is 0.5.
            "ShaderNodeTexCoord" | "ShaderNodeUVMap" | "ShaderNodeNewGeometry" => Some(BVal::f(0.5, false)),
            // Point mapping is affine, so the mean maps to the mean:
            // `v * scale + location` (rotation keeps the centre of a 0..1 box
            // near itself for the small angles garments use).
            "ShaderNodeMapping" => {
                let v = arg("Vector")?;
                let scale = arg("Scale").map_or(1.0, |s| s.value());
                let loc = arg("Location").map_or(0.0, |l| l.value());
                Some(BVal::f(v.value() * scale + loc, false))
            }
            "ShaderNodeTexImage" => {
                // A file Blender cannot find draws magenta — the pink the
                // viewport shows for a missing texture — with alpha 1.
                let present = self
                    .f
                    .block_at(self.f.ptr(node, self.off("bNode", "*id")))
                    .is_none_or(|im| self.f.image_present(im, self.dna));
                Some(match (out_id == "Alpha", present) {
                    (true, _) => BVal::f(1.0, false),
                    (false, true) => BVal::c([1.0; 4], false),
                    (false, false) => BVal::c([1.0, 0.0, 1.0, 1.0], false),
                })
            }
            k if k.starts_with("ShaderNodeTex") => {
                Some(if out_name == "Color" { BVal::c([0.5, 0.5, 0.5, 1.0], false) } else { BVal::f(0.5, false) })
            }
            _ => None,
        }
    }
}

