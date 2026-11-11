// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/geometry_attributes.cpp`
//! (`GeometryManager::device_update_attributes`, `AttributeTableBuilder`):
//! the attributes that are used, packed by type into the device's
//! attribute arrays, with the positions in `tri_verts`.
//!
//! Cycles counts first and allocates once; a `Vec` grows as it is filled.
//! Not ported: voxel attributes, an object's own attribute values, and
//! the position with its radius of hair and points.

use super::{AttributeIdMap, GeometryInput, GeometryManager};
use crate::scene::{Attribute, AttributeData, AttributeRequest, AttributeRequestSet, AttributeStandard, DeviceScene, Mesh, Object, ObjectManager};
use crate::util::Transform;

/// Every step of an attribute's values added to a device array; the
/// answer is where the first one is.
fn append<T: Copy>(out: &mut Vec<T>, attribute: &Attribute, view: impl for<'a> Fn(&'a AttributeData) -> &'a [T]) -> usize {
    let offset = out.len();
    (0..attribute.motion_step_count()).for_each(|step| out.extend_from_slice(view(attribute.data(step))));
    offset
}

/// `AttributeTableBuilder::add`: an attribute's values into the array of
/// its type, and where they are into the request.
fn pack(dscene: &mut DeviceScene, attribute: &Attribute, prim_offset: usize, request: &mut AttributeRequest) {
    let offset = match (&attribute.center, attribute.std == AttributeStandard::of("POSITION")) {
        (AttributeData::Byte(_), _) => append(&mut dscene.attribute_uchar4, attribute, AttributeData::byte),
        (AttributeData::Normal(_), _) => append(&mut dscene.attribute_normal, attribute, AttributeData::normal),
        (AttributeData::Float(_), _) => append(&mut dscene.attribute_float, attribute, AttributeData::float),
        (AttributeData::Float2(_), _) => append(&mut dscene.attribute_float2, attribute, AttributeData::float2),
        (AttributeData::Float4(_), _) => append(&mut dscene.attribute_float4, attribute, AttributeData::float4),
        // A matrix is three rows of four floats.
        (AttributeData::Matrix(_), _) => {
            let offset = dscene.attribute_float4.len();
            let row = |matrix: &Transform| [matrix.x, matrix.y, matrix.z];
            (0..attribute.motion_step_count()).for_each(|step| dscene.attribute_float4.extend(attribute.data(step).matrix().iter().flat_map(row)));
            offset
        }
        // Positions have an array of their own, which is faster to intersect.
        (AttributeData::Float3(_), true) => append(&mut dscene.tri_vert, attribute, AttributeData::float3),
        (AttributeData::Float3(_), false) => append(&mut dscene.attribute_float3, attribute, AttributeData::float3),
    };
    // A primitive is numbered among every mesh's, a vertex within its
    // own mesh: the offset makes up for that.
    let ahead = match (attribute.element.has("FACE"), attribute.element.has("CORNER")) {
        (true, _) => prim_offset,
        (false, true) => 3 * prim_offset,
        (false, false) => 0,
    };
    request.kind = attribute.kind;
    request.desc.element = attribute.element;
    request.desc.offset = offset as i32 - ahead as i32;
}

impl GeometryManager {
    /// What a mesh must have on the device: what the scene and its
    /// shaders ask for, and always its positions and normals.
    fn request_of(mesh: &Mesh, input: &GeometryInput) -> AttributeRequestSet {
        let mut request = AttributeRequestSet::new();
        request.add_set(input.global_attribute);
        request.add_set(&mesh.geometry.needed_attribute(input.shader));
        let always = ["POSITION", "VERTEX_NORMAL", "CORNER_NORMAL", "SHADOW_TRANSPARENCY"].map(AttributeStandard::of);
        let has = mesh.geometry.attribute.attribute.iter().map(|attribute| attribute.std).filter(|std| always.contains(std));
        has.for_each(|std| request.add_standard(std));
        request
    }

    /// `device_update_attributes`.
    pub(super) fn device_update_attribute(dscene: &mut DeviceScene, mesh: &mut [Mesh], object: &mut [Object], id: &mut AttributeIdMap, input: &GeometryInput) {
        let mut request: Vec<AttributeRequestSet> = mesh.iter().map(|mesh| Self::request_of(mesh, input)).collect();
        for (index, (mesh, request)) in mesh.iter_mut().zip(&mut request).enumerate() {
            mesh.geometry.index = index;
            for request in &mut request.request {
                match mesh.geometry.attribute.find_request(request) {
                    Some(attribute) => pack(dscene, attribute, mesh.geometry.prim_offset, request),
                    // An attribute the mesh does not have.
                    None => request.desc = Default::default(),
                }
            }
        }
        Self::update_svm_attribute(dscene, mesh, object, &request, id);
        // The values and the map are in place: the objects are told where.
        ObjectManager::device_update_geom_offset(dscene, object, mesh);
    }
}
