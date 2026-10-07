//! Scene-level properties Blender shows on open: frame range / fps
//! (`Scene.r.cfra/sfra/efra/frs_sec/frs_sec_base`), output resolution
//! (`Scene.r.xsch/ysch/size`), render engine (`Scene.r.engine`) and the
//! world's horizon colour (`Scene.world → World.horr/horg/horb`).
//! `Scene.r` is an embedded `RenderData`, so its fields are read at
//! `offset(Scene.r) + offset(RenderData.<field>)`.

use crate::blend::file::types::*;

#[derive(Debug, Clone, PartialEq)]
pub struct BlendSceneProps {
    pub frame_current: i32,
    pub frame_start: i32,
    pub frame_end: i32,
    pub fps: f32,
    pub resolution_x: i32,
    pub resolution_y: i32,
    pub resolution_percent: i32,
    pub engine: String,
    /// Colour management (`Scene.view_settings`): view transform name
    /// ("Standard", "Filmic", "AgX" …), exposure (stops) and gamma.
    pub view_transform: String,
    pub exposure: f32,
    pub gamma: f32,
    /// Film ▸ Transparent (`Scene.r.alphamode == R_ALPHAPREMUL`): the
    /// background rendered as alpha 0 rather than the world.
    pub film_transparent: bool,
    /// Workbench's screen-space ambient occlusion (`Scene.display`,
    /// `VIEW3D_PT_shading_options_ssao`): `[matcap_ssao_distance,
    /// matcap_ssao_attenuation, matcap_ssao_samples]` — 0.2, 1, 16 by
    /// default (`DNA_scene_defaults.h`).
    pub ssao: [f32; 3],
    /// Workbench's shadow (`Scene.display`, `VIEW3D_PT_shading_options_shadow`):
    /// `light_direction` (unit, Blender's "user-friendly" axes — `ShadowPass::
    /// init` swaps y and z and negates x and z), `shadow_shift`,
    /// `shadow_focus`; (1/√3, 1/√3, 1/√3), 0.1, 0 by default.
    pub light_direction: [f32; 3],
    pub shadow: [f32; 2],
    /// The Workbench render engine's shading (`Scene.display.shading`, what
    /// F12 draws with) and its anti-aliasing (`render_aa`: `eSceneDisplay_AA`
    /// — 0 off, 1 FXAA, 5 / 8 / 11 / 16 / 32 samples; 8 by default).
    pub display_shading: crate::blend::BlendShading,
    pub render_aa: i16,
    pub world_color: [f32; 3],
    /// The world's Background node (what actually lights a 2.8+ render).
    pub world: crate::blend::BlendWorld,
    /// `Scene.camera` — the object a final render looks through (F12), which
    /// is not necessarily the viewport's view. Empty when the scene has none.
    pub camera: String,
    /// Properties → Output: where a render is written, and in what format
    /// (`scene/output.rs`).
    pub output: crate::blend::scene::output::BlendOutput,
    /// The render region (`Scene.r.border`, `R_BORDER`, `R_CROP`).
    pub border: crate::blend::BlendRenderBorder,
    /// Output → Encoding, for FFmpeg Video (`scene/movie.rs`).
    pub movie: crate::blend::scene::movie::BlendMovie,
    /// Which render passes the view layer asks for (`scene/pass.rs`).
    pub pass: crate::blend::scene::pass::BlendPassSet,
    /// `scene["cycles"]` — the Cycles add-on's settings, as ID properties.
    /// Only values that differ from the RNA default are in the file, so this
    /// is usually a handful of entries and often empty.
    pub cycles: crate::blend::BIdGroup,
    pub view_layer_name: String,
    /// File version from the header ("3.6", "5.2").
    pub file_version: String,
    /// Workspace + 3D view to restore ("Load UI", `ui_state.rs`).
    pub ui: crate::blend::scene::ui::BlendUiState,
    /// Packed image bytes by image key (`image.rs`).
    pub packed_images: crate::blend::shade::image::PackedImages,
    /// Modifier-evaluated meshes by object index (`modifier/`); the
    /// object's own geometry stays the rest mesh (`Object.data`).
    pub evaluated: std::collections::HashMap<usize, crate::blend::modifier::BlendEvalMesh>,
    /// Every action in the file and who uses it (`anim/summary.rs`).
    pub actions: Vec<crate::blend::anim::summary::BlendActionSummary>,
    /// Every animated object's NLA — action line and tracks (`anim/nla.rs`).
    pub nla: Vec<crate::blend::anim::nla::BlendNla>,
    /// Every driven object's driver F-curves (`AnimData.drivers`).
    pub drivers: Vec<crate::blend::anim::drivers::BlendDrivers>,
    /// Every object's transform over time, in object order (`anim/motion.rs`).
    pub motion: crate::blend::anim::motion::BlendMotionList,
    /// Every material that carries a driver, by material name
    /// (`shade/driver.rs`). Applied per frame over the materials the
    /// objects were read with, after the objects' own driver pass.
    pub material_drivers: Vec<crate::blend::shade::driver::BMaterialDrivers>,
    /// Every light that carries a driver, by `Light` name
    /// (`object/light_driver.rs`).
    pub light_drivers: Vec<crate::blend::object::light_driver::BLightDrivers>,
    /// Every world that carries a driver, over its own `color` and its
    /// Background node's sockets (`shade/world_driver.rs`).
    pub world_drivers: Vec<crate::blend::shade::world_driver::BWorldDrivers>,
    /// Every object that carries a driver on one of its modifiers'
    /// settings (`modifier/driver.rs`). Read, typed and clamped; not yet
    /// written into the evaluation stack — see that module's header.
    pub modifier_drivers: Vec<crate::blend::modifier::driver::BModifierDrivers>,
    /// The timeline markers (`scene/marker.rs`).
    pub markers: Vec<crate::blend::scene::marker::BlendMarker>,
    /// The Playback popover's settings (`scene/playback.rs`).
    pub playback: crate::blend::BlendPlayback,
    /// The rigs that move and the meshes they deform (`rig/source.rs`).
    pub rigs: crate::blend::rig::BlendRigs,
    /// The 3D cursor (`scene/cursor.rs`).
    pub cursor: crate::blend::BlendCursor,
    /// The scene's compositor tree (`comp/read.rs`), read whichever of the
    /// two DNA spellings the file was saved with. `None` when the file has
    /// no tree at all; a tree that is switched off still comes through, so
    /// the editor can draw it.
    pub compositor: Option<crate::comp::BCompTree>,
}

