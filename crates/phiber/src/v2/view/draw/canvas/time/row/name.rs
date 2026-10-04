//! What an F-curve is called in the channel list — `getname_anim_fcurve`.
//!
//! Ported from Blender 5.2.0 in `crates/phiber/refs/blender`,
//! `editors/animation/anim_ipo_utils.cc:48`. A channel's name is not stored
//! anywhere in the file: it is BUILT, every draw, from the curve's RNA path and
//! array index.
//!
//! Blender assembles three pieces and joins them as `"%s%s (%s)"` — array letter,
//! property name, struct name — or `"%s%s"` when there is no struct. Its own
//! comment calls this "option 2" and admits it "does have problems with looking
//! rather odd though"; it is what ships, so it is what this does.
//! `pose.bones["Chest"].location` index 0 is `X Location (Chest)`.
//!
//! **This is the one implementation.** It began life as `BTimeRow::driver_name`
//! in `time/driver/`, because the Drivers editor was the first channel list that
//! needed names. The Dope Sheet's F-curve rows need exactly the same function, so
//! it moved here rather than being written a second time — which is what nearly
//! happened: a second, slightly different version was written before anyone
//! grepped. Two namings would mean the same curve reading one way in the Drivers
//! editor and another in the Dope Sheet.
//!
//! The letters are `RNA_property_array_item_char`
//! (`makesrna/intern/rna_access.cc:1370`), the piece most likely to be guessed
//! wrong: vectors run `XYZW` but quaternions run `WXYZ`, so index 0 of a
//! quaternion is **W**, not X. Colours run `RGBA`. Past index 3 every subtype
//! falls through to Blender's `[%d]` form.
//!
//! The property names are RNA's `ui_text`, not the path segment:
//! `rotation_euler` displays as "Euler Rotation" and `rotation_quaternion` as
//! "Quaternion Rotation" (`rna_object.cc:3158-3199`). Writing the segment through
//! unchanged would give "X rotation_euler". A property not in the table falls
//! back to `title_case`, which is how RNA derives a UI name from an identifier
//! when one is not given.

use crate::phiphx::blend::BFCurve;

/// `RNA_property_ui_name` of the properties a channel list reaches, and the axis
/// letters `RNA_property_array_item_char` gives their items.
///
/// An empty letter set means the property is not an array, so it takes no letter
/// at all — `arrayname = ""` in Blender, which is why the format string can
/// concatenate it unconditionally.
const PROPERTY_NAMES: &[(&str, &str, &str)] = &[
    ("influence", "Influence", ""),
    ("mute", "Disable", ""),
    ("hide", "Hide", ""),
    ("hide_viewport", "Disable in Viewports", ""),
    ("hide_render", "Disable in Renders", ""),
    ("weight", "Weight", ""),
    ("value", "Value", ""),
    ("location", "Location", "XYZW"),
    ("scale", "Scale", "XYZW"),
    ("rotation_euler", "Euler Rotation", "XYZW"),
    ("rotation_quaternion", "Quaternion Rotation", "WXYZ"),
    ("rotation_axis_angle", "Axis-Angle Rotation", "WXYZ"),
    ("delta_location", "Delta Location", "XYZW"),
    ("delta_scale", "Delta Scale", "XYZW"),
    ("delta_rotation_euler", "Delta Euler Rotation", "XYZW"),
    ("delta_rotation_quaternion", "Delta Quaternion Rotation", "WXYZ"),
    ("color", "Color", "RGBA"),
];

/// `ConstraintTargetBone.weight`: the `targets[…]` of an Armature constraint.
const TARGET_WEIGHT: &str = "Blend Weight";

/// `BLI_str_quoted_substr(path, prefix, …)`: the quoted name after `prefix`.
fn quoted_after<'a>(path: &'a str, prefix: &str) -> Option<&'a str> {
    let rest = &path[path.find(prefix)? + prefix.len()..];
    let rest = rest.strip_prefix('"')?;
    rest.split_once('"').map(|(name, _)| name)
}

