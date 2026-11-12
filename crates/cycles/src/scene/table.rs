// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/tables.h` and `tables.cpp`
//! (`LookupTables`): the tables of numbers the kernel looks things up in,
//! such as the pixel filter, packed into one array of the device scene.

use crate::kernel::constant;

/// One table's place in the array.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Table {
    offset: usize,
    size: usize,
}

/// `LookupTables`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LookupTableSet {
    /// The tables, in the order of their offsets.
    table: Vec<Table>,
    need_update: bool,
}

impl LookupTableSet {
    pub fn new() -> LookupTableSet {
        LookupTableSet::default()
    }

    /// `need_update`: a table was added or removed since the device took
    /// the array.
    pub fn need_update(&self) -> bool {
        self.need_update
    }

    /// `device_update`: the device has the array.
    pub fn device_update(&mut self) {
        self.need_update = false;
    }

    /// How many tables there are.
    pub fn table_count(&self) -> usize {
        self.table.len()
    }

    /// `add_table`: copy `data` into `lookup_table`, in the first gap it
    /// fits or at the end, and return where it starts. A table takes whole
    /// chunks of `TABLE_CHUNK_SIZE` floats.
    pub fn add_table(&mut self, lookup_table: &mut Vec<f32>, data: &[f32]) -> usize {
        self.need_update = true;
        let chunk = constant("TABLE_CHUNK_SIZE").unwrap_or(1).max(1) as usize;
        let size = data.len().div_ceil(chunk) * chunk;
        // The end of the table before each gap, and the first gap wide enough.
        let start = std::iter::once(0).chain(self.table.iter().map(|table| table.offset + table.size));
        let gap = start.zip(self.table.iter().map(|table| table.offset)).position(|(start, next)| start + size <= next);
        let index = gap.unwrap_or(self.table.len());
        let offset = match index {
            0 => 0,
            index => self.table[index - 1].offset + self.table[index - 1].size,
        };
        self.table.insert(index, Table { offset, size });
        if lookup_table.len() < offset + size {
            lookup_table.resize(offset + size, 0.0);
        }
        lookup_table[offset..offset + data.len()].copy_from_slice(data);
        offset
    }

    /// `remove_table`: give the table at `offset` up. `offset` is `None`
    /// afterwards, and nothing happens when it is `None` already.
    pub fn remove_table(&mut self, offset: &mut Option<usize>) {
        let Some(at) = offset.take() else {
            return;
        };
        self.need_update = true;
        self.table.retain(|table| table.offset != at);
    }
}
