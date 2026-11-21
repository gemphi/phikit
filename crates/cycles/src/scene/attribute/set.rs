// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/attribute.h` and `attribute.cpp`
//! (`AttributeSet`): the attributes of one geometry.
//!
//! Cycles' set points back at its geometry to ask how many vertices,
//! corners and faces it has. Here the geometry gives the count when it
//! adds an attribute, and a function of the element when it resizes them.

use super::{Attribute, AttributeElement, AttributePrimitive, AttributeRequest, AttributeStandard, AttributeType, KernelDataType};
use crate::scene::GeometryKind;

/// `AttributeSet`.
#[derive(Debug, Clone, PartialEq)]
pub struct AttributeSet {
    /// The kind of geometry it is on, which decides what a standard
    /// attribute is there.
    pub geometry: GeometryKind,
    pub prim: AttributePrimitive,
    /// `attributes`.
    pub attribute: Vec<Attribute>,
    /// A bit for each device array an attribute was added to or removed
    /// from since the device was updated.
    modified_flag: u32,
}

impl AttributeSet {
    pub fn new(geometry: GeometryKind, prim: AttributePrimitive) -> AttributeSet {
        AttributeSet { geometry, prim, attribute: Vec::new(), modified_flag: u32::MAX }
    }

    fn tag_modified(&mut self, kernel_type: KernelDataType) {
        self.modified_flag |= 1 << kernel_type as u32;
    }

    fn remove_at(&mut self, index: usize) {
        let removed = self.attribute.remove(index);
        self.tag_modified(removed.kernel_type());
    }

    fn push(&mut self, attribute: Attribute) -> usize {
        self.tag_modified(attribute.kernel_type());
        self.attribute.push(attribute);
        self.attribute.len() - 1
    }

    /// `add(name, type, element)`: the attribute named so, made with
    /// `size` values when there is none; one of another type or element
    /// is replaced.
    pub fn add(&mut self, name: &str, kind: AttributeType, element: AttributeElement, size: usize) -> &mut Attribute {
        let found = self.attribute.iter().position(|attribute| attribute.name == name);
        let same = found.filter(|index| self.attribute[*index].kind == kind && self.attribute[*index].element == element);
        let index = match (same, found) {
            (Some(index), _) => index,
            (None, Some(index)) => {
                self.remove_at(index);
                self.push(Attribute::new(name, kind, element, size))
            }
            (None, None) => self.push(Attribute::new(name, kind, element, size)),
        };
        &mut self.attribute[index]
    }

    /// `add(std, name)`: a standard attribute, under its own name when
    /// `name` is empty, with the type and element it has on this kind of
    /// geometry.
    pub fn add_standard(&mut self, std: AttributeStandard, name: &str, size: usize) -> &mut Attribute {
        let name = match name.is_empty() {
            true => std.standard_name(),
            false => name,
        };
        let kind = std.type_on(self.geometry);
        let element = std.element_on(self.geometry);
        debug_assert!(kind.is_some() && element.is_some(), "{:?} has no standard attribute {}", self.geometry, std.0);
        let attribute = self.add(name, kind.unwrap_or(AttributeType::Float), element.unwrap_or(AttributeElement::NONE), size);
        attribute.std = std;
        attribute
    }

    /// `add_from`: `other` joins the set, or gives its values to the one
    /// of its name, type and element already there.
    pub fn add_from(&mut self, other: Attribute) -> &mut Attribute {
        let found = self.attribute.iter().position(|attribute| attribute.name == other.name);
        let same = found.filter(|index| self.attribute[*index].kind == other.kind && self.attribute[*index].element == other.element);
        let index = match (same, found) {
            (Some(index), _) => {
                self.attribute[index].std = other.std;
                self.attribute[index].set_data_from(other);
                index
            }
            (None, Some(index)) => {
                self.remove_at(index);
                self.push(other)
            }
            (None, None) => self.push(other),
        };
        &mut self.attribute[index]
    }

    /// `find(name)`.
    pub fn find(&self, name: &str) -> Option<&Attribute> {
        self.attribute.iter().find(|attribute| attribute.name == name)
    }

    pub fn find_mut(&mut self, name: &str) -> Option<&mut Attribute> {
        self.attribute.iter_mut().find(|attribute| attribute.name == name)
    }

