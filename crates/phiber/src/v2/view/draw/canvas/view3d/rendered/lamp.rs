//! A file's light as the tracer takes it: one [`SceneLight`] (`phiphx`'s
//! `scene/extras.rs`) mapped onto a `phiray::Light` record. The shape
//! fields are the ones that decide how soft the lamp's shadow is
//! (`phiray`'s `shadow.wgsl`, `rendering.md` §44–46), and each kind keeps
//! its shape in a different quantity, so this is where the two
//! vocabularies meet.

use crate::phiphx::blend::AREA_ENERGY_SCALE;
use crate::phiphx::scene::SceneLight;
use glam::Mat4;
use phiray::{AREA_DISK, AREA_ELLIPSE, AREA_RECT, AREA_SQUARE, Light};

/// Kinds, as the kernels read `Light::kind`.
const SUN: u32 = 0;
const POINT: u32 = 1;
const SPOT: u32 = 2;
const AREA: u32 = 3;

/// What a Blender light is to the tracer.
pub trait BLamp {
    /// `Light::kind`.
    fn kind(&self) -> u32;
    /// `Light::shape`, an area light's `Light.area_shape`.
    fn shape(&self) -> u32;
    /// A spot's `Light::spot_scale`, `scale.z / scale.xy` of the object.
    fn spot_scale(&self, m: &Mat4) -> [f32; 2];
    /// The record the kernels read.
    fn record(&self) -> Light;
}

impl BLamp for SceneLight {
    fn kind(&self) -> u32 {
        match self.kind.as_str() {
            "SUN" => SUN,
            "SPOT" => SPOT,
            "AREA" => AREA,
            _ => POINT,
        }
    }

    fn shape(&self) -> u32 {
        match self.shape.as_str() {
            "RECTANGLE" => AREA_RECT,
            "DISK" => AREA_DISK,
            "ELLIPSE" => AREA_ELLIPSE,
            _ => AREA_SQUARE,
        }
    }

    /// EEVEE's `spot_size_inv` (`eevee_light.cc`): the object's Z scale
    /// over its X and Y, which turns the cone's cross-section into an
    /// ellipse. An unscaled spot gives `[1, 1]` and stays round.
    fn spot_scale(&self, m: &Mat4) -> [f32; 2] {
        let z = m.z_axis.truncate().length();
        [m.x_axis, m.y_axis].map(|axis| z / axis.truncate().length().max(1e-8))
    }

    fn record(&self) -> Light {
        let m = Mat4::from_cols_array_2d(&self.matrix_world);
        let kind = self.kind();
        // A sun's shape is its Angle (`Light.sun_angle`) and a point's or
        // spot's its Radius; both are the file's as of §45.
        //
        // A spot's cone is the cosine of its half angle, and its Blend is
        // how much of that cone fades, which EEVEE turns into a width in
        // cosine on the CPU (`eevee_light.cc`: `spot_blend =
        // (1 - spot_size) * spotblend`) so the kernels only smoothstep.
        //
        // Its cone follows the object's scale, so a spot squashed in one
        // axis lights an ellipse rather than a circle (`spot_scale`).
        let (cone, blend, spot_scale) = match kind == SPOT {
            true => {
                let cone = (self.spot_size * 0.5).cos();
                (cone, ((1.0 - cone) * self.spot_blend).max(1e-8), self.spot_scale(&m))
            }
            false => (0.0, 0.0, [0.0; 2]),
        };
        // An area light's shape lies in its own plane, so its axes are the
        // object's and its edges carry the object's scale, as EEVEE's do
        // (`eevee_light.cc`: `size *= scale.xy()`). A spot has no plane but
        // needs the same axes, for the ellipse; a sun and a point need
        // neither and leave `right` and `size_y` at zero.
        let area = kind == AREA;
        let (right, size, size_y) = match (area, kind == SPOT) {
            (true, _) => (
                m.x_axis.truncate().normalize_or_zero().to_array(),
                self.size * m.x_axis.truncate().length(),
                self.size_y * m.y_axis.truncate().length(),
            ),
            (_, true) => (m.x_axis.truncate().normalize_or_zero().to_array(), self.size, 0.0),
            _ => ([0.0; 3], self.size, 0.0),
        };
        // An area light's `energy` is the value the **file stores** — 4/pi
        // of the watts the UI shows, pre-multiplied so Blender's own
        // radiance comes out of `energy / (4 A)` — while the kernel wants
        // watts (`phiray/tests/lamp_power.rs` fed it 100 W by hand and
        // matched Cycles). Dividing by the same constant the driver pass
        // multiplies by on its way in (`BLightDrivers::apply`) keeps the
        // field meaning one thing on both sides of this boundary; anything
        // else a file's area light reaches the kernel 27% too bright, which
        // is what it did — 2.430 against Cycles' 1.862 — before this
        // (`tests/a_files_area_light_reaches_the_kernel_at_its_watts.rs`,
        // `rendering.md` §71).
        let energy = if area { self.energy / AREA_ENERGY_SCALE } else { self.energy };
        Light {
            pos: m.w_axis.truncate().to_array(),
            kind,
            color: self.color.map(|c| c * energy.max(0.0)),
            size,
            dir: (-m.z_axis.truncate()).normalize_or_zero().to_array(),
            cone,
            blend,
            right,
            size_y,
            shape: if area { self.shape() } else { AREA_SQUARE },
            spot_scale,
        }
    }
}
