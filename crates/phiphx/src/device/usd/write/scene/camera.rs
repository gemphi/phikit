//! `SceneCamera` as a `UsdGeomCamera`.
//!
//! USD and Blender agree on the frame: a camera looks down its local -Z with
//! +Y up, so the object's matrix is the camera's matrix as it is. Lens and
//! film are in millimetres on both sides. USD names apertures "tenths of a
//! scene unit"; for a perspective camera only focal length over aperture
//! matters, and for an orthographic one the aperture is the view width, so
//! that is written as `ortho_scale x 10` (scene units are metres here).
use super::head::UsdHead;
use super::out::UsdOut;
use super::text::UsdText;
use crate::scene::SceneCamera;

/// Writes one camera prim.
pub(crate) struct UsdCamera;

impl UsdCamera {
    /// The film `(horizontal, vertical)` the camera exposes for an image of
    /// `aspect = width / height`, following `Camera.sensor_fit` the way
    /// `BKE_camera_sensor_size` does: AUTO spans `sensor_width` along the
    /// image's longer side; HORIZONTAL and VERTICAL pin one side.
    pub fn film(cam: &SceneCamera, aspect: f32) -> (f32, f32) {
        if cam.kind == "ORTHO" {
            let side = cam.ortho_scale * 10.0;
            return if aspect >= 1.0 { (side, side / aspect) } else { (side * aspect, side) };
        }
        match cam.sensor_fit.as_str() {
            "HORIZONTAL" => (cam.sensor_width, cam.sensor_width / aspect),
            "VERTICAL" => (cam.sensor_height * aspect, cam.sensor_height),
            _ if aspect >= 1.0 => (cam.sensor_width, cam.sensor_width / aspect),
            _ => (cam.sensor_width * aspect, cam.sensor_width),
        }
    }

    /// Open the camera prim and write its lens and film; the caller writes
    /// any children and closes it. `aspect` is the render's width over height.
    pub fn open(out: &mut UsdOut, head: &UsdHead, cam: &SceneCamera, aspect: f32) {
        head.open(out, "Camera", &[]);
        let (h, v) = Self::film(cam, aspect);
        let ortho = cam.kind == "ORTHO";
        out.line(&format!("token projection = \"{}\"", if ortho { "orthographic" } else { "perspective" }));
        out.line(&format!("float focalLength = {}", UsdText::float(cam.focal_length_mm)));
        out.line(&format!("float horizontalAperture = {}", UsdText::float(h)));
        out.line(&format!("float verticalAperture = {}", UsdText::float(v)));
        // Shift is a fraction of the fitted side (`Camera.shiftx / shifty`).
        let side = if h >= v { h } else { v };
        if cam.shift != [0.0, 0.0] {
            out.line(&format!("float horizontalApertureOffset = {}", UsdText::float(cam.shift[0] * side)));
            out.line(&format!("float verticalApertureOffset = {}", UsdText::float(cam.shift[1] * side)));
        }
        out.line(&format!("float2 clippingRange = {}", UsdText::vec2([cam.clip_start, cam.clip_end])));
        if cam.dof.on {
            out.line(&format!("float focusDistance = {}", UsdText::float(cam.dof.focus_distance)));
            out.line(&format!("float fStop = {}", UsdText::float(cam.dof.fstop)));
        }
    }
}
