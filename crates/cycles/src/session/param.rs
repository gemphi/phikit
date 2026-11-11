// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/session/session.h` (`SessionParams`).

/// `ShadingSystem`: how shader graphs run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShadingSystem {
    /// `SHADINGSYSTEM_SVM`.
    #[default]
    Svm,
    /// `SHADINGSYSTEM_OSL`. Not ported.
    Osl,
}

/// `SessionParams`: what a session is started with. The two `DeviceInfo`
/// fields are not here: the device is wgpu, opened by phiray.
#[derive(Debug, Clone, PartialEq)]
pub struct SessionParam {
    pub headless: bool,
    pub background: bool,
    /// `samples`.
    pub sample_count: i32,
    pub use_sample_subset: bool,
    pub sample_subset_offset: i32,
    pub sample_subset_length: i32,
    pub pixel_size: i32,
    /// `threads`: zero lets the system choose.
    pub thread_count: i32,
    /// Seconds path tracing may run. Zero is no limit.
    pub time_limit: f64,
    pub use_profiling: bool,
    pub use_auto_tile: bool,
    pub tile_size: i32,
    pub use_resolution_divider: bool,
    /// `shadingsystem`.
    pub shading_system: ShadingSystem,
    /// Where in-progress tile files go.
    pub temp_dir: String,
}

impl Default for SessionParam {
    fn default() -> Self {
        Self {
            headless: false,
            background: false,
            sample_count: 1024,
            use_sample_subset: false,
            sample_subset_offset: 0,
            sample_subset_length: 1024,
            pixel_size: 1,
            thread_count: 0,
            time_limit: 0.0,
            use_profiling: false,
            use_auto_tile: true,
            tile_size: 2048,
            use_resolution_divider: true,
            shading_system: ShadingSystem::Svm,
            temp_dir: String::new(),
        }
    }
}

impl SessionParam {
    /// Whether the session has to be made again. A change an existing
    /// session can take, such as the sample count, is not one.
    pub fn modified(&self, other: &SessionParam) -> bool {
        !(self.headless == other.headless
            && self.background == other.background
            && self.pixel_size == other.pixel_size
            && self.thread_count == other.thread_count
            && self.use_profiling == other.use_profiling
            && self.use_auto_tile == other.use_auto_tile
            && self.tile_size == other.tile_size
            && self.use_resolution_divider == other.use_resolution_divider
            && self.shading_system == other.shading_system)
    }
}
