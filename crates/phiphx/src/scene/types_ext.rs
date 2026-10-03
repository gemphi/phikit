//! Extended scene surfaces read by non-mesh spaces (Animation / Rendering /
//! Shading / Compositing / Info). Split out of `scene/mod.rs` so the parent
//! stays under the 200-line cap. Every type here has a sensible `Default` so
//! consumers see Blender-factory values before a `.blend` is loaded.

/// Timeline surface — Blender's `Scene.r.sfra`/`efra`/`cfra`/`frs_sec`.
/// Read by Time host, Animation space and the viewport overlay.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneTimeline {
    pub frame_start: i32,
    pub frame_end: i32,
    pub current_frame: i32,
    /// Frames per second, resolved from `frs_sec / frs_sec_base`.
    pub fps: f32,
}

impl Default for SceneTimeline {
    fn default() -> Self {
        // Blender's factory default range and framerate.
        Self {
            frame_start: 1,
            frame_end: 250,
            current_frame: 1,
            fps: 24.0,
        }
    }
}

/// World / environment surface — Blender's `Scene.world → World.horr/g/b`.
/// The horizon RGB is the flat background colour every non-node render falls
/// back to when no environment texture is bound.
#[derive(Clone, Debug, PartialEq)]
pub struct SceneWorld {
    pub horizon_color: [f32; 3],
    /// The Background node that lights the render: colour × strength, the
    /// environment image behind it and its Z rotation.
    pub background: crate::blend::BlendWorld,
    /// Decoded environment image (linear float RGB, equirectangular), when
    /// the file names one and it could be found.
    pub environment: Option<super::texture::SceneEnvironment>,
}

impl Default for SceneWorld {
    fn default() -> Self {
        // Blender's default mid-grey background.
        Self {
            horizon_color: [0.05, 0.05, 0.05],
            background: Default::default(),
            environment: None,
        }
    }
}

/// Render settings surface — `Scene.r` output configuration.
/// The rendering / compositing spaces read this to display the output format
/// and target resolution without touching the raw DNA fields themselves.
#[derive(Clone, Debug, PartialEq)]
pub struct SceneRender {
    /// `Scene.camera` — the object a final render looks through.
    pub camera: String,
    /// `scene["cycles"]` — the Cycles add-on's saved settings. Only values
    /// that differ from the RNA default are in the file, so a missing entry
    /// means "the default", not "zero".
    pub cycles: crate::blend::BIdGroup,
    pub resolution_x: i32,
    pub resolution_y: i32,
    /// `Scene.r.size` — percentage of `resolution_[x,y]` actually rendered.
    pub resolution_percent: i32,
    /// Blender render engine identifier ("BLENDER_EEVEE_NEXT", "CYCLES", …).
    pub engine: String,
    /// `Scene.view_settings.view_transform` ("Standard", "Filmic", "AgX" …).
    pub view_transform: String,
    /// `Scene.view_settings.exposure` (stops) and `.gamma`.
    pub exposure: f32,
    pub gamma: f32,
    /// Film ▸ Transparent (`Scene.r.alphamode`).
    pub film_transparent: bool,
    /// Properties → Output: where a render is written and in what format
    /// (`blend/scene/output.rs`).
    pub output: crate::blend::BlendOutput,
    /// Output → Encoding (`BlendMovie`), for FFmpeg Video.
    pub movie: crate::blend::BlendMovie,
    /// Which render passes the view layer asks for (`blend/scene/pass.rs`).
    pub pass: crate::blend::BlendPassSet,
    /// Workbench's SSAO settings (`Scene.display`).
    pub ssao: SceneSsao,
    /// Workbench's shadow settings (`Scene.display`).
    pub shadow: SceneShadow,
}

/// `Scene.display`'s shadow — the Shading popover's gear popover
/// `VIEW3D_PT_shading_options_shadow`: `light_direction` (Direction, a unit
/// vector in the axes the panel shows), `shadow_shift` (Offset, 0 ..= 1)
/// and `shadow_focus` (Focus, 0 ..= 1). Blender's defaults are
/// (1/√3, 1/√3, 1/√3), 0.1 and 0.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneShadow {
    pub light_direction: [f32; 3],
    pub shift: f32,
    pub focus: f32,
}

