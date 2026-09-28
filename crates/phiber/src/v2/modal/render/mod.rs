//! Render window — Blender opens `Render ▸ Render Image` (F12) in a new
//! window titled "Blender Render" that hosts an **Image Editor** showing the
//! `Render Result` image (`render_display_type == 'WINDOW'`,
//! `render_view_open` in `render_view.cc`). Phiber does the same with its
//! Image host inside a window-sized modal card.

use crate::v2::host::BHostItem;
use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;

#[derive(Clone, Copy, Debug, Default)]
pub struct BRenderModal;

impl BViewItem for BRenderModal {
    fn build(&self, s: &BState) -> BView {
        BHostItem::new(BHostType::Image).build(s)
    }
}