/// `RNA_def_property`'s default UI name: the identifier, spaced and capitalised.
fn title_case(identifier: &str) -> String {
    identifier
        .split('_')
        .map(|word| {
            let mut letters = word.chars();
            letters.next().map_or(String::new(), |first| first.to_uppercase().chain(letters).collect())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// `arrayname`: the item's letter and a space, or Blender's bracketed index when
/// the subtype has no letter for it (`if (c)` fails past index 3).
fn array_prefix(letters: &str, index: u32) -> String {
    if letters.is_empty() {
        return String::new();
    }
    match letters.chars().nth(index as usize) {
        Some(letter) => format!("{letter} "),
        None => format!("[{index}]"),
    }
}

/// The `structname` half: the bone, the constraint, or both as `bone : constraint`.
///
/// Blender's first branch is exactly that pair; its remaining branches ask RNA
/// for a struct's name property, which has no equivalent here. A modifier is
/// kept because a driver on one is common and reads as nothing otherwise.
fn owner(path: &str) -> Option<String> {
    let bone = quoted_after(path, "bones[");
    let constraint = quoted_after(path, "constraints[");
    match (bone, constraint) {
        (Some(bone), Some(constraint)) => Some(format!("{bone} : {constraint}")),
        (Some(bone), None) => Some(bone.to_string()),
        (None, Some(constraint)) => Some(constraint.to_string()),
        (None, None) => quoted_after(path, "modifiers[").map(str::to_string),
    }
}

/// `getname_anim_fcurve`: `<axis> <property> (<struct>)`.
pub fn fcurve_name(curve: &BFCurve) -> String {
    // `if (fcu->rna_path == nullptr)` — a curve the file gave no path.
    if curve.path.is_empty() {
        return "<no path>".to_string();
    }
    let property = curve.property();
    let (label, letters) = PROPERTY_NAMES
        .iter()
        .find(|(name, _, _)| *name == property)
        .map_or_else(|| (title_case(property), ""), |(_, label, letters)| ((*label).to_string(), *letters));
    let label = if property == "weight" && curve.path.contains(".targets[") { TARGET_WEIGHT.to_string() } else { label };
    let axis = array_prefix(letters, curve.index);
    match owner(&curve.path) {
        Some(owner) => format!("{axis}{label} ({owner})"),
        None => format!("{axis}{label}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named(path: &str, index: u32) -> String {
        fcurve_name(&BFCurve::new(path, index, Vec::new()))
    }

    #[test]
    fn an_objects_transform_reads_the_way_blender_prints_it() {
        // `"%s%s"` — no struct, because the path is the object's own property.
        assert_eq!(named("location", 0), "X Location");
        assert_eq!(named("location", 2), "Z Location");
        assert_eq!(named("scale", 1), "Y Scale");
    }

    #[test]
    fn a_quaternion_starts_at_w_and_a_vector_starts_at_x() {
        // `quatitem = "WXYZ"` against `vectoritem = "XYZW"` — index 0 is W for
        // one and X for the other, which is the easiest thing here to get wrong.
        assert_eq!(named("rotation_quaternion", 0), "W Quaternion Rotation");
        assert_eq!(named("rotation_quaternion", 1), "X Quaternion Rotation");
        assert_eq!(named("rotation_euler", 0), "X Euler Rotation");
        // The display names are RNA's, never the raw identifier.
        assert!(!named("rotation_euler", 0).contains('_'));
    }

    #[test]
    fn a_bone_channel_names_its_bone_and_a_constraint_names_both() {
        assert_eq!(named(r#"pose.bones["Chest"].location"#, 0), "X Location (Chest)");
        assert_eq!(
            named(r#"pose.bones["Chest"].constraints["Copy Location"].influence"#, 0),
            "Influence (Chest : Copy Location)"
        );
    }

    #[test]
    fn a_property_that_is_not_an_array_takes_no_letter() {
        // `arrayname = ""` when `RNA_property_array_check` is false, so this must
        // not come out as "X Influence" or "[0] Influence".
        assert_eq!(named("influence", 0), "Influence");
        assert_eq!(named("hide_render", 0), "Disable in Renders");
    }

    #[test]
    fn past_the_fourth_item_the_index_is_written_out() {
        // `if (c)` fails for index >= 4 and Blender writes `[%d]`. Note the space
        // after a letter and none after a bracket, which is why both are built in
        // one place.
        assert_eq!(named("location", 4), "[4]Location");
        assert_eq!(named("color", 3), "A Color");
    }

    #[test]
    fn an_unknown_property_takes_rnas_default_ui_name() {
        // RNA derives a UI name from the identifier when one is not declared, so
        // an unlisted property reads as words rather than as a raw path.
        assert_eq!(named("some_new_property", 0), "Some New Property");
        assert_eq!(named("", 0), "<no path>");
    }

    #[test]
    fn every_name_is_something_a_row_can_print() {
        for (path, index) in [
            ("location", 0),
            ("", 0),
            ("unknown", 9),
            (r#"pose.bones["A"].scale"#, 2),
            ("rotation_quaternion", 7),
        ] {
            let name = named(path, index);
            assert!(!name.is_empty() && !name.contains('\n'), "{path:?}[{index}] → {name:?}");
        }
    }
}
