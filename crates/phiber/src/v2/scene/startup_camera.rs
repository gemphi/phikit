//! The camera a render looks through when no file is open — Blender's
//! startup file has one (`startup.blend`: a 50 mm camera at (7.36, −6.93,
//! 4.96) looking down at the cube), so F12 on a fresh session renders.
//! Phiber's default scene is three primitives without a camera; this
//! stands the same camera on the same line of sight, pulled back to frame
//! them, so Render Image, Render Animation and the Rendering workspace
//! have something to render before a file is opened.

use super::{BObject3D, view_op::bounds_of};
use crate::phiphx::blend::BlendDof;
use crate::phiphx::scene::{SceneCamera, SceneModel};
use glam::Vec3;

/// The startup file's camera, standing where it frames the default scene.
pub struct BStartupCamera;

impl BStartupCamera {
    /// `Camera` in `startup.blend`: where it stands, as a direction from the
    /// cube it looks at.
    pub const DIRECTION: [f32; 3] = [7.358_9, -6.925_8, 4.958_3];
    pub const NAME: &'static str = "Camera";
    /// Its lens and sensor (`DNA_camera_defaults.h`), its clipping.
    pub const LENS_MM: f32 = 50.0;
    pub const SENSOR_MM: f32 = 36.0;
    pub const CLIP: (f32, f32) = (0.1, 1000.0);
    /// Room left around what is framed.
    const MARGIN: f32 = 1.1;

    /// The camera framing `objects` for a render `aspect` wide per high:
    /// their bounding sphere fitted to the tighter field of view.
    pub fn camera(objects: &[BObject3D], aspect: f32) -> SceneCamera {
        let all: Vec<&BObject3D> = objects.iter().collect();
        let (centre, extent) = bounds_of(&all).unwrap_or((Vec3::ZERO, 2.0));
        // The sphere around the box, and the half field of view the sensor
        // gives across — `BKE_camera_sensor_fit` AUTO, the width — and up.
        let radius = extent * 0.5 * 3f32.sqrt();
        let half_across = (Self::SENSOR_MM * 0.5 / Self::LENS_MM).atan();
        let half_up = (Self::SENSOR_MM * 0.5 / Self::LENS_MM / aspect.max(1e-3)).atan();
        let distance = radius * Self::MARGIN / half_across.min(half_up).sin().max(1e-3);
        let back = Vec3::from(Self::DIRECTION).normalize();
        let right = Vec3::Z.cross(back).normalize();
        let up = back.cross(right);
        let eye = centre + back * distance;
        let col = |v: Vec3| [v.x, v.y, v.z, 0.0];
        SceneCamera {
            name: Self::NAME.into(),
            location: eye.to_array(),
            matrix_world: [col(right), col(up), col(back), [eye.x, eye.y, eye.z, 1.0]],
            kind: "PERSP".into(),
            focal_length_mm: Self::LENS_MM,
            sensor_width: Self::SENSOR_MM,
            ortho_scale: 6.0,
            clip_start: Self::CLIP.0,
            clip_end: Self::CLIP.1,
            display_size: 1.0,
            dof: BlendDof::default(),
        }
    }

    /// A model with that camera alone — what a render plan reads when no
    /// file is open: the camera, named as the scene's.
    pub fn model(objects: &[BObject3D]) -> SceneModel {
        let mut model = SceneModel::default();
        let (w, h) = super::render_job::render_size(&model);
        model.cameras.push(Self::camera(objects, w as f32 / h.max(1) as f32));
        model.render.camera = Self::NAME.into();
        model
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_startup_camera_frames_the_default_scene_from_blenders_corner() {
        let objects = super::super::current_scene(&crate::v2::state::BState::new(), None);
        let model = BStartupCamera::model(&objects);
        let cam = super::super::render_job::active_camera(&model).expect("the scene's camera");
        assert_eq!(cam.name, "Camera");
        // Above, in front and to the right, as the startup file's.
        assert!(cam.location[0] > 0.0 && cam.location[1] < 0.0 && cam.location[2] > 0.0, "{:?}", cam.location);
        let eye = Vec3::from(cam.location);
        let back = Vec3::new(cam.matrix_world[2][0], cam.matrix_world[2][1], cam.matrix_world[2][2]);
        assert!((back - Vec3::from(BStartupCamera::DIRECTION).normalize()).length() < 1e-5, "looking along the startup camera's line");
        // Every vertex of the scene lies within the tighter field of view.
        let all: Vec<&BObject3D> = objects.iter().collect();
        let (centre, extent) = bounds_of(&all).unwrap();
        let depth = (centre - eye).dot(-back);
        let half_up = (18.0_f32 / 50.0 / (1920.0 / 1080.0)).atan();
        assert!(extent * 0.5 * 3f32.sqrt() <= depth * half_up.tan() * 1.0001, "{extent} at {depth}");
        assert!((cam.focal_length_mm, cam.sensor_width, cam.clip_end) == (50.0, 36.0, 1000.0));
    }
}
