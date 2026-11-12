// SPDX-License-Identifier: Apache-2.0
//! Tests of `kernel/bit.rs`: each constant is the number Cycles' header
//! gives it, which `data/constant.tsv` holds. A new ray and a new hit are
//! tested in `ray.rs` and `intersection.rs`; a new shading point here.
use crate::kernel::{ShaderData, bit, constant, flag};
#[test]
fn every_constant_is_the_number_the_generated_table_has() {
    for (name, value) in bit::TABLE {
        let expect = constant(name).unwrap_or_else(|| panic!("data/constant.tsv has no {name}"));
        // The table writes a mask with its sign when it is over 2^31.
        assert_eq!(*value, expect as u32, "{name}");
    }
    assert!(bit::TABLE.len() > 50);
    assert_eq!(i64::from(bit::NONE), constant("PRIM_NONE").unwrap());
}
#[test]
fn the_shader_mask_clears_the_bits_that_only_flag_a_shader() {
    let shader = 7 | bit::SHADER_SMOOTH_NORMAL | bit::SHADER_CAST_SHADOW | bit::SHADER_USE_MIS;
    assert_eq!(shader & bit::SHADER_MASK, 7);
    assert_eq!(flag("SHADER_MASK"), bit::SHADER_MASK);
}
#[test]
fn a_shading_point_starts_with_room_for_closures_and_none_in_it() {
    let sd = ShaderData::default();
    assert_eq!((sd.num_closure, sd.num_closure_left, sd.flag), (0, bit::MAX_CLOSURE, 0));
}
