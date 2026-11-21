// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! The values of one motion step of an attribute: `Attribute::Buffer` in
//! `intern/cycles/scene/attribute.h`. Cycles keeps the bytes and casts
//! them (`Attribute::data<T>`, `data_sizeof`); here the type is kept with
//! the values.

use super::{AttributeElement, AttributeType};
use crate::util::{PackedNormal, Transform};

/// One step's values, typed as `Attribute::data_sizeof` sizes them.
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeData {
    Float(Vec<f32>),
    Float2(Vec<[f32; 2]>),
    /// A point, a vector or a colour: `packed_float3`.
    Float3(Vec<[f32; 3]>),
    Float4(Vec<[f32; 4]>),
    /// A colour in bytes: `uchar4`.
    Byte(Vec<[u8; 4]>),
    Normal(Vec<PackedNormal>),
    Matrix(Vec<Transform>),
}

/// The typed views of the values: empty for values of another type.
macro_rules! view {
    ($get:ident, $get_mut:ident, $variant:ident, $item:ty) => {
        pub fn $get(&self) -> &[$item] {
            match self {
                AttributeData::$variant(value) => value,
                _ => &[],
            }
        }

        pub fn $get_mut(&mut self) -> &mut [$item] {
            match self {
                AttributeData::$variant(value) => value,
                _ => &mut [],
            }
        }
    };
}

impl AttributeData {
    /// No values yet, of the type `data_sizeof` gives an attribute of this
    /// type on this element.
    pub fn of(kind: AttributeType, element: AttributeElement) -> AttributeData {
        match (element.has("IS_BYTE"), element.has("IS_NORMAL"), kind) {
            (true, _, _) => AttributeData::Byte(Vec::new()),
            (_, true, _) => AttributeData::Normal(Vec::new()),
            (_, _, AttributeType::Float) => AttributeData::Float(Vec::new()),
            (_, _, AttributeType::Float2) => AttributeData::Float2(Vec::new()),
            (_, _, AttributeType::Matrix) => AttributeData::Matrix(Vec::new()),
            (_, _, AttributeType::Float4 | AttributeType::Rgba) => AttributeData::Float4(Vec::new()),
            _ => AttributeData::Float3(Vec::new()),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            AttributeData::Float(value) => value.len(),
            AttributeData::Float2(value) => value.len(),
            AttributeData::Float3(value) => value.len(),
            AttributeData::Float4(value) => value.len(),
            AttributeData::Byte(value) => value.len(),
            AttributeData::Normal(value) => value.len(),
            AttributeData::Matrix(value) => value.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// `count` values: those there are kept, new ones are zero. (Cycles
    /// leaves new ones unset.)
    pub fn resize(&mut self, count: usize) {
        match self {
            AttributeData::Float(value) => value.resize(count, 0.0),
            AttributeData::Float2(value) => value.resize(count, [0.0; 2]),
            AttributeData::Float3(value) => value.resize(count, [0.0; 3]),
            AttributeData::Float4(value) => value.resize(count, [0.0; 4]),
            AttributeData::Byte(value) => value.resize(count, [0; 4]),
            AttributeData::Normal(value) => value.resize(count, PackedNormal::default()),
            AttributeData::Matrix(value) => value.resize(count, Transform::EMPTY),
        }
    }

    view!(float, float_mut, Float, f32);
    view!(float2, float2_mut, Float2, [f32; 2]);
    view!(float3, float3_mut, Float3, [f32; 3]);
    view!(float4, float4_mut, Float4, [f32; 4]);
    view!(byte, byte_mut, Byte, [u8; 4]);
    view!(normal, normal_mut, Normal, PackedNormal);
    view!(matrix, matrix_mut, Matrix, Transform);
}
