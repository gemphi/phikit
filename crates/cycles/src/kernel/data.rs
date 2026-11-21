// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/types.h` (`KernelData`) and
//! `kernel/data_template.h`: the constant data every kernel reads, such as
//! `kernel_data.film.pass_combined`.
//!
//! Cycles builds these structs from a template so a GPU kernel can be
//! compiled with their members as constants. Here the template is
//! `data/kernel.tsv`, written by `tool/kernel.py`, and the data is the
//! words it lays out: the same buffer for the CPU and for a GPU. The light
//! link sets are not in it yet: they come with the lights.

use std::sync::OnceLock;

use crate::util::{ProjectionTransform, Transform, row_list};

/// What a member holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    Int,
    Uint,
    Float,
    Float2,
    Float4,
    /// Twelve floats: three rows of a matrix.
    Transform,
    /// Sixteen floats: a full matrix.
    Projection,
}

/// One member, such as `film.pass_combined`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Field {
    /// The member of `KernelData` it is in, such as `film`.
    pub parent: &'static str,
    pub name: &'static str,
    pub kind: FieldKind,
    /// Its first 32-bit word in the data.
    pub word: usize,
    pub word_count: usize,
    /// Cycles may make it a constant when it compiles a kernel for a scene.
    pub specialize: bool,
}

/// One struct of the data, such as `KernelFilm`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataStruct {
    pub parent: &'static str,
    /// The C++ struct.
    pub name: &'static str,
    pub word: usize,
    pub word_count: usize,
}

struct Layout {
    block: Vec<DataStruct>,
    field: Vec<Field>,
}

fn layout() -> &'static Layout {
    static LAYOUT: OnceLock<Layout> = OnceLock::new();
    LAYOUT.get_or_init(|| {
        let row: Vec<Vec<&'static str>> = row_list(include_str!("../../data/kernel.tsv")).collect();
        let number = |row: &[&str], index: usize| row.get(index).and_then(|cell| cell.parse::<usize>().ok()).unwrap_or(0);
        let block = row.iter().filter(|row| row[0] == "struct" && row.len() >= 5);
        let field = row.iter().filter(|row| row[0] == "member" && row.len() >= 7);
        Layout {
            block: block.map(|row| DataStruct { parent: row[1], name: row[2], word: number(row, 3), word_count: number(row, 4) }).collect(),
            field: field.filter_map(|row| Some(Field { parent: row[1], name: row[2], kind: kind(row[3])?, word: number(row, 4), word_count: number(row, 5), specialize: row[6] == "1" })).collect(),
        }
    })
}

fn kind(name: &str) -> Option<FieldKind> {
    match name {
        "int" => Some(FieldKind::Int),
        "uint" => Some(FieldKind::Uint),
        "float" => Some(FieldKind::Float),
        "float2" => Some(FieldKind::Float2),
        "float4" => Some(FieldKind::Float4),
        "Transform" => Some(FieldKind::Transform),
        "ProjectionTransform" => Some(FieldKind::Projection),
        _ => None,
    }
}

/// `KernelData`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelData {
    word: Vec<u32>,
}

impl Default for KernelData {
    fn default() -> Self {
        Self::new()
    }
}

impl KernelData {
    /// The data, every member zero.
    pub fn new() -> KernelData {
        let word_count = layout().block.iter().map(|block| block.word + block.word_count).max().unwrap_or(0);
        KernelData { word: vec![0; word_count] }
    }

    /// The structs of the data, in order.
    pub fn struct_list() -> &'static [DataStruct] {
        &layout().block
    }

    /// Every member, in order.
    pub fn field_list() -> &'static [Field] {
        &layout().field
    }

    /// The member `parent.name`, such as `film`, `pass_combined`. A kernel
    /// looks its members up once and keeps them.
    pub fn field(parent: &str, name: &str) -> Option<&'static Field> {
        let found = layout().field.iter().find(|field| field.parent == parent && field.name == name);
        debug_assert!(found.is_some(), "data/kernel.tsv has no {parent}.{name}");
        found
    }

    /// The words, as a device takes them.
    pub fn word(&self) -> &[u32] {
        &self.word
    }

    /// The words of a member.
    pub fn at(&self, field: &Field) -> &[u32] {
        &self.word[field.word..field.word + field.word_count]
    }

    fn put(&mut self, parent: &str, name: &str, value: &[u32]) {
        let Some(field) = Self::field(parent, name) else {
            return;
        };
        debug_assert!(field.word_count == value.len(), "{parent}.{name} is {} words", field.word_count);
        let count = field.word_count.min(value.len());
        self.word[field.word..field.word + count].copy_from_slice(&value[..count]);
    }

    fn take(&self, parent: &str, name: &str) -> &[u32] {
        Self::field(parent, name).map(|field| self.at(field)).unwrap_or(&[])
    }

    pub fn set_int(&mut self, parent: &str, name: &str, value: i32) {
        self.put(parent, name, &[value as u32]);
    }

    pub fn set_uint(&mut self, parent: &str, name: &str, value: u32) {
        self.put(parent, name, &[value]);
    }

    pub fn set_float(&mut self, parent: &str, name: &str, value: f32) {
        self.put(parent, name, &[value.to_bits()]);
    }

    pub fn set_float2(&mut self, parent: &str, name: &str, value: [f32; 2]) {
        self.put(parent, name, &value.map(f32::to_bits));
    }

    pub fn set_float4(&mut self, parent: &str, name: &str, value: [f32; 4]) {
        self.put(parent, name, &value.map(f32::to_bits));
    }

    pub fn set_transform(&mut self, parent: &str, name: &str, value: &Transform) {
        self.put(parent, name, &value.word().map(f32::to_bits));
    }

    pub fn set_projection(&mut self, parent: &str, name: &str, value: &ProjectionTransform) {
        self.put(parent, name, &value.word().map(f32::to_bits));
    }

    /// The floats of a member of any size: a vector or a matrix, row by
    /// row.
    pub fn float_list(&self, parent: &str, name: &str) -> Vec<f32> {
        self.take(parent, name).iter().map(|word| f32::from_bits(*word)).collect()
    }

    pub fn int(&self, parent: &str, name: &str) -> i32 {
        self.uint(parent, name) as i32
    }

    pub fn uint(&self, parent: &str, name: &str) -> u32 {
        self.take(parent, name).first().copied().unwrap_or(0)
    }

    pub fn float(&self, parent: &str, name: &str) -> f32 {
        f32::from_bits(self.uint(parent, name))
    }

    pub fn float2(&self, parent: &str, name: &str) -> [f32; 2] {
        let word = self.take(parent, name);
        [0, 1].map(|index| word.get(index).copied().map(f32::from_bits).unwrap_or(0.0))
    }

    pub fn float4(&self, parent: &str, name: &str) -> [f32; 4] {
        self.row_list(parent, name)[0]
    }

    pub fn transform(&self, parent: &str, name: &str) -> Transform {
        let [x, y, z, _] = self.row_list(parent, name);
        Transform { x, y, z }
    }

    pub fn projection(&self, parent: &str, name: &str) -> ProjectionTransform {
        let [x, y, z, w] = self.row_list(parent, name);
        ProjectionTransform { x, y, z, w }
    }

    /// A member's floats in rows of four, as a matrix has them; a row the
    /// member does not reach is zero.
    fn row_list(&self, parent: &str, name: &str) -> [[f32; 4]; 4] {
        let word = self.take(parent, name);
        std::array::from_fn(|row| std::array::from_fn(|column| word.get(row * 4 + column).copied().map(f32::from_bits).unwrap_or(0.0)))
    }
}