    /// `find(std)`.
    pub fn find_standard(&self, std: AttributeStandard) -> Option<&Attribute> {
        self.attribute.iter().find(|attribute| attribute.std == std)
    }

    pub fn find_standard_mut(&mut self, std: AttributeStandard) -> Option<&mut Attribute> {
        self.attribute.iter_mut().find(|attribute| attribute.std == std)
    }

    /// Where the attribute a request asks for is in `attribute`.
    pub fn position(&self, request: &AttributeRequest) -> Option<usize> {
        match request.std == AttributeStandard::NONE {
            true => self.attribute.iter().position(|attribute| attribute.name == request.name),
            false => self.attribute.iter().position(|attribute| attribute.std == request.std),
        }
    }

    /// `find(AttributeRequest)`: by name, or by standard when it asks for one.
    pub fn find_request(&self, request: &AttributeRequest) -> Option<&Attribute> {
        self.position(request).map(|index| &self.attribute[index])
    }

    /// `find_matching`: the one with the name, standard, type and element
    /// of `other`.
    pub fn find_matching(&self, other: &Attribute) -> Option<&Attribute> {
        let same = |attribute: &&Attribute| attribute.name == other.name && attribute.std == other.std && attribute.kind == other.kind && attribute.element == other.element;
        self.attribute.iter().find(same)
    }

    /// `remove(name)`.
    pub fn remove(&mut self, name: &str) {
        let found = self.attribute.iter().position(|attribute| attribute.name == name);
        found.into_iter().for_each(|index| self.remove_at(index));
    }

    /// `remove(std)`.
    pub fn remove_standard(&mut self, std: AttributeStandard) {
        let found = self.attribute.iter().position(|attribute| attribute.std == std);
        found.into_iter().for_each(|index| self.remove_at(index));
    }

    /// `copy`: an attribute like `other`, with room for its values and its
    /// motion steps but not the values themselves.
    pub fn copy(&mut self, other: &Attribute, size: usize, motion_step_count: u32) -> &mut Attribute {
        let attribute = self.add(&other.name, other.kind, other.element, size);
        attribute.std = other.std;
        match other.has_motion() {
            true => attribute.add_motion(motion_step_count),
            false => {}
        }
        attribute
    }

    /// `resize`: every attribute to the number of its element the
    /// geometry now has.
    pub fn resize(&mut self, size: impl Fn(AttributeElement) -> usize) {
        self.attribute.iter_mut().for_each(|attribute| attribute.resize(size(attribute.element)));
    }

    /// `clear`. With `preserve_voxel_data`, voxel attributes and the
    /// generated transform stay.
    pub fn clear(&mut self, preserve_voxel_data: bool) {
        let generated = AttributeStandard::of("GENERATED_TRANSFORM");
        self.attribute.retain(|attribute| preserve_voxel_data && (attribute.element.has("VOXEL") || attribute.std == generated));
    }

    /// `update`: become `new`. Attributes `new` lacks are removed, the
    /// others take its values. `true` when the positions or the radii
    /// changed, which the geometry must be tagged for; its transform no
    /// longer counts as applied either way.
    pub fn update(&mut self, new: AttributeSet) -> bool {
        let gone: Vec<usize> = (0..self.attribute.len()).filter(|index| new.find_matching(&self.attribute[*index]).is_none()).collect();
        gone.into_iter().rev().for_each(|index| self.remove_at(index));
        let shape = [AttributeStandard::of("POSITION"), AttributeStandard::of("RADIUS")];
        let mut changed = false;
        for attribute in new.attribute {
            let added = self.add_from(attribute);
            changed |= added.modified && shape.contains(&added.std);
        }
        changed
    }

    /// `modified(kernel_type)`: an attribute of that device array was
    /// added or removed. Whether values changed is `Attribute::modified`.
    pub fn modified(&self, kernel_type: KernelDataType) -> bool {
        self.modified_flag & (1 << kernel_type as u32) != 0
    }

    /// `clear_modified`.
    pub fn clear_modified(&mut self) {
        self.attribute.iter_mut().for_each(|attribute| attribute.modified = false);
        self.modified_flag = 0;
    }
}
