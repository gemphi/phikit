//! Which render passes a view layer asks for — `ViewLayer.passflag`
//! (`DNA_scene_enums.h`'s `SCE_PASS_*`).
//!
//! The flag is a bitfield with holes in it: several bits are `SCE_PASS_UNUSED_*`
//! left behind by passes Blender removed, so the numbering is not a sequence
//! and cannot be derived. Every value here is copied from the enum.
//!
//! A pass's *name* matters as much as its bit, because it becomes the layer
//! name inside a multi-layer EXR and is what a compositor looks for. Those
//! come from `RE_PASSNAME_*`, also in `DNA_scene_enums.h`.

use crate::blend::file::types::*;

/// One render pass. Only the ones phiray can actually produce are named;
/// a file asking for others still reads, it just does not get them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BPassKind {
    Combined,
    Depth,
    Normal,
    DiffuseColor,
    Position,
    Emission,
    Mist,
    ObjectIndex,
    MaterialIndex,
    AmbientOcclusion,
    Environment,
    Uv,
    Vector,
}

impl BPassKind {
    /// `SCE_PASS_*`.
    pub fn bit(self) -> u32 {
        match self {
            Self::Combined => 1 << 0,
            Self::Depth => 1 << 1,
            Self::AmbientOcclusion => 1 << 6,
            Self::Position => 1 << 7,
            Self::Normal => 1 << 8,
            Self::Vector => 1 << 9,
            Self::ObjectIndex => 1 << 11,
            Self::Uv => 1 << 12,
            Self::Mist => 1 << 14,
            Self::Emission => 1 << 16,
            Self::Environment => 1 << 17,
            Self::MaterialIndex => 1 << 18,
            Self::DiffuseColor => 1 << 21,
        }
    }

    /// `RE_PASSNAME_*` — the name the pass carries into an EXR.
    pub fn name(self) -> &'static str {
        match self {
            Self::Combined => "Combined",
            Self::Depth => "Depth",
            Self::Normal => "Normal",
            Self::DiffuseColor => "Diffuse Color",
            Self::Position => "Position",
            Self::Emission => "Emission",
            Self::Mist => "Mist",
            Self::ObjectIndex => "Object Index",
            Self::MaterialIndex => "Material Index",
            Self::AmbientOcclusion => "Ambient Occlusion",
            Self::Environment => "Environment",
            Self::Uv => "UV",
            Self::Vector => "Vector",
        }
    }

    /// The EXR channel names this pass writes, in order. Read off a
    /// multi-layer EXR Blender rendered (`truth/blender_pass.py`) rather
    /// than guessed, because several are not what they look like: Mist's
    /// one channel is "Z", the two index passes use "X", and Ambient
    /// Occlusion is three channels rather than one.
    pub fn channel_list(self) -> &'static [&'static str] {
        match self {
            Self::Combined => &["R", "G", "B", "A"],
            Self::Depth | Self::Mist => &["Z"],
            Self::ObjectIndex | Self::MaterialIndex => &["X"],
            Self::Normal | Self::Position => &["X", "Y", "Z"],
            Self::Vector => &["X", "Y", "Z", "W"],
            Self::Uv => &["U", "V", "A"],
            Self::DiffuseColor | Self::Emission | Self::Environment | Self::AmbientOcclusion => &["R", "G", "B"],
        }
    }

    /// How many channels the pass has.
    pub fn channel(self) -> usize {
        self.channel_list().len()
    }

    /// Every pass this module knows, in the order Blender lists them.
    pub fn list() -> &'static [Self] {
        &[
            Self::Combined,
            Self::Depth,
            Self::Mist,
            Self::Position,
            Self::Normal,
            Self::Vector,
            Self::Uv,
            Self::DiffuseColor,
            Self::Emission,
            Self::Environment,
            Self::AmbientOcclusion,
            Self::ObjectIndex,
            Self::MaterialIndex,
        ]
    }
}

/// `ViewLayer.passflag`, kept as the raw bitfield so a pass phiber does not
/// model is neither lost nor silently claimed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlendPassSet {
    pub flag: u32,
}

impl Default for BlendPassSet {
    fn default() -> Self {
        // Blender's own default: Combined only.
        Self { flag: BPassKind::Combined.bit() }
    }
}

impl BlendPassSet {
    pub fn has(&self, kind: BPassKind) -> bool {
        self.flag & kind.bit() != 0
    }

    /// The passes asked for, named, in Blender's order. Combined is always
    /// among them: Blender's UI does not let it be switched off, and a render
    /// with no image at all would be a strange thing to write.
    pub fn list(&self) -> Vec<BPassKind> {
        std::iter::once(BPassKind::Combined)
            .chain(BPassKind::list().iter().copied().filter(|k| *k != BPassKind::Combined && self.has(*k)))
            .collect()
    }
}

impl BlendFile {
    /// Read `ViewLayer.passflag` off the view layer Blender opens.
    pub(crate) fn extract_pass_set(&self, scene_block: &BlendBlock, dna: &DnaType) -> BlendPassSet {
        let Some(vl) = self.view_layer_block_for(scene_block, dna) else { return Default::default() };
        match self.off("ViewLayer", "passflag", dna) {
            Some(o) if o + 4 <= vl.data.len() => BlendPassSet { flag: self.read_i32(&vl.data[o..o + 4]) as u32 },
            _ => Default::default(),
        }
    }
}
