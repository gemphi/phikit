//! The UV editor's tool settings — `ToolSettings` as `IMAGE_HT_header`
//! draws them in UV editing, under `uv.*`: UV sync selection
//! (`use_uv_select_sync`), the select mode (the mesh's while syncing,
//! `mesh_select_mode`, else `uv_select_mode`), island selection, sticky
//! selection, the pivot (`SpaceImage.pivot_point`), snapping and
//! proportional editing.

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
        };
        BPath::owned(vec!["uv", tail])
    }
}
