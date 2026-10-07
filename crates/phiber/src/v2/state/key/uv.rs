//! The UV editor's tool settings — `ToolSettings` as `IMAGE_HT_header`
//! draws them in UV editing, under `uv.*`: UV sync selection
//! (`use_uv_select_sync`), the select mode (the mesh's while syncing,
//! `mesh_select_mode`, else `uv_select_mode`), island selection, sticky
//! selection, the pivot (`SpaceImage.pivot_point`), snapping and
//! proportional editing — and the editor's own session edits, the selected
//! corners and their offsets (`scene::uv_edit`).

use super::BKey;
use crate::v2::state::BPath;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UvKey {
    Sync,
    SelectMode,
    MeshSelectMode,
    Island,
    Sticky,
    Pivot,
    Snap,
    SnapElement,
    Proportional,
    Falloff,
    /// The UV corners selected, and how far each moved corner has moved
    /// (`scene::uv_edit`). Session state over a model that stays the
    /// file's, as `ctx.scene.*` is for objects.
    Selected,
    Moved,
    /// The corners the user has **pinned** (`uv.pin`, P in the UV editor).
    /// A pin is what an unwrap solves around: a conformal map is only
    /// defined up to a similarity, so the pinned UVs are the fixed points
    /// that pick one out of the family (`phiphx::uv::lscm`).
    Pinned,
    /// Where the pointer was when the drag last moved, as `u:v` on the
    /// square. In state rather than in the element, because the press
    /// writes the selection and the re-render that follows replaces the
    /// element — and with it any cell the handlers were holding.
    Drag,
}

impl BKey for UvKey {
    fn path(&self) -> BPath {
        let tail: &'static str = match self {
            Self::Sync => "sync",
            Self::SelectMode => "select_mode",
            Self::MeshSelectMode => "mesh_select_mode",
            Self::Island => "island",
            Self::Sticky => "sticky",
            Self::Pivot => "pivot",
            Self::Snap => "snap",
            Self::SnapElement => "snap_element",
            Self::Proportional => "proportional",
            Self::Falloff => "falloff",
            Self::Selected => "selected",
            Self::Moved => "moved",
            Self::Pinned => "pinned",
            Self::Drag => "drag",
        };
        BPath::owned(vec!["uv", tail])
    }
}
