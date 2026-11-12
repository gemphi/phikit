// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::scene::attribute::test::{element, std};
use crate::scene::*;

#[test]
fn a_standard_is_cycles_number_for_it() {
    assert_eq!((AttributeStandard::NONE.0, std("POSITION").0, std("UV").0), (0, 1, 5));
    assert_eq!(AttributeStandard::count(), 35);
}

#[test]
fn a_standard_has_the_name_a_shader_asks_for_it_by() {
    assert_eq!((std("POSITION").standard_name(), std("UV").standard_name(), std("GENERATED").standard_name()), ("P", "uv", "generated"));
    assert_eq!((std("VERTEX_NORMAL").standard_name(), std("CORNER_NORMAL").standard_name()), ("N", "N"));
    assert_eq!(AttributeStandard::NONE.standard_name(), "");
}

#[test]
fn a_name_is_the_first_standard_that_has_it() {
    assert_eq!(AttributeStandard::name_standard("uv"), std("UV"));
    assert_eq!(AttributeStandard::name_standard("N"), std("VERTEX_NORMAL"));
    assert_eq!(AttributeStandard::name_standard("my_map"), AttributeStandard::NONE);
}

#[test]
fn what_a_standard_is_depends_on_the_kind_of_geometry() {
    let mesh = GeometryKind::Mesh;
    assert_eq!((std("POSITION").type_on(mesh), std("POSITION").element_on(mesh)), (Some(AttributeType::Point), Some(element("VERTEX"))));
    assert_eq!((std("UV").type_on(mesh), std("UV").element_on(mesh)), (Some(AttributeType::Float2), Some(element("CORNER"))));
    assert_eq!(std("VERTEX_COLOR").element_on(mesh), Some(element("CORNER_BYTE")));
    assert_eq!(std("CORNER_NORMAL").element_on(mesh), Some(element("CORNER_NORMAL")));
    // On hair a UV is one for each curve, and positions are on the keys.
    assert_eq!((std("UV").element_on(GeometryKind::Hair), std("POSITION").element_on(GeometryKind::Hair)), (Some(element("CURVE")), Some(element("CURVE_KEY"))));
    // A mesh has no radius, and a light has no attributes.
    assert_eq!((std("RADIUS").type_on(mesh), std("POSITION").type_on(GeometryKind::SunLight)), (None, None));
}

#[test]
fn an_element_is_what_the_values_are_for_and_how_they_are_stored() {
    let normal = element("VERTEX_NORMAL");
    assert!(normal.has("VERTEX") && normal.has("IS_NORMAL") && !normal.has("IS_BYTE") && !normal.has("CORNER"));
    assert_eq!((normal.domain(), element("CORNER_BYTE").domain(), element("FACE").domain()), (element("VERTEX"), element("CORNER"), element("FACE")));
    assert_eq!(AttributeElement::NONE, element("NONE"));
}

#[test]
fn colours_points_vectors_and_normals_are_stored_alike() {
    assert!(AttributeType::Color.same_storage(AttributeType::Normal) && AttributeType::Float.same_storage(AttributeType::Float));
    assert!(!AttributeType::Float.same_storage(AttributeType::Float2) && !AttributeType::Point.same_storage(AttributeType::Rgba));
    assert_eq!(AttributeType::named("TypePoint"), Some(AttributeType::Point));
    assert_eq!(AttributeType::named("TypeString"), None);
}

#[test]
fn the_attribute_map_names_a_type_by_the_kernel_s_number() {
    let all = [AttributeType::Float, AttributeType::Float2, AttributeType::Point, AttributeType::Float4, AttributeType::Rgba, AttributeType::Matrix];
    assert_eq!(all.map(AttributeType::node_attribute), [0, 1, 2, 3, 4, 5]);
    assert_eq!(AttributeType::Normal.node_attribute(), AttributeType::Color.node_attribute());
    assert_eq!(AttributePrimitive::count(), 2);
}
