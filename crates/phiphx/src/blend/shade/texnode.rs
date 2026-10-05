//! A texture node as `procedural.rs` hands it to the tracer: which node and
//! output ([`BlendProcKind`]), its socket values folded through `eval.rs`
//! and its settings from the node's storage — `NodeTexNoise` (type,
//! normalize), `NodeTexVoronoi` (feature, distance, normalize),
//! `NodeTexGradient`, `NodeTexMagic` (depth), `NodeTexWave` (type,
//! directions, profile), `NodeTexBrick` (offset and squash) — with each
//! socket's default from `node_shader_tex_*.cc` when the file lacks it.

use super::procedural::BProcFinder;
use crate::blend::file::types::*;

/// What the texture is and the output read (`Proc::kind` in the tracer).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BlendProcKind {
    #[default]
    NoiseFac,
    NoiseColor,
    VoronoiDistance,
    VoronoiColor,
    /// Voronoi ▸ Radius (N-Sphere Radius).
    VoronoiRadius,
    CheckerFac,
    CheckerColor,
    BrickFac,
    BrickColor,
    /// Gradient ▸ Fac or Color (the same grey).
    Gradient,
    MagicFac,
    MagicColor,
    /// Wave ▸ Fac or Color (the same grey).
    Wave,
}

/// The coordinate the texture samples (`ShaderNodeTexCoord`'s outputs).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BlendProcCoord {
    #[default]
    Generated,
    Object,
    Uv,
}

/// One procedural texture as a material socket reads it. The fields
/// past `distortion` are the ones a node type has (the tracer's `Proc`
/// words, `procedural.wgsl`'s head); the rest stay at their default.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlendProc {
    pub kind: BlendProcKind,
    pub coords: BlendProcCoord,
    /// Mapping ▸ Location and Scale (Point), the identity without one.
    pub location: [f32; 3],
    pub scale3: [f32; 3],
    pub scale: f32,
    pub detail: f32,
    pub roughness: f32,
    pub lacunarity: f32,
    /// Noise / Magic / Wave ▸ Distortion, or Voronoi ▸ Randomness.
    pub distortion: f32,
    /// Normalize: Voronoi's (Blender 4.0+, off on a new node; a file from
    /// before has none and draws the raw distance) or Noise's (4.0+, on
    /// by default; always on before).
    pub normalize: bool,
    /// The node's type: `NodeTexNoise.type` (fBm 1 when the file predates
    /// it), `NodeTexVoronoi.feature`, `NodeTexGradient.gradient_type`,
    /// `NodeTexWave.wave_type`.
    pub variant: u32,
    /// Voronoi's metric; Wave's bands direction; Brick's offset frequency.
    pub direction: u32,
    /// Wave's rings direction; Brick's squash frequency; Magic's depth.
    pub rings: u32,
    /// Wave's profile.
    pub profile: u32,
    /// Noise Offset; Brick's offset amount; Wave Detail Scale.
    pub offset: f32,
    /// Noise Gain; Brick's squash amount; Wave Detail Roughness.
    pub gain: f32,
    /// Voronoi Smoothness; Wave Phase Offset; Brick Mortar Size.
    pub smoothness: f32,
    /// Voronoi Exponent; Brick Mortar Smooth.
    pub exponent: f32,
    /// Brick Bias, Brick Width, Row Height.
    pub bias: f32,
    pub width: f32,
    pub height: f32,
    /// Checker / Brick Color1 and Color2, Brick Mortar (linear).
    pub colors: [[f32; 3]; 3],
    /// The Color Ramp between the texture and the socket, as 256 RGBA
    /// bytes of linear colour, `None` for the texture's own output.
    pub ramp: Option<Vec<u8>>,
}

/// `SHD_NOISE_FBM`, the type of every Noise before Blender 4.1.
const NOISE_FBM: u32 = 1;
/// `SHD_VORONOI_N_SPHERE_RADIUS`.
const VORONOI_N_SPHERE_RADIUS: u32 = 4;

