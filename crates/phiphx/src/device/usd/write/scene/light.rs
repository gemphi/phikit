//! `SceneLight` as a UsdLux light.
//!
//! ## Units
//!
//! `SceneLight::energy` is Blender's stored number (`scene/extras.rs`):
//! watts for POINT and SPOT, irradiance (W/m^2) for SUN, and 4/pi of the
//! UI watts for AREA (`AREA_ENERGY_SCALE`). Every light is written with
//! `inputs:normalize = true`, which makes UsdLux read `intensity` as the
//! light's *power* (flux in watts, or irradiance for a distant light) and
//! spread it over the shape's area, the same law Blender follows:
//! `E = P / (4 pi d^2)` for a point (`phiray/tests/lamp_power.rs`, checked
//! against Cycles) and `L = P / (pi A)` for an area light (same file). So:
//!
//! - POINT, SPOT, SUN: `intensity = energy`.
//! - AREA: `intensity = energy x pi / 4`, the UI watts, undoing the storage
//!   scale as the tracer's lamp record does (`view3d/rendered/lamp.rs`).
//!
//! A light has no exposure here (`inputs:exposure` stays 0).
use crate::device::usd::write::scene::head::UsdHead;
use crate::device::usd::write::scene::out::UsdOut;
use crate::device::usd::write::scene::text::UsdText;
use crate::scene::SceneLight;
use std::f32::consts::PI;

/// Writes one light prim.
pub(crate) struct UsdLight;

impl UsdLight {
    /// The UsdLux prim type for `light`: a SphereLight for POINT and SPOT,
    /// DistantLight for SUN, RectLight for a SQUARE or RECTANGLE area and
    /// DiskLight for a DISK or ELLIPSE one.
    pub fn kind(light: &SceneLight) -> &'static str {
        match light.kind.as_str() {
            "SUN" => "DistantLight",
            "AREA" if matches!(light.shape.as_str(), "DISK" | "ELLIPSE") => "DiskLight",
            "AREA" => "RectLight",
            _ => "SphereLight",
        }
    }

    /// `inputs:intensity` for the units above.
    pub fn intensity(light: &SceneLight) -> f32 {
        if light.kind == "AREA" { light.energy * PI / 4.0 } else { light.energy }
    }

    /// Open the light prim and write it; the caller writes any children
    /// and closes it.
    pub fn open(out: &mut UsdOut, head: &UsdHead, light: &SceneLight) {
        let spot = light.kind == "SPOT";
        let shaping = ["prepend apiSchemas = [\"ShapingAPI\"]".to_string()];
        head.open(out, Self::kind(light), if spot { &shaping } else { &[] });
        out.line(&format!("color3f inputs:color = {}", UsdText::vec3(light.color)));
        out.line(&format!("float inputs:intensity = {}", UsdText::float(Self::intensity(light))));
        out.line("bool inputs:normalize = true");
        match Self::kind(light) {
            "DistantLight" => out.line(&format!("float inputs:angle = {}", UsdText::float(light.size.to_degrees()))),
            "DiskLight" => out.line(&format!("float inputs:radius = {}", UsdText::float((light.size + light.size_y) / 4.0))),
            "RectLight" => {
                let height = if light.shape == "RECTANGLE" { light.size_y } else { light.size };
                out.line(&format!("float inputs:width = {}", UsdText::float(light.size)));
                out.line(&format!("float inputs:height = {}", UsdText::float(height)));
            }
            _ => out.line(&format!("float inputs:radius = {}", UsdText::float(light.size))),
        }
        if spot {
            // `spot_size` is the full cone; UsdLux wants the half angle in degrees.
            out.line(&format!("float inputs:shaping:cone:angle = {}", UsdText::float(light.spot_size.to_degrees() / 2.0)));
            out.line(&format!("float inputs:shaping:cone:softness = {}", UsdText::float(light.spot_blend)));
        }
    }
}
