//! The context keys — Blender's `bContext`: which workspace, scene, view
//! layer, object, mode and tool are active, and where the frame is.
//! Top-level paths, `ctx.<field>`.

use super::BKey;
use crate::v2::state::BPath;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CtxKey {
    /// The active workspace (`BSpaceType::slug`).
    Space,
    Scene,
    ViewLayer,
    /// The active object's name.
    Object,
    /// The selected objects' names, `|`-separated (`Base.flag & BASE_SELECTED`).
    Selected,
    /// The interaction mode (Object, Edit, Sculpt, …).
    Mode,
    /// The active tool.
    Tool,
    /// The current frame.
    Frame,
    /// `"true"` while playing back.
    Playing,
    /// A transport request (`start`, `end`, `+1`, `-1`), consumed by `follow`.
    FrameStep,
    FrameStart,
    FrameEnd,
    Fps,
    /// `ToolSettings.use_keyframe_insert_auto` — the record toggle.
    AutoKey,
    /// The frame playback started from (`ScreenAnimData::sfra`), for the
    /// Restore Frame loop mode.
    PlayFrom,
    /// `Scene.use_preview_range`, and the range it plays when on.
    PreviewRange,
    PreviewStart,
    PreviewEnd,
    /// `Scene.markers` as edited: `frame=name|frame=name`; unset until edited.
    Markers,
    /// The selected markers, by frame: `12|40` (`TimeMarker.flag & SELECT`).
    MarkerSelected,
    /// A marker grab under way (`scene::grab`).
    MarkerGrab,
    /// A box swept over the marker band (`scene::grab::BMarkerBox`).
    MarkerBox,
    /// `ToolSettings.lock_markers` — "Lock Markers" in the Timeline's Marker
    /// menu: a scene tool setting that gates every marker edit (add, move,
    /// delete, duplicate) in Blender's `anim_markers.cc`. Checked here with
    /// the same gate at the head of `BMarkerOpRule`.
    MarkersLocked,
    /// The Dope Sheet's selected keys, its key grab and its box
    /// (`scene::action::dope`).
    KeySelected,
    KeyGrab,
    KeyBox,
}

impl BKey for CtxKey {
    fn path(&self) -> BPath {
        let seg: &'static str = match self {
            Self::Space => "space",
            Self::Scene => "scene",
            Self::ViewLayer => "view_layer",
            Self::Object => "object",
            Self::Selected => "selected",
            Self::Mode => "mode",
            Self::Tool => "tool",
            Self::Frame => "frame",
            Self::Playing => "playing",
            Self::FrameStep => "frame_step",
            Self::FrameStart => "frame_start",
            Self::FrameEnd => "frame_end",
            Self::Fps => "fps",
            Self::AutoKey => "auto_key",
            Self::PlayFrom => "play_from",
            Self::PreviewRange => "preview_range",
            Self::PreviewStart => "preview_start",
            Self::PreviewEnd => "preview_end",
            Self::Markers => "markers",
            Self::MarkerSelected => return BPath::owned(vec!["markers", "selected"]),
            Self::MarkerGrab => return BPath::owned(vec!["markers", "grab"]),
            Self::MarkerBox => return BPath::owned(vec!["markers", "box"]),
            Self::MarkersLocked => return BPath::owned(vec!["markers", "locked"]),
            Self::KeySelected => return BPath::owned(vec!["keys", "selected"]),
            Self::KeyGrab => return BPath::owned(vec!["keys", "grab"]),
            Self::KeyBox => return BPath::owned(vec!["keys", "box"]),
        };
        BPath::owned(vec![seg])
    }
}
