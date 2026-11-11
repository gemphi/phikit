//! The supported modifier kinds: `eModifierType` to reader.
use super::*;

type Reader = fn(&BlendFile, &BlendBlock, &BlendBlock, &DnaType) -> Option<Box<dyn BModifier>>;

/// One supported modifier type.
pub(crate) struct BModifierKind {
    pub dna_type: u32,
    pub name: &'static str,
    pub read: Reader,
}

/// Supported modifiers, by `eModifierType`.
pub(crate) const KINDS: &[BModifierKind] = &[
    BModifierKind { dna_type: 8, name: "ARMATURE", read: armature::read },
    BModifierKind { dna_type: 1, name: "SUBSURF", read: subsurf::read },
    BModifierKind { dna_type: 5, name: "MIRROR", read: mirror::read },
    BModifierKind { dna_type: 12, name: "ARRAY", read: array::read },
    BModifierKind { dna_type: 44, name: "TRIANGULATE", read: triangulate::read },
    BModifierKind { dna_type: 33, name: "SOLIDIFY", read: solidify::read },
    BModifierKind { dna_type: 14, name: "DISPLACE", read: displace::read },
    BModifierKind { dna_type: 55, name: "WELD", read: weld::read },
    BModifierKind { dna_type: 2, name: "LATTICE", read: lattice::read },
    BModifierKind { dna_type: 51, name: "CORRECTIVE_SMOOTH", read: corrective_smooth::read },
    BModifierKind { dna_type: 25, name: "SHRINKWRAP", read: shrinkwrap::read },
    BModifierKind { dna_type: 27, name: "MASK", read: mask::read },
    BModifierKind { dna_type: 3, name: "CURVE", read: curve::read },
    BModifierKind { dna_type: 29, name: "MULTIRES", read: multires::read },
    BModifierKind { dna_type: 28, name: "SIMPLE_DEFORM", read: simple_deform::read },
    BModifierKind { dna_type: 53, name: "SURFACE_DEFORM", read: surface_deform::read },
    BModifierKind { dna_type: 22, name: "CLOTH", read: cloth::read },
    BModifierKind { dna_type: nodes::DNA_TYPE, name: "NODES", read: nodes::read },
];

/// Names of the modifiers the viewport evaluates (Blender's RNA type ids).
pub fn supported_modifiers() -> impl Iterator<Item = &'static str> {
    KINDS.iter().map(|k| k.name)
}
