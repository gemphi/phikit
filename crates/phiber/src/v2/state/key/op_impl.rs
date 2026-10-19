//! `impl OpKey` — the walk list (`ALL`), the tail name of each key, and the
//! parse (`of`). Split from `op.rs` so the enum variants and their doc stay
//! under the 240-line read cap (`tests/level_structure.rs`).

use super::OpKey;
use crate::v2::state::BPath;

impl OpKey {
    /// Every op, so the enum can be walked both ways.
    pub const ALL: &'static [Self] = &[
        Self::Report,
        Self::Flip,
        Self::Quit,
        Self::Fullscreen,
        Self::Undo,
        Self::Redo,
        Self::NewFile,
        Self::Revert,
        Self::SpaceStep,
        Self::Select,
        Self::SceneEdit,
        Self::View3d,
        Self::FrameRange,
        Self::Outliner,
        Self::OutlinerDrop,
        Self::Action,
        Self::ActionRange,
        Self::ActionCyclic,
        Self::ActionStart,
        Self::ActionEnd,
        Self::Keyframe,
        Self::Nla,
        Self::Props,
        Self::AreaMaximize,
        Self::Factory,
        Self::Render,
        Self::ExportFormat,
        Self::Marker,
        Self::Transform,
        Self::Rename,
        Self::ImageCopy,
        Self::RenderBorder,
        Self::Cursor,
        Self::Keys,
        Self::Save,
        Self::FileClose,
        Self::UndoHistory,
        Self::TimeView,
        Self::NodeView,
        Self::NodeSelect,
        Self::NodeDelete,
        Self::NodeDuplicate,
        Self::Channels,
        Self::ObjectClear,
        Self::ObjectParent,
        Self::EditorType,
        Self::PreviewRange,
    ];

    /// The last segment of this op's path.
    ///
    /// `path` builds on it and [`Self::of`] reads it back, so the spelling
    /// exists once. It used to exist once too — but only forwards, which left
    /// the dispatcher in `app/op/` comparing an ENCODED path against ten
    /// encoded keys in a row to answer a question the key can answer itself.
    pub const fn tail(self) -> &'static str {
        match self {
            Self::Report => "report",
            Self::Flip => "flip",
            Self::Quit => "quit",
            Self::Fullscreen => "fullscreen",
            Self::Undo => "undo",
            Self::Redo => "redo",
            Self::NewFile => "new_file",
            Self::Revert => "revert",
            Self::SpaceStep => "space_step",
            Self::Select => "select",
            Self::SceneEdit => "scene_edit",
            Self::View3d => "view3d",
            Self::FrameRange => "frame_range",
            Self::Outliner => "outliner",
            Self::OutlinerDrop => "outliner_drop",
            Self::Action => "action",
            Self::ActionRange => "action_range",
            Self::ActionCyclic => "action_cyclic",
            Self::ActionStart => "action_start",
            Self::ActionEnd => "action_end",
            Self::Keyframe => "keyframe",
            Self::Nla => "nla",
            Self::Props => "props",
            Self::AreaMaximize => "area_maximize",
            Self::Factory => "factory",
            Self::Render => "render",
            Self::ExportFormat => "export_format",
            Self::Marker => "marker",
            Self::Transform => "transform",
            Self::Rename => "rename",
            Self::ImageCopy => "image_copy",
            Self::RenderBorder => "render_border",
            Self::Cursor => "cursor",
            Self::Keys => "keys",
            Self::Save => "save",
            Self::FileClose => "file_close",
            Self::UndoHistory => "undo_history",
            Self::TimeView => "time_view",
            Self::NodeView => "node_view",
            Self::NodeSelect => "node_select",
            Self::NodeDelete => "node_delete",
            Self::NodeDuplicate => "node_duplicate",
            Self::Channels => "channels",
            Self::ObjectClear => "object_clear",
            Self::ObjectParent => "object_parent",
            Self::EditorType => "editor_type",
            Self::PreviewRange => "preview_range",
        }
    }

    /// The op a path names, if it names one.
    ///
    /// The WHOLE path, not just its tail. Every op's path is exactly
    /// `op.<tail>` and nothing else in the tree is, so a key elsewhere that
    /// happens to end in a word an op uses is not that op. The dispatcher in
    /// `app/op/` used to ask the family separately, one line above its ladder;
    /// it is one question about one path, so it is answered once, here.
    pub fn of(path: &BPath) -> Option<Self> {
        let [family, tail] = path.segs.as_ref() else { return None };
        if *family != "op" {
            return None;
        }
        Self::ALL.iter().copied().find(|op| op.tail() == *tail)
    }
}
