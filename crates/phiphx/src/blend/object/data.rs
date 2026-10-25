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
    /// `Camera.sensor_y`: the sensor's height, read when `sensor_fit` is
    /// vertical (`BKE_camera_sensor_size`).
    pub sensor_y: f32,
    /// `Camera.sensor_fit`: 0 AUTO, 1 HORIZONTAL, 2 VERTICAL.
    pub sensor_fit: u8,
    /// `Camera.shiftx / shifty`: lens shift, fractions of the fitted side.
    pub shift: [f32; 2],
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
    /// The `Light` datablock's saved address (`BlendBlock.old_address`).
    ///
    /// Which light datablock this came from, so a driver set read off the
    /// same datablock can be matched to it (`anim/driver/wire.rs`). The
    /// address rather than the name because this struct is `Copy` — a
    /// `String` would take that away from every caller — and because an
    /// address cannot collide where two datablocks share a name.
    pub address: u64,
}

impl BlendFile {
    /// The DNA name of a light's datablock. RNA renamed `Lamp` to `Light`
    /// in 2.80 and **the DNA struct kept the old name** — Blender 5.0.1
    /// still writes `Lamp` — so the modern spelling is checked first only
    /// in case that ever changes, and `Lamp` is what every file in hand
    /// actually uses.
    pub(crate) fn lamp_struct(dna: &DnaType) -> &'static str {
        if dna.types.iter().any(|t| t == "Light") { "Light" } else { "Lamp" }
    }
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
                        sensor_y: f("sensor_y", 24.0),
                        sensor_fit: self.uint_field(ca, "Camera", "sensor_fit", dna).unwrap_or(0) as u8,
                        shift: [f("shiftx", 0.0), f("shifty", 0.0)],
                        clip_start: f("clip_start", 0.1),
                        clip_end: f("clip_end", 1000.0),
                        drawsize: f("drawsize", 1.0),
                    })
                }
                None => BlendObjectData::None,
            },
            "LIGHT" => match self.block_at_code(data, "LA") {
                Some(la) => {
                    let st = Self::lamp_struct(dna);
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
                        address: la.old_address,
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

#[cfg(test)]
mod digest {
    /// Not a fixture test: point `SCENE_CAM_DIGEST` at a `.blend` and this
    /// prints the extracted scene's camera objects and `Scene.camera`, for
    /// checking which camera a render picks. Does nothing without the var.
    #[test]
    fn scene_cam_digest_of_env_file() {
        let Ok(path) = std::env::var("SCENE_CAM_DIGEST") else { return };
        let bytes = std::fs::read(&path).expect("read");
        let f = crate::blend::file::types::BlendFile::parse_bytes(&bytes).expect("parse");
        let scene = f.extract_scene().expect("scene");
        println!("Scene.camera = {:?} cfra {} range {}..{}", scene.props.camera, scene.props.frame_current, scene.props.frame_start, scene.props.frame_end);
        println!("motion tracks: {}", scene.props.motion.list.len());
        for o in scene.objects.iter().filter(|o| o.object_type == "CAMERA") {
            println!("camera object {:?} loc {:?}", o.name, o.location);
        }
        println!("{} objects in all", scene.objects.len());
        let rh = scene.objects.iter().filter(|o| o.visibility.hide_render).count();
        let hv = scene.objects.iter().filter(|o| o.visibility.hide_viewport).count();
        println!("objects hide_render {rh}, hide_viewport {hv}");
        let w = &scene.props.world;
        println!("world: color {:?} strength {} texture {:?} missing {:?}", w.color, w.strength, w.texture, w.missing);
        for o in scene.objects.iter().filter(|o| o.object_type == "LIGHT" || o.object_type == "LAMP") {
            if let crate::blend::BlendObjectData::Light(l) = &o.data {
                println!("light {:?}: kind {} energy {} color {:?} hr {}", o.name, l.kind, l.energy, l.color, o.visibility.hide_render);
            }
        }
        fn walk(c: &crate::blend::BlendCollection, depth: usize) {
            println!(
                "col {}{:?}: {} objs, exclude {} hide_v {} vis(hv {} hs {} hr {})",
                "  ".repeat(depth), c.name, c.object_names.len(), c.exclude, c.hide_viewport,
                c.visibility.hide_viewport, c.visibility.hide_select, c.visibility.hide_render
            );
            for k in &c.children { walk(k, depth + 1); }
        }
        for c in &scene.collections { walk(c, 0); }
    }

    /// Not a fixture test: point `CAM_DIGEST` at a `.blend` and this prints
    /// every camera's framing fields and the scene's render size, for
    /// checking a real file against Blender. Does nothing without the var.
    #[test]
    fn cam_digest_of_env_file() {
        let Ok(path) = std::env::var("CAM_DIGEST") else { return };
        let bytes = std::fs::read(&path).expect("read");
        let f = crate::blend::file::types::BlendFile::parse_bytes(&bytes).expect("parse");
        let dna = f.dna.clone().expect("dna");
        for ca in f.blocks.iter().filter(|b| b.code_str == "CA") {
            let name = f.extract_name_from_block(ca, "id.name").unwrap_or_default();
            let g = |n: &str| f.float_field(ca, "Camera", n, &dna, f32::NAN);
            println!(
                "CA {name} at {:#x}: lens {} sensor {}x{} fit {} shift ({}, {}) type {}",
                ca.old_address,
                g("lens"),
                g("sensor_x"),
                g("sensor_y"),
                f.uint_field(ca, "Camera", "sensor_fit", &dna).unwrap_or(99),
                g("shiftx"),
                g("shifty"),
                f.uint_field(ca, "Camera", "type", &dna).unwrap_or(99),
            );
        }
        let ps = f.header.pointer_size as usize;
        let cams: std::collections::HashSet<u64> = f.blocks.iter().filter(|b| b.code_str == "CA").map(|b| b.old_address).collect();
        let data_off = crate::blend::file::types::BlendFile::struct_field_offset_exact("Object", "*data", &dna, ps).unwrap();
        for ob in f.blocks.iter().filter(|b| b.code_str == "OB") {
            let data = crate::blend::file::types::BlendFile::read_ptr(&ob.data, data_off, ps, f.header.little_endian);
            if cams.contains(&data) {
                println!("OB {} -> CA {data:#x} at {:#x}", f.extract_name_from_block(ob, "id.name").unwrap_or_default(), ob.old_address);
            }
        }
        let r_off = crate::blend::file::types::BlendFile::struct_field_offset_exact("Scene", "r", &dna, ps).expect("Scene.r");
        for sc in f.blocks.iter().filter(|b| b.code_str == "SC") {
            let name = f.extract_name_from_block(sc, "id.name").unwrap_or_default();
            let r = |n: &str| {
                let o = r_off + crate::blend::file::types::BlendFile::struct_field_offset_exact("RenderData", n, &dna, ps)?;
                (o + 4 <= sc.data.len()).then(|| f.read_i32(&sc.data[o..o + 4]))
            };
            println!("SC {name}: res {:?}x{:?}", r("xsch"), r("ysch"));
            if let Some(o) = crate::blend::file::types::BlendFile::struct_field_offset_exact("Scene", "*camera", &dna, ps) {
                let addr = crate::blend::file::types::BlendFile::read_ptr(&sc.data, o, ps, f.header.little_endian);
                let ob = f.blocks.iter().find(|b| b.code_str == "OB" && b.old_address == addr);
                let cam_name = ob.map(|b| f.extract_name_from_block(b, "id.name").unwrap_or_default());
                println!("SC {name}: camera -> {addr:#x} {cam_name:?}");
            }
        }
    }
}

#[cfg(test)]
mod bone_digest {
    /// Not a fixture test: point `BONE_DIGEST` at a `.blend` and this
    /// prints every bone's saved rest fields (`arm_mat`, `bone_mat`,
    /// `head`), for checking a scripted file against Blender's layout.
    #[test]
    fn bone_digest_of_env_file() {
        let Ok(path) = std::env::var("BONE_DIGEST") else { return };
        let bytes = std::fs::read(&path).expect("read");
        let f = crate::blend::file::types::BlendFile::parse_bytes(&bytes).expect("parse");
        let dna = f.dna.clone().expect("dna");
        let layouts = dna.layout(f.header.pointer_size as usize);
        let ps = f.header.pointer_size as usize;
        {
            use std::collections::HashMap;
            let mut by_type: HashMap<&str, usize> = HashMap::new();
            for b in &f.blocks {
                let t = dna.structs.get(b.sdna_index as usize).and_then(|s| dna.types.get(s.type_index as usize)).map_or("?", String::as_str);
                *by_type.entry(t).or_default() += 1;
            }
            let mut rows: Vec<_> = by_type.into_iter().collect();
            rows.sort();
            println!("TYPES {rows:?}");
        }
        for pc in f.blocks.iter().filter(|b| {
            dna.structs.get(b.sdna_index as usize).and_then(|s| dna.types.get(s.type_index as usize)).map(String::as_str) == Some("bPoseChannel")
        }) {
            if let Some(o) = crate::blend::file::types::BlendFile::struct_field_offset_exact("bPoseChannel", "*bone", &dna, ps) {
                let addr = crate::blend::file::types::BlendFile::read_ptr(&pc.data, o, ps, f.header.little_endian);
                println!("PCHAN *bone = {addr:#x} block_at {:?}", f.block_at(addr).map(|b| (b.code_str.clone(), b.old_address, b.count)));
            }
        }
        for b in f.blocks.iter().filter(|b| {
            dna.structs.get(b.sdna_index as usize).and_then(|s| dna.types.get(s.type_index as usize)).map(String::as_str) == Some("Bone")
        }) {
            println!("BONEBLOCK code {} at {:#x} count {} size {}", b.code_str, b.old_address, b.count, b.size);
        }
        if let Some(st) = dna.structs.iter().find(|s| dna.types.get(s.type_index as usize).map(String::as_str) == Some("bPoseChannel")) {
            for fld in &st.fields {
                let n = dna.names.get(fld.name_index as usize).cloned().unwrap_or_default();
                if n.contains("bone") || n.contains("Bone") {
                    println!("DNA bPoseChannel field: {n} (type {})", dna.types.get(fld.type_index as usize).map_or("?", String::as_str));
                }
            }
        }
        if let Some(st) = dna.structs.iter().find(|s| dna.types.get(s.type_index as usize).map(String::as_str) == Some("Bone")) {
            for fld in &st.fields {
                let n = dna.names.get(fld.name_index as usize).cloned().unwrap_or_default();
                if n.contains("mat") || n.contains("head") || n.contains("tail") || n.contains("length") {
                    println!("DNA Bone field: {n}");
                }
            }
        }
        for b in f.blocks.iter().filter(|b| b.code_str == "DATA") {
            let Some(layout) = layouts.structure("Bone") else { continue };
            // Bones are DATA blocks whose SDNA struct is Bone.
            if dna.structs.get(b.sdna_index as usize).map(|s| dna.types.get(s.type_index as usize).map(String::as_str)) != Some(Some("Bone")) {
                continue;
            }
            let name_o = layout.fields.get("name").map(|(o, _)| *o).unwrap_or(0);
            let name = crate::blend::shade::nodes::read_cstr(&b.data, name_o, 64);
            let g = |field: &str, n: usize| -> Vec<f32> {
                layout.fields.get(field).map_or(vec![], |(o, _)| (0..n).map(|k| f.read_f32(&b.data[o + 4 * k..o + 4 * k + 4])).collect())
            };
            println!("BONE {name}: head {:?} arm_mat row3 {:?} bone_mat row0 {:?}", g("head[3]", 3), &g("arm_mat[4][4]", 16)[12..16], &g("bone_mat[3][3]", 3)[0..3]);
        }
    }
}
