// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use super::element;
use crate::scene::*;
use crate::util::{PackedNormal, Transform};

#[test]
fn the_values_are_typed_by_the_element_first_and_then_the_type() {
    let of = |kind: AttributeType, short: &str| AttributeData::of(kind, element(short));
    assert_eq!(of(AttributeType::Rgba, "CORNER_BYTE"), AttributeData::Byte(Vec::new()));
    assert_eq!(of(AttributeType::Normal, "CORNER_NORMAL"), AttributeData::Normal(Vec::new()));
    assert_eq!(of(AttributeType::Float, "VERTEX"), AttributeData::Float(Vec::new()));
    assert_eq!(of(AttributeType::Float2, "CORNER"), AttributeData::Float2(Vec::new()));
    assert_eq!(of(AttributeType::Matrix, "MESH"), AttributeData::Matrix(Vec::new()));
    assert_eq!((of(AttributeType::Float4, "FACE"), of(AttributeType::Rgba, "FACE")), (AttributeData::Float4(Vec::new()), AttributeData::Float4(Vec::new())));
    // A normal not on a normal element is three plain floats, like a point.
    for kind in [AttributeType::Color, AttributeType::Point, AttributeType::Vector, AttributeType::Normal] {
        assert_eq!(of(kind, "VERTEX"), AttributeData::Float3(Vec::new()));
    }
}

#[test]
fn new_values_are_zero() {
    let mut data = AttributeData::Normal(Vec::new());
    assert!(data.is_empty());
    data.resize(2);
    assert_eq!((data.len(), data.normal()), (2, &[PackedNormal(0); 2][..]));
    let mut matrix = AttributeData::Matrix(Vec::new());
    matrix.resize(1);
    assert_eq!(matrix.matrix(), [Transform::EMPTY]);
}

#[test]
fn a_view_of_another_type_is_empty() {
    let mut data = AttributeData::Float(vec![1.0, 2.0]);
    assert_eq!(data.float(), [1.0, 2.0]);
    assert!(data.float3().is_empty() && data.byte().is_empty() && data.float2_mut().is_empty());
    data.float_mut()[0] = 5.0;
    assert_eq!(data.float(), [5.0, 2.0]);
}
