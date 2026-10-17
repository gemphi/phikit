//! Displace modifier (`DisplaceModifierData`, Blender `MOD_displace.cc`),
//! textureless: every vertex moves along the chosen direction by
//! `strength * (1 - midlevel)`, which is what Blender shows for a Displace
//! with no texture assigned — the common case in a viewport-only read.
//!
//! Not yet: texture sampling (so `RGB_XYZ`, which needs one, is a no-op, as
//! it is in Blender), custom normals and per-vertex-group strength.

use super::geom::vertex_normals;
use super::driver::BModValue;
use super::{BModifier, BlendEvalMesh, EvalCtx};
use crate::blend::file::types::*;
use glam::Vec3;

/// `MOD_DISP_DIR_*`.
const DIR_X: u32 = 0;
const DIR_Y: u32 = 1;
const DIR_Z: u32 = 2;
const DIR_NOR: u32 = 3;
const DIR_CLNOR: u32 = 5;

pub(crate) struct BDisplace {
    pub strength: f32,
    pub midlevel: f32,
    pub direction: u32,
}

pub(super) fn read(
    f: &BlendFile,
    md: &BlendBlock,
    _ob: &BlendBlock,
    dna: &DnaType,
) -> Option<Box<dyn BModifier>> {
    Some(Box::new(BDisplace {
        strength: f.float_field(md, "DisplaceModifierData", "strength", dna, 1.0),
        midlevel: f.float_field(md, "DisplaceModifierData", "midlevel", dna, 0.5),
        direction: f.int_field(md, "DisplaceModifierData", "direction", dna),
    }))
}

impl BDisplace {
    #[cfg(test)]
    pub(crate) fn for_test() -> Self {
        Self { strength: 1.0, midlevel: 0.5, direction: 0 }
    }

    fn driven(&mut self, prop: &str, value: BModValue) -> bool {
        match prop {
            "strength" => self.strength = value.as_f32(),
            // The path's name and the field's differ by an underscore.
            "mid_level" => self.midlevel = value.as_f32(),
            _ => return false,
        }
        true
    }
}
impl BModifier for BDisplace {
    fn clone_box(&self) -> Box<dyn BModifier> {
        Box::new(Self { strength: self.strength, midlevel: self.midlevel, direction: self.direction })
    }

    fn set_driven(&mut self, prop: &str, _component: usize, value: BModValue) -> bool {
        self.driven(prop, value)
    }

    /// `MOD_displace.cc`'s `is_disabled`:
    /// `(!texture && direction == MOD_DISP_DIR_RGB_XYZ) || strength == 0`.
    ///
    /// This reader is textureless, so `!texture` always holds and the first
    /// branch reduces to the direction alone: an RGB→XYZ displace with no
    /// texture to read the colours from has nothing to displace along, and
    /// Blender drops it rather than falling back to an axis.
    fn disabled(&self) -> bool {
        const MOD_DISP_DIR_RGB_XYZ: u32 = 4;
        self.direction == MOD_DISP_DIR_RGB_XYZ || self.strength == 0.0
    }

    fn apply(&self, mesh: &mut BlendEvalMesh, _ctx: &EvalCtx) {
        // With no texture the sampled value is 1, so this is the whole delta.
        let delta = self.strength * (1.0 - self.midlevel);
        if delta == 0.0 || mesh.vertices.is_empty() {
            return;
        }
        let axis = match self.direction {
            DIR_X => Some(Vec3::X),
            DIR_Y => Some(Vec3::Y),
            DIR_Z => Some(Vec3::Z),
            // Custom normals fall back to the computed ones, as Blender does
            // when the mesh carries none.
            DIR_NOR | DIR_CLNOR => None,
            // `RGB_XYZ` reads the texture's colour; without one, nothing moves.
            _ => return,
        };
        match axis {
            Some(dir) => {
                for v in &mut mesh.vertices {
                    *v = (Vec3::from(*v) + dir * delta).into();
                }
            }
            None => {
                let normals = vertex_normals(mesh);
                for (v, n) in mesh.vertices.iter_mut().zip(normals) {
                    *v = (Vec3::from(*v) + n * delta).into();
                }
            }
        }
    }
}