impl Default for BlendSceneProps {
    fn default() -> Self {
        Self {
            compositor: None,
            frame_current: 1,
            frame_start: 1,
            frame_end: 250,
            fps: 24.0,
            resolution_x: 1920,
            resolution_y: 1080,
            resolution_percent: 100,
            engine: "BLENDER_EEVEE".to_string(),
            view_transform: "Standard".to_string(),
            exposure: 0.0,
            gamma: 1.0,
            film_transparent: false,
            ssao: [0.2, 1.0, 16.0],
            light_direction: [0.577_350_3; 3],
            shadow: [0.1, 0.0],
            display_shading: Default::default(),
            render_aa: 8,
            world_color: [0.05, 0.05, 0.05],
            camera: String::new(),
            world: Default::default(),
            output: Default::default(),
            border: Default::default(),
            movie: Default::default(),
            pass: Default::default(),
            cycles: Default::default(),
            view_layer_name: "ViewLayer".to_string(),
            file_version: String::new(),
            ui: Default::default(),
            packed_images: Vec::new(),
            evaluated: Default::default(),
            actions: Vec::new(),
            nla: Vec::new(),
            drivers: Vec::new(),
            motion: Default::default(),
            material_drivers: Vec::new(),
            light_drivers: Vec::new(),
            world_drivers: Vec::new(),
            modifier_drivers: Vec::new(),
            markers: Vec::new(),
            playback: Default::default(),
            rigs: Default::default(),
            cursor: Default::default(),
        }
    }
}

