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
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn every_op_is_found_by_its_own_path() {
        // The round trip is the whole point of `tail` existing: `path` writes
        // the spelling and `of` reads it, so neither can drift from the other.
        for op in OpKey::ALL {
            assert_eq!(OpKey::of(&op.path()), Some(*op), "{op:?}");
        }
    }

    #[test]
    fn all_lists_every_variant_exactly_once() {
        // `ALL` is hand-kept because Rust cannot walk variants. Nothing but
        // this notices a variant added to the enum and not to the list — and
        // `of` would then silently never return it.
        let tails: BTreeSet<&str> = OpKey::ALL.iter().map(|op| op.tail()).collect();
        assert_eq!(tails.len(), OpKey::ALL.len(), "two ops share a tail");
        let source = include_str!("op.rs");
        let body = {
            let from = source.find("pub enum OpKey {").expect("the enum");
            let rest = &source[from..];
            &rest[..rest.find("\n}\n").expect("its end")]
        };
        let declared = body
            .lines()
            .filter_map(|line| {
                let name = line.split("//").next()?.trim().strip_suffix(',')?;
                (!name.is_empty()
                    && name.chars().next()?.is_ascii_uppercase()
                    && !name.contains(' '))
                .then_some(name)
            })
            .count();
        assert_eq!(declared, OpKey::ALL.len(), "the enum declares {declared}");
    }

    #[test]
    fn a_path_outside_the_op_family_is_not_an_op() {
        // `of` is strict about the family, which is what let the dispatcher
        // stop checking it separately. A key that ends in an op's word — and
        // `ctx.save` would — must not be read as that op.
        assert_eq!(OpKey::of(&BPath::owned(vec!["ctx", "save"])), None);
        assert_eq!(OpKey::of(&BPath::owned(vec!["op", "save", "as"])), None, "two segments only");
        assert_eq!(OpKey::of(&BPath::owned(vec!["op"])), None);
        assert_eq!(OpKey::of(&BPath::owned(vec!["op", "nothing_of_the_sort"])), None);
    }
}
