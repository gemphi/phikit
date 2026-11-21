// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/session/buffers.h` and `buffers.cpp`
//! (`BufferPass`, and the pass half of `BufferParams`).

use super::BufferParam;
use crate::scene::{Pass, PassInfo, PassMode, PassType};

/// `PASS_UNUSED` (`kernel/types.h:52`): a pass with no floats in the buffer.
pub const PASS_UNUSED: i32 = -1;

/// `BufferPass`: one pass as it sits in a render buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufferPass {
    pub pass_type: PassType,
    pub mode: PassMode,
    pub name: String,
    pub include_albedo: bool,
    pub lightgroup: String,
    /// Where its floats start in a pixel, or [`PASS_UNUSED`].
    pub offset: i32,
}

impl BufferPass {
    /// `BufferPass(const Pass *scene_pass)`: not yet placed in a pixel.
    pub fn of(scene_pass: &Pass) -> BufferPass {
        BufferPass {
            pass_type: scene_pass.pass_type,
            mode: scene_pass.mode,
            name: scene_pass.name.clone(),
            include_albedo: scene_pass.include_albedo,
            lightgroup: scene_pass.lightgroup.clone(),
            offset: PASS_UNUSED,
        }
    }

    /// `get_info`.
    pub fn info(&self) -> PassInfo {
        PassInfo::of(self.pass_type, self.mode, self.include_albedo, !self.lightgroup.is_empty())
    }
}

/// `pass_type_mode_to_index`: two slots a pass type, noisy then denoised.
fn index_of(pass_type: PassType, mode: PassMode) -> usize {
    let base = pass_type.0 as usize * 2;
    match mode {
        PassMode::Noisy => base,
        PassMode::Denoised => base + 1,
    }
}

impl BufferParam {
    /// `update_passes()`: work out the stride and each type's first offset
    /// from the passes already placed.
    pub fn update_pass(&mut self) {
        self.update_offset_stride();
        self.pass_offset.fill(PASS_UNUSED);
        self.pass_stride = 0;
        for pass in &self.pass {
            let placed = pass.offset != PASS_UNUSED;
            let slot = self.pass_offset.get_mut(index_of(pass.pass_type, pass.mode)).filter(|_| placed);
            if let Some(slot) = slot {
                if *slot == PASS_UNUSED {
                    *slot = self.pass_stride;
                }
                self.pass_stride += pass.info().component_count;
            }
        }
    }

    /// `update_passes(scene_passes)`: lay the scene's passes out in a pixel,
    /// one after another, skipping those that are composed from others.
    pub fn update_pass_from(&mut self, scene_pass: &[Pass]) {
        self.pass.clear();
        let mut stride = 0;
        for asked in scene_pass {
            let mut placed = BufferPass::of(asked);
            if asked.is_written() {
                placed.offset = stride;
                stride += asked.info().component_count;
            }
            self.pass.push(placed);
        }
        self.update_pass();
    }

    /// `get_pass_offset`: [`PASS_UNUSED`] when the buffer has no such pass.
    pub fn pass_offset(&self, pass_type: PassType, mode: PassMode) -> i32 {
        match pass_type == PassType::NONE {
            true => PASS_UNUSED,
            false => self.pass_offset.get(index_of(pass_type, mode)).copied().unwrap_or(PASS_UNUSED),
        }
    }

    /// `find_pass(name)`.
    pub fn find_pass_named(&self, name: &str) -> Option<&BufferPass> {
        self.pass.iter().find(|pass| pass.name == name)
    }

    /// `find_pass(type, mode)`.
    pub fn find_pass(&self, pass_type: PassType, mode: PassMode) -> Option<&BufferPass> {
        self.pass.iter().find(|pass| pass.pass_type == pass_type && pass.mode == mode)
    }

    /// `get_actual_display_pass`: the pass to show for `pass_type`. The
    /// combined pass gives way to the shadow catcher's matte when there is one.
    pub fn actual_display_pass(&self, pass_type: PassType, mode: PassMode) -> Option<&BufferPass> {
        let pass = self.find_pass(pass_type, mode)?;
        let combined = Some(pass.pass_type) == PassType::named("PASS_COMBINED") && pass.lightgroup.is_empty();
        let matte = PassType::named("PASS_SHADOW_CATCHER_MATTE").filter(|_| combined);
        Some(matte.and_then(|matte| self.find_pass(matte, pass.mode)).unwrap_or(pass))
    }

    /// `modified`: whether a buffer made for `other` cannot be used for this.
    pub fn modified(&self, other: &BufferParam) -> bool {
        let size = (self.width, self.height) != (other.width, other.height);
        let full = (self.full_x, self.full_y, self.full_width, self.full_height)
            != (other.full_x, other.full_y, other.full_width, other.full_height);
        let window = (self.window_x, self.window_y, self.window_width, self.window_height)
            != (other.window_x, other.window_y, other.window_width, other.window_height);
        let layout = (self.offset, self.stride, self.pass_stride) != (other.offset, other.stride, other.pass_stride);
        let name = (&self.layer, &self.view) != (&other.layer, &other.view);
        let look = self.exposure != other.exposure
            || self.use_approximate_shadow_catcher != other.use_approximate_shadow_catcher
            || self.use_transparent_background != other.use_transparent_background;
        size || full || window || layout || name || look || self.pass != other.pass
    }
}
