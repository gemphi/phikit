//! The Math and Map Range nodes around a Color Ramp, and the ramp's bake.
//!
//! A scalar node between the texture and the ramp is composed into the
//! ramp's 256 stops — so `Noise → Map Range → Color Ramp` samples the ramp
//! where Blender samples it — and one between the ramp and the socket rides
//! as the slot's tail (`BlendMath`, `svm_math` in the kernel), a linear Map
//! Range as the Multiply Add it is.

use super::procedural::{BProcFinder, RAMP_STOPS};
use super::texnode::BlendMath;
use super::scalar::BScalarOp;
use crate::blend::file::types::BlendBlock;

/// `NODE_MATH_MULTIPLY_ADD`: what a linear Map Range is as a Math tail.
const MATH_MULTIPLY_ADD: u32 = 36;

/// `eCustomDataType::CD_PROP_FLOAT3`: a Map Range over vectors, which is
/// not a scalar node.
const CD_PROP_FLOAT3: u8 = 48;

impl BProcFinder<'_> {
    /// A node's `storage` byte `field` of struct `st`.
    pub(super) fn storage_byte(&self, node: &BlendBlock, st: &str, field: &str) -> Option<u8> {
        let storage = self.f.block_at(self.f.ptr(node, self.f.off("bNode", "*storage", self.dna)))?;
        storage.data.get(self.f.off(st, field, self.dna)?).copied()
    }

    /// The scalar node `node` with its other inputs folded, or `None` for
    /// a node that is not one (a vector Map Range).
    pub(super) fn scalar_op(&self, node: &BlendBlock, kind: &str) -> Option<BScalarOp> {
        match kind {
            "ShaderNodeMath" => {
                let m = self.math(node);
                Some(BScalarOp::Math { op: m.op, b: m.b, c: m.c, clamp: m.clamp })
            }
            "ShaderNodeMapRange" if self.storage_byte(node, "NodeMapRange", "data_type") != Some(CD_PROP_FLOAT3) => Some(BScalarOp::MapRange {
                ty: u32::from(self.storage_byte(node, "NodeMapRange", "interpolation_type").unwrap_or(0)),
                from_min: self.float(node, "From Min", 0.0),
                from_max: self.float(node, "From Max", 1.0),
                to_min: self.float(node, "To Min", 0.0),
                to_max: self.float(node, "To Max", 1.0),
                steps: self.float(node, "Steps", 4.0),
                clamp: self.storage_byte(node, "NodeMapRange", "clamp").unwrap_or(1) != 0,
            }),
            _ => None,
        }
    }

    /// The op as the slot's tail: a Math node as is, a linear Map Range as
    /// `x · k + b` — its clamp kept only when the target is 0..1, which is
    /// the tail's one clamp.
    pub(super) fn tail(op: BScalarOp) -> Option<BlendMath> {
        match op {
            BScalarOp::Math { op, b, c, clamp } => Some(BlendMath { op, b, c, clamp }),
            BScalarOp::MapRange { ty: 0, from_min, from_max, to_min, to_max, clamp, .. } if from_max != from_min => {
                let k = (to_max - to_min) / (from_max - from_min);
                Some(BlendMath { op: MATH_MULTIPLY_ADD, b: k, c: to_min - from_min * k, clamp: clamp && to_min == 0.0 && to_max == 1.0 })
            }
            _ => None,
        }
    }

    /// A Color Ramp baked: its colour at each of `RAMP_STOPS` positions
    /// of the texture's value, through the scalar ops `before` it (the
    /// one nearest the ramp last), linear, clamped to bytes.
    pub(super) fn bake(&self, ramp: &BlendBlock, before: &[BScalarOp]) -> Option<Vec<u8>> {
        let band = self.f.block_at(self.f.ptr(ramp, self.f.off("bNode", "*storage", self.dna)))?;
        let mut out = Vec::with_capacity(RAMP_STOPS * 4);
        for i in 0..RAMP_STOPS {
            let x = before.iter().rev().fold(i as f32 / (RAMP_STOPS - 1) as f32, |x, op| op.apply(x));
            let c = super::blend::ramp(self.f, self.dna, band, x)?;
            out.extend(c.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8));
        }
        Some(out)
    }
}
