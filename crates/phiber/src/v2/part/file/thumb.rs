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

/// The card sizes the View settings offer (`FileSelectParams.thumbnail_size`
/// runs 16–256): a thumbnail grid can be a contact sheet or a gallery.
pub const SIZES: [(&str, f32); 4] = [("Small", 64.0), ("Medium", THUMB_SIZE), ("Large", 160.0), ("Extra Large", 256.0)];

/// The card size in force: the one picked, else Medium.
pub fn size(state: &BState) -> f32 {
    let picked = state.get(&FileKey::ThumbSize.path()).unwrap_or("");
    SIZES.iter().find(|(label, _)| *label == picked).map_or(THUMB_SIZE, |(_, px)| *px)
}

fn tile(state: &BState, file: &BFileRow, selected: &str, size: f32) -> BView {
    let is_blend = crate::v2::scene::is_blend_path(&file.path);
    // A picture is its own preview; a .blend its stored thumbnail.
    let shows = is_blend || file.kind == super::kind::BFileKind::Image;
    let action = if file.is_dir() {
        BAction::Set(FileKey::Path.path(), file.path.clone())
    } else {
        super::op::pick_or_commit(state, &file.path)
    };
    BView::thumbnail(BThumb {
        preview_path: if shows { file.path.clone() } else { String::new() },
        icon: file.icon().to_string(),
        label: file.name.clone(),
        subtitle: if is_blend { preview::subtitle(&file.path) } else { String::new() },
        size,
        selected: file.path == selected,
        action: Some(action),
    })
}

pub fn view(state: &BState) -> BView {
    let selected = super::op::picked(state);
    let body = match listing::rows_for(state) {
        Ok(rows) if rows.is_empty() => BView::label("No files in this directory"),
        Ok(rows) => BView::grid(rows.iter().map(|f| tile(state, f, selected, size(state))).collect()),
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