impl Default for SceneShadow {
    fn default() -> Self {
        Self { light_direction: [0.577_350_3; 3], shift: 0.1, focus: 0.0 }
    }
}

/// `Scene.display`'s screen-space ambient occlusion — Cavity ▸ World's
/// settings popover (`VIEW3D_PT_shading_options_ssao`): `matcap_ssao_samples`
/// (1 ..= 500), `matcap_ssao_distance` (Blender units) and
/// `matcap_ssao_attenuation`. Blender's defaults are 16, 0.2 and 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneSsao {
    pub samples: u32,
    pub distance: f32,
    pub attenuation: f32,
}

impl Default for SceneSsao {
    fn default() -> Self {
        Self { samples: 16, distance: 0.2, attenuation: 1.0 }
    }
}

impl Default for SceneRender {
    fn default() -> Self {
        Self {
            camera: String::new(),
            cycles: Default::default(),
            resolution_x: 1920,
            resolution_y: 1080,
            resolution_percent: 100,
            engine: String::from("BLENDER_EEVEE_NEXT"),
            view_transform: String::from("Standard"),
            output: Default::default(),
            movie: Default::default(),
            pass: Default::default(),
            ssao: Default::default(),
            shadow: Default::default(),
            exposure: 0.0,
            gamma: 1.0,
            film_transparent: false,
        }
    }
}

pub use super::extras::{SceneCamera, SceneEmpty, SceneLight};

/// One action / animation clip as the animation editors list it: its
/// range, its channel count, the frames that carry a key, and its users.
/// Built from `blend::anim::summary::BlendActionSummary`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SceneAction {
    pub name: String,
    pub frame_start: f32,
    pub frame_end: f32,
    /// Number of F-curves in the action (channels).
    pub fcurve_count: usize,
    /// Sorted, distinct keyed frames across every curve.
    pub keys: Vec<f32>,
    /// Objects whose animation data points at this action.
    pub users: Vec<String>,
    /// Every F-curve, with its keys and Graph-editor colour state.
    pub curves: Vec<crate::blend::BFCurve>,
    /// The slots, each with its run of `curves` (`ActionSlot`).
    pub slots: Vec<crate::blend::BlendActionSlot>,
    /// `(object, slot handle)` per user.
    pub bindings: Vec<(String, i32)>,
    /// Manual Frame Range, when on: `(frame_start, frame_end)`.
    pub manual_range: Option<(f32, f32)>,
    /// Cyclic Animation.
    pub cyclic: bool,
}

/// One animated object's NLA as the NLA editor lists it: the action line
/// and the tracks with their strips (`blend/anim/nla.rs`).
pub type SceneNla = crate::blend::BlendNla;

/// One driven object's driver F-curves (`AnimData.drivers`).
pub type SceneDrivers = crate::blend::BlendDrivers;

impl SceneAction {
    pub fn used_by(&self, object: &str) -> bool {
        self.users.iter().any(|user| user == object)
    }

    /// The slot with `handle`.
    pub fn slot(&self, handle: i32) -> Option<&crate::blend::BlendActionSlot> {
        self.slots.iter().find(|slot| slot.handle == handle)
    }

    /// The curves `handle`'s slot keys; none for a handle the action has no
    /// slot for (an object assigned the action with no slot moves not).
    pub fn curves_of(&self, handle: i32) -> &[crate::blend::BFCurve] {
        self.slot(handle).and_then(|slot| self.curves.get(slot.curves.clone())).unwrap_or(&[])
    }

    /// The slot `object` is bound to in the file.
    pub fn slot_of(&self, object: &str) -> Option<&crate::blend::BlendActionSlot> {
        self.bindings.iter().find(|(o, _)| o == object).and_then(|(_, h)| self.slot(*h))
    }
}
