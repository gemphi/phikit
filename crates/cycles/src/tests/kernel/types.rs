// SPDX-License-Identifier: Apache-2.0
//! Tests of `kernel/types.rs`: each constant is the number Cycles' header
//! gives it, which `data/constant.tsv` holds. A new ray and a new hit are
//! tested in `ray.rs` and `intersection.rs`.
use crate::kernel::{constant, flag, types};
#[test]
fn every_constant_is_the_number_the_generated_table_has() {
    for (name, value) in types::TABLE {
        let expect = constant(name).unwrap_or_else(|| panic!("data/constant.tsv has no {name}"));
        // The table writes a mask with its sign when it is over 2^31.
        assert_eq!(*value, expect as u32, "{name}");
    }
    assert!(types::TABLE.len() > 50);
    assert_eq!(i64::from(types::NONE), constant("PRIM_NONE").unwrap());
}
#[test]
fn the_shader_mask_clears_the_bits_that_only_flag_a_shader() {
    let shader = 7 | types::SHADER_SMOOTH_NORMAL | types::SHADER_CAST_SHADOW | types::SHADER_USE_MIS;
    assert_eq!(shader & types::SHADER_MASK, 7);
    assert_eq!(flag("SHADER_MASK"), types::SHADER_MASK);
}
#[test]
fn a_shading_point_starts_with_room_for_closures_and_none_in_it() {
    let sd = types::ShaderData::default();
    assert_eq!((sd.num_closure, sd.num_closure_left, sd.flag), (0, types::MAX_CLOSURE, 0));
}
