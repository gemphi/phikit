//! Baking: the same walk as the fold in `eval.rs`, emitting a program (`bake.rs`)
//! instead of one value, so an image that drives a Mix's factor keeps doing
//! so per texel instead of folding to white.
use crate::device::blend::shade::eval::{BGraph, SOCK_RGBA};
use crate::device::blend::file::kind::*;
use crate::device::blend::shade::bake::{BBakeImage, BBakeOp, BlendBake, IDENTITY, compose, mapping_affine};

impl BlendFile {
    /// The Base Color chain compiled to a bake, when it holds at least one
    /// image and every node on the way is one the compiler knows.
    pub(crate) fn base_bake(&self, ma: &BlendBlock, dna: &DnaType) -> Option<BlendBake> {
        let bsdf = self.surface_principled(ma, dna)?;
        let sock = self.socket_named(bsdf, "Base Color", dna)?;
        let cut = self.socket_named(bsdf, "Alpha", dna);
        let g = BGraph::new(self, ma, dna);
        let cuts = cut.is_some_and(|c| g.linked(c));
        if !g.linked(sock) && !cuts {
            return None;
        }
        let mut bake = BlendBake { images: Vec::new(), ops: Vec::new(), alpha: None };
        if let Some(cut) = cut.filter(|_| cuts) {
            // Only worth a program when an image reaches it; a chain of
            // procedurals keeps the value it folds to (`principled.rs`).
            if g.emit_input(cut, &mut bake, 0).is_some() && !bake.images.is_empty() {
                bake.alpha = Some(std::mem::take(&mut bake.ops));
            }
            bake.ops.clear();
            if bake.alpha.is_none() {
                bake.images.clear();
            }
        }
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
                if self.text(node, "bNode", "idname") == "ShaderNodeGroup" {
                    let v = self.default(out)?;
                    bake.ops.push(if v.float { BBakeOp::ConstF(v.rgba[0]) } else { BBakeOp::Const(v.rgba) });
                    return Some(());
                }
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
    pub(super) fn emit_arg(&self, node: &BlendBlock, id: &str, bake: &mut BlendBake, depth: u32) -> Option<()> {
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
                let is_alpha = out_id == "Alpha" || out_name == "Alpha";
                // No image on the node: Cycles answers its missing-image
                // colour (`IMAGE_MISSING_RGBA`), and the graph goes on.
                let picked = self.f.block_at(self.f.ptr(node, self.off("bNode", "*id"))).map(|im| (self.f.image_key(im, self.dna), im));
                let Some((key, im)) = picked.filter(|(key, _)| !key.is_empty()) else {
                    bake.ops.push(if is_alpha { BBakeOp::ConstF(1.0) } else { BBakeOp::Const([1.0, 0.0, 1.0, 1.0]) });
                    return Some(());
                };
                let affine = self.uv_affine(node, 0);
                let alpha_linked = self.f.output_sockets(node, self.dna).into_iter().any(|s| self.f.socket_name(s, self.dna) == "Alpha" && self.links.iter().any(|l| l.from_sock == s));
                let straight = alpha_linked || self.f.image_color_is_stored(im, self.dna);
                let same = |img: &BBakeImage| img.key == key && img.affine == affine && img.straight == straight;
                let i = match bake.images.iter().position(same) {
                    Some(i) => i,
                    None => {
                        bake.images.push(BBakeImage { key, affine, straight });
                        bake.images.len() - 1
                    }
                } as u16;
                bake.ops.push(if is_alpha { BBakeOp::ImageAlpha(i) } else { BBakeOp::Image(i) });
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
            _ => self.emit_more(node, out_id, out_name, bake, depth),
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
        let f = crate::device::blend::file::kind::BlendFile::parse_bytes(&bytes).expect("parse");
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
        let f = crate::device::blend::file::kind::BlendFile::parse_bytes(&bytes).expect("parse");
        let dna = f.dna.clone().expect("dna");
        let ps = f.header.pointer_size as usize;
        for im in f.blocks.iter().filter(|b| b.code_str == "IM") {
            let name = f.extract_name_from_block(im, "id.name").unwrap_or_default();
            let path_o = crate::device::blend::file::kind::BlendFile::struct_field_offset_exact("Image", "filepath[1024]", &dna, ps)
                .or_else(|| crate::device::blend::file::kind::BlendFile::struct_field_offset_exact("Image", "name[1024]", &dna, ps));
            let fp = path_o.map(|o| crate::device::blend::shade::node::read_cstr(&im.data, o, 1024)).unwrap_or_default();
            let legacy = f.ptr(im, f.off("Image", "*packedfile", &dna));
            let list = f.ptr(im, f.off("Image", "packedfiles", &dna));
            println!("IM {name}: packed {} path {fp}", legacy != 0 || list != 0);
        }
    }
}
