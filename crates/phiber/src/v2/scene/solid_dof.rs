//! Workbench's Depth of Field (`workbench_effect_dof.cc`, `DofPass::init`):
//! Solid's `use_dof` through the camera looked through — `SceneState::
//! draw_dof` wants the camera (`RV3D_CAMOB`), its `CAM_DOF_ENABLED` and
//! the shading's flag together. The camera's part is `BCameraLens`; the
//! words the kernel takes (`phiray::Camera::dof`) are `BSolidFx`'s.

use super::BSolidFx;
use crate::phiphx::scene::SceneCamera;

/// What `DofPass::init` reads off the camera: its depth of field switch,
/// focal length and sensor (metres, `scale_camera`), f-stop, the distance
/// it focuses at (Blender units; the focus object's resolved by the
/// caller, `render_job::focus_on`) and its clip end — the depth under a
/// pixel with nothing drawn.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BCameraLens {
    pub on: bool,
    pub lens: f32,
    pub sensor: f32,
    pub fstop: f32,
    pub focus: f32,
    pub clip_end: f32,
}

impl BCameraLens {
    /// `Camera.lens`, `BKE_camera_sensor_size` (AUTO: `sensor_x`),
    /// `dof.aperture_fstop`, `dof.focus_distance`, `clip_end`.
    pub fn of(cam: &SceneCamera) -> Self {
        Self { on: cam.dof.on, lens: cam.focal_length_mm * 0.001, sensor: cam.sensor_width * 0.001, fstop: cam.dof.fstop, focus: cam.dof.focus_distance, clip_end: cam.clip_end }
    }

    /// Focused where `BKE_camera_object_dof_distance` puts it: the focus
    /// object's distance along the axis when the camera names one.
    pub fn focused_at(self, focus: Option<f32>) -> Self {
        Self { focus: focus.unwrap_or(self.focus).max(1e-5), ..self }
    }
}

impl BSolidFx {
    /// `phiray::Camera::dof`: `DofPass::init`'s `aperture_size_`
    /// (`½ · lens / fstop · |lens / (focus − lens)|`), `distance_`
    /// (`−focus`), `invsensor_size_` per frame pixel — the camera frame's
    /// share of the frame's width (`rv3d->viewcamtexcofac[0]` inverted; 1
    /// for a render) over the sensor — and `far_` (`−clip_end`). All zero
    /// — the effect off — unless Depth of Field is on, the view looks
    /// through a camera and that camera's own depth of field is on.
    pub fn dof_words(&self, lens: Option<BCameraLens>, frame_fraction: f32) -> [f32; 4] {
        let Some(l) = lens.filter(|l| l.on && self.dof) else { return [0.0; 4] };
        let aperture = 0.5 * l.lens / l.fstop.max(1e-5);
        let aperture_size = aperture * (l.lens / (l.focus - l.lens)).abs();
        [aperture_size, -l.focus, frame_fraction.max(0.0) / l.sensor.max(1e-6), -l.clip_end.max(1e-3)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn camera(on: bool) -> SceneCamera {
        let eye = [[1.0, 0.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, -1.0, 0.0, 0.0], [0.0, -10.0, 0.0, 1.0]];
        let dof = crate::phiphx::blend::BlendDof { on, fstop: 2.0, focus_distance: 5.0, ..Default::default() };
        SceneCamera { name: "Camera".into(), location: [0.0, -10.0, 0.0], matrix_world: eye, kind: "PERSP".into(), focal_length_mm: 50.0, sensor_width: 36.0, sensor_height: 24.0, sensor_fit: "AUTO".into(), shift: [0.0, 0.0],  ortho_scale: 6.0, clip_start: 0.1, clip_end: 100.0, display_size: 1.0, dof }
    }

    #[test]
    fn the_words_follow_dofpass_init_and_are_zero_unless_everything_is_on() {
        let fx = BSolidFx { dof: true, ..BSolidFx::FACTORY };
        let lens = BCameraLens::of(&camera(true));
        let w = fx.dof_words(Some(lens), 1.0);
        // ½ · 0.05 / 2 = 0.0125; × 0.05 / 4.95; 1 / 0.036.
        assert!((w[0] - 0.0125 * (0.05 / 4.95)).abs() < 1e-7, "{w:?}");
        assert_eq!((w[1], w[3]), (-5.0, -100.0));
        assert!((w[2] - 1.0 / 0.036).abs() < 1e-3);
        // The frame a quarter of the viewport wide: a quarter the pixels a sensor.
        assert!((fx.dof_words(Some(lens), 0.25)[2] - 0.25 / 0.036).abs() < 1e-3);
        // Focused on an object: its distance takes over.
        assert_eq!(fx.dof_words(Some(lens.focused_at(Some(8.0))), 1.0)[1], -8.0);
        assert_eq!(fx.dof_words(Some(lens.focused_at(None)), 1.0)[1], -5.0);
        assert_eq!(fx.dof_words(None, 1.0), [0.0; 4], "not through a camera");
        assert_eq!(fx.dof_words(Some(BCameraLens::of(&camera(false))), 1.0), [0.0; 4], "the camera's depth of field off");
        assert_eq!(BSolidFx::FACTORY.dof_words(Some(lens), 1.0), [0.0; 4], "the shading's switch off");
    }
}
