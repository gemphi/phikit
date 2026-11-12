// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Tests of this module. It also holds what the others make attributes with.


pub(crate) use crate::scene::*;

pub(crate) fn std(short: &str) -> AttributeStandard {
    AttributeStandard::of(short)
}

pub(crate) fn element(short: &str) -> AttributeElement {
    AttributeElement::of(short)
}

/// A point for each of `size` vertices.
pub(crate) fn point(size: usize) -> Attribute {
    Attribute::new("P", AttributeType::Point, element("VERTEX"), size)
}

#[test]
fn a_new_attribute_has_a_zero_for_each_element_and_counts_as_changed() {
    let attribute = point(3);
    assert_eq!((attribute.size(), attribute.center.float3()), (3, &[[0.0f32; 3]; 3][..]));
    assert!(attribute.modified && !attribute.has_motion());
    assert_eq!((attribute.std, attribute.motion_step_count()), (AttributeStandard::NONE, 1));
}

#[test]
fn resizing_keeps_the_values_there_are() {
    let mut attribute = point(2);
    attribute.center.float3_mut()[1] = [1.0, 2.0, 3.0];
    attribute.resize(4);
    assert_eq!(attribute.center.float3(), [[0.0; 3], [1.0, 2.0, 3.0], [0.0; 3], [0.0; 3]]);
    attribute.resize(1);
    assert_eq!(attribute.size(), 1);
}

#[test]
fn motion_adds_a_buffer_for_each_step_but_the_centre() {
    let mut attribute = point(2);
    attribute.modified = false;
    attribute.add_motion(0);
    assert!(!attribute.has_motion() && !attribute.modified);
    attribute.add_motion(3);
    assert!(attribute.modified);
    assert_eq!((attribute.motion.len(), attribute.motion_step_count(), attribute.data(2).len()), (2, 3, 2));
    attribute.resize(5);
    assert_eq!(attribute.data(1).len(), 5);
    attribute.add_motion(2);
    assert_eq!(attribute.motion.len(), 1);
    attribute.remove_motion();
    assert!(!attribute.has_motion());
}

#[test]
fn a_step_is_its_own_values() {
    let mut attribute = point(1);
    attribute.add_motion(3);
    attribute.data_mut(1).float3_mut()[0] = [1.0; 3];
    attribute.data_mut(2).float3_mut()[0] = [2.0; 3];
    assert_eq!([0, 1, 2].map(|step| attribute.data(step).float3()[0][0]), [0.0, 1.0, 2.0]);
}

#[test]
fn in_the_order_of_time_the_centre_is_in_the_middle() {
    // Three steps: before, centre, after are stored as centre, before, after.
    assert_eq!([0, 1, 2].map(|step| Attribute::time_step_to_attr_step(step, 3)), [1, 0, 2]);
    assert_eq!([0, 1, 2, 3, 4].map(|step| Attribute::time_step_to_attr_step(step, 5)), [1, 2, 0, 3, 4]);
    assert_eq!(Attribute::time_step_to_attr_step(0, 1), 0);
}

#[test]
fn motion_steps_can_be_taken_from_another_attribute() {
    let (mut one, mut other) = (point(2), point(2));
    other.add_motion(3);
    (one.modified, other.modified) = (false, false);
    one.take_motion_from(&mut other);
    assert_eq!((one.motion.len(), other.motion.len()), (2, 0));
    assert!(one.modified && other.modified);
}

#[test]
fn taking_the_same_values_is_no_change() {
    let mut attribute = point(2);
    attribute.modified = false;
    attribute.set_data_from(point(2));
    assert!(!attribute.modified);
    let mut other = point(2);
    other.center.float3_mut()[0] = [1.0; 3];
    attribute.set_data_from(other);
    assert!(attribute.modified);
    assert_eq!(attribute.center.float3()[0], [1.0; 3]);
    attribute.modified = false;
    attribute.set_data_from(point(3));
    assert!(attribute.modified && attribute.size() == 3);
}

#[test]
fn the_device_array_follows_the_element_first_and_then_the_type() {
    let of = |kind: AttributeType, short: &str| Attribute::new("a", kind, element(short), 0).kernel_type();
    assert_eq!(of(AttributeType::Rgba, "CORNER_BYTE"), KernelDataType::Uchar4);
    assert_eq!(of(AttributeType::Normal, "VERTEX_NORMAL"), KernelDataType::Normal);
    assert_eq!((of(AttributeType::Float, "FACE"), of(AttributeType::Float2, "CORNER")), (KernelDataType::Float, KernelDataType::Float2));
    assert_eq!((of(AttributeType::Rgba, "CORNER"), of(AttributeType::Matrix, "MESH")), (KernelDataType::Float4, KernelDataType::Float4));
    assert_eq!((of(AttributeType::Point, "VERTEX"), of(AttributeType::Normal, "VERTEX")), (KernelDataType::Float3, KernelDataType::Float3));
}

#[test]
fn a_uv_map_names_the_tiles_it_reaches_into() {
    let mut uv = Attribute::new("uv", AttributeType::Float2, element("CORNER"), 5);
    // In tile 1001, in 1012, on the right edge of 1001, left of every tile, past the tenth.
    uv.center.float2_mut().copy_from_slice(&[[0.5, 0.5], [1.5, 1.2], [1.0, 0.5], [-0.5, 0.5], [10.5, 0.0]]);
    assert_eq!(uv.uv_tile_list().into_iter().collect::<Vec<i32>>(), [1001, 1012]);
    assert!(point(2).uv_tile_list().is_empty());
}
