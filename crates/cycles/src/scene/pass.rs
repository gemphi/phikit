// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/kernel/types.h` (`PassType`) and
//! `intern/cycles/scene/pass.h`, `pass.cpp` (`PassMode`, `PassInfo`, `Pass`).
//! The pass names, numbers and layouts are `data/pass.tsv`, which
//! `tool/pass.py` writes from those same files.

use std::sync::OnceLock;

/// `PassType`: a pass by Cycles' own number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct PassType(pub i32);

/// `PassMode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PassMode {
    #[default]
    Noisy,
    Denoised,
}

/// `PassInfo`: what a pass holds and how it is read out.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PassInfo {
    /// `num_components`: floats a pixel.
    pub component_count: i32,
    pub use_filter: bool,
    pub use_exposure: bool,
    pub is_written: bool,
    pub scale: f32,
    /// `scale` is set from the render timer's frequency (render time only).
    pub scale_by_timer: bool,
    pub divide_type: PassType,
    pub direct_type: PassType,
    pub indirect_type: PassType,
    pub use_compositing: bool,
    pub use_denoising_albedo: bool,
    pub support_denoise: bool,
}

/// One line of `data/pass.tsv`.
struct Row {
    name: String,
    value: i32,
    /// The columns after the value; empty for an enum marker such as `PASS_NUM`.
    cell: Vec<String>,
}

fn table() -> &'static [Row] {
    static TABLE: OnceLock<Vec<Row>> = OnceLock::new();
    TABLE.get_or_init(|| include_str!("../../data/pass.tsv").lines().filter_map(row).collect())
}

fn row(line: &str) -> Option<Row> {
    let mut cell = line.split('\t');
    let name = cell.next().filter(|name| !name.is_empty() && !name.starts_with('#'))?;
    let value = cell.next()?.parse().ok()?;
    let rest: Vec<String> = cell.map(str::to_string).collect();
    let cell = match rest.len() >= 14 {
        true => rest,
        false => Vec::new(),
    };
    Some(Row { name: name.to_string(), value, cell })
}

impl PassType {
    /// `PASS_NONE`.
    pub const NONE: PassType = PassType(0);

    /// The pass Cycles names so, such as `PASS_COMBINED`.
    pub fn named(name: &str) -> Option<PassType> {
        table().iter().find(|row| row.name == name && !row.cell.is_empty()).map(|row| PassType(row.value))
    }

    /// Cycles' name for the pass.
    pub fn name(self) -> Option<&'static str> {
        table().iter().find(|row| row.value == self.0 && !row.cell.is_empty()).map(|row| row.name.as_str())
    }

    /// `PASS_NUM`: one past the largest pass number.
    pub fn count() -> usize {
        table().iter().find(|row| row.name == "PASS_NUM").map_or(0, |row| row.value as usize)
    }
}

impl PassInfo {
    /// `Pass::get_info(type, mode, include_albedo, is_lightgroup)`. A type
    /// the table does not hold has no components, as Cycles' default case.
    pub fn of(pass_type: PassType, mode: PassMode, include_albedo: bool, is_lightgroup: bool) -> PassInfo {
        let found = table().iter().find(|row| row.value == pass_type.0 && !row.cell.is_empty());
        let Some(row) = found else {
            return PassInfo::empty();
        };
        let cell = |at: usize| row.cell[at].as_str();
        let number = |at: usize| cell(at).parse::<i32>().ok();
        let flag = |at: usize| cell(at) == "1";
        let pass = |at: usize| PassType::named(cell(at)).unwrap_or(PassType::NONE);
        let base = number(0).unwrap_or(0);
        let component_count = match (is_lightgroup, mode) {
            (true, _) => number(1).unwrap_or(base),
            (false, PassMode::Denoised) => number(2).unwrap_or(base),
            (false, PassMode::Noisy) => base,
        };
        let divide_type = match (cell(7), include_albedo) {
            ("without_albedo", true) => PassType::NONE,
            _ => pass(6),
        };
        let support_denoise = match cell(12) {
            "yes" => true,
            "not_lightgroup" => !is_lightgroup,
            _ => false,
        };
        PassInfo {
            component_count,
            use_filter: flag(3),
            use_exposure: flag(4),
            is_written: flag(5),
            scale: 1.0,
            scale_by_timer: cell(13) == "timer",
            divide_type,
            direct_type: pass(8),
            indirect_type: pass(9),
            use_compositing: flag(10),
            use_denoising_albedo: flag(11),
            support_denoise,
        }
    }

    fn empty() -> PassInfo {
        PassInfo {
            component_count: 0,
            use_filter: true,
            use_exposure: false,
            is_written: true,
            scale: 1.0,
            scale_by_timer: false,
            divide_type: PassType::NONE,
            direct_type: PassType::NONE,
            indirect_type: PassType::NONE,
            use_compositing: false,
            use_denoising_albedo: true,
            support_denoise: false,
        }
    }
}

/// `Pass`: a pass as the scene asks for it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Pass {
    pub pass_type: PassType,
    pub mode: PassMode,
    pub name: String,
    pub include_albedo: bool,
    pub lightgroup: String,
    /// `is_auto_`: the film added the pass by itself, and takes it out
    /// again when it works the passes out anew.
    pub is_auto: bool,
}

impl Pass {
    pub fn new(pass_type: PassType, mode: PassMode) -> Pass {
        Pass { pass_type, mode, ..Pass::default() }
    }

    /// `get_info`.
    pub fn info(&self) -> PassInfo {
        PassInfo::of(self.pass_type, self.mode, self.include_albedo, !self.lightgroup.is_empty())
    }

    /// `is_written`: whether the pass has floats of its own in the buffer.
    pub fn is_written(&self) -> bool {
        self.info().is_written
    }
}

#[cfg(test)]
mod test;
