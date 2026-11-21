// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/camera.cpp` (`Camera::update`,
//! lines 214-507): the camera's matrices, from the raster through the
//! screen and the camera to the world and back, and what of the camera the
//! kernel is given.

use super::{Camera, MotionType};
use crate::scene::DeviceScene;
use crate::util::{ProjectionTransform, Transform, motion_decompose};

fn subtract(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn normalize(vector: [f32; 3]) -> [f32; 3] {
    let length = vector.iter().map(|part| part * part).sum::<f32>().sqrt();
    match length == 0.0 {
        true => vector,
        false => vector.map(|part| part / length),
    }
}

/// What one pixel across (`step`) is in camera space.
fn differential(kind: &str, rastertocamera: &ProjectionTransform, step: [f32; 3]) -> [f32; 3] {
    match kind {
        "orthograph" => rastertocamera.direction(step),
        "perspective" => subtract(rastertocamera.point(step), rastertocamera.point([0.0; 3])),
        _ => [0.0; 3],
    }
}

impl Camera {
    /// The projection of the camera's type for a field of view.
    fn cameratoscreen(&self, fov: f32) -> ProjectionTransform {
        let (near, far) = (self.node.float("nearclip"), self.node.float("farclip"));
        match self.kind() {
            "perspective" => ProjectionTransform::perspective(fov, near, far),
            "orthograph" => ProjectionTransform::orthographic(near, far),
            _ => ProjectionTransform::IDENTITY,
        }
    }

    /// `update`: work the matrices out again when a socket changed. A
    /// change of what motion is needed for asks for a device update too.
    pub fn update(&mut self, need_motion: MotionType) {
        if self.previous_need_motion != Some(need_motion) {
            self.need_device_update = true;
        }
        if !self.node.is_modified() {
            return;
        }
        let node = &self.node;
        let kind = self.kind();
        let (full_width, full_height) = (node.int("full_width") as f32, node.int("full_height") as f32);
        let matrix = node.transform("matrix");
        let fov = node.float("fov");

        // The full viewport to the camera's border in it, NDC to raster,
        // and the screen to NDC.
        let fulltoborder = Transform::from_viewplane(&self.bound("viewport_camera_border"));
        let bordertofull = fulltoborder.inverse();
        let ndctoraster = Transform::scale(self.width as f32, self.height as f32, 1.0) * bordertofull;
        let full_ndctoraster = Transform::scale(full_width, full_height, 1.0) * bordertofull;
        let screentondc = fulltoborder * Transform::from_viewplane(&self.bound("viewplane"));
        let screentoraster = ndctoraster * screentondc;
        let rastertoscreen = screentoraster.inverse();
        let full_rastertoscreen = (full_ndctoraster * screentondc).inverse();

        let cameratoscreen = self.cameratoscreen(fov);
        let screentocamera = cameratoscreen.inverse();
        let rastertocamera = screentocamera * rastertoscreen;
        let full_rastertocamera = screentocamera * full_rastertoscreen;
        let rastertoworld = matrix * rastertocamera;
        // Built up again, not inverted from the above: inverting matrices
        // that are nearly singular loses precision in a large scene.
        let worldtocamera = matrix.inverse();
        let worldtoscreen = cameratoscreen * worldtocamera;
        let worldtondc = screentondc * worldtoscreen;
        let worldtoraster = ndctoraster * worldtondc;

        // The frustum's planes, from the corners of the full raster.
        let far_corner = full_rastertocamera.point([full_width, full_height, 0.0]);
        let near_corner = full_rastertocamera.point([0.0; 3]);
        let frustum = match kind {
            "perspective" => [
                normalize([far_corner[2], 0.0, -far_corner[0]]),
                normalize([0.0, far_corner[2], -far_corner[1]]),
                normalize([-near_corner[2], 0.0, near_corner[0]]),
                normalize([0.0, -near_corner[2], near_corner[1]]),
            ],
            _ => [self.frustum_right_normal, self.frustum_top_normal, self.frustum_left_normal, self.frustum_bottom_normal],
        };

        // Motion: where the camera, and its field of view, were at the
        // shutter's open and close.
        let motion = node.transform_array("motion");
        let have_motion = motion.iter().any(|step| *step != matrix);
        let (first, last) = match (have_motion, motion.first(), motion.last()) {
            (true, Some(first), Some(last)) => (first.inverse(), last.inverse()),
            _ => (worldtocamera, worldtocamera),
        };
        // For blur, a camera that moves has every place decomposed.
        let camera_motion = match need_motion == MotionType::Blur && have_motion {
            true => motion_decompose(motion),
            false => Vec::new(),
        };
        let (fov_pre, fov_post) = (node.float("fov_pre"), node.float("fov_post"));
        let flat = kind != "panorama" && kind != "custom";
        let moving = have_motion || fov != fov_pre || fov != fov_post;
        let blur_perspective = need_motion == MotionType::Blur && node.boolean("use_perspective_motion") && kind == "perspective";
        let (perspective_pre, perspective_post) = match (need_motion.is_pass() && flat, moving, blur_perspective) {
            // For motion vectors: the world to the raster at each end.
            (true, true, _) => (screentoraster * self.cameratoscreen(fov_pre) * first, screentoraster * self.cameratoscreen(fov_post) * last),
            (true, false, _) => (worldtoraster, worldtoraster),
            // For blur: the raster to the camera at each end.
            (false, _, true) => (self.cameratoscreen(fov_pre).inverse() * rastertoscreen, self.cameratoscreen(fov_post).inverse() * rastertoscreen),
            (false, _, false) => (self.perspective_pre, self.perspective_post),
        };

        self.dx = matrix.direction(differential(kind, &rastertocamera, [1.0, 0.0, 0.0]));
        self.dy = matrix.direction(differential(kind, &rastertocamera, [0.0, 1.0, 0.0]));
        self.full_dx = matrix.direction(differential(kind, &full_rastertocamera, [1.0, 0.0, 0.0]));
        self.full_dy = matrix.direction(differential(kind, &full_rastertocamera, [0.0, 1.0, 0.0]));
        [self.frustum_right_normal, self.frustum_top_normal, self.frustum_left_normal, self.frustum_bottom_normal] = frustum;
        self.cameratoworld = matrix;
        self.screentoworld = matrix * screentocamera;
        self.rastertoworld = rastertoworld;
        self.ndctoworld = rastertoworld * ndctoraster;
        self.worldtocamera = worldtocamera;
        self.worldtoscreen = worldtoscreen;
        self.worldtondc = worldtondc;
        self.worldtoraster = worldtoraster;
        self.rastertocamera = rastertocamera;
        self.full_rastertocamera = full_rastertocamera;
        // Only motion vectors read these; otherwise they stay as they were.
        if need_motion.is_pass() {
            (self.motion_pass_pre, self.motion_pass_post) = (first, last);
        }
        (self.perspective_pre, self.perspective_post) = (perspective_pre, perspective_post);
        self.have_perspective_motion = blur_perspective;
        self.kernel_camera_motion = camera_motion;

        self.node.clear_modified();
        self.need_device_update = true;
        self.need_flag_update = true;
        self.previous_need_motion = Some(need_motion);
    }

    /// The kernel's camera, from the matrices and the sockets: the second
    /// half of `update` and `dscene->data.cam = kernel_camera`.
    pub(super) fn write(&self, dscene: &mut DeviceScene, need_motion: MotionType) {
        let node = &self.node;
        let data = &mut dscene.data;
        let projection = [
            ("screentoworld", self.screentoworld),
            ("rastertoworld", self.rastertoworld),
            ("rastertocamera", self.rastertocamera),
            ("worldtoscreen", self.worldtoscreen),
            ("worldtoraster", self.worldtoraster),
            ("worldtondc", self.worldtondc),
            ("ndctoworld", self.ndctoworld),
            ("perspective_pre", self.perspective_pre),
            ("perspective_post", self.perspective_post),
        ];
        projection.iter().for_each(|(name, value)| data.set_projection("cam", name, value));
        let transform = [("cameratoworld", self.cameratoworld), ("worldtocamera", self.worldtocamera), ("motion_pass_pre", self.motion_pass_pre), ("motion_pass_post", self.motion_pass_post)];
        transform.iter().for_each(|(name, value)| data.set_transform("cam", name, value));
        data.set_int("cam", "num_motion_steps", self.kernel_camera_motion.len() as i32);
        data.set_int("cam", "have_perspective_motion", i32::from(self.have_perspective_motion));

        let blades = node.int("blades");
        let shuttertime = match need_motion == MotionType::Blur {
            true => node.float("shuttertime"),
            false => -1.0,
        };
        let interocular = node.float("interocular_distance") * 0.5;
        let eye = node.node_type().enum_named("stereo_eye_enum").and_then(|table| table.key(node.int("stereo_eye")));
        let interocular_offset = match eye {
            Some("left") => -interocular,
            Some("right") => interocular,
            _ => 0.0,
        };
        let pole_merge = match node.boolean("use_pole_merge") {
            true => [node.float("pole_merge_angle_from"), node.float("pole_merge_angle_to")],
            false => [-1.0; 2],
        };
        let (near, far) = (node.float("nearclip"), node.float("farclip"));
        let cliplength = match far == f32::MAX {
            true => f32::MAX,
            false => far - near,
        };
        let (width, height) = (self.width as f32, self.height as f32);
        let float = [
            ("aperturesize", node.float("aperturesize")),
            ("focaldistance", node.float("focaldistance").max(1e-5)),
            ("blades", f32::from(u8::from(blades >= 3)) * blades as f32),
            ("bladesrotation", node.float("bladesrotation")),
            ("shuttertime", shuttertime),
            ("inv_aperture_ratio", 1.0 / node.float("aperture_ratio")),
            ("fisheye_fov", node.float("fisheye_fov")),
            ("fisheye_lens", node.float("fisheye_lens")),
            ("fisheye_lens_polynomial_bias", node.float("fisheye_polynomial_k0")),
            ("interocular_offset", interocular_offset),
            ("convergence_distance", node.float("convergence_distance")),
            ("pole_merge_angle_from", pole_merge[0]),
            ("pole_merge_angle_to", pole_merge[1]),
            ("sensorwidth", node.float("sensorwidth")),
            ("sensorheight", node.float("sensorheight")),
            ("width", width),
            ("height", height),
            // So the same texture tiles are used as at the full size.
            ("differential_scale", 0.5 * (width / node.int("full_width") as f32 + height / node.int("full_height") as f32)),
            ("nearclip", near),
            ("cliplength", cliplength),
            ("rolling_shutter_duration", node.float("rolling_shutter_duration")),
        ];
        float.iter().for_each(|(name, value)| data.set_float("cam", name, *value));
        let int = [("motion_position", "motion_position"), ("type", "camera_type"), ("panorama_type", "panorama_type"), ("rolling_shutter_type", "rolling_shutter_type")];
        int.iter().for_each(|(name, socket)| data.set_int("cam", name, node.int(socket)));

        let (longitude_min, latitude_min) = (node.float("longitude_min"), node.float("latitude_min"));
        let equirectangular = [longitude_min - node.float("longitude_max"), -longitude_min, latitude_min - node.float("latitude_max"), -latitude_min + std::f32::consts::FRAC_PI_2];
        let polynomial = ["fisheye_polynomial_k1", "fisheye_polynomial_k2", "fisheye_polynomial_k3", "fisheye_polynomial_k4"].map(|socket| node.float(socket));
        let cylindrical = [-node.float("central_cylindrical_range_u_min"), -node.float("central_cylindrical_range_u_max"), node.float("central_cylindrical_range_v_min"), node.float("central_cylindrical_range_v_max")];
        data.set_float4("cam", "equirectangular_range", equirectangular);
        data.set_float4("cam", "fisheye_lens_polynomial_coefficients", polynomial);
        data.set_float4("cam", "central_cylindrical_range", cylindrical);
        data.set_float4("cam", "dx", [self.dx[0], self.dx[1], self.dx[2], 0.0]);
        data.set_float4("cam", "dy", [self.dy[0], self.dy[1], self.dy[2], 0.0]);
    }
}
