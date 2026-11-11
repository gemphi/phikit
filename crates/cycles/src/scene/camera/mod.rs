// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/camera.h` and `camera.cpp` (`Camera`,
//! `compute_auto_viewplane`, `device_update`, `device_free`,
//! `set_screen_size`): where rays start. The camera works its matrices
//! out (`update.rs`) and writes them, and its lens, into
//! `kernel_data.cam`; its places over the shutter (`motion.rs`) go into
//! the `camera_motion` array.
//!
//! The sockets are the `camera` type of `data/node.tsv`. Not ported:
//! whether it is inside a volume (`device_update_volume`), the bounds and
//! sizes adaptive subdivision asks for, and a camera written in OSL.

mod motion;
mod update;

pub use motion::MotionType;

use super::{DeviceScene, LookupTableSet};
use crate::graph::{Node, Value};
use crate::kernel::constant;
use crate::util::{BoundBox2D, DecomposedTransform, ProjectionTransform, Transform, cdf_inverted};

/// `Camera`.
#[derive(Debug, Clone, PartialEq)]
pub struct Camera {
    /// The settings, by socket name.
    pub node: Node,
    /// The size rendered, which a resolution divider makes smaller than
    /// the `full_width` and `full_height` sockets.
    pub width: i32,
    pub height: i32,
    pub screentoworld: ProjectionTransform,
    pub rastertoworld: ProjectionTransform,
    pub ndctoworld: ProjectionTransform,
    pub cameratoworld: Transform,
    pub worldtoraster: ProjectionTransform,
    pub worldtoscreen: ProjectionTransform,
    pub worldtondc: ProjectionTransform,
    pub worldtocamera: Transform,
    pub rastertocamera: ProjectionTransform,
    /// The same at the full size.
    pub full_rastertocamera: ProjectionTransform,
    /// What one pixel to the right and one down are in the world.
    pub dx: [f32; 3],
    pub dy: [f32; 3],
    pub full_dx: [f32; 3],
    pub full_dy: [f32; 3],
    /// The planes of the view frustum, for a perspective camera.
    pub frustum_right_normal: [f32; 3],
    pub frustum_top_normal: [f32; 3],
    pub frustum_left_normal: [f32; 3],
    pub frustum_bottom_normal: [f32; 3],
    /// The world to the camera, and to the raster, at the shutter's open
    /// and close: for motion vectors and for a field of view that moves.
    motion_pass_pre: Transform,
    motion_pass_post: Transform,
    perspective_pre: ProjectionTransform,
    perspective_post: ProjectionTransform,
    have_perspective_motion: bool,
    /// `kernel_camera_motion`: for motion blur, each place the camera has
    /// over the shutter, when it moves.
    kernel_camera_motion: Vec<DecomposedTransform>,
    shutter_table_offset: Option<usize>,
    pub need_device_update: bool,
    pub need_flag_update: bool,
    previous_need_motion: Option<MotionType>,
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

impl Camera {
    /// A camera as Cycles makes one: 1024 by 512, a shutter that is open
    /// all the time, and the viewplane of that size. The border sockets
    /// start as the whole view, which is the C++ class's default and not
    /// the sockets'.
    pub fn new() -> Camera {
        let mut node = Node::new("camera").expect("data/node.tsv has the camera");
        let ramp = constant("RAMP_TABLE_SIZE").unwrap_or(0) as usize;
        node.set("shutter_curve", Value::FloatArray(vec![1.0; ramp]));
        let identity = ProjectionTransform::IDENTITY;
        let mut camera = Camera {
            node,
            width: 1024,
            height: 512,
            screentoworld: identity,
            rastertoworld: identity,
            ndctoworld: identity,
            cameratoworld: Transform::IDENTITY,
            worldtoraster: identity,
            worldtoscreen: identity,
            worldtondc: identity,
            worldtocamera: Transform::IDENTITY,
            rastertocamera: identity,
            full_rastertocamera: identity,
            dx: [0.0; 3],
            dy: [0.0; 3],
            full_dx: [0.0; 3],
            full_dy: [0.0; 3],
            frustum_right_normal: [0.0; 3],
            frustum_top_normal: [0.0; 3],
            frustum_left_normal: [0.0; 3],
            frustum_bottom_normal: [0.0; 3],
            motion_pass_pre: Transform::IDENTITY,
            motion_pass_post: Transform::IDENTITY,
            perspective_pre: identity,
            perspective_post: identity,
            have_perspective_motion: false,
            kernel_camera_motion: Vec::new(),
            shutter_table_offset: None,
            need_device_update: true,
            need_flag_update: true,
            previous_need_motion: None,
        };
        camera.set_bound("border", &BoundBox2D::default());
        camera.set_bound("viewport_camera_border", &BoundBox2D::default());
        camera.compute_auto_viewplane();
        camera
    }

