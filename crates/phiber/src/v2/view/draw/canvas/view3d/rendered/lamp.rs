//! A file's light as the tracer takes it: one [`SceneLight`] (`phiphx`'s
//! `scene/extras.rs`) mapped onto a `phiray::Light` record. The shape
//! fields are the ones that decide how soft the lamp's shadow is
//! (`phiray`'s `shadow.wgsl`, `rendering.md` §44–46), and each kind keeps
//! its shape in a different quantity, so this is where the two
//! vocabularies meet.

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

    fn record(&self) -> Light {
        let m = Mat4::from_cols_array_2d(&self.matrix_world);
        let kind = self.kind();
        // A sun's shape is its Angle (`Light.sun_angle`) and a point's or
        // spot's its Radius; both are the file's as of §45.
        let cone = if kind == SPOT { (self.spot_size * 0.5).cos() } else { 0.0 };
        // An area light's shape lies in its own plane, so its axes are the
        // object's and its edges carry the object's scale, as EEVEE's do
        // (`eevee_light.cc`: `size *= scale.xy()`). The other kinds have no
        // plane and leave `right` and `size_y` at zero.
        let area = kind == AREA;
        let (right, size, size_y) = match area {
            true => (
                m.x_axis.truncate().normalize_or_zero().to_array(),
                self.size * m.x_axis.truncate().length(),
                self.size_y * m.y_axis.truncate().length(),
            ),
            false => ([0.0; 3], self.size, 0.0),
        };
        Light {
            pos: m.w_axis.truncate().to_array(),
            kind,
            color: self.color.map(|c| c * self.energy.max(0.0)),
            size,
            dir: (-m.z_axis.truncate()).normalize_or_zero().to_array(),
            cone,
            right,
            size_y,
            shape: if area { self.shape() } else { AREA_SQUARE },
            _pad: [0; 3],
        }
    }
}
