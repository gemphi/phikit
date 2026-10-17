//! Report viewport-enabled modifiers that phiphx does not evaluate.
//!
//! `modifier_stack` silently drops unsupported modifiers from the
//! evaluation stack — correctly, because it cannot run them — but the
//! user has no way to know which geometry is missing. This walks the
//! same linked list and returns the names of the ones that were dropped,
//! so the open can surface them.

use super::KINDS;
use crate::blend::file::types::*;

/// `eModifierType` → user-facing Blender name (`DNA_modifier_types.h`).
/// Only the ones NOT in [`KINDS`] ever appear in a report, but having
/// the full table means the report says "Boolean" instead of "modifier
/// type 11".
const MODIFIER_NAMES: &[(u32, &str)] = &[
    (0, "None"),
    (1, "Subdivision Surface"),
    (2, "Lattice"),
    (3, "Curve"),
    (4, "Build"),
    (5, "Mirror"),
    (6, "Decimate"),
    (7, "Wave"),
    (8, "Armature"),
    (9, "Hook"),
    (10, "Soft Body"),
    (11, "Boolean"),
    (12, "Array"),
    (13, "Edge Split"),
    (14, "Displace"),
    (15, "UV Project"),
    (16, "Smooth"),
    (17, "Cast"),
    (18, "Mesh Deform"),
    (19, "Particle System"),
    (20, "Particle Instance"),
    (21, "Explode"),
    (22, "Cloth"),
    (23, "Collision"),
    (24, "Bevel"),
    (25, "Shrinkwrap"),
    (26, "Fluid Simulation"),
    (27, "Simple Deform"),
    (28, "Multires"),
    // 29 unused
    (30, "Surface"),
    (31, "Fluid"),
    (32, "Shape Key"),
    (33, "Solidify"),
    (34, "Screw"),
    (35, "Warp"),
    (36, "Vertex Weight Edit"),
    (37, "Vertex Weight Mix"),
    (38, "Vertex Weight Proximity"),
    (39, "Ocean"),
    (40, "Dynamic Paint"),
    (41, "Remesh"),
    (42, "Skin"),
    (43, "Laplacian Smooth"),
    (44, "Triangulate"),
    (45, "UV Warp"),
    (46, "Mesh Cache"),
    (47, "Laplacian Deform"),
    (48, "Wireframe"),
    (49, "Data Transfer"),
    (50, "Normal Edit"),
    (51, "Corrective Smooth"),
    (52, "Mesh Sequence Cache"),
    (53, "Surface Deform"),
    (54, "Weighted Normal"),
    (55, "Weld"),
    (56, "Fluid"),
    (57, "Geometry Nodes"),
    (58, "Mesh to Volume"),
    (59, "Volume to Mesh"),
    (60, "Volume Displace"),
    (88, "Line Art"),
];

/// User-facing name for a modifier type number, falling back to
/// `"modifier type N"` when the table does not cover it.
fn type_name(dna_type: u32) -> String {
    MODIFIER_NAMES
        .iter()
        .find(|(t, _)| *t == dna_type)
        .map(|(_, n)| (*n).to_string())
        .unwrap_or_else(|| format!("modifier type {dna_type}"))
}

/// Modifier-data flag: enabled in the viewport (`eModifierMode_Realtime`).
const MODE_REALTIME: u32 = 1;

impl BlendFile {
    /// Names of viewport-enabled modifiers on `ob` that the evaluation
    /// stack does not support — the geometry the open drops.
    ///
    /// Each entry is `"TypeName (user name)"`, e.g. `"Boolean (BoolCut)"`.
    /// The walk mirrors `modifier_stack` but collects what that one
    /// discards.
    pub(crate) fn skipped_modifiers(
        &self,
        ob: &BlendBlock,
        dna: &DnaType,
    ) -> Vec<String> {
        let first = self.ptr(ob, self.off("Object", "modifiers", dna));
        let next = self.off("ModifierData", "*next", dna);
        let name_at = self.modifier_name_offset(dna);
        self.scoped(ob, || self.list(first, next))
            .into_iter()
            .filter_map(|md| {
                let mode = self.int_field(md, "ModifierData", "mode", dna);
                if mode & MODE_REALTIME == 0 {
                    return None; // not viewport-enabled
                }
                let ty = self.int_field(md, "ModifierData", "type", dna);
                if KINDS.iter().any(|k| k.dna_type == ty) {
                    return None; // supported
                }
                let user = self.modifier_name(md, name_at).unwrap_or_default();
                let label = type_name(ty);
                if user.is_empty() || user == label {
                    Some(label)
                } else {
                    Some(format!("{label} ({user})"))
                }
            })
            .collect()
    }
}
