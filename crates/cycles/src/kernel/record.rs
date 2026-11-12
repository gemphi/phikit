// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/types.h`: the structs a device array
//! holds one of for each element, such as `KernelObject` for each object
//! and `AttributeMap` for each entry of the attribute map.
//!
//! The layouts are `data/record.tsv`, written by `tool/kernel.py` from
//! those structs. As with the constant data, an array is the words the
//! table lays out, not Cycles' bytes: a member takes whole words, a 64-bit
//! one two.

use std::sync::OnceLock;

use crate::util::{Transform, row_list};

/// One member of a struct, such as `KernelObject::tfm`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordField {
    pub name: &'static str,
    /// The C++ type of one of its parts, such as `float`.
    pub kind: &'static str,
    /// Its first 32-bit word in an element.
    pub word: usize,
    pub word_count: usize,
}

/// One struct, such as `KernelObject`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordLayout {
    /// The C++ struct.
    pub name: &'static str,
    /// The words of one element.
    pub word_count: usize,
    pub field: Vec<RecordField>,
}

fn layout_list() -> &'static [RecordLayout] {
    static LAYOUT: OnceLock<Vec<RecordLayout>> = OnceLock::new();
    LAYOUT.get_or_init(|| {
        let row: Vec<Vec<&'static str>> = row_list(include_str!("../../data/record.tsv")).collect();
        let number = |row: &[&str], index: usize| row.get(index).and_then(|cell| cell.parse::<usize>().ok()).unwrap_or(0);
        let field = |name: &str| {
            let member = row.iter().filter(|row| row[0] == "member" && row.len() >= 6 && row[1] == name);
            member.map(|row| RecordField { name: row[2], kind: row[3], word: number(row, 4), word_count: number(row, 5) }).collect()
        };
        let block = row.iter().filter(|row| row[0] == "struct" && row.len() >= 3);
        block.map(|row| RecordLayout { name: row[1], word_count: number(row, 2), field: field(row[1]) }).collect()
    })
}

impl RecordLayout {
    /// The layout of the struct Cycles names so.
    pub fn of(name: &str) -> Option<&'static RecordLayout> {
        layout_list().iter().find(|layout| layout.name == name)
    }

    /// The member named so.
    pub fn field(&self, name: &str) -> Option<&RecordField> {
        let found = self.field.iter().find(|field| field.name == name);
        debug_assert!(found.is_some(), "data/record.tsv has no {}.{name}", self.name);
        found
    }
}

/// A device array of structs: the words of every element, one after
/// another.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordArray {
    layout: &'static RecordLayout,
    word: Vec<u32>,
}

impl RecordArray {
    /// An empty array of the struct Cycles names so; `None` when
    /// `data/record.tsv` has no such struct.
    pub fn of(name: &str) -> Option<RecordArray> {
        RecordLayout::of(name).map(|layout| RecordArray { layout, word: Vec::new() })
    }

    pub fn layout(&self) -> &'static RecordLayout {
        self.layout
    }

    /// How many elements it has.
    pub fn len(&self) -> usize {
        self.word.len() / self.layout.word_count.max(1)
    }

    pub fn is_empty(&self) -> bool {
        self.word.is_empty()
    }

    /// `alloc`: `count` elements, every member zero.
    pub fn alloc(&mut self, count: usize) {
        self.word.clear();
        self.word.resize(count * self.layout.word_count, 0);
    }

    /// The words, as a device takes them.
    pub fn word(&self) -> &[u32] {
        &self.word
    }

    fn place(&self, index: usize, name: &str) -> Option<std::ops::Range<usize>> {
        let field = self.layout.field(name)?;
        let start = index * self.layout.word_count + field.word;
        debug_assert!(index < self.len(), "{} has no element {index}", self.layout.name);
        (index < self.len()).then_some(start..start + field.word_count)
    }

    fn put(&mut self, index: usize, name: &str, value: &[u32]) {
        let Some(place) = self.place(index, name) else {
            return;
        };
        debug_assert!(place.len() == value.len(), "{}.{name} is {} words", self.layout.name, place.len());
        let count = place.len().min(value.len());
        self.word[place.start..place.start + count].copy_from_slice(&value[..count]);
    }

    fn take(&self, index: usize, name: &str) -> &[u32] {
        self.place(index, name).map(|place| &self.word[place]).unwrap_or(&[])
    }

    pub fn set_int(&mut self, index: usize, name: &str, value: i32) {
        self.put(index, name, &[value as u32]);
    }

    pub fn set_uint(&mut self, index: usize, name: &str, value: u32) {
        self.put(index, name, &[value]);
    }

    /// A 64-bit member: the low word first.
    pub fn set_uint64(&mut self, index: usize, name: &str, value: u64) {
        self.put(index, name, &[value as u32, (value >> 32) as u32]);
    }

    pub fn set_float(&mut self, index: usize, name: &str, value: f32) {
        self.put(index, name, &[value.to_bits()]);
    }

    /// A member of several floats, such as `color`.
    pub fn set_float_list(&mut self, index: usize, name: &str, value: &[f32]) {
        let word: Vec<u32> = value.iter().map(|part| part.to_bits()).collect();
        self.put(index, name, &word);
    }

    pub fn set_transform(&mut self, index: usize, name: &str, value: &Transform) {
        self.set_float_list(index, name, &value.word());
    }

    pub fn uint(&self, index: usize, name: &str) -> u32 {
        self.take(index, name).first().copied().unwrap_or(0)
    }

    pub fn int(&self, index: usize, name: &str) -> i32 {
        self.uint(index, name) as i32
    }

    pub fn uint64(&self, index: usize, name: &str) -> u64 {
        let word = self.take(index, name);
        let part = |at: usize| u64::from(word.get(at).copied().unwrap_or(0));
        part(0) | (part(1) << 32)
    }

    pub fn float(&self, index: usize, name: &str) -> f32 {
        f32::from_bits(self.uint(index, name))
    }

    pub fn float_list(&self, index: usize, name: &str) -> Vec<f32> {
        self.take(index, name).iter().map(|word| f32::from_bits(*word)).collect()
    }
}

#[cfg(test)]
mod test;
