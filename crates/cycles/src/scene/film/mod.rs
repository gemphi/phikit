// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/film.h` and `film.cpp` (`Film`,
//! `Film::device_update`, `get_aov_offset`, `update_lightgroups`,
//! `get_kernel_features`): what the image holds. The film lays the
//! scene's passes out in a pixel and tells the kernel where each one is.
//!
//! The sockets are the `film` type of `data/node.tsv`. Which member of the
//! kernel's film data a pass type's offset goes to is `data/film.tsv`,
//! which `tool/film.py` writes from `Film::device_update`.

mod filter;
mod layout;
mod pass;

pub use filter::filter_table;
pub use layout::PassLayout;
pub use pass::{FilmPassChange, FilmPassInput};

use std::collections::HashSet;

use super::{DeviceScene, LookupTableSet, Pass, PassMode, PassType};
use crate::graph::Node;
use crate::kernel::{constant, flag};
use crate::util::row_list;

/// The rows of `data/film.tsv` of one kind.
fn table(kind: &'static str) -> impl Iterator<Item = Vec<&'static str>> {
    row_list(include_str!("../../../data/film.tsv")).filter(move |row| row.first() == Some(&kind))
}

/// A pass type by Cycles' name; none for a name the pass table lacks.
pub(super) fn pass_type(name: &str) -> PassType {
    PassType::named(name).unwrap_or(PassType::NONE)
}

/// `is_volume_guiding_pass` (`scene/pass.cpp`).
pub(super) fn is_volume_guiding_pass(pass: PassType) -> bool {
    pass == pass_type("PASS_VOLUME_SCATTER") || pass == pass_type("PASS_VOLUME_TRANSMIT")
}

/// `Film`.
#[derive(Debug, Clone, PartialEq)]
pub struct Film {
    /// The settings, by socket name.
    pub node: Node,
    /// `filter_table_offset_`: where the pixel filter is in the lookup
    /// tables.
    filter_table_offset: Option<usize>,
    prev_have_uv_pass: bool,
    prev_have_motion_pass: bool,
    prev_have_ao_pass: bool,
}

impl Default for Film {
    fn default() -> Self {
        Self::new()
    }
}

impl Film {
    /// A film with every socket at Cycles' default.
    pub fn new() -> Film {
        let node = Node::new("film").expect("data/node.tsv has the film");
        Film { node, filter_table_offset: None, prev_have_uv_pass: false, prev_have_motion_pass: false, prev_have_ao_pass: false }
    }

    /// `add_default`: the one pass a scene starts with.
    pub fn default_pass() -> Pass {
        Pass::new(pass_type("PASS_COMBINED"), PassMode::Noisy)
    }

    /// `get_display_pass`.
    pub fn display_pass(&self) -> PassType {
        PassType(self.node.int("display_pass"))
    }

    /// `device_update`: write the film's settings and the offset of every
    /// pass into `kernel_data.film`, and the pixel filter into the lookup
    /// tables. `motion_pass` is whether the scene has motion vectors
    /// (`Scene::need_motion` is `MOTION_PASS`); without them a motion pass
    /// keeps its floats and gets no offset. Nothing when no setting changed.
    pub fn device_update(&mut self, dscene: &mut DeviceScene, lookup: &mut LookupTableSet, pass: &[Pass], motion_pass: bool) {
        if !self.node.is_modified() {
            return;
        }
        lookup.remove_table(&mut self.filter_table_offset);
        let node = &self.node;
        let data = &mut dscene.data;
        data.set_float("film", "exposure", node.float("exposure"));
        data.set_float("film", "pass_alpha_threshold", node.float("pass_alpha_threshold"));
        data.set_int("film", "use_approximate_shadow_catcher", i32::from(node.boolean("use_approximate_shadow_catcher")));
        // Unused, so the kernel need not test a mask before it writes.
        let unused = constant("PASS_UNUSED").unwrap_or(-1) as i32;
        table("unused").for_each(|row| data.set_int("film", row[2], unused));

        let layout = Self::pass_layout(pass, motion_pass);
        layout.offset.iter().for_each(|(member, offset)| data.set_int("film", member, *offset));
        data.set_int("film", "pass_stride", layout.stride);
        data.set_int("film", "light_pass_flag", layout.light_pass_flag as i32);
        data.set_int("film", "pass_flag", layout.pass_flag as i32);
        data.set_int("film", "denoising_pass_flag", layout.denoising_pass_flag as i32);

        let filter = node.node_type().enum_named("filter_enum").and_then(|table| table.key(node.int("filter_type"))).unwrap_or("box");
        let offset = lookup.add_table(&mut dscene.lookup_table, &filter_table(filter, node.float("filter_width")));
        self.filter_table_offset = Some(offset);
        data.set_int("tables", "filter_table_offset", offset as i32);

        let mist_depth = node.float("mist_depth");
        let mist_inv_depth = match mist_depth > 0.0 {
            true => 1.0 / mist_depth,
            false => 0.0,
        };
        data.set_float("film", "mist_start", node.float("mist_start"));
        data.set_float("film", "mist_inv_depth", mist_inv_depth);
        data.set_float("film", "mist_falloff", node.float("mist_falloff"));
        data.set_int("film", "cryptomatte_passes", node.int("cryptomatte_passes"));
        data.set_int("film", "cryptomatte_depth", node.int("cryptomatte_depth"));

        let option = [("denoising_pass_follow_reflections", "DENOISING_PASS_FOLLOW_REFLECTIONS"), ("denoising_pass_use_albedo_roughness_weighting", "DENOISING_PASS_USE_ALBEDO_ROUGHNESS_WEIGHTING")];
        let option_flag = option.into_iter().filter(|(socket, _)| node.boolean(socket)).fold(0, |mask, (_, name)| mask | flag(name));
        data.set_int("film", "denoising_pass_options_flag", option_flag as i32);
        self.node.clear_modified();
    }

    /// `device_free`: give the pixel filter's table up.
    pub fn device_free(&mut self, lookup: &mut LookupTableSet) {
        lookup.remove_table(&mut self.filter_table_offset);
    }

    /// `get_aov_offset`: where the AOV pass named so starts among the AOVs
    /// of its kind, and whether it is a colour. `None` when there is none.
    pub fn aov_offset(pass: &[Pass], name: &str) -> Option<(i32, bool)> {
        let (value, color) = (pass_type("PASS_AOV_VALUE"), pass_type("PASS_AOV_COLOR"));
        let is_aov = |pass: &&Pass| pass.pass_type == value || pass.pass_type == color;
        let index = pass.iter().position(|pass| pass.name == name && is_aov(&pass))?;
        let kind = pass[index].pass_type;
        let before = pass[..index].iter().filter(|pass| pass.pass_type == kind);
        Some((before.map(|pass| pass.info().component_count).sum(), kind == color))
    }

    /// `update_lightgroups`: every light group a pass names, numbered in
    /// the order the passes name them.
    pub fn lightgroup_list(pass: &[Pass]) -> Vec<String> {
        let mut seen = HashSet::new();
        pass.iter().map(|pass| &pass.lightgroup).filter(|name| !name.is_empty() && seen.insert(name.as_str())).cloned().collect()
    }

    /// `get_kernel_features`: what the passes ask of the kernel.
    pub fn kernel_features(pass: &[Pass]) -> u32 {
        let between = |pass: PassType, first: &str, last: &str| pass.0 >= pass_type(first).0 && pass.0 <= pass_type(last).0;
        let feature = |pass: &Pass| {
            let denoised = pass.mode == PassMode::Denoised && !is_volume_guiding_pass(pass.pass_type);
            let rule = [
                (denoised || between(pass.pass_type, "PASS_DENOISING_ALBEDO", "PASS_DENOISING_BACKWARD_MOTION"), "KERNEL_FEATURE_DENOISING"),
                (between(pass.pass_type, "PASS_DIFFUSE", "PASS_VOLUME_TRANSMIT"), "KERNEL_FEATURE_LIGHT_PASSES"),
                (pass.pass_type == pass_type("PASS_AO"), "KERNEL_FEATURE_AO_PASS"),
            ];
            rule.into_iter().filter(|(holds, _)| *holds).fold(0, |mask, (_, name)| mask | flag(name))
        };
        pass.iter().filter(|pass| pass.is_written()).fold(0, |mask, pass| mask | feature(pass))
    }
}