impl BProcFinder<'_> {
    /// `node`'s storage field `field` of struct `st`, `len` bytes wide
    /// (1 or 4), or `None` when the file's DNA has no such field.
    fn stored(&self, node: &BlendBlock, st: &str, field: &str, len: usize) -> Option<u32> {
        let storage = self.f.block_at(self.f.ptr(node, self.f.off("bNode", "*storage", self.dna)))?;
        let bytes = storage.data.get(self.f.off(st, field, self.dna)?..)?.get(..len)?;
        Some(if len == 1 { bytes[0] as u32 } else { self.f.read_i32(bytes) as u32 })
    }

    /// The node `from`, read at its output `output`; `None` for a node or
    /// an output the tracer does not evaluate (Voronoi ▸ Position, W).
    pub(super) fn texture(&self, from: &BlendBlock, output: &str) -> Option<BlendProc> {
        let idname = self.kind(from);
        let kind = idname.strip_prefix("ShaderNodeTex").unwrap_or("");
        let color = output == "Color";
        let base = BlendProc { scale: self.float(from, "Scale", 5.0), scale3: [1.0; 3], ..Default::default() };
        Some(match kind {
            "Noise" => BlendProc {
                kind: if color { BlendProcKind::NoiseColor } else { BlendProcKind::NoiseFac },
                detail: self.float(from, "Detail", 2.0),
                roughness: self.float(from, "Roughness", 0.5),
                lacunarity: self.float(from, "Lacunarity", 2.0),
                distortion: self.float(from, "Distortion", 0.0),
                offset: self.float(from, "Offset", 0.0),
                gain: self.float(from, "Gain", 1.0),
                variant: self.stored(from, "NodeTexNoise", "type", 1).unwrap_or(NOISE_FBM),
                normalize: self.stored(from, "NodeTexNoise", "normalize", 1).is_none_or(|n| n != 0),
                ..base
            },
            "Voronoi" => {
                let variant = self.stored(from, "NodeTexVoronoi", "feature", 4).unwrap_or(0);
                let kind = match (output, variant) {
                    ("Color", _) => BlendProcKind::VoronoiColor,
                    ("Radius", _) | (_, VORONOI_N_SPHERE_RADIUS) => BlendProcKind::VoronoiRadius,
                    ("Distance", _) => BlendProcKind::VoronoiDistance,
                    _ => return None,
                };
                BlendProc {
                    kind,
                    variant,
                    direction: self.stored(from, "NodeTexVoronoi", "distance", 4).unwrap_or(0),
                    normalize: self.stored(from, "NodeTexVoronoi", "normalize", 4).is_some_and(|n| n != 0),
                    detail: self.float(from, "Detail", 0.0),
                    roughness: self.float(from, "Roughness", 0.5),
                    lacunarity: self.float(from, "Lacunarity", 2.0),
                    smoothness: self.float(from, "Smoothness", 1.0),
                    exponent: self.float(from, "Exponent", 0.5),
                    distortion: self.float(from, "Randomness", 1.0),
                    ..base
                }
            }
            "Checker" => BlendProc {
                kind: if color { BlendProcKind::CheckerColor } else { BlendProcKind::CheckerFac },
                colors: [self.color(from, "Color1", [0.8; 3]), self.color(from, "Color2", [0.2; 3]), [0.0; 3]],
                ..base
            },
            "Brick" => BlendProc {
                kind: if color { BlendProcKind::BrickColor } else { BlendProcKind::BrickFac },
                colors: [self.color(from, "Color1", [0.8; 3]), self.color(from, "Color2", [0.2; 3]), self.color(from, "Mortar", [0.0; 3])],
                smoothness: self.float(from, "Mortar Size", 0.02),
                exponent: self.float(from, "Mortar Smooth", 0.1),
                bias: self.float(from, "Bias", 0.0),
                width: self.float(from, "Brick Width", 0.5),
                height: self.float(from, "Row Height", 0.25),
                direction: self.stored(from, "NodeTexBrick", "offset_freq", 4).unwrap_or(2),
                rings: self.stored(from, "NodeTexBrick", "squash_freq", 4).unwrap_or(2),
                offset: self.stored(from, "NodeTexBrick", "offset", 4).map_or(0.5, f32::from_bits),
                gain: self.stored(from, "NodeTexBrick", "squash", 4).map_or(1.0, f32::from_bits),
                ..base
            },
            "Gradient" => BlendProc { kind: BlendProcKind::Gradient, scale: 1.0, variant: self.stored(from, "NodeTexGradient", "gradient_type", 4).unwrap_or(0), ..base },
            "Magic" => BlendProc {
                kind: if color { BlendProcKind::MagicColor } else { BlendProcKind::MagicFac },
                distortion: self.float(from, "Distortion", 1.0),
                rings: self.stored(from, "NodeTexMagic", "depth", 4).unwrap_or(2),
                ..base
            },
            "Wave" => BlendProc {
                kind: BlendProcKind::Wave,
                variant: self.stored(from, "NodeTexWave", "wave_type", 4).unwrap_or(0),
                direction: self.stored(from, "NodeTexWave", "bands_direction", 4).unwrap_or(0),
                rings: self.stored(from, "NodeTexWave", "rings_direction", 4).unwrap_or(0),
                profile: self.stored(from, "NodeTexWave", "wave_profile", 4).unwrap_or(0),
                distortion: self.float(from, "Distortion", 0.0),
                detail: self.float(from, "Detail", 2.0),
                offset: self.float(from, "Detail Scale", 1.0),
                gain: self.float(from, "Detail Roughness", 0.5),
                smoothness: self.float(from, "Phase Offset", 0.0),
                ..base
            },
            _ => return None,
        })
    }
}
