//! Thumbnail tile + wrapping grid — the File Browser's Thumbnails display
//! (`FILE_IMGDISPLAY`), one tile per entry: preview (or type icon), name,
//! and an optional subtitle.

use super::BAction;
use crate::v2::view::BView;

#[derive(Clone, Debug, PartialEq)]
pub struct BThumb {
    /// File whose embedded preview to show (`.blend` `TEST` block). Empty
    /// for entries without one — the tile then shows `icon`.
    pub preview_path: String,
    pub icon: String,
    pub label: String,
    pub subtitle: String,
    /// Tile edge in px (Blender `FileSelectParams.thumbnail_size`).
    pub size: f32,
    pub selected: bool,
    pub action: Option<BAction>,
}

impl BView {
    pub fn thumbnail(thumb: BThumb) -> Self {
        Self::Thumbnail(Box::new(thumb))
    }
    /// Children laid out left-to-right, wrapping onto new rows.
    pub fn grid(list: Vec<BView>) -> Self {
        Self::Grid { list }
    }
}
