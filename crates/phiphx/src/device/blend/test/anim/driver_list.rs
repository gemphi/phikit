use crate::device::blend::anim::driver_list::*;

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