    /// A rectangle held in four sockets: `viewplane`, `border` or
    /// `viewport_camera_border`.
    pub fn bound(&self, name: &str) -> BoundBox2D {
        let side = |side: &str| self.node.float(&format!("{name}.{side}"));
        BoundBox2D { left: side("left"), right: side("right"), bottom: side("bottom"), top: side("top") }
    }

    pub fn set_bound(&mut self, name: &str, bound: &BoundBox2D) {
        let side = [("left", bound.left), ("right", bound.right), ("bottom", bound.bottom), ("top", bound.top)];
        side.into_iter().for_each(|(side, value)| {
            self.node.set(&format!("{name}.{side}"), Value::Float(value));
        });
    }

    /// The camera type's key in `type_enum`: `perspective`, `orthograph`,
    /// `panorama` or `custom`.
    pub fn kind(&self) -> &'static str {
        let table = self.node.node_type().enum_named("type_enum");
        table.and_then(|table| table.key(self.node.int("camera_type"))).unwrap_or("perspective")
    }

    /// `compute_auto_viewplane`: the viewplane that fits the full size, one
    /// unit to the short side. A panorama or custom camera has the unit
    /// square.
    pub fn compute_auto_viewplane(&mut self) {
        let (full_width, full_height) = (self.node.int("full_width"), self.node.int("full_height"));
        let aspect = full_width as f32 / full_height as f32;
        let viewplane = match (self.kind(), full_width >= full_height) {
            ("panorama" | "custom", _) => BoundBox2D::default(),
            (_, true) => BoundBox2D::of_radius([aspect, 1.0]),
            (_, false) => BoundBox2D::of_radius([1.0, 1.0 / aspect]),
        };
        self.set_bound("viewplane", &viewplane);
    }

    /// `set_screen_size`: `true` when the size changed.
    pub fn set_screen_size(&mut self, width: i32, height: i32) -> bool {
        let changed = (width, height) != (self.width, self.height);
        if changed {
            (self.width, self.height) = (width, height);
            self.node.tag_modified();
        }
        changed
    }

    /// `shutter_curve_eval`: how open the shutter is at `x`, from 0 to 1.
    fn shutter_curve_eval(curve: &[f32], x: f32) -> f32 {
        let Some(last) = curve.last() else {
            return 1.0;
        };
        let at = x.clamp(0.0, 1.0) * curve.len() as f32 - 1.0;
        let index = at as i32;
        match index < curve.len() as i32 - 1 {
            // Cycles reads the entry before the first here; the first is
            // what it stands for.
            true => {
                let (from, to) = (curve[index.max(0) as usize], curve[(index + 1) as usize]);
                from + (to - from) * (at - index as f32)
            }
            false => *last,
        }
    }

    /// `device_update`: bring the matrices up to date and write the camera
    /// into `kernel_data.cam`. With motion blur the shutter curve goes into
    /// the lookup tables, and a moving camera's places into `camera_motion`.
    pub fn device_update(&mut self, dscene: &mut DeviceScene, lookup: &mut LookupTableSet, need_motion: MotionType) {
        self.update(need_motion);
        if !self.need_device_update {
            return;
        }
        lookup.remove_table(&mut self.shutter_table_offset);
        if need_motion == MotionType::Blur {
            let curve = self.node.float_array("shutter_curve");
            let size = constant("SHUTTER_TABLE_SIZE").unwrap_or(0) as usize;
            let table = cdf_inverted(size, 0.0, 1.0, |x| Self::shutter_curve_eval(curve, x), false);
            let offset = lookup.add_table(&mut dscene.lookup_table, &table);
            self.shutter_table_offset = Some(offset);
            dscene.data.set_int("cam", "shutter_table_offset", offset as i32);
        }
        self.write(dscene, need_motion);
        dscene.camera_motion = self.kernel_camera_motion.clone();
        // Cycles lowers the flag after the volume pass, which is not ported.
        self.need_device_update = false;
    }

    /// `device_free`: give the shutter's table and the motion array up.
    pub fn device_free(&mut self, dscene: &mut DeviceScene, lookup: &mut LookupTableSet) {
        lookup.remove_table(&mut self.shutter_table_offset);
        dscene.camera_motion.clear();
    }
}
