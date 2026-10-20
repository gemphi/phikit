//! Camera / light / empty objects → `SceneCamera` / `SceneLight` /
//! `SceneEmpty`, from the DNA read by `importer::blend::object_data`.

use super::{SceneCamera, SceneEmpty, SceneLight};
use crate::blend::{BlendObjectData, BlendSceneObject};

pub(super) fn camera_from_blend(obj: &BlendSceneObject) -> SceneCamera {
    let mut cam = SceneCamera {
        name: obj.name.clone(),
        location: obj.location,
        matrix_world: obj.matrix_world,
        kind: "PERSP".into(),
        focal_length_mm: 50.0,
        sensor_width: 36.0,
        sensor_height: 24.0,
        sensor_fit: "AUTO".into(),
        shift: [0.0, 0.0],
        ortho_scale: 6.0,
        clip_start: 0.1,
        clip_end: 1000.0,
        display_size: 1.0,
        dof: Default::default(),
    };
    if let BlendObjectData::Camera(c) = &obj.data {
        cam.kind = ["PERSP", "ORTHO", "PANO"].get(c.kind as usize).unwrap_or(&"PERSP").to_string();
        cam.focal_length_mm = c.lens;
        cam.sensor_width = c.sensor_x;
        cam.sensor_height = c.sensor_y;
        cam.sensor_fit = ["AUTO", "HORIZONTAL", "VERTICAL"].get(c.sensor_fit as usize).unwrap_or(&"AUTO").to_string();
        cam.shift = c.shift;
        cam.ortho_scale = c.ortho_scale;
        cam.clip_start = c.clip_start;
        cam.clip_end = c.clip_end;
        cam.display_size = c.drawsize;
        cam.dof = c.dof.clone();
    }
    cam
}

pub(super) fn light_from_blend(obj: &BlendSceneObject) -> SceneLight {
    let mut light = SceneLight {
        name: obj.name.clone(),
        location: obj.location,
        matrix_world: obj.matrix_world,
        color: [1.0, 1.0, 1.0],
        energy: 1000.0,
        kind: "POINT".into(),
        spot_size: std::f32::consts::FRAC_PI_4,
        spot_blend: 0.15,
        size: 0.1,
        size_y: 0.1,
        shape: "SQUARE".into(),
    };
    if let BlendObjectData::Light(l) = obj.data {
        light.kind = match l.kind {
            1 => "SUN",
            2 => "SPOT",
            4 => "AREA",
            _ => "POINT",
        }
        .into();
        light.color = l.color;
        light.energy = l.energy;
        light.spot_size = l.spot_size;
        light.spot_blend = l.spot_blend;
        // A sun's shape is an angle, not a length: `Light.radius` is not
        // what Blender's Angle writes, and a sun left on `radius` drew a
        // crisp shadow where Blender's is soft.
        light.size = match l.kind {
            1 => l.sun_angle,
            4 => l.area_size,
            _ => l.radius,
        };
        // Blender reads Size Y only for the two irregular shapes; a Square
        // and a Disk take one size both ways (`eevee_light.cc`).
        light.shape = match l.area_shape {
            1 => "RECTANGLE",
            4 => "DISK",
            5 => "ELLIPSE",
            _ => "SQUARE",
        }
        .into();
        let irregular = matches!(l.area_shape, 1 | 5);
        light.size_y = if irregular { l.area_sizey } else { light.size };
    }
    light
}

/// `OB_ARROWS = 1 … OB_EMPTY_CONE = 7, OB_EMPTY_IMAGE = 8` (DNA_object_types.h).
const EMPTY_TYPES: &[&str] = &[
    "PLAIN_AXES", "ARROWS", "PLAIN_AXES", "CIRCLE", "SINGLE_ARROW", "CUBE", "SPHERE", "CONE", "IMAGE",
];

pub(super) fn empty_from_blend(obj: &BlendSceneObject) -> SceneEmpty {
    let (ty, size) = match obj.data {
        BlendObjectData::Empty(e) => (e.draw_type as usize, e.draw_size),
        _ => (2, 1.0),
    };
    SceneEmpty {
        name: obj.name.clone(),
        matrix_world: obj.matrix_world,
        display_type: EMPTY_TYPES.get(ty).unwrap_or(&"PLAIN_AXES").to_string(),
        display_size: size,
    }
}
