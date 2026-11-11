use crate::device::blend::modifier::driver_read::modifier_path;

#[test]
fn a_modifier_path_is_a_name_and_a_property() {
    assert_eq!(modifier_path("modifiers[\"Subsurf\"].levels"), Some(("Subsurf".into(), "levels".into())));
    assert_eq!(
        modifier_path("modifiers[\"My Array\"].constant_offset_displace"),
        Some(("My Array".into(), "constant_offset_displace".into()))
    );
}

#[test]
fn a_path_that_is_not_a_modifier_setting_is_not_read_as_one() {
    assert_eq!(modifier_path("location"), None);
    assert_eq!(modifier_path("[\"gain\"]"), None);
    assert_eq!(modifier_path("modifiers[\"M\"]."), None, "no property");
    assert_eq!(modifier_path("modifiers[\"\"].levels"), None, "no name");
    // A nested path reaches through the modifier to another datablock,
    // which is not a setting on it.
    assert_eq!(modifier_path("modifiers[\"Disp\"].texture.contrast"), None);
    assert_eq!(modifier_path("modifiers[\"Disp\"].vertex_group[0]"), None);
}

#[test]
fn the_modifier_name_ends_at_its_own_quote() {
    // The name is `a`, so the property would be `x"].levels` — which
    // holds a `.` and is therefore not a setting on it. A name chosen
    // to look like the rest of the path cannot redirect the read.
    assert_eq!(modifier_path("modifiers[\"a\"].x\"].levels"), None);
    // And a plain name with a quote after it reads as itself.
    assert_eq!(modifier_path("modifiers[\"a b\"].offset"), Some(("a b".into(), "offset".into())));
}
