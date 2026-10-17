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
    /// I: `insert` keys the selected objects' Location, Rotation and Scale
    /// (`anim.keyframe_insert`), `channels` every curve they have
    /// (`action.keyframe_insert` / `graph.keyframe_insert`, All Channels)
    /// (`scene::action::keying`).
    Keyframe,
    /// The NLA editor's operators: `click:…`, `channel:…`, `select:all|none|
    /// invert`, `delete`, `mute`, `tracks_add`, `tracks_add_above`,
    /// `tracks_delete` (`scene::action::nla_editor`).
    Nla,
    /// `screen.region_toggle`-ish for the Properties panels: `expand` / `collapse`.
    Props,
    /// `screen.screen_full_area` (Ctrl Space): the slug of the editor whose
    /// menu fired it — the area showing that editor fills the workspace, or
    /// the maximized area is put back (`area::maximize`).
    AreaMaximize,
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
    /// `image.clipboard_copy`: the shown image to the clipboard.
    ImageCopy,
    /// The render region's operators (`scene::render_border`): `arm`,
    /// `arm_image`, `set:…`, `off`, `clear`.
    RenderBorder,
    /// The 3D cursor (`view3d.snap_cursor_to_*`, `view3d.snap_selected_to_cursor`,
    /// `view3d.view_all(center=True)`): `origin`, `selected`, `active`,
    /// `selection_to_cursor`, `center_frame_all` (`scene::cursor`).
    Cursor,
    /// The Dope Sheet's keys (`action.select_all`, `action.delete`, a key
    /// grab): `all`, `none`, `invert`, `delete`, `grab:…`
    /// (`scene::action::dope`).
    Keys,
    /// `wm.save_mainfile`: empty saves to the opened path (or asks for one),
    /// `incremental` to the next numbered name.
    Save,
    /// The "Save changes before closing?" dialog's answer: `save`,
    /// `discard` or `cancel` (`wm_block_file_close_*`).
    FileClose,
    /// `ed.undo_history`: the index of the step to stand at.
    UndoHistory,
    /// An animation editor's `View2D`: `<editor>:all` (`action.view_all`,
    /// Home), `<editor>:scene` (`anim.scene_range_frame`),
    /// `<editor>:zoom:<steps>` (`view2d.zoom_in` / `_out`, the wheel) and
    /// `<editor>:scroll:<pixels>` (`view2d.scroll_left` / `_right`, Ctrl
    /// wheel) (`canvas/time/view/op.rs`).
    TimeView,
    /// The node editor's `View2D`: `<editor>:all` (`NODE_OT_view_all`, Home).
    ///
    /// Pan and zoom are not here — they are gestures, and write
    /// `NodeKey::View` straight from the canvas, which is the only place that
    /// knows the pointer and the region (`canvas/node/view/op.rs` says why).
    NodeView,
    /// `NODE_OT_select_all`: `<editor>:all` (A), `<editor>:none` (Alt A),
    /// `<editor>:invert` (Ctrl I) — the whole-graph selection ops the node
    /// editors share. Picking is a gesture (`canvas/node/select`); these are
    /// the keystrokes that act on every node at once (`canvas/node/select/op.rs`).
    NodeSelect,
    /// `NODE_OT_delete` (X): the selected nodes come out of the graph. The
    /// value names the editor (`<editor>`); the rule reads that editor's
    /// selection (`canvas/node/remove/op.rs`).
    NodeDelete,
    /// `NODE_OT_duplicate` (Shift D): the selected nodes copied as additions,
    /// the copies selected. The value names the editor
    /// (`canvas/node/add/duplicate.rs`).
    NodeDuplicate,
    /// The animation editors' channel operators: `extend:const|lin`,
    /// `protect:on|off|toggle`, `mute:on|off|toggle` — a curve's
    /// extrapolation and its Protect / Mute (`scene/action/channel.rs`).
    Channels,
    /// Object ▸ Clear (Alt G / R / S): `location` / `rotation` / `scale` back
    /// to identity on the selected objects (`scene/transform`).
    ObjectClear,
    /// Object ▸ Parent: `set` makes the selected objects children of the
    /// active one (Ctrl P `PAR_OBJECT`); `clear` takes a parent off (Alt P
    /// `CLEAR_PARENT_ALL`) — `scene/parent`.
    ObjectParent,
    /// `wm.context_set_enum` with `area.type` — the Toggle Graph Editor and
    /// Toggle Dope Sheet entries (`space_dopesheet.py`, `space_graph.py`).
    /// Value is `<from>:<to>`: the area currently showing `from` switches to
    /// `to` (`area/current::area_key`).
    EditorType,
    /// Scene ▸ Preview Range: `selected` reads the selected keyframe columns
    /// and makes their extents the preview range (`ACTION_OT_previewrange_set`,
    /// `action_edit.cc`); `clear` turns the preview range off.
    PreviewRange,
}

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

impl BKey for OpKey {
    fn path(&self) -> BPath {
        BPath::owned(vec!["op", self.tail()])
    }
}

#[cfg(test)]
#[path = "op_tests.rs"]
mod tests;
