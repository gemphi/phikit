// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::scene::geometry::test::*;
use crate::kernel::flag;

#[test]
fn a_shader_s_id_carries_its_flags_in_the_top_bits() {
    let shader = Shader { id: 5, ..Shader::default() };
    assert_eq!(shader.shader_id(false), 5 | flag("SHADER_CAST_SHADOW"));
    assert_eq!(shader.shader_id(true), 5 | flag("SHADER_CAST_SHADOW") | flag("SHADER_SMOOTH_NORMAL"));
}

#[test]
fn a_named_attribute_gets_a_number_after_the_standards_and_keeps_it() {
    let mut id = AttributeIdMap::new();
    let count = AttributeStandard::count() as u64;
    assert_eq!([id.id("weight"), id.id("mask"), id.id("weight")], [count, count + 1, count]);
    assert_eq!(AttributeIdMap::standard_id(std("UV")), 5);
}
