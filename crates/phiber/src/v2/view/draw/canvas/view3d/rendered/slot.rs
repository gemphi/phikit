//! A file's texture node as the tracer takes it: one [`BlendProc`]
//! (`phiphx`'s `shade/texnode.rs`) mapped onto a `phiray::Proc` slot — the
//! node's kind and output, its coordinates, and its colour ramp uploaded to
//! the atlas as a 256 × 1 image. The two sides keep their own vocabularies,
//! so this is where they meet.

use super::surface::BAtlasIds;
use crate::phiphx::blend::shade::procedural::{BlendProc, BlendProcCoord, BlendProcKind};
use phiray::proc::*;
use phiray::{NO_TEX, Proc, Scene};

/// What a texture node is to the tracer.
pub(super) trait BProcSlot {
    /// `Proc::kind`'s low byte: the node and the output read.
    fn kind(&self) -> u32;
    /// `Proc::coords`: 0 Generated, 1 Object, 2 UV.
    fn coords(&self) -> u32;
    /// The slot driving `target` (`Proc::target`), its ramp uploaded once
    /// per material build (`ids` keys them by their pixel buffer).
    fn slot(&self, target: u32, out: &mut Scene, ids: &mut BAtlasIds) -> Proc;
}

impl BProcSlot for BlendProc {
    fn kind(&self) -> u32 {
        match self.kind {
            BlendProcKind::NoiseFac => PROC_NOISE_FAC,
            BlendProcKind::NoiseColor => PROC_NOISE_COLOR,
            BlendProcKind::VoronoiDistance => PROC_VORONOI_DISTANCE,
            BlendProcKind::VoronoiColor => PROC_VORONOI_COLOR,
            BlendProcKind::VoronoiRadius => PROC_VORONOI_RADIUS,
            BlendProcKind::CheckerFac => PROC_CHECKER_FAC,
            BlendProcKind::CheckerColor => PROC_CHECKER_COLOR,
            BlendProcKind::BrickFac => PROC_BRICK_FAC,
            BlendProcKind::BrickColor => PROC_BRICK_COLOR,
            BlendProcKind::Gradient => PROC_GRADIENT,
            BlendProcKind::MagicFac => PROC_MAGIC_FAC,
            BlendProcKind::MagicColor => PROC_MAGIC_COLOR,
            BlendProcKind::Wave => PROC_WAVE,
        }
    }

    fn coords(&self) -> u32 {
        match self.coords {
            BlendProcCoord::Generated => 0,
            BlendProcCoord::Object => 1,
            BlendProcCoord::Uv => 2,
        }
    }

    fn slot(&self, target: u32, out: &mut Scene, ids: &mut BAtlasIds) -> Proc {
        let ramp = self.ramp.as_ref().map_or(NO_TEX, |r| *ids.entry(r as *const Vec<u8>).or_insert_with(|| out.atlas.add(256, 1, r)));
        let kind = self.kind();
        Proc {
            kind: if self.normalize { kind | PROC_NORMALIZE } else { kind },
            coords: self.coords(),
            ramp,
            target,
            location: self.location,
            scale3: self.scale3,
            scale: self.scale,
            detail: self.detail,
            roughness: self.roughness,
            lacunarity: self.lacunarity,
            distortion: self.distortion,
            variant: self.variant,
            direction: self.direction,
            rings: self.rings,
            profile: self.profile,
            offset: self.offset,
            gain: self.gain,
            smoothness: self.smoothness,
            exponent: self.exponent,
            bias: self.bias,
            width: self.width,
            height: self.height,
            color1: self.colors[0],
            color2: self.colors[1],
            color3: self.colors[2],
            ..Default::default()
        }
    }
}
