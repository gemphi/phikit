// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/film.cpp` (`Film::update_passes`,
//! `add_auto_pass`, `remove_auto_passes`, `compare_pass_order`,
//! `finalize_passes`, lines 534-803): the passes a render needs besides
//! the ones asked for, added by the film itself, and the order all of them
//! are laid out in.

use super::{Film, is_volume_guiding_pass, pass_type, table};
use crate::device::DenoiserPass;
use crate::scene::{Integrator, Pass, PassMode, PassType};

/// What `update_pass` reads from the rest of the scene. Each comes from a
/// part of the scene that is not ported yet, so it is handed in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FilmPassInput {
    /// `scene->background->get_transparent()`.
    pub background_transparent: bool,
    /// `scene->bake_manager->get_baking()`.
    pub baking: bool,
    /// `scene->bake_manager->get_use_seed()`.
    pub bake_use_seed: bool,
    /// `scene->has_shadow_catcher()`.
    pub has_shadow_catcher: bool,
    /// `scene->has_volume()`.
    pub has_volume: bool,
}

/// What changed in `update_pass` that other managers have to hear of.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FilmPassChange {
    /// There is a UV pass now, or no longer: the geometry and the shaders
    /// are to be updated.
    pub uv_pass: bool,
    /// The same for a motion pass: the geometry.
    pub motion_pass: bool,
    /// The same for an AO pass: the integrator.
    pub ao_pass: bool,
}

/// `add_auto_pass`: a pass the film adds by itself.
fn auto_pass(name: &str, mode: PassMode) -> Pass {
    Pass { is_auto: true, ..Pass::new(pass_type(name), mode) }
}

/// `add_auto_pass` with a name: the one Cycles gives the pass
/// (`data/film.tsv`), if any.
fn named_pass(name: &str, mode: PassMode) -> Pass {
    let mode_name = match mode {
        PassMode::Noisy => "noisy",
        PassMode::Denoised => "denoised",
    };
    let label = table("name").find(|row| row[1] == name && row[2] == mode_name).map(|row| row[3]).unwrap_or("");
    Pass { name: label.to_string(), ..auto_pass(name, mode) }
}

/// `Pass::contains`.
fn contains(pass: &[Pass], name: &str) -> bool {
    pass.iter().any(|pass| pass.pass_type == pass_type(name))
}

/// `finalize_passes`: no denoised pass without a denoiser, one pass for
/// each type and mode, and the order the kernel takes them in: wide passes
/// first, then those of no light group, then by type. The sort is stable,
/// so AOV and cryptomatte passes keep the order they were given in.
fn finalize(pass: Vec<Pass>, use_denoise: bool) -> Vec<Pass> {
    let mut kept: Vec<Pass> = Vec::new();
    for mut pass in pass {
        let need_denoise = pass.info().support_denoise && (use_denoise || is_volume_guiding_pass(pass.pass_type));
        if !need_denoise {
            pass.mode = PassMode::Noisy;
        }
        // Two passes of one type and mode are one, unless both have a name
        // and the names differ. The name either has is kept.
        let same = |other: &&mut Pass| other.pass_type == pass.pass_type && other.mode == pass.mode && (pass.name.is_empty() || other.name.is_empty() || other.name == pass.name);
        match kept.iter_mut().find(same) {
            Some(other) => {
                if other.name.is_empty() {
                    other.name = pass.name.clone();
                }
                other.is_auto &= pass.is_auto;
            }
            None => kept.push(pass),
        }
    }
    kept.sort_by_key(|pass| (-pass.info().component_count, !pass.lightgroup.is_empty(), pass.pass_type.0));
    kept
}

