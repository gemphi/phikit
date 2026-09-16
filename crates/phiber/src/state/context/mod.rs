//! Central context state - mirrors Blender's `bpy.context`.
//!
//! Tracks which space, BAreaSpec, and host is active.
//!
//! Structure (mod.rs/def.rs/ctx.rs/use.rs):
//!   mod.rs  → BMode enum, BContext struct
//!   def.rs  → Default impl, BMode::for_host()
//!   ctx.rs  → Context-sync logic (set_active_host, set_space, set_mode)
//!   use.rs  → Pure state mutations (select_object, deselect_all, set_frame, toggle_playback)

pub mod ctx;
pub mod def;
pub mod r#use;

use crate::store::hosts::BHostType;
use crate::store::regions::BRegionType;
use crate::store::spaces::BSpaceType;
use crate::store::zones::BZoneType;

/// The current interaction mode - mirrors Blender's `bpy.context.mode`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BMode {
    Object,
    Edit,
    Sculpt,
    UvEdit,
    TexturePaint,
    WeightPaint,
    VertexPaint,
    Pose,
    Particle,
    Paint,
    Mask,
    View,
    Default,
}

impl BMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Object => "Object Mode",
            Self::Edit => "Edit Mode",
            Self::Sculpt => "Sculpt Mode",
            Self::UvEdit => "UV Editing",
            Self::TexturePaint => "Texture Paint",
            Self::WeightPaint => "Weight Paint",
            Self::VertexPaint => "Vertex Paint",
            Self::Pose => "Pose Mode",
            Self::Particle => "Particle Mode",
            Self::Paint => "Paint",
            Self::Mask => "Mask",
            Self::View => "View",
            Self::Default => "Default",
        }
    }
}

/// Central context state - the single source of truth for what's active.
#[derive(Clone, Debug)]
pub struct BContext {
    pub active_space: BSpaceType,
    pub active_zone: BZoneType,
    pub active_host: BHostType,
    pub active_region: BRegionType,
    pub mode: BMode,
    pub active_object: Option<String>,
    pub selected_objects: Vec<String>,
    pub active_tool: Option<String>,
    pub active_scene: String,
    pub view_layer: String,
    pub frame_current: i32,
    pub playing: bool,
}
