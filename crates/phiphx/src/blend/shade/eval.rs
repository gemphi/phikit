//! Constant folding of a material's colour graph (Mix, Color Ramp,
//! Hue/Saturation, Bright/Contrast, Gamma, Math, Clamp, Invert, RGB/Value,
//! Reroute) into one representative
//! value per Principled input. Textures have no single value: image colour
//! counts as white (so the result is the tint the image is multiplied by)
//! and procedural outputs as their mean 0.5; such results are marked inexact.
//! Formulas follow Blender's GPU nodes (`gpu_shader_material_mix_color.glsl`,
//! `…_hue_sat_val.glsl`, `BKE_colorband_evaluate`).

use super::links::BLink;
use crate::blend::shade::nodes::read_cstr;
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
            // The fold is what a camera ray sees (`svm_node_light_path`
            // for a ray straight from the camera): Is Camera Ray 1, every
            // other Is-ray 0, the depths 0, the ray length 1 — so a Mix
            // Shader keyed on Is Shadow Ray (the fake-caustics setup) folds
            // to the shader the camera is shown.
            "ShaderNodeLightPath" => Some(BVal::f(if out_name == "Is Camera Ray" || out_name == "Ray Length" { 1.0 } else { 0.0 }, true)),
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


// ---------------------------------------------------------------------------
// Baking: the same walk as the fold above, emitting a program (`bake.rs`)
// instead of one value, so an image that drives a Mix's factor keeps doing
// so per texel instead of folding to white.

use super::bake::{BBakeImage, BBakeOp, BlendBake, IDENTITY, compose, mapping_affine};

impl BlendFile {
    /// The Base Color chain compiled to a bake, when it holds at least one
    /// image and every node on the way is one the compiler knows.
    pub(crate) fn base_bake(&self, ma: &BlendBlock, dna: &DnaType) -> Option<BlendBake> {
        let bsdf = self.surface_principled(ma, dna)?;
        let sock = self.socket_named(bsdf, "Base Color", dna)?;
        let g = BGraph::new(self, ma, dna);
        if !g.linked(sock) {
            return None;
        }
        let mut bake = BlendBake { images: Vec::new(), ops: Vec::new() };
        g.emit_input(sock, &mut bake, 0)?;
        (!bake.images.is_empty()).then_some(bake)
    }
}

