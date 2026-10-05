//! Non-mesh object data Blender's overlay engine draws: `Camera`
//! (`DNA_camera_types.h`), `Light` (`DNA_light_types.h`) and the empty
//! display fields on `Object` (`empty_drawtype`, `empty_drawsize`).

use crate::blend::file::types::*;

/// `Camera` display data. `kind`: 0 = PERSP, 1 = ORTHO, 2 = PANO.
#[derive(Debug, Clone, PartialEq)]
pub struct BlendCameraData {
    pub kind: u8,
    pub lens: f32,
    pub ortho_scale: f32,
    pub sensor_x: f32,
    pub clip_start: f32,
    pub clip_end: f32,
    pub drawsize: f32,
    /// `Camera.dof` (`object/dof.rs`). Not `Copy`, because a camera can name
    /// the object it focuses on.
    pub dof: super::dof::BlendDof,
}

/// `Light` data. `kind`: 0 = POINT (LA_LOCAL), 1 = SUN, 2 = SPOT, 4 = AREA.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlendLightData {
    pub kind: u8,
    pub color: [f32; 3],
    pub energy: f32,
    pub spot_size: f32,
    /// `Light.spotblend`: how much of the cone fades, 0 ..= 1. Blender's
    /// default is 0.15, which both kernels had hard-coded before it was read.
    pub spot_blend: f32,
    pub radius: f32,
    pub area_size: f32,
    /// `Light.area_sizey`: an area light's edge across its own Y. Blender
    /// reads it only for a Rectangle and an Ellipse (`eevee_light.cc`).
    pub area_sizey: f32,
    /// `Light.area_shape` (`eLightAreaShape`): 0 square, 1 rectangle,
    /// 4 disk, 5 ellipse.
    pub area_shape: u8,
    /// `Light.sun_angle`: a sun's angular diameter in radians. Blender's
    /// default is the real sun's, 0.526°.
    pub sun_angle: f32,
}

/// `Light.sun_angle`'s default: the real sun's angular diameter, 0.526°.
pub const SUN_ANGLE: f32 = 0.009180432;

/// `Object.empty_drawtype` (OB_ARROWS = 1 … OB_EMPTY_CONE = 7) + size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlendEmptyData {
    pub draw_type: u8,
    pub draw_size: f32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum BlendObjectData {
    #[default]
    None,
    Camera(BlendCameraData),
    Light(BlendLightData),
    Empty(BlendEmptyData),
}

impl BlendFile {
    /// Read an unsigned field whose DNA width is 1, 2 or 4 bytes (enum-typed
    /// fields are `char` / `short` / `int` depending on the struct).
    pub(crate) fn uint_field(&self, block: &BlendBlock, st: &str, field: &str, dna: &DnaType) -> Option<u32> {
        let ps = self.header.pointer_size as usize;
        let layout = dna.layout(ps);
        let &(off, width) = layout.structure(st)?.fields.get(field)?;
        let d = &block.data;
        match width {
            1 => d.get(off).map(|b| *b as u32),
            2 if off + 2 <= d.len() => Some(self.read_u16(&d[off..off + 2]) as u32),
            4 if off + 4 <= d.len() => Some(self.read_i32(&d[off..off + 4]) as u32),
            _ => None,
        }
    }

    pub(crate) fn float_field(&self, block: &BlendBlock, st: &str, field: &str, dna: &DnaType, dflt: f32) -> f32 {
        let ps = self.header.pointer_size as usize;
        match Self::struct_field_offset_exact(st, field, dna, ps) {
            Some(o) if o + 4 <= block.data.len() => self.read_f32(&block.data[o..o + 4]),
            _ => dflt,
        }
    }

    /// Display data for `ob` given its type name.
    pub(crate) fn extract_object_data(&self, ob: &BlendBlock, ty: &str, dna: &DnaType) -> BlendObjectData {
        let ps = self.header.pointer_size as usize;
        let data = Self::struct_field_offset_exact("Object", "*data", dna, ps)
            .map(|o| Self::read_ptr(&ob.data, o, ps, self.header.little_endian))
            .unwrap_or(0);
        match ty {
            "CAMERA" => match self.block_at_code(data, "CA") {
                Some(ca) => {
                    let f = |n: &str, d: f32| self.float_field(ca, "Camera", n, dna, d);
                    BlendObjectData::Camera(BlendCameraData {
                        kind: self.uint_field(ca, "Camera", "type", dna).unwrap_or(0) as u8,
                        lens: f("lens", 50.0),
                        ortho_scale: f("ortho_scale", 6.0),
                        dof: self.extract_dof(ca, dna),
                        sensor_x: f("sensor_x", 36.0),
                        clip_start: f("clip_start", 0.1),
                        clip_end: f("clip_end", 1000.0),
                        drawsize: f("drawsize", 1.0),
                    })
                }
                None => BlendObjectData::None,
            },
            "LIGHT" => match self.block_at_code(data, "LA") {
                Some(la) => {
                    // Blender ≤ 3.x files still name the DNA struct `Lamp`.
                    let st = if dna.types.iter().any(|t| t == "Light") { "Light" } else { "Lamp" };
                    let f = |n: &str, d: f32| self.float_field(la, st, n, dna, d);
                    BlendObjectData::Light(BlendLightData {
                        kind: self.uint_field(la, st, "type", dna).unwrap_or(0) as u8,
                        color: [f("r", 1.0), f("g", 1.0), f("b", 1.0)],
                        energy: f("energy", 10.0),
                        spot_size: f("spotsize", std::f32::consts::FRAC_PI_4),
                        spot_blend: f("spotblend", 0.15),
                        radius: f("radius", 0.1),
                        area_size: f("area_size", 0.25),
                        area_sizey: f("area_sizey", 0.25),
                        area_shape: self.uint_field(la, st, "area_shape", dna).unwrap_or(0) as u8,
                        sun_angle: f("sun_angle", SUN_ANGLE),
                    })
                }
                None => BlendObjectData::None,
            },
            "EMPTY" => BlendObjectData::Empty(BlendEmptyData {
                draw_type: self.uint_field(ob, "Object", "empty_drawtype", dna).unwrap_or(2) as u8,
                draw_size: self.float_field(ob, "Object", "empty_drawsize", dna, 1.0),
            }),
            _ => BlendObjectData::None,
        }
    }
}
