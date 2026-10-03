//! The Image editor's header, right of the menus — `IMAGE_HT_header`. Two
//! parts, because Blender puts a spacer between them: the image data-block
//! in the centre (`template_ID`), and at the right end the gizmo and overlay
//! toggles, the render slot, view layer and pass (`template_image_layers`)
//! and the display channels.

pub mod set;

use crate::v2::host::BHostType;
use crate::v2::scene::render_job;
use crate::v2::state::{BKey as _, BState, ImageKey, ModalKey, OpKey, val::V_MODAL_FILE_BROWSER};
use crate::v2::view::prim::{BAction, BControl};
use crate::v2::view::{BView, BViewItem};

/// `template_ID(sima, "image", new="image.new", open="image.open")`: the
/// image shown — the Render Result once there is one — or New and Open.
#[derive(Clone, Copy, Debug)]
pub struct BImageBlock {
    pub host: BHostType,
}

impl BImageBlock {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }

    /// The Render Result exists as a data-block once a render has been
    /// asked for; the UV editor never shows it.
    pub fn shows_render(&self) -> bool {
        self.host == BHostType::Image && render_job::shot().0.is_some()
    }
}

impl BViewItem for BImageBlock {
    fn build(&self, _s: &BState) -> BView {
        if self.shows_render() {
            let mut row = vec![BView::Tab(BControl::new().with_icon("image").with_label(set::RENDER_RESULT).with_tooltip("Image"))];
            // While the render runs, its progress and a Cancel beside the
            // image it is drawing — Blender keeps the job's bar and X in the
            // status bar (`WM_jobs_stop`), which stays too; here it is also
            // where the eye is.
            if render_job::rendering()
                && let Some(shot) = render_job::shot().0
            {
                row.push(BView::progress_bar(shot.fraction(), set::RENDERING));
                row.push(BView::Button(
                    BControl::new().with_icon("x").with_label(set::CANCEL).with_tooltip("Stop the render and keep what it has drawn (Esc)").with_action(BAction::Set(OpKey::Render.path(), "cancel".to_string())),
                ));
            }
            return BView::row(row);
        }
        // `image.new` opens Blender's new-image popup, which phiber does not
        // have yet; `image.open` is the file browser.
        BView::row(vec![
            BView::Button(BControl::new().with_icon("file_new").with_label(set::NEW)),
            BView::Button(
                BControl::new().with_icon("folder_open").with_label(set::OPEN).with_action(BAction::Set(ModalKey::Open.path(), V_MODAL_FILE_BROWSER.to_string())),
            ),
        ])
    }
}

/// The right end: with an image, the slot, view layer and pass dropdowns and
/// the display channels (`template_image_layers`), and then the two controls
/// `IMAGE_HT_header` ends with — the gizmo display and the overlays.
#[derive(Clone, Copy, Debug)]
pub struct BImageDisplay {
    pub host: BHostType,
}

impl BImageDisplay {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BImageDisplay {
    fn build(&self, s: &BState) -> BView {
        let mut list = Vec::new();
        if BImageBlock::new(self.host).shows_render() {
            let (shot, _) = render_job::shot();
            let passes = set::passes(shot.as_ref());
            list.push(set::dropdown(s, self.host, "slot", "render", ImageKey::Slot, &set::SLOTS, set::SLOTS[0]));
            list.push(set::dropdown(s, self.host, "view_layer", "render_layer", ImageKey::ViewLayer, &[set::VIEW_LAYER], set::VIEW_LAYER));
            list.push(set::dropdown(s, self.host, "pass", "layer", ImageKey::Pass, &passes, set::COMBINED));
            list.push(set::dropdown(s, self.host, "channels", "image", ImageKey::Channels, &set::CHANNELS, set::CHANNEL_DEFAULT));
        }
        // The two Blender ends this header with. The overlay one was a toggle
        // that nothing read and nothing drew — `IMAGE_PT_overlay` is a
        // popover of seven boxes, and it is the same table the UV editor
        // reads (`filter::overlay`), because they are one space in Blender.
        list.push(set::toggle(s, ImageKey::Gizmo, "transform_gizmo", "Show Gizmo"));
        list.push(crate::v2::part::filter::overlay::view(s, self.host));
        BView::row(list)
    }
}
