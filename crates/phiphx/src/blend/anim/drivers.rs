//! Reading an object's drivers — `Object.adt → AnimData.drivers` — as the
//! Drivers editor lists them, with the load-time step Blender 5.0 applies
//! to older files (`version_bone_hide_property_driver`, versioning_500.cc):
//! a driver on `bones["…"].hide` used to live on the armature data; Blender
//! moves a copy to every object using that armature as
//! `pose.bones["…"].hide`, after the object's own drivers.

use super::fcurve::BFCurve;
use crate::blend::file::types::*;
use crate::blend::scene::types::BlendSceneObject;

/// `hide_prop_prefix` and `hide_prop_suffix`.
const HIDE_PREFIX: &str = "bones[";
const HIDE_SUFFIX: &str = "\"].hide";
/// The pose path the moved driver gets.
const POSE_PREFIX: &str = "pose.";
/// `MAIN_VERSION_FILE_ATLEAST(bmain, 500, 110)`: files below this still
/// carry the armature-side drivers.
const HIDE_DRIVER_VERSION: (u32, u32) = (500, 110);
/// `Object.type` of an armature object.
const ARMATURE: &str = "ARMATURE";

/// One object's drivers, as the Drivers editor lists them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlendDrivers {
    pub object: String,
    pub curves: Vec<BFCurve>,
}

/// `BLI_str_escape_find_quote`: the position of the next unescaped `"`.
fn escape_find_quote(text: &str) -> Option<usize> {
    let mut escape = false;
    for (position, character) in text.char_indices() {
        if character == '"' && !escape {
            return Some(position);
        }
        escape = !escape && character == '\\';
    }
    None
}

/// `BLI_str_quoted_substr_range(path, prefix, …)`: the range of the quoted
/// text right after `prefix`, when there is one.
fn quoted_substr_range(path: &str, prefix: &str) -> Option<(usize, usize)> {
    let start = path.find(prefix)? + prefix.len();
    if !path[start..].starts_with('"') {
        return None;
    }
    let start = start + 1;
    let end = start + escape_find_quote(&path[start..])?;
    Some((start, end))
}

impl BFCurve {
    /// `is_prefix_found && STREQ(rna_path + quoted_bone_name_end, hide_prop_suffix)`:
    /// a driver on a bone's hide property, as stored on the armature.
    fn drives_bone_hide(&self) -> bool {
        quoted_substr_range(&self.path, HIDE_PREFIX).is_some_and(|(_, end)| &self.path[end..] == HIDE_SUFFIX)
    }

    /// The copy an armature user gets: the same curve at `pose.` + path.
    fn posed(&self) -> BFCurve {
        BFCurve { path: format!("{POSE_PREFIX}{}", self.path), ..self.clone() }
    }
}

impl BlendFile {
    /// `FileGlobal.subversion` of the file.
    pub(crate) fn subversion(&self, dna: &DnaType) -> u32 {
        self.blocks.iter().find(|block| block.code_str == "GLOB").map_or(0, |glob| self.int_field(glob, "FileGlobal", "subversion", dna))
    }

    /// Whether this file is older than the version that moved the bone
    /// hide drivers onto objects — `!MAIN_VERSION_FILE_ATLEAST(bmain, 500, 110)`.
    fn has_armature_hide_drivers(&self, dna: &DnaType) -> bool {
        let (version, subversion) = HIDE_DRIVER_VERSION;
        let file = self.header.version_number;
        !(file > version || (file == version && self.subversion(dna) >= subversion))
    }

    /// `version_bone_hide_property_driver` for one user of an armature: the
    /// armature's `bones["…"].hide` drivers, re-pathed under `pose.`.
    fn bone_hide_drivers(&self, object: &BlendSceneObject, ob: &BlendBlock, dna: &DnaType) -> Vec<BFCurve> {
        if object.object_type != ARMATURE {
            return Vec::new();
        }
        let Some(armature) = self.block_at(self.ptr(ob, self.off("Object", "*data", dna))) else { return Vec::new() };
        self.id_drivers(armature, "bArmature", dna).iter().filter(|curve| curve.drives_bone_hide()).map(BFCurve::posed).collect()
    }

    /// Every object's drivers, in object order, for objects that have any —
    /// the object's own first, then the ones versioning moves onto it.
    pub(crate) fn drivers_list(&self, objects: &[BlendSceneObject], dna: &DnaType) -> Vec<BlendDrivers> {
        let versioned = self.has_armature_hide_drivers(dna);
        objects
            .iter()
            .filter_map(|object| {
                let ob = self.block_at(object.block_address)?;
                let mut curves = self.drivers(ob, dna);
                if versioned {
                    curves.extend(self.bone_hide_drivers(object, ob, dna));
                }
                (!curves.is_empty()).then(|| BlendDrivers { object: object.name.clone(), curves })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn curve(path: &str) -> BFCurve {
        BFCurve::new(path, 0, Vec::new())
    }

    #[test]
    fn a_bone_hide_driver_is_the_quoted_bone_name_between_the_prefix_and_the_suffix() {
        assert!(curve("bones[\"upper_arm_ik_target.L\"].hide").drives_bone_hide());
        assert!(curve("bones[\"quote \\\" inside\"].hide").drives_bone_hide(), "an escaped quote is part of the name");
        assert!(!curve("bones[\"thigh\"].hide_select").drives_bone_hide(), "the suffix is the whole rest of the path");
        assert!(curve("pose.bones[\"thigh\"].hide").drives_bone_hide(), "`strstr` finds the prefix anywhere in the path");
        assert!(!curve("bones[0].hide").drives_bone_hide(), "an index is not a quoted name");
        assert!(!curve("hide").drives_bone_hide());
    }

    #[test]
    fn the_moved_copy_is_the_same_curve_under_pose() {
        let moved = curve("bones[\"thigh.R\"].hide").posed();
        assert_eq!(moved.path, "pose.bones[\"thigh.R\"].hide");
        assert_eq!(moved.index, 0);
    }
}
