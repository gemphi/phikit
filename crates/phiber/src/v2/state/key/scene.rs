//! The loaded scene's keys — what is being read and how far it has got,
//! under `ctx.scene.*`.

use super::BKey;
use crate::v2::state::BPath;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SceneKey {
    /// `"true"` while a .blend is being read; absent otherwise.
    Loading,
    /// The stage label the progress screen shows.
    LoadStage,
    /// 0..1, the progress bar.
    LoadProgress,
    /// The Cancel intent, consumed by the loader.
    LoadCancel,
    /// The last load error, if any.
    Error,
    /// The path of the file on screen.
    LoadedPath,
    /// Objects added since the file was read: `name=shape|name=shape`.
    Added,
    /// Objects deleted since the file was read: `name|name`.
    Deleted,
    /// Objects duplicated since the file was read: `copy=source|copy=source`.
    Copied,
    /// Objects renamed since the file was read: `new=old|new=old`.
    Renamed,
    /// Objects the Outliner moved to another collection: `name=path|…`.
    Moved,
    /// Objects the Outliner parented: `child=parent|…`.
    Parented,
    /// Objects Ctrl-dragged into another collection as well: `name=path|…`.
    Linked,
    /// Collections the Outliner nested in another: `path=parent path|…`
    /// (the file's paths, which stay their ids).
    Nested,
    /// Objects given another action, or none: `object=action#slot|…`
    /// (`scene::action`).
    Assigned,
    /// Actions made by New: `name=source|…` (an empty source: an empty action).
    MadeActions,
    /// Actions' Manual Frame Range and Cyclic: `action=start,end,cyclic|…`
    /// (no range: `-,-,cyclic`).
    ActionRanges,
    /// Keys inserted (I), in order:
    /// `action#slot#frame=path:index:value;…|…` (`scene::action::keys`).
    Keyed,
    /// Each NLA stack Push Down, Stash and the NLA editor edited, whole,
    /// as JSON (`scene::action::nla`).
    Nla,
    /// Objects the Transform panel moved, turned or scaled:
    /// `name=lx,ly,lz,rx,ry,rz,sx,sy,sz|…` (rotation in degrees, XYZ).
    Transformed,
    /// The 3D cursor when the session moved it: `x,y,z,w,qx,qy,qz` — its
    /// place and its turn (`scene::cursor`).
    Cursor,
    /// `"true"` while the document has changed since it was read or
    /// written — Blender's `!wm->file_saved` (`WM_file_tag_modified`).
    Unsaved,
}

impl BKey for SceneKey {
    fn path(&self) -> BPath {
        let tail: &'static str = match self {
            Self::Loading => "loading",
            Self::LoadStage => "load_stage",
            Self::LoadProgress => "load_progress",
            Self::LoadCancel => "load_cancel",
            Self::Error => "error",
            Self::LoadedPath => "loaded_path",
            Self::Added => "added",
            Self::Deleted => "deleted",
            Self::Copied => "copied",
            Self::Renamed => "renamed",
            Self::Unsaved => "unsaved",
            Self::Moved => "moved",
            Self::Parented => "parented",
            Self::Linked => "linked",
            Self::Nested => "nested",
            Self::Assigned => "assigned",
            Self::MadeActions => "made_actions",
            Self::ActionRanges => "action_ranges",
            Self::Keyed => "keyed",
            Self::Nla => "nla",
            Self::Transformed => "transformed",
            Self::Cursor => "cursor",
        };
        BPath::owned(vec!["scene", tail])
    }
}