impl BlendFile {
    fn rd_off(&self, field: &str, dna: &DnaType) -> Option<usize> {
        let ps = self.header.pointer_size as usize;
        let r = Self::struct_field_offset_exact("Scene", "r", dna, ps)?;
        Some(r + Self::struct_field_offset_exact("RenderData", field, dna, ps)?)
    }

    fn sp_i32(&self, d: &[u8], off: Option<usize>, dflt: i32) -> i32 {
        match off {
            Some(o) if o + 4 <= d.len() => self.read_i32(&d[o..o + 4]),
            _ => dflt,
        }
    }
    fn sp_i16(&self, d: &[u8], off: Option<usize>, dflt: i32) -> i32 {
        match off {
            Some(o) if o + 2 <= d.len() => self.read_u16(&d[o..o + 2]) as i16 as i32,
            _ => dflt,
        }
    }
    fn sp_f32(&self, d: &[u8], off: Option<usize>, dflt: f32) -> f32 {
        match off {
            Some(o) if o + 4 <= d.len() => self.read_f32(&d[o..o + 4]),
            _ => dflt,
        }
    }

    pub(crate) fn extract_scene_props(&self, sc: &BlendBlock, dna: &DnaType) -> BlendSceneProps {
        let ps = self.header.pointer_size as usize;
        let le = self.header.little_endian;
        let d = &sc.data;
        let dflt = BlendSceneProps::default();
        let frs_sec = self.sp_i16(d, self.rd_off("frs_sec", dna), 24) as f32;
        let frs_sec_base = self.sp_f32(d, self.rd_off("frs_sec_base", dna), 1.0).max(1e-6);
        let engine = self
            .rd_off("engine[32]", dna)
            .map(|o| {
                let end = (o + 32).min(d.len());
                let b = &d[o.min(end)..end];
                let n = b.iter().position(|&c| c == 0).unwrap_or(b.len());
                String::from_utf8_lossy(&b[..n]).to_string()
            })
            .filter(|e| !e.is_empty())
            .unwrap_or(dflt.engine.clone());
        // `Scene.view_settings` is an embedded `ColorManagedViewSettings`.
        let vs = |field: &str| {
            Some(Self::struct_field_offset_exact("Scene", "view_settings", dna, ps)?
                + Self::struct_field_offset_exact("ColorManagedViewSettings", field, dna, ps)?)
        };
        let view_transform = vs("view_transform[64]")
            .map(|o| {
                let end = (o + 64).min(d.len());
                let b = &d[o.min(end)..end];
                let n = b.iter().position(|&c| c == 0).unwrap_or(b.len());
                String::from_utf8_lossy(&b[..n]).to_string()
            })
            .filter(|v| !v.is_empty())
            .unwrap_or(dflt.view_transform.clone());
        // `Scene.display` is an embedded `SceneDisplay`.
        let disp = |field: &str| {
            Some(Self::struct_field_offset_exact("Scene", "display", dna, ps)? + Self::struct_field_offset_exact("SceneDisplay", field, dna, ps)?)
        };
        let wo = Self::struct_field_offset_exact("Scene", "*world", dna, ps)
            .map(|o| Self::read_ptr(d, o, ps, le))
            .and_then(|addr| self.block_at_code(addr, "WO"));
        let world_color = wo
            .map(|wo| {
                let c = |f: &str| {
                    self.sp_f32(&wo.data, Self::struct_field_offset_exact("World", f, dna, ps), 0.05)
                };
                [c("horr"), c("horg"), c("horb")]
            })
            .unwrap_or(dflt.world_color);
        let world = wo.map(|wo| self.world(wo, dna)).unwrap_or_default();
        // `scene["cycles"]` is an ID property group, not DNA (`idprop.rs`).
        let camera = Self::struct_field_offset_exact("Scene", "*camera", dna, ps)
            .map(|o| Self::read_ptr(d, o, ps, le))
            .and_then(|addr| self.block_at_code(addr, "OB"))
            .and_then(|ob| self.extract_name_from_block(ob, "name").ok())
            .unwrap_or_default();
        let cycles = match self.id_properties(sc, dna).get("cycles") {
            Some(crate::blend::BIdValue::Group(g)) => g.clone(),
            _ => Default::default(),
        };
        BlendSceneProps {
            // Read from the Scene block in `read.rs`, which has the DNA to
            // hand; the props reader does not reach the node tree.
            compositor: None,
            frame_current: self.sp_i32(d, self.rd_off("cfra", dna), 1),
            frame_start: self.sp_i32(d, self.rd_off("sfra", dna), 1),
            frame_end: self.sp_i32(d, self.rd_off("efra", dna), 250),
            fps: frs_sec / frs_sec_base,
            resolution_x: self.sp_i32(d, self.rd_off("xsch", dna), 1920),
            resolution_y: self.sp_i32(d, self.rd_off("ysch", dna), 1080),
            resolution_percent: self.sp_i16(d, self.rd_off("size", dna), 100),
            engine,
            view_transform,
            exposure: self.sp_f32(d, vs("exposure"), 0.0),
            gamma: self.sp_f32(d, vs("gamma"), 1.0),
            // `R_ADDSKY` 0, `R_ALPHAPREMUL` 1.
            film_transparent: self.sp_i16(d, self.rd_off("alphamode", dna), 0) == 1,
            ssao: [self.sp_f32(d, disp("matcap_ssao_distance"), 0.2), self.sp_f32(d, disp("matcap_ssao_attenuation"), 1.0), self.sp_i32(d, disp("matcap_ssao_samples"), 16).clamp(1, 500) as f32],
            light_direction: disp("light_direction[3]").map_or(dflt.light_direction, |o| std::array::from_fn(|i| self.sp_f32(d, Some(o + 4 * i), dflt.light_direction[i]))),
            shadow: [self.sp_f32(d, disp("shadow_shift"), 0.1), self.sp_f32(d, disp("shadow_focus"), 0.0)],
            display_shading: self.shading_at(Some(sc), disp("shading"), dna),
            // `render_aa` is a `char` in DNA.
            render_aa: disp("render_aa").and_then(|o| d.get(o)).map_or(8, |&b| i16::from(b as i8)),
            world_color,
            camera,
            world,
            output: self.extract_output(sc, dna),
            border: crate::blend::BlendRenderBorder::of(
                self.sp_i32(d, self.rd_off("mode", dna), 0) as u32,
                self.rd_off("border", dna).map_or([0.0, 1.0, 0.0, 1.0], |o| std::array::from_fn(|i| self.sp_f32(d, Some(o + 4 * i), [0.0, 1.0, 0.0, 1.0][i]))),
            ),
            movie: self.extract_movie(sc, dna),
            pass: self.extract_pass_set(sc, dna),
            cycles,
            view_layer_name: dflt.view_layer_name,
            file_version: self.header.version_label(),
            ui: Default::default(),
            packed_images: Vec::new(),
            evaluated: Default::default(),
            actions: Vec::new(),
            nla: Vec::new(),
            drivers: Vec::new(),
            motion: Default::default(),
            material_drivers: Vec::new(),
            light_drivers: Vec::new(),
            world_drivers: Vec::new(),
            modifier_drivers: Vec::new(),
            markers: self.markers(sc, dna),
            playback: self.extract_playback(sc, dna),
            rigs: Default::default(),
            cursor: self.cursor(sc, dna),
        }
    }
}
