//! Thumbnails display (`FILE_IMGDISPLAY`): a wrapping grid of tiles — the
//! `.blend` preview when the file has one, the type icon otherwise.

use crate::v2::state::BKey as _;
use crate::v2::state::FileKey;
use super::listing;
use super::preview;
use super::BFileRow;
use crate::v2::state::BState;
use crate::v2::view::prim::{BCard, BColumn};
use crate::v2::view::prim::{BAction, BThumb};
use crate::v2::view::BView;

/// Blender's default `thumbnail_size` ("Normal", `FileSelectParams`).
pub const THUMB_SIZE: f32 = 96.0;

fn tile(file: &BFileRow, selected: &str) -> BView {
    let is_blend = crate::v2::scene::is_blend_path(&file.path);
    let action = if file.is_dir() {
        BAction::Set(FileKey::Path.path(), file.path.clone())
    } else {
        BAction::Set(FileKey::Selected.path(), file.path.clone())
    };
    BView::thumbnail(BThumb {
        preview_path: if is_blend { file.path.clone() } else { String::new() },
        icon: file.icon().to_string(),
        label: file.name.clone(),
        subtitle: if is_blend { preview::subtitle(&file.path) } else { String::new() },
        size: THUMB_SIZE,
        selected: file.path == selected,
        action: Some(action),
    })
}

pub fn view(state: &BState) -> BView {
    let selected = state.get(&FileKey::Selected.path()).unwrap_or("");
    let body = match listing::rows_for(state) {
        Ok(rows) if rows.is_empty() => BView::label("No files in this directory"),
        Ok(rows) => BView::grid(rows.iter().map(|f| tile(f, selected)).collect()),
        Err(why) => BView::label(why),
    };
    BView::from(BCard::column(BColumn::Body, vec![body]))
}

/// Whether the browser is in Thumbnails display mode.
pub fn active(state: &BState) -> bool {
    use super::set::{BFileDisplayType, DISPLAY_MODES};
    let Some(current) = state.get(&FileKey::Display.path()) else { return false };
    DISPLAY_MODES
        .iter()
        .any(|m| m.kind == BFileDisplayType::Thumbnail && m.label == current)
}
