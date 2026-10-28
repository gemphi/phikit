//! The value, vector and colour nodes the fold (`eval.rs`) continues
//! into: Map Range, Separate / Combine XYZ, Separate / Combine Color (and
//! the legacy RGB / HSV pairs), Vector Math, Vector Rotate, Normal,
//! Fresnel, Layer Weight, Vertex Color and Attribute — each as Cycles
//! evaluates it (`svm_node_map_range`, `svm_node_separate_color`,
//! `svm_vector_math`, `svm_node_vector_rotate`, `svm_node_normal`,
//! `svm_node_fresnel`, `svm_node_layer_weight`), or, where the value is
//! the surface's and not the graph's, the mean a camera sees: a Fresnel
//! is `fresnel_dielectric_Fss` (`closure/bsdf_util.h`, the integral of
//! `2·cos·F(cos)` over the hemisphere) and a Facing weight the same
//! integral of `1 − cosᵖ`, which is `1 − 2 / (p + 2)`. A vertex colour the
//! graph cannot read is white, as an image is.
//!
//! The same nodes emitted to a Base Color bake (`bake.rs`): the colour
//! ones as ops, the rest as the constant they fold to when exact.

use super::bake::{BBakeOp, BlendBake};
use super::eval::{BGraph, BVal};
use super::{scalar, sepcomb, vector};
use crate::blend::file::types::BlendBlock;

/// `eCustomDataType`: `CD_PROP_FLOAT3`, a Map Range over vectors.
const CD_PROP_FLOAT3: u8 = 48;

/// Cycles' `fresnel_dielectric_Fss`: the hemispherical mean of the
/// dielectric Fresnel at `eta` (Kulla–Conty's fit).
pub(crate) fn fresnel_fss(eta: f32) -> f32 {
    match eta < 1.0 {
        true => 0.997118 + eta * (0.1014 - eta * (0.965241 + eta * 0.130607)),
        false => (eta - 1.0) / (4.08567 + 1.00071 * eta),
    }
}

/// `svm_node_layer_weight`'s Facing: the Blend remapped to the power
/// the cosine is raised to, and `1 − cosᵖ` averaged over the hemisphere.
pub(crate) fn facing_mean(blend: f32) -> f32 {
    let p = match blend == 0.5 {
        true => 1.0,
        false => {
            let b = blend.clamp(0.0, 1.0 - 1e-5);
            if b < 0.5 { 2.0 * b } else { 0.5 / (1.0 - b) }
        }
    };
    1.0 - 2.0 / (p + 2.0)
}

fn all_exact(vals: &[BVal]) -> bool {
    vals.iter().all(|v| v.exact)
}

