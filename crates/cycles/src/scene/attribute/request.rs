// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/attribute.h` and `attribute.cpp`
//! (`AttributeRequest`, `AttributeRequestSet`) and `kernel/types.h`
//! (`AttributeDescriptor`): what a shader asks a geometry to have, so only
//! the attributes that are used go to the device.

use super::{AttributeElement, AttributeStandard, AttributeType};

/// `AttributeDescriptor`: where an attribute's values are on the device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AttributeDescriptor {
    pub element: AttributeElement,
    /// Its first value in the device array of its type.
    pub offset: i32,
}

/// `AttributeRequest`: an attribute asked for by name, or as a standard.
#[derive(Debug, Clone, PartialEq)]
pub struct AttributeRequest {
    pub name: String,
    pub std: AttributeStandard,
    /// `type`: filled in when the geometry's attributes are packed.
    pub kind: AttributeType,
    /// Filled in when the geometry's attributes are packed.
    pub desc: AttributeDescriptor,
}

impl AttributeRequest {
    fn new(name: &str, std: AttributeStandard) -> AttributeRequest {
        AttributeRequest { name: name.to_string(), std, kind: AttributeType::Float, desc: AttributeDescriptor::default() }
    }
}

/// `AttributeRequestSet`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AttributeRequestSet {
    /// `requests`.
    pub request: Vec<AttributeRequest>,
}

impl AttributeRequestSet {
    pub fn new() -> AttributeRequestSet {
        AttributeRequestSet::default()
    }

    /// `add(name)`: once.
    pub fn add_name(&mut self, name: &str) {
        match self.find_name(name) {
            true => {}
            false => self.request.push(AttributeRequest::new(name, AttributeStandard::NONE)),
        }
    }

    /// `add(std)`: once.
    pub fn add_standard(&mut self, std: AttributeStandard) {
        match self.find_standard(std) {
            true => {}
            false => self.request.push(AttributeRequest::new("", std)),
        }
    }

    /// `add(AttributeRequestSet)`.
    pub fn add_set(&mut self, other: &AttributeRequestSet) {
        for request in &other.request {
            match request.std == AttributeStandard::NONE {
                true => self.add_name(&request.name),
                false => self.add_standard(request.std),
            }
        }
    }

    /// `add_standard(name)`: the standard attribute of that name when
    /// there is one, else the attribute by name. An empty name asks for
    /// nothing.
    pub fn add_named(&mut self, name: &str) {
        match (name.is_empty(), AttributeStandard::name_standard(name)) {
            (true, _) => {}
            (false, AttributeStandard::NONE) => self.add_name(name),
            (false, std) => self.add_standard(std),
        }
    }

    /// `find(name)`.
    pub fn find_name(&self, name: &str) -> bool {
        self.request.iter().any(|request| request.name == name)
    }

    /// `find(std)`.
    pub fn find_standard(&self, std: AttributeStandard) -> bool {
        self.request.iter().any(|request| request.std == std)
    }

    /// `size`.
    pub fn len(&self) -> usize {
        self.request.len()
    }

    pub fn is_empty(&self) -> bool {
        self.request.is_empty()
    }

    pub fn clear(&mut self) {
        self.request.clear();
    }

    /// `modified`: `other` does not ask for the same attributes.
    pub fn modified(&self, other: &AttributeRequestSet) -> bool {
        let asked = |request: &super::AttributeRequest| other.request.iter().any(|theirs| theirs.name == request.name && theirs.std == request.std);
        self.request.len() != other.request.len() || !self.request.iter().all(asked)
    }
}
