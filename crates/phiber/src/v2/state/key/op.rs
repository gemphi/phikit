//! The operator keys — Blender's `wm.*` / `object.*` / `screen.*` operators
//! as a menu item fires them: one `Set` of an intent, consumed by the rule
//! that owns it (`state/rule.rs`), or by the app when the intent needs the
//! window or the loaded file. Under `ctx.op.*`; none of them is kept.

use super::BKey;
use crate::v2::state::BPath;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OpKey {
    /// A menu item that has no operator yet: its `Menu ▸ Item` path. The
    /// rule reports it to the status bar and the Info editor.
    Report,
    /// Flip a boolean key: the encoded key to flip (`ctx.…`).
    Flip,
    /// `wm.quit_blender`.
    Quit,
    /// `wm.window_fullscreen_toggle`.
    Fullscreen,
    /// `ed.undo` / `ed.redo`.
    Undo,
    Redo,
    /// `wm.read_homefile`: the template slug to start from.
    NewFile,
    /// `wm.revert_mainfile`.
    Revert,
    /// `screen.workspace_cycle`: `+1` / `-1`.
    SpaceStep,
    /// `object.select_all`: `all` / `none` / `invert`, or `extend:<name>`.
    Select,
    /// Scene edits: `add:<shape>`, `delete`, `duplicate`, `hide`, `unhide`.
    SceneEdit,
    /// 3D view operators: `frame_all`, `frame_selected`, `orbit_left`, …
    View3d,
    /// `anim.start_frame_set` / `anim.end_frame_set`: `start` / `end`.
    FrameRange,
    /// `outliner.show_hierarchy` / `expanded_toggle`: `hierarchy`, `expand`, `collapse`.
    Outliner,
    /// An Outliner drop: `<dragged>><target>`, each `ob:<name>` or
    /// `col:<path>`, led by `&` when Ctrl was held
    /// (`part::outliner::drop::BOutlinerDropRule`).
    OutlinerDrop,
    /// The Action Editor's `template_action` on the active object:
    /// `assign:<action>`, `slot:<identifier>`, `new`, `unlink`
    /// (`scene::action::rule`).
    Action,
    /// The Action panel's Manual Frame Range and Cyclic Animation boxes, and
    /// its Start and End typed (`scene::action::rule`).
    ActionRange,
    ActionCyclic,
    ActionStart,
    ActionEnd,
    /// `screen.region_toggle`-ish for the Properties panels: `expand` / `collapse`.
    Props,
    /// `wm.read_factory_settings`.
    Factory,
    /// `render.render`: `image` (F12) or `animation` (Ctrl F12).
    Render,
    /// Export the scene in a format: `gltf` / `usd` / `ply`, once a path is picked.
    ExportFormat,
    /// `marker.add` / `delete` / `next` / `prev`.
    Marker,
    /// `transform.translate` / `rotate` / `resize` (G / R / S) and, while
    /// one is under way, `confirm`, `cancel` and `axis:x|y|z`.
    Transform,
    /// `wm.rename`-ish (F2, `TOPBAR_PT_name`): empty opens the dialog on the
    /// active object.
    Rename,
    /// `wm.save_mainfile`: empty saves to the opened path (or asks for one),
    /// `incremental` to the next numbered name.
    Save,
    /// The "Save changes before closing?" dialog's answer: `save`,
    /// `discard` or `cancel` (`wm_block_file_close_*`).
    FileClose,
    /// `ed.undo_history`: the index of the step to stand at.
    UndoHistory,
}

impl BKey for OpKey {
    fn path(&self) -> BPath {
        let tail: &'static str = match self {
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
            Self::Props => "props",
            Self::Factory => "factory",
            Self::Render => "render",
            Self::ExportFormat => "export_format",
            Self::Marker => "marker",
            Self::Transform => "transform",
            Self::Rename => "rename",
            Self::Save => "save",
            Self::FileClose => "file_close",
            Self::UndoHistory => "undo_history",
        };
        BPath::owned(vec!["op", tail])
    }
}
