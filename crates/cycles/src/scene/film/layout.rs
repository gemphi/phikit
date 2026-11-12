// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/film.cpp` (`Film::device_update`,
//! lines 221-454): where each pass starts in a pixel, and which passes
//! there are, as the kernel is told. The member a pass type's offset goes
//! to is `data/film.tsv`.

use std::collections::HashSet;

use super::{Film, pass_type, table};
use crate::kernel::constant;
use crate::scene::{Pass, PassMode, PassType};

/// The passes laid out in a pixel.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PassLayout {
    /// The members of the kernel's film data that get an offset.
    pub offset: Vec<(String, i32)>,
    /// `pass_stride`: floats a pixel.
    pub stride: i32,
    /// A bit for each light pass there is, by its number modulo 32.
    pub light_pass_flag: u32,
    /// The same for the data passes.
    pub pass_flag: u32,
    /// The same for the denoising passes.
    pub denoising_pass_flag: u32,
}

impl PassLayout {
    /// Give `member` the offset. When it has one already, the lowest
    /// stays with `keep_lowest`, else the new one replaces it.
    fn put(&mut self, member: &str, offset: i32, keep_lowest: bool) {
        match self.offset.iter_mut().find(|(name, _)| name == member) {
            Some((_, kept)) if keep_lowest => *kept = offset.min(*kept),
            Some((_, kept)) => *kept = offset,
            None => self.offset.push((member.to_string(), offset)),
        }
    }

    /// Count the pass type in the flags of its category.
    fn count(&mut self, pass: PassType) {
        let bit = 1u32 << (pass.0 % 32);
        let end = |name: &str| constant(name).unwrap_or(0) as i32;
        let flag = [("PASS_CATEGORY_LIGHT_END", &mut self.light_pass_flag), ("PASS_CATEGORY_DATA_END", &mut self.pass_flag), ("PASS_CATEGORY_DENOISING_END", &mut self.denoising_pass_flag)];
        if let Some((_, flag)) = flag.into_iter().find(|(name, _)| pass.0 <= end(name)) {
            *flag |= bit;
        }
    }
}

impl Film {
    /// Lay `pass` out, in its order. A pass that is not written takes no
    /// floats. Every other one does, but not every one gets an offset:
    /// a denoised pass has none (the kernel writes noisy passes; only the
    /// two volume guiding passes are read back denoised), and a motion
    /// pass has none without motion vectors.
    pub fn pass_layout(pass: &[Pass], motion_pass: bool) -> PassLayout {
        let motion = ["PASS_MOTION", "PASS_MOTION_WEIGHT", "PASS_DENOISING_BACKWARD_MOTION"].map(pass_type);
        let mut layout = PassLayout::default();
        let mut first_done = HashSet::new();
        for pass in pass.iter().filter(|pass| pass.pass_type != PassType::NONE && pass.is_written()) {
            let offset = layout.stride;
            layout.stride += pass.info().component_count;
            let name = pass.pass_type.name().unwrap_or("").to_lowercase();
            let member = table("offset").find(|row| pass_type(row[1]) == pass.pass_type);
            match (pass.mode, motion.contains(&pass.pass_type) && !motion_pass, pass.lightgroup.is_empty(), member) {
                (PassMode::Denoised, _, _, _) => {
                    if super::is_volume_guiding_pass(pass.pass_type) {
                        layout.put(&format!("{name}_denoised"), offset, false);
                    }
                }
                (PassMode::Noisy, true, _, _) => {}
                // The passes of all light groups follow one another, so
                // the kernel is told where the first is.
                (PassMode::Noisy, false, false, _) => {
                    layout.count(pass.pass_type);
                    layout.put("pass_lightgroup", offset, true);
                }
                (PassMode::Noisy, false, true, Some(row)) => {
                    layout.count(pass.pass_type);
                    // Of several AOVs the kernel is told the first.
                    if row[3] != "first" || first_done.insert(row[2]) {
                        layout.put(row[2], offset, row[3] == "lowest");
                    }
                }
                (PassMode::Noisy, false, true, None) => layout.count(pass.pass_type),
            }
        }
        layout
    }
}

#[cfg(test)]
mod test;