impl BGraph<'_> {
    /// A vector socket's stored components.
    fn default_vec(&self, sock: u64) -> Option<[f32; 3]> {
        let s = self.f.block_at(sock)?;
        let v = self.f.block_at(self.f.ptr(s, self.off("bNodeSocket", "*default_value")))?;
        let fl = |i: usize| v.data.get(i..i + 4).map(|b| self.f.read_f32(b));
        Some([fl(4)?, fl(8)?, fl(12)?])
    }

    /// The UV affine feeding an Image Texture's Vector input: identity for
    /// UV coordinates, Mapping nodes composed, anything else identity (the
    /// chain is kept rather than refused — an Object-driven pattern bakes
    /// at its UV stand-in).
    fn uv_affine(&self, node: &BlendBlock, depth: u32) -> [[f32; 3]; 2] {
        if depth > 8 {
            return IDENTITY;
        }
        let Some(sock) = self.f.socket_named(node, "Vector", self.dna) else { return IDENTITY };
        let Some(link) = self.links.iter().find(|l| l.to_sock == sock) else { return IDENTITY };
        let Some(from) = self.nodes.iter().copied().find(|n| n.old_address == link.from) else { return IDENTITY };
        let kind = self.text(from, "bNode", "idname");
        match kind.as_str() {
            "ShaderNodeMapping" => {
                let inner = self.uv_affine(from, depth + 1);
                let vec = |name: &str, d: [f32; 3]| {
                    self.f.socket_named(from, name, self.dna).and_then(|s| self.default_vec(s)).unwrap_or(d)
                };
                let loc = vec("Location", [0.0; 3]);
                let rot = vec("Rotation", [0.0; 3]);
                let scale = vec("Scale", [1.0; 3]);
                compose(mapping_affine(loc, rot[2], scale), inner)
            }
            "NodeReroute" => self.uv_affine(from, depth + 1),
            _ => IDENTITY,
        }
    }

    /// Emit the value arriving at `sock` (its default when unlinked).
    fn emit_input(&self, sock: u64, bake: &mut BlendBake, depth: u32) -> Option<()> {
        if depth > 64 || bake.ops.len() > 1024 {
            return None;
        }
        match self.links.iter().find(|l| l.to_sock == sock) {
            Some(l) => {
                let node = self.nodes.iter().copied().find(|n| n.old_address == l.from)?;
                let out = self.f.block_at(l.from_sock)?;
                let out_id = self.text(out, "bNodeSocket", "identifier");
                let out_name = self.text(out, "bNodeSocket", "name");
                self.emit_output(node, &out_id, &out_name, bake, depth + 1)
            }
            None => {
                let v = self.default(self.f.block_at(sock)?)?;
                bake.ops.push(if v.float { BBakeOp::ConstF(v.rgba[0]) } else { BBakeOp::Const(v.rgba) });
                Some(())
            }
        }
    }

    /// Emit `node`'s input by identifier (falling back to name).
    fn emit_arg(&self, node: &BlendBlock, id: &str, bake: &mut BlendBake, depth: u32) -> Option<()> {
        let socks = self.f.input_sockets(node, self.dna);
        let find = |field: &str| {
            socks.iter().copied().find(|&a| self.f.block_at(a).is_some_and(|b| self.text(b, "bNodeSocket", field) == id))
        };
        self.emit_input(find("identifier").or_else(|| find("name"))?, bake, depth)
    }

    fn emit_output(&self, node: &BlendBlock, out_id: &str, out_name: &str, bake: &mut BlendBake, depth: u32) -> Option<()> {
        let kind = self.text(node, "bNode", "idname");
        let storage = || self.f.block_at(self.f.ptr(node, self.off("bNode", "*storage")));
        let byte = |o: Option<usize>| o.and_then(|o| node.data.get(o)).copied().unwrap_or(0);
        let short = |o: Option<usize>| o.and_then(|o| node.data.get(o..o + 2)).map_or(0, |b| self.f.read_u16(b));
        match kind.as_str() {
            "ShaderNodeTexImage" => {
                let im = self.f.block_at(self.f.ptr(node, self.off("bNode", "*id")))?;
                let key = self.f.image_key(im, self.dna);
                if key.is_empty() {
                    return None;
                }
                let affine = self.uv_affine(node, 0);
                let same = |img: &BBakeImage| img.key == key && img.affine == affine;
                let i = match bake.images.iter().position(same) {
                    Some(i) => i,
                    None => {
                        bake.images.push(BBakeImage { key, affine });
                        bake.images.len() - 1
                    }
                } as u16;
                bake.ops.push(if out_id == "Alpha" || out_name == "Alpha" { BBakeOp::ImageAlpha(i) } else { BBakeOp::Image(i) });
                Some(())
            }
            "ShaderNodeMix" => {
                let st = storage()?;
                let field = |f: &str| self.off("NodeShaderMix", f).and_then(|o| st.data.get(o)).copied().unwrap_or(0);
                self.emit_arg(node, "Factor_Float", bake, depth)?;
                let (a, b) = match field("data_type") as u16 {
                    SOCK_RGBA => ("A_Color", "B_Color"),
                    _ => ("A_Float", "B_Float"),
                };
                self.emit_arg(node, a, bake, depth)?;
                self.emit_arg(node, b, bake, depth)?;
                bake.ops.push(BBakeOp::Mix { blend: field("blend_type"), clamp_fac: field("clamp_factor") != 0, clamp_res: field("clamp_result") != 0 });
                Some(())
            }
            "ShaderNodeMixRGB" => {
                self.emit_arg(node, "Fac", bake, depth)?;
                self.emit_arg(node, "Color1", bake, depth)?;
                self.emit_arg(node, "Color2", bake, depth)?;
                bake.ops.push(BBakeOp::Mix { blend: byte(self.off("bNode", "custom1")), clamp_fac: true, clamp_res: false });
                Some(())
            }
            "ShaderNodeValToRGB" => {
                self.emit_arg(node, "Fac", bake, depth)?;
                let (stops, ipo) = super::blend::ramp_stops(self.f, self.dna, storage()?)?;
                bake.ops.push(BBakeOp::Ramp { stops, ipo, alpha: out_name == "Alpha" });
                Some(())
            }
            "ShaderNodeInvert" => {
                self.emit_arg(node, "Fac", bake, depth)?;
                self.emit_arg(node, "Color", bake, depth)?;
                bake.ops.push(BBakeOp::Invert);
                Some(())
            }
            "ShaderNodeGamma" => {
                self.emit_arg(node, "Color", bake, depth)?;
                self.emit_arg(node, "Gamma", bake, depth)?;
                bake.ops.push(BBakeOp::Gamma);
                Some(())
            }
            "ShaderNodeBrightContrast" => {
                self.emit_arg(node, "Color", bake, depth)?;
                self.emit_arg(node, "Bright", bake, depth)?;
                self.emit_arg(node, "Contrast", bake, depth)?;
                bake.ops.push(BBakeOp::BrightContrast);
                Some(())
            }
            "ShaderNodeHueSaturation" => {
                self.emit_arg(node, "Color", bake, depth)?;
                self.emit_arg(node, "Hue", bake, depth)?;
                self.emit_arg(node, "Saturation", bake, depth)?;
                self.emit_arg(node, "Value", bake, depth)?;
                self.emit_arg(node, "Fac", bake, depth)?;
                bake.ops.push(BBakeOp::HueSat);
                Some(())
            }
            "ShaderNodeMath" => {
                self.emit_arg(node, "Value", bake, depth)?;
                let b = self.emit_arg(node, "Value_001", bake, depth).is_some();
                let c = b && self.emit_arg(node, "Value_002", bake, depth).is_some();
                let args = 1 + u8::from(b) + u8::from(c);
                bake.ops.push(BBakeOp::Math { op: short(self.off("bNode", "custom1")), clamp: short(self.off("bNode", "custom2")) & super::scalar::MATH_CLAMP != 0, args });
                Some(())
            }
            "ShaderNodeClamp" => {
                self.emit_arg(node, "Value", bake, depth)?;
                self.emit_arg(node, "Min", bake, depth)?;
                self.emit_arg(node, "Max", bake, depth)?;
                bake.ops.push(BBakeOp::Clamp { range: short(self.off("bNode", "custom1")) });
                Some(())
            }
            "ShaderNodeRGB" | "ShaderNodeValue" => {
                let outs = self.f.list(self.f.ptr(node, self.off("bNode", "outputs")), self.off("bNodeSocket", "*next"));
                let v = self.default(outs.first()?)?;
                bake.ops.push(if v.float { BBakeOp::ConstF(v.rgba[0]) } else { BBakeOp::Const(v.rgba) });
                Some(())
            }
            "NodeReroute" => self.emit_input(*self.f.input_sockets(node, self.dna).first()?, bake, depth),
            "ShaderNodeRGBCurve" => self.emit_arg(node, "Color", bake, depth),
            "ShaderNodeAmbientOcclusion" => {
                if out_name == "AO" {
                    bake.ops.push(BBakeOp::ConstF(1.0));
                    Some(())
                } else {
                    self.emit_arg(node, "Color", bake, depth)
                }
            }
            "ShaderNodeTexCoord" | "ShaderNodeUVMap" | "ShaderNodeNewGeometry" | "ShaderNodeMapping" => {
                bake.ops.push(BBakeOp::ConstF(0.5));
                Some(())
            }
            "ShaderNodeLightPath" => {
                bake.ops.push(BBakeOp::ConstF(if out_name == "Is Camera Ray" || out_name == "Ray Length" { 1.0 } else { 0.0 }));
                Some(())
            }
            k if k.starts_with("ShaderNodeTex") => {
                // A procedural folds to its mean here; the kernel's own
                // procedural slot still drives what it can (`procedural.rs`).
                bake.ops.push(if out_name == "Color" { BBakeOp::Const([0.5, 0.5, 0.5, 1.0]) } else { BBakeOp::ConstF(0.5) });
                Some(())
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod bake_digest {
    /// Not a fixture test: point `BAKE_DIGEST` at a `.blend` and this prints
    /// every material's compiled Base Color bake, for checking a real file's
    /// chains against Blender's node graphs. Does nothing without the var.
    #[test]
    fn bake_digest_of_env_file() {
        let Ok(path) = std::env::var("BAKE_DIGEST") else { return };
        let bytes = std::fs::read(&path).expect("read");
        let f = crate::blend::file::types::BlendFile::parse_bytes(&bytes).expect("parse");
        let dna = f.dna.clone().expect("dna");
        for ma in f.blocks.iter().filter(|b| b.code_str == "MA") {
            let name = f.extract_name_from_block(ma, "id.name").unwrap_or_default();
            match f.base_bake(ma, &dna) {
                Some(b) => {
                    let imgs: Vec<String> = b.images.iter().map(|i| format!("{} @{:?}", i.key, i.affine[0])).collect();
                    println!("{name}: {} ops, imgs {:?}", b.ops.len(), imgs);
                }
                None => println!("{name}: -"),
            }
        }
    }
}

#[cfg(test)]
mod image_digest {
    /// Not a fixture test: point `IMG_DIGEST` at a `.blend` and this prints
    /// every image datablock's path and whether its pixels are packed in the
    /// file. Does nothing without the var.
    #[test]
    fn image_digest_of_env_file() {
        let Ok(path) = std::env::var("IMG_DIGEST") else { return };
        let bytes = std::fs::read(&path).expect("read");
        let f = crate::blend::file::types::BlendFile::parse_bytes(&bytes).expect("parse");
        let dna = f.dna.clone().expect("dna");
        let ps = f.header.pointer_size as usize;
        for im in f.blocks.iter().filter(|b| b.code_str == "IM") {
            let name = f.extract_name_from_block(im, "id.name").unwrap_or_default();
            let path_o = crate::blend::file::types::BlendFile::struct_field_offset_exact("Image", "filepath[1024]", &dna, ps)
                .or_else(|| crate::blend::file::types::BlendFile::struct_field_offset_exact("Image", "name[1024]", &dna, ps));
            let fp = path_o.map(|o| crate::blend::shade::nodes::read_cstr(&im.data, o, 1024)).unwrap_or_default();
            let legacy = f.ptr(im, f.off("Image", "*packedfile", &dna));
            let list = f.ptr(im, f.off("Image", "packedfiles", &dna));
            println!("IM {name}: packed {} path {fp}", legacy != 0 || list != 0);
        }
    }
}
