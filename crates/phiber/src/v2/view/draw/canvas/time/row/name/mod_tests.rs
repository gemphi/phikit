//! The shape of a channel name. Its *content* — that "Power" and not
//! "Energy" is what a light's `energy` reads as — is
//! `tests/channel_name_truth.rs`, which holds every row of Blender's own
//! dump. These pin the assembly: the letter, the owner, the fallback, and
//! the two records this module keeps about owners it cannot resolve.

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
    // An axis-angle takes the quaternion letters too (`PROP_AXISANGLE` sits
    // in the same `ELEM` as `PROP_QUATERNION`), which is easy to miss.
    assert_eq!(named("rotation_axis_angle", 0), "W Axis-Angle Rotation");
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
fn the_three_owner_sensitive_names_follow_the_path() {
    // `OWNED`, one assertion per row and both sides of each.
    assert_eq!(named("matrix_basis", 0), "Input Matrix");
    assert_eq!(named(r#"pose.bones["Chest"].matrix_basis"#, 0), "Basis Matrix (Chest)");
    assert_eq!(named(r#"constraints["Limit"].mute"#, 0), "Disable (Limit)");
    assert_eq!(named(r#"key_blocks["Smile"].mute"#, 0), "Mute (Smile)");
    assert_eq!(named(r#"constraints["IK"].weight"#, 0), "Weight (IK)");
    assert_eq!(
        named(r#"pose.bones["Arm"].constraints["Arm"].targets[0].weight"#, 0),
        "Blend Weight (Arm : Arm)"
    );
}

#[test]
fn a_shape_key_value_names_its_key() {
    // The other half of a rig, and the one `ShapeKey` property that is
    // almost always driven.
    assert_eq!(named(r#"key_blocks["Smile"].value"#, 0), "Value (Smile)");
}

#[test]
fn the_four_names_the_path_cannot_resolve_print_their_recorded_spelling() {
    // `AMBIGUOUS` is a record, and this is what keeps it true: each row
    // says what phiber prints, so a table edit that changes one of these
    // without updating the record fails here rather than going unnoticed.
    for row in AMBIGUOUS {
        let printed = row.printed.0;
        assert_eq!(named(row.property, 0), printed, "{} is recorded as printing {printed:?}", row.property);
    }
}

#[test]
fn no_identifier_is_in_two_tables_at_once() {
    // The four tables are split only to keep each file readable, and the
    // lookup takes the first hit — so an identifier in two of them would
    // make the name depend on the chain order rather than on Blender.
    let mut seen: Vec<&str> = Vec::new();
    let mut twice: Vec<&str> = Vec::new();
    for (property, _, _) in set::OBJECT.iter().chain(data::DATABLOCK).chain(rig::RIG).chain(modifier::MODIFIER) {
        if seen.contains(property) {
            twice.push(property);
        }
        seen.push(property);
    }
    assert!(twice.is_empty(), "identifiers in more than one table: {twice:?}");
    // And nothing owner-sensitive sits in a table, where it would be found
    // first and the path never consulted.
    for row in OWNED {
        assert!(!seen.contains(&row.property), "{} is owner-sensitive and must not be tabulated", row.property);
    }
}

#[test]
fn every_name_is_something_a_row_can_print() {
    for (path, index) in [
        ("location", 0),
        ("", 0),
        ("unknown", 9),
        (r#"pose.bones["A"].scale"#, 2),
        ("rotation_quaternion", 7),
        (r#"nodes["Principled BSDF"].inputs[0].default_value"#, 1),
    ] {
        let name = named(path, index);
        assert!(!name.is_empty() && !name.contains('\n'), "{path:?}[{index}] → {name:?}");
    }
}
