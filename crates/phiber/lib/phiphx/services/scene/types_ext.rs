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
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneWorld {
    pub horizon_color: [f32; 3],
}

impl Default for SceneWorld {
    fn default() -> Self {
        // Blender's default mid-grey background.
        Self {
            horizon_color: [0.05, 0.05, 0.05],
        }
    }
}

/// Render settings surface — `Scene.r` output configuration.
/// The rendering / compositing spaces read this to display the output format
/// and target resolution without touching the raw DNA fields themselves.
#[derive(Clone, Debug, PartialEq)]
pub struct SceneRender {
    pub resolution_x: i32,
    pub resolution_y: i32,
    /// `Scene.r.size` — percentage of `resolution_[x,y]` actually rendered.
    pub resolution_percent: i32,
    /// Blender render engine identifier ("BLENDER_EEVEE_NEXT", "CYCLES", …).
    pub engine: String,
}

impl Default for SceneRender {
    fn default() -> Self {
        Self {
            resolution_x: 1920,
            resolution_y: 1080,
            resolution_percent: 100,
            engine: String::from("BLENDER_EEVEE_NEXT"),
        }
    }
}

/// A camera object — matches Blender's `Camera` DNA plus the owning Object's
/// name. Used by the Rendering space to display / preview render cameras.
#[derive(Clone, Debug, PartialEq)]
pub struct SceneCamera {
    pub name: String,
    /// Owning Object's world location.
    pub location: [f32; 3],
    /// Camera focal length in millimetres.
    pub focal_length_mm: f32,
    /// Near / far clip distances.
    pub clip_start: f32,
    pub clip_end: f32,
}

/// A light object — matches Blender's `Light` DNA plus the owning Object.
/// Used by Shading + Rendering spaces.
#[derive(Clone, Debug, PartialEq)]
pub struct SceneLight {
    pub name: String,
    pub location: [f32; 3],
    pub color: [f32; 3],
    /// Blender's `Light.energy`. Watts for POINT/SPOT/AREA, unitless factor
    /// for SUN.
    pub energy: f32,
    /// Blender light kind: `"POINT"`, `"SUN"`, `"SPOT"`, `"AREA"`.
    pub kind: String,
}

/// One action / animation clip. A skeleton stub for now — the fields let the
/// Animation host list clips and show their frame range without opening a
/// full curve editor.
#[derive(Clone, Debug, PartialEq)]
pub struct SceneAction {
    pub name: String,
    pub frame_start: f32,
    pub frame_end: f32,
    /// Number of F-curves in the action (channels).
    pub fcurve_count: usize,
}
