use super::{own_channel, own_prop, split_index};

#[test]
fn a_path_names_a_channel_only_when_it_is_the_objects_own_and_in_range() {
    assert_eq!(own_channel("location", 2), Some(("location".into(), 2)));
    assert_eq!(own_channel("rotation_quaternion", 3), Some(("rotation_quaternion".into(), 3)));
    assert_eq!(own_channel("location", 3), None, "a vector has three");
    assert_eq!(own_channel("pose.bones[\"thigh\"].location", 0), None, "that one is the rig's");
    assert_eq!(own_channel("hide_render", 0), None);
    assert_eq!(split_index("location[1]"), ("location".to_string(), 1));
    assert_eq!(split_index("scale"), ("scale".to_string(), 0));
    assert_eq!(split_index("rotation_euler[2]"), ("rotation_euler".to_string(), 2));
}

#[test]
fn a_custom_property_path_is_the_whole_bracketed_name_and_nothing_else() {
    assert_eq!(own_prop("[\"IK_FK\"]").as_deref(), Some("IK_FK"));
    assert_eq!(own_prop("[\"a \\\"b\\\"\"]").as_deref(), Some("a \"b\""), "an escaped quote is part of the name");
    assert_eq!(own_prop("pose.bones[\"x\"][\"IK_FK\"]"), None, "that one is the rig's");
    assert_eq!(own_prop("[\"\"]"), None, "an empty name is no property");
    assert_eq!(own_prop("location"), None);
    assert_eq!(own_prop("[\"a\"][\"b\"]"), None, "a property of a property is not one of the object's");
}
