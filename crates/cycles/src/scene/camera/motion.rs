// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/scene.h` (`Scene::MotionType`) and
//! `scene/camera.cpp` (`Camera::use_motion`, `motion_time`, `motion_step`,
//! `update_interactive_motion`): the places the camera has over the
//! shutter. For motion blur they are decomposed in `update.rs`, so that a
//! kernel can find where the camera is at any moment between them.

use super::Camera;
use crate::graph::Value;

/// `Scene::MotionType`: what a render needs motion for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MotionType {
    /// `MOTION_NONE`.
    #[default]
    None,
    /// `MOTION_PASS`: a pass of motion vectors.
    Pass,
    /// `MOTION_BLUR`.
    Blur,
    /// `MOTION_PASS_INTERACTIVE`: motion vectors in a viewport.
    PassInteractive,
}

impl MotionType {
    /// Motion vectors are wanted, not blur.
    pub fn is_pass(self) -> bool {
        self == MotionType::Pass || self == MotionType::PassInteractive
    }
}

impl Camera {
    /// `use_motion`: the camera has more than one place over the shutter.
    pub fn use_motion(&self) -> bool {
        self.node.transform_array("motion").len() > 1
    }

    /// `motion_time`: when motion step `step` is, from -1 at the shutter's
    /// open to 1 at its close.
    pub fn motion_time(&self, step: usize) -> f32 {
        let count = self.node.transform_array("motion").len();
        match count > 1 {
            true => 2.0 * step as f32 / (count - 1) as f32 - 1.0,
            false => 0.0,
        }
    }

    /// `motion_step`: the step that is at `time`, if one is.
    pub fn motion_step(&self, time: f32) -> Option<usize> {
        let count = self.node.transform_array("motion").len();
        (0..count).filter(|_| count > 1).find(|step| self.motion_time(*step) == time)
    }

    /// `update_interactive_motion`: in a viewport, once a frame is
    /// written, where the camera is and how wide it sees become where it
    /// was, for the next frame's motion vectors. A camera that moved is
    /// left modified, so its last move does not stay in the vectors.
    pub fn update_interactive_motion(&mut self) {
        let mut motion = self.node.transform_array("motion").to_vec();
        if !motion.is_empty() {
            motion[0] = self.node.transform("matrix");
            self.node.set("motion", Value::TransformArray(motion));
        }
        let fov = self.node.float("fov");
        self.node.set("fov_pre", Value::Float(fov));
    }
}

#[cfg(test)]
mod test;
