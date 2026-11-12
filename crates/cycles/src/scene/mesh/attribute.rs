// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/mesh.cpp`
//! (`Mesh::copy_center_to_motion_step`, `get_uv_tiles`,
//! `update_generated`, `add_undisplaced`): the attributes a mesh makes
//! for itself from the ones it was given.

use std::collections::BTreeSet;

use super::Mesh;
use crate::scene::{Attribute, AttributeData, AttributeStandard};

/// The centre's values, copied to motion step `step` of an attribute
/// that moves.
fn copy_center(attribute: Option<&mut Attribute>, step: usize) {
    if let Some(attribute) = attribute.filter(|attribute| attribute.has_motion()) {
        let center = attribute.center.clone();
        *attribute.data_mut(step) = center;
    }
}

impl Mesh {
    /// `copy_center_to_motion_step`: a motion step that came without
    /// positions of its own stands still at the centre's.
    pub fn copy_center_to_motion_step(&mut self, motion_step: usize) {
        let step = motion_step + 1;
        for std in ["POSITION", "VERTEX_NORMAL"].map(AttributeStandard::of) {
            copy_center(self.geometry.attribute.find_standard_mut(std), step);
            copy_center(self.subd_attribute.find_standard_mut(std), step);
        }
        copy_center(self.geometry.attribute.find_standard_mut(AttributeStandard::of("CORNER_NORMAL")), step);
    }

    /// `get_uv_tiles`: the UDIM tiles the UV map named `map` reaches
    /// into, or the standard UV map when it has no name.
    pub fn uv_tile_list(&self, map: &str) -> BTreeSet<i32> {
        let uv = AttributeStandard::of("UV");
        let found = [&self.geometry.attribute, &self.subd_attribute].map(|set| match map.is_empty() {
            true => set.find_standard(uv),
            false => set.find(map),
        });
        found.into_iter().flatten().flat_map(Attribute::uv_tile_list).collect()
    }

    /// `update_generated`: generated coordinates are the positions,
    /// copied when `needed` and the mesh has none.
    pub fn update_generated(&mut self, needed: bool) {
        let (generated, subd) = (AttributeStandard::of("GENERATED"), self.subd_face_count() > 0);
        let set = match subd {
            true => &self.subd_attribute,
            false => &self.geometry.attribute,
        };
        let empty = self.triangle_count() == 0 && !subd;
        let position = set.find_standard(AttributeStandard::of("POSITION")).map(|attribute| attribute.center.clone());
        let position = position.filter(|_| needed && !empty && set.find_standard(generated).is_none());
        if let Some(position) = position {
            let attribute = match subd {
                true => self.add_subd_attribute(generated),
                false => self.add_attribute(generated),
            };
            attribute.center = position;
        }
    }

    /// `add_undisplaced`: the positions and the normals as they are
    /// before displacement, kept when a shader needs them.
    pub fn add_undisplaced(&mut self, need_position: bool, need_normal: bool) {
        let set = &self.geometry.attribute;
        let (position, normal) = (AttributeStandard::of("POSITION_UNDISPLACED"), AttributeStandard::of("NORMAL_UNDISPLACED"));
        if need_position && set.find_standard(position).is_none() {
            let value = AttributeData::Float3(self.geometry.position().to_vec());
            self.add_attribute(position).center = value;
        }
        // The corner normals when there are some, else the vertex normals,
        // on the element they are on.
        let set = &self.geometry.attribute;
        let from = ["CORNER_NORMAL", "VERTEX_NORMAL"].into_iter().find_map(|std| set.find_standard(AttributeStandard::of(std)));
        let from = from.filter(|_| need_normal && set.find_standard(normal).is_none()).map(|from| (from.kind, from.element, from.center.clone()));
        if let Some((kind, element, value)) = from {
            let attribute = self.geometry.attribute.add(normal.standard_name(), kind, element, value.len());
            attribute.std = normal;
            attribute.center = value;
        }
    }
}

#[cfg(test)]
mod test;