impl Film {
    /// `update_passes`: take the passes the film added before out, add the
    /// ones this render needs, and put all of them in order.
    pub fn update_pass(&mut self, pass: &mut Vec<Pass>, integrator: &Integrator, input: &FilmPassInput) -> FilmPassChange {
        // `remove_auto_passes`.
        pass.retain(|pass| !pass.is_auto);
        let mut add = |name: &str, mode: PassMode| pass.push(auto_pass(name, mode));

        // The pass shown, and the combined pass: adaptive sampling is
        // always judged on it.
        let display = self.display_pass();
        add(display.name().unwrap_or("PASS_COMBINED"), PassMode::Noisy);
        if display != pass_type("PASS_COMBINED") {
            add("PASS_COMBINED", PassMode::Noisy);
        }
        if integrator.adaptive_sampling().enabled {
            add("PASS_SAMPLE_COUNT", PassMode::Noisy);
            add("PASS_ADAPTIVE_AUX_BUFFER", PassMode::Noisy);
        }
        let use_denoise = integrator.node.boolean("use_denoise");
        let denoiser_pass = integrator.node.int("denoiser_passes");
        let guide = [
            (DenoiserPass::ALBEDO, "PASS_DENOISING_ALBEDO"),
            (DenoiserPass::SPECULAR_ALBEDO, "PASS_DENOISING_SPECULAR_ALBEDO"),
            (DenoiserPass::NORMAL, "PASS_DENOISING_NORMAL"),
            (DenoiserPass::ROUGHNESS, "PASS_DENOISING_ROUGHNESS"),
            (DenoiserPass::DEPTH, "PASS_DENOISING_DEPTH"),
            (DenoiserPass::MOTION, "PASS_MOTION"),
            (DenoiserPass::BACKWARD_MOTION, "PASS_DENOISING_BACKWARD_MOTION"),
        ];
        guide.into_iter().filter(|(bit, _)| use_denoise && denoiser_pass & bit != 0).for_each(|(_, name)| add(name, PassMode::Noisy));

        self.add_shadow_catcher_pass(pass, input);
        // What a pass is composed from, and its denoised twin: with every
        // denoised pass stored, the denoiser can be tuned afterwards.
        for asked in pass.clone() {
            let info = asked.info();
            let part = [info.divide_type, info.direct_type, info.indirect_type];
            part.into_iter().filter(|part| *part != PassType::NONE).for_each(|part| pass.push(auto_pass(part.name().unwrap_or(""), PassMode::Noisy)));
            if info.support_denoise && use_denoise {
                pass.push(auto_pass(asked.pass_type.name().unwrap_or(""), PassMode::Denoised));
            }
        }
        self.add_scene_pass(pass, input);
        *pass = finalize(std::mem::take(pass), use_denoise);

        let have = [contains(pass, "PASS_UV"), contains(pass, "PASS_MOTION") || contains(pass, "PASS_DENOISING_BACKWARD_MOTION"), contains(pass, "PASS_AO")];
        let change = FilmPassChange { uv_pass: have[0] != self.prev_have_uv_pass, motion_pass: have[1] != self.prev_have_motion_pass, ao_pass: have[2] != self.prev_have_ao_pass };
        [self.prev_have_uv_pass, self.prev_have_motion_pass, self.prev_have_ao_pass] = have;
        self.node.tag_modified();
        change
    }

    /// The passes a shadow catcher needs: with one in the scene all three,
    /// and the background when the approximate catcher has to put it back;
    /// with only the pass asked for, its sample count.
    fn add_shadow_catcher_pass(&self, pass: &mut Vec<Pass>, input: &FilmPassInput) {
        let need_background = self.node.boolean("use_approximate_shadow_catcher") && !input.background_transparent;
        let name: &[&str] = match (input.has_shadow_catcher, need_background, contains(pass, "PASS_SHADOW_CATCHER")) {
            (true, true, _) => &["PASS_SHADOW_CATCHER", "PASS_SHADOW_CATCHER_SAMPLE_COUNT", "PASS_SHADOW_CATCHER_MATTE", "PASS_BACKGROUND"],
            (true, false, _) => &["PASS_SHADOW_CATCHER", "PASS_SHADOW_CATCHER_SAMPLE_COUNT", "PASS_SHADOW_CATCHER_MATTE"],
            (false, _, true) => &["PASS_SHADOW_CATCHER", "PASS_SHADOW_CATCHER_SAMPLE_COUNT"],
            (false, _, false) => &[],
        };
        name.iter().for_each(|name| pass.push(auto_pass(name, PassMode::Noisy)));
    }

    /// The passes baking, tiled rendering and volumes need.
    fn add_scene_pass(&self, pass: &mut Vec<Pass>, input: &FilmPassInput) {
        let mut add = |wanted: bool, name: &str, mode: PassMode| {
            if wanted {
                pass.push(named_pass(name, mode));
            }
        };
        add(input.baking, "PASS_BAKE_PRIMITIVE", PassMode::Noisy);
        add(input.baking, "PASS_BAKE_DIFFERENTIAL", PassMode::Noisy);
        add(input.baking && input.bake_use_seed, "PASS_BAKE_SEED", PassMode::Noisy);
        // A sample count pass for tiled rendering, and for volumes.
        let no_sample_count = !contains(pass, "PASS_SAMPLE_COUNT");
        if self.node.boolean("use_sample_count") && no_sample_count {
            pass.push(auto_pass("PASS_SAMPLE_COUNT", PassMode::Noisy));
        }
        if !input.has_volume {
            return;
        }
        let no_sample_count = !contains(pass, "PASS_SAMPLE_COUNT");
        let no_majorant = !contains(pass, "PASS_VOLUME_MAJORANT");
        let volume = [
            (true, "PASS_VOLUME_SCATTER", PassMode::Noisy),
            (true, "PASS_VOLUME_SCATTER", PassMode::Denoised),
            (true, "PASS_VOLUME_TRANSMIT", PassMode::Noisy),
            (true, "PASS_VOLUME_TRANSMIT", PassMode::Denoised),
            (no_sample_count, "PASS_SAMPLE_COUNT", PassMode::Noisy),
            (no_majorant, "PASS_VOLUME_MAJORANT", PassMode::Noisy),
            (true, "PASS_VOLUME_MAJORANT_SAMPLE_COUNT", PassMode::Noisy),
        ];
        volume.into_iter().filter(|(wanted, _, _)| *wanted).for_each(|(_, name, mode)| pass.push(named_pass(name, mode)));
    }
}

#[cfg(test)]
mod test;