impl BGraph<'_> {
    fn storage(&self, node: &BlendBlock) -> Option<&BlendBlock> {
        self.f.block_at(self.f.ptr(node, self.off("bNode", "*storage")))
    }

    fn stored_byte(&self, node: &BlendBlock, st: &str, field: &str) -> Option<u8> {
        self.storage(node)?.data.get(self.off(st, field)?).copied()
    }

    /// `NodeCombSepColor.mode` — one byte, not four: the enum's base type
    /// is `int8_t` (`DNA_node_types.h`), so a four-byte read falls off the
    /// end of the struct and every Separate / Combine Color came out RGB.
    fn comb_sep_mode(&self, node: &BlendBlock) -> u32 {
        u32::from(self.stored_byte(node, "NodeCombSepColor", "mode").unwrap_or(0))
    }

    /// `bNode.custom1` / `custom2`, 0 when the file lacks the field.
    fn custom(&self, node: &BlendBlock, field: &str) -> u32 {
        self.off("bNode", field).and_then(|o| node.data.get(o..o + 2)).map_or(0, |b| u32::from(self.f.read_u16(b)))
    }

    /// The node's first output socket's stored value (a Normal node keeps
    /// its direction there).
    fn first_output(&self, node: &BlendBlock) -> Option<BVal> {
        let outs = self.f.list(self.f.ptr(node, self.off("bNode", "outputs")), self.off("bNodeSocket", "*next"));
        self.default(outs.first()?)
    }

    /// Inputs `ids`, each folded, or `None` when one is missing.
    fn args(&self, node: &BlendBlock, ids: &[&str], depth: u32) -> Option<Vec<BVal>> {
        ids.iter().map(|id| self.arg(node, id, depth)).collect()
    }

    pub(super) fn fold_more(&self, node: &BlendBlock, _out_id: &str, out_name: &str, depth: u32) -> Option<BVal> {
        let kind = self.text(node, "bNode", "idname");
        let channel = |n: &str, names: [&str; 3]| names.iter().position(|c| *c == n).unwrap_or(0);
        match kind.as_str() {
            "ShaderNodeMapRange" => {
                let ty = u32::from(self.stored_byte(node, "NodeMapRange", "interpolation_type").unwrap_or(0));
                let clamp = self.stored_byte(node, "NodeMapRange", "clamp").unwrap_or(1) != 0;
                match self.stored_byte(node, "NodeMapRange", "data_type") == Some(CD_PROP_FLOAT3) {
                    true => {
                        let v = self.args(node, &["Vector", "From_Min_FLOAT3", "From_Max_FLOAT3", "To_Min_FLOAT3", "To_Max_FLOAT3", "Steps_FLOAT3"], depth)?;
                        let at = |i: usize| scalar::map_range(ty, v[0].rgba[i], v[1].rgba[i], v[2].rgba[i], v[3].rgba[i], v[4].rgba[i], v[5].rgba[i], clamp);
                        Some(BVal::v([at(0), at(1), at(2)], all_exact(&v)))
                    }
                    false => {
                        let v = self.args(node, &["Value", "From Min", "From Max", "To Min", "To Max", "Steps"], depth)?;
                        let x: Vec<f32> = v.iter().map(BVal::value).collect();
                        Some(BVal::f(scalar::map_range(ty, x[0], x[1], x[2], x[3], x[4], x[5], clamp), all_exact(&v)))
                    }
                }
            }
            "ShaderNodeSeparateXYZ" => {
                let v = self.arg(node, "Vector", depth)?;
                Some(BVal::f(v.xyz()[channel(out_name, ["X", "Y", "Z"])], v.exact))
            }
            "ShaderNodeCombineXYZ" => {
                let v = self.args(node, &["X", "Y", "Z"], depth)?;
                Some(BVal::v([v[0].value(), v[1].value(), v[2].value()], all_exact(&v)))
            }
            "ShaderNodeSeparateColor" | "ShaderNodeSeparateRGB" | "ShaderNodeSeparateHSV" => {
                let (mode, input, names) = match kind.as_str() {
                    "ShaderNodeSeparateRGB" => (sepcomb::COMBSEP_RGB, "Image", ["R", "G", "B"]),
                    "ShaderNodeSeparateHSV" => (sepcomb::COMBSEP_HSV, "Color", ["H", "S", "V"]),
                    _ => (self.comb_sep_mode(node), "Color", ["Red", "Green", "Blue"]),
                };
                let c = self.arg(node, input, depth)?;
                Some(BVal::f(sepcomb::separate_color(mode, c.xyz())[channel(out_name, names)], c.exact))
            }
            "ShaderNodeCombineColor" | "ShaderNodeCombineRGB" | "ShaderNodeCombineHSV" => {
                let (mode, ids) = match kind.as_str() {
                    "ShaderNodeCombineRGB" => (sepcomb::COMBSEP_RGB, ["R", "G", "B"]),
                    "ShaderNodeCombineHSV" => (sepcomb::COMBSEP_HSV, ["H", "S", "V"]),
                    _ => (self.comb_sep_mode(node), ["Red", "Green", "Blue"]),
                };
                let v = self.args(node, &ids, depth)?;
                let [r, g, b] = sepcomb::combine_color(mode, [v[0].value(), v[1].value(), v[2].value()]);
                Some(BVal::c([r, g, b, 1.0], all_exact(&v)))
            }
            "ShaderNodeVectorMath" => {
                let v = self.args(node, &["Vector", "Vector_001", "Vector_002", "Scale"], depth)?;
                let (vec, val) = vector::vector_math(self.custom(node, "custom1"), v[0].xyz(), v[1].xyz(), v[2].xyz(), v[3].value())?;
                Some(if out_name == "Value" { BVal::f(val, all_exact(&v)) } else { BVal::v(vec, all_exact(&v)) })
            }
            "ShaderNodeVectorRotate" => {
                let v = self.args(node, &["Vector", "Center", "Axis", "Angle", "Rotation"], depth)?;
                let out = vector::vector_rotate(self.custom(node, "custom1"), self.custom(node, "custom2") != 0, v[0].xyz(), v[1].xyz(), v[2].xyz(), v[3].value(), v[4].xyz());
                Some(BVal::v(out, all_exact(&v)))
            }
            // `svm_node_normal`: the stored direction normalized is the
            // Normal output; Dot is its dot with the input normalized.
            "ShaderNodeNormal" => {
                let dir = vector::safe_normalize(self.first_output(node)?.xyz());
                match out_name {
                    "Dot" => {
                        let n = self.arg(node, "Normal", depth)?;
                        Some(BVal::f(vector::dot(dir, vector::safe_normalize(n.xyz())), n.exact))
                    }
                    _ => Some(BVal::v(dir, true)),
                }
            }
            "ShaderNodeFresnel" => {
                let ior = self.arg(node, "IOR", depth).map_or(1.45, |v| v.value()).max(1e-5);
                Some(BVal::f(fresnel_fss(ior), false))
            }
            "ShaderNodeLayerWeight" => {
                let blend = self.arg(node, "Blend", depth).map_or(0.5, |v| v.value());
                Some(BVal::f(
                    match out_name {
                        "Facing" => facing_mean(blend),
                        _ => fresnel_fss(1.0 / (1.0 - blend).max(1e-5)),
                    },
                    false,
                ))
            }
            // A colour attribute the graph cannot read per vertex: white,
            // as an image folds, so what multiplies it keeps its tint.
            "ShaderNodeVertexColor" | "ShaderNodeAttribute" => Some(match out_name {
                "Alpha" | "Fac" => BVal::f(1.0, false),
                _ => BVal::c([1.0; 4], false),
            }),
            _ => None,
        }
    }

    pub(super) fn emit_more(&self, node: &BlendBlock, out_id: &str, out_name: &str, bake: &mut BlendBake, depth: u32) -> Option<()> {
        let kind = self.text(node, "bNode", "idname");
        let channel = |names: [&str; 3]| names.iter().position(|c| *c == out_name).unwrap_or(0) as u8;
        match kind.as_str() {
            "ShaderNodeMapRange" if self.stored_byte(node, "NodeMapRange", "data_type") != Some(CD_PROP_FLOAT3) => {
                for id in ["Value", "From Min", "From Max", "To Min", "To Max", "Steps"] {
                    self.emit_arg(node, id, bake, depth)?;
                }
                let ty = u32::from(self.stored_byte(node, "NodeMapRange", "interpolation_type").unwrap_or(0));
                bake.ops.push(BBakeOp::MapRange { ty, clamp: self.stored_byte(node, "NodeMapRange", "clamp").unwrap_or(1) != 0 });
                Some(())
            }
            "ShaderNodeSeparateColor" => {
                self.emit_arg(node, "Color", bake, depth)?;
                let mode = self.comb_sep_mode(node);
                bake.ops.push(BBakeOp::SepColor { mode, channel: channel(["Red", "Green", "Blue"]) });
                Some(())
            }
            "ShaderNodeSeparateRGB" => {
                self.emit_arg(node, "Image", bake, depth)?;
                bake.ops.push(BBakeOp::SepColor { mode: sepcomb::COMBSEP_RGB, channel: channel(["R", "G", "B"]) });
                Some(())
            }
            "ShaderNodeCombineColor" => {
                for id in ["Red", "Green", "Blue"] {
                    self.emit_arg(node, id, bake, depth)?;
                }
                bake.ops.push(BBakeOp::CombColor { mode: self.comb_sep_mode(node) });
                Some(())
            }
            "ShaderNodeCombineRGB" => {
                for id in ["R", "G", "B"] {
                    self.emit_arg(node, id, bake, depth)?;
                }
                bake.ops.push(BBakeOp::CombColor { mode: sepcomb::COMBSEP_RGB });
                Some(())
            }
            "ShaderNodeSeparateXYZ" => {
                self.emit_arg(node, "Vector", bake, depth)?;
                bake.ops.push(BBakeOp::SepColor { mode: sepcomb::COMBSEP_RGB, channel: channel(["X", "Y", "Z"]) });
                Some(())
            }
            "ShaderNodeCombineXYZ" => {
                for id in ["X", "Y", "Z"] {
                    self.emit_arg(node, id, bake, depth)?;
                }
                bake.ops.push(BBakeOp::CombColor { mode: sepcomb::COMBSEP_RGB });
                Some(())
            }
            // The rest bake as the constant they fold to when the fold is
            // exact; a vector node over a texture is not a bake.
            _ => {
                let v = self.fold_more(node, out_id, out_name, depth).filter(|v| v.exact || matches!(kind.as_str(), "ShaderNodeFresnel" | "ShaderNodeLayerWeight" | "ShaderNodeVertexColor" | "ShaderNodeAttribute"))?;
                bake.ops.push(if v.float { BBakeOp::ConstF(v.rgba[0]) } else { BBakeOp::Const(v.rgba) });
                Some(())
            }
        }
    }
}
