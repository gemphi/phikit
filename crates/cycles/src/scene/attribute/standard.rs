// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/types.h` (`AttributeStandard`,
//! `AttributeElement`, `AttributePrimitive`) and `scene/attribute.h`,
//! `attribute.cpp` (`AttrKernelDataType`, `Attribute::standard_name`,
//! `name_standard`, `same_storage`, `find_type_from_geometry_std`,
//! `find_element_from_geometry_std`).
//!
//! The numbers are `data/constant.tsv`; the names, and the type and element
//! a standard attribute has on each kind of geometry, are
//! `data/attribute.tsv`, which `tool/attribute.py` writes from
//! `scene/attribute.cpp`.

use std::sync::OnceLock;

use crate::kernel::{constant, flag};
use crate::scene::GeometryKind;
use crate::util::row_list;

/// `AttributeStandard`: an attribute Cycles knows the meaning of, by its
/// own number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct AttributeStandard(pub i32);

/// `AttributeElement`: what an attribute has a value for, as Cycles' bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct AttributeElement(pub u32);

/// `TypeDesc`, as far as attributes use it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttributeType {
    Float,
    Float2,
    Float4,
    Rgba,
    Color,
    Point,
    Vector,
    Normal,
    Matrix,
}

/// `AttributePrimitive`: a mesh's triangles, or its faces before they are
/// subdivided.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AttributePrimitive {
    #[default]
    Geometry,
    Subd,
}

/// `AttrKernelDataType`: the device array an attribute's values go to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KernelDataType {
    Float,
    Float2,
    Float3,
    Float4,
    Uchar4,
    Normal,
}

struct Table {
    /// A standard and the name a shader asks for it by.
    name: Vec<(AttributeStandard, &'static str)>,
    /// A kind of geometry, a standard and its type there.
    kind: Vec<(&'static str, AttributeStandard, AttributeType)>,
    element: Vec<(&'static str, AttributeStandard, AttributeElement)>,
}

fn table() -> &'static Table {
    static TABLE: OnceLock<Table> = OnceLock::new();
    TABLE.get_or_init(|| {
        let row: Vec<Vec<&'static str>> = row_list(include_str!("../../../data/attribute.tsv")).collect();
        let standard = |name: &str| constant(name).map(|number| AttributeStandard(number as i32));
        let of = |head: &'static str, width: usize| row.iter().filter(move |row| row[0] == head && row.len() >= width);
        Table {
            name: of("name", 3).filter_map(|row| Some((standard(row[1])?, row[2]))).collect(),
            kind: of("type", 4).filter_map(|row| Some((row[1], standard(row[2])?, AttributeType::named(row[3])?))).collect(),
            element: of("element", 4).filter_map(|row| Some((row[1], standard(row[2])?, AttributeElement(constant(row[3])? as u32)))).collect(),
        }
    })
}

impl AttributeStandard {
    /// `ATTR_STD_NONE`: not a standard attribute.
    pub const NONE: AttributeStandard = AttributeStandard(0);

    /// The standard `ATTR_STD_<short>`, such as `POSITION`.
    pub fn of(short: &str) -> AttributeStandard {
        let found = constant(&format!("ATTR_STD_{short}"));
        debug_assert!(found.is_some(), "data/constant.tsv has no ATTR_STD_{short}");
        AttributeStandard(found.unwrap_or(0) as i32)
    }

    /// `ATTR_STD_NUM`: one past the last standard.
    pub fn count() -> i32 {
        constant("ATTR_STD_NUM").unwrap_or(0) as i32
    }

    /// `Attribute::standard_name`: the name a shader asks for it by; empty
    /// for one that has none.
    pub fn standard_name(self) -> &'static str {
        table().name.iter().find(|(standard, _)| *standard == self).map(|(_, name)| *name).unwrap_or("")
    }

    /// `Attribute::name_standard`: the first standard with that name.
    pub fn name_standard(name: &str) -> AttributeStandard {
        let found = table().name.iter().filter(|(_, given)| *given == name).map(|(standard, _)| *standard);
        found.min_by_key(|standard| standard.0).unwrap_or(AttributeStandard::NONE)
    }

    /// `find_type_from_geometry_std`.
    pub fn type_on(self, geometry: GeometryKind) -> Option<AttributeType> {
        let key = geometry.key()?;
        table().kind.iter().find(|(kind, standard, _)| *kind == key && *standard == self).map(|row| row.2)
    }

    /// `find_element_from_geometry_std`.
    pub fn element_on(self, geometry: GeometryKind) -> Option<AttributeElement> {
        let key = geometry.key()?;
        table().element.iter().find(|(kind, standard, _)| *kind == key && *standard == self).map(|row| row.2)
    }
}

impl AttributeElement {
    /// `ATTR_ELEMENT_NONE`.
    pub const NONE: AttributeElement = AttributeElement(0);

    /// The element `ATTR_ELEMENT_<short>`, such as `VERTEX`.
    pub fn of(short: &str) -> AttributeElement {
        AttributeElement(flag(&format!("ATTR_ELEMENT_{short}")))
    }

    /// It has the bits of `ATTR_ELEMENT_<short>`.
    pub fn has(self, short: &str) -> bool {
        self.0 & AttributeElement::of(short).0 != 0
    }

    /// What the values are for, without how they are stored: the element
    /// less `IS_MOTION`, `IS_NORMAL` and `IS_BYTE`.
    pub fn domain(self) -> AttributeElement {
        let storage = ["IS_MOTION", "IS_NORMAL", "IS_BYTE"].iter().fold(0, |mask, short| mask | AttributeElement::of(short).0);
        AttributeElement(self.0 & !storage)
    }
}

impl AttributeType {
    /// The type Cycles writes so, such as `TypePoint`.
    pub fn named(name: &str) -> Option<AttributeType> {
        match name {
            "TypeFloat" => Some(AttributeType::Float),
            "TypeFloat2" => Some(AttributeType::Float2),
            "TypeFloat4" => Some(AttributeType::Float4),
            "TypeRGBA" => Some(AttributeType::Rgba),
            "TypeColor" => Some(AttributeType::Color),
            "TypePoint" => Some(AttributeType::Point),
            "TypeVector" => Some(AttributeType::Vector),
            "TypeNormal" => Some(AttributeType::Normal),
            "TypeMatrix" => Some(AttributeType::Matrix),
            _ => None,
        }
    }

    /// Three floats: a colour, a point, a vector or a normal.
    pub fn is_float3(self) -> bool {
        matches!(self, AttributeType::Color | AttributeType::Point | AttributeType::Vector | AttributeType::Normal)
    }

    /// `Attribute::same_storage`.
    pub fn same_storage(self, other: AttributeType) -> bool {
        self == other || (self.is_float3() && other.is_float3())
    }

    /// `NodeAttributeType`: how an attribute map names the type.
    pub fn node_attribute(self) -> i32 {
        let name = match self {
            AttributeType::Float => "NODE_ATTR_FLOAT",
            AttributeType::Matrix => "NODE_ATTR_MATRIX",
            AttributeType::Float2 => "NODE_ATTR_FLOAT2",
            AttributeType::Float4 => "NODE_ATTR_FLOAT4",
            AttributeType::Rgba => "NODE_ATTR_RGBA",
            _ => "NODE_ATTR_FLOAT3",
        };
        constant(name).unwrap_or(0) as i32
    }
}

impl AttributePrimitive {
    /// `ATTR_PRIM_TYPES`: how many there are.
    pub fn count() -> usize {
        constant("ATTR_PRIM_TYPES").unwrap_or(2) as usize
    }
}

#[cfg(test)]
mod test;
