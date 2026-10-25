//! The file browser's details column — what the picked file is: its
//! preview, its name and folder, its kind, size and date. Blender keeps
//! these in a tooltip; Finder and the Project Browser keep them in a column
//! beside the list, which is where you look after picking a file.

use super::listing;
use crate::v2::state::BState;
use crate::v2::view::prim::{BCard, BColumn, BThumb};
use crate::v2::view::{BView, BViewItem};

/// How big the preview in the details column is.
const PREVIEW: f32 = 160.0;

#[derive(Clone, Copy, Debug)]
pub struct BFileInfo;

impl BViewItem for BFileInfo {
    fn build(&self, s: &BState) -> BView {
        let picked = super::op::picked(s);
        let row = listing::rows_for(s).ok().and_then(|rows| rows.into_iter().find(|row| row.path == picked && !picked.is_empty()));
        let Some(file) = row else {
            return BView::from(BCard::column(BColumn::Sidebar, vec![BView::label("Select a file to see its details.")]));
        };
        let shows = crate::v2::scene::is_blend_path(&file.path) || file.kind == super::kind::BFileKind::Image;
        let preview = BView::thumbnail(BThumb {
            preview_path: if shows { file.path.clone() } else { String::new() },
            icon: file.icon().to_string(),
            label: file.name.clone(),
            subtitle: String::new(),
            size: PREVIEW,
            selected: false,
            action: None,
        });
        let folder = crate::v2::modal::splash::recent::parent_dir(&file.path).to_string();
        let mut rows = vec![BView::pair("Name", file.name.clone()), BView::pair("Folder", folder)];
        rows.push(BView::pair("Kind", file.kind.label().to_string()));
        if !file.is_dir() {
            rows.push(BView::pair("Size", file.format_size()));
        }
        rows.push(BView::pair("Modified", file.format_time()));
        let cards = vec![BCard::panel("Preview").body(vec![preview]).into(), BCard::panel("Details").icon(file.icon()).body(rows).into()];
        BView::from(BCard::column(BColumn::Sidebar, cards))
    }
}
