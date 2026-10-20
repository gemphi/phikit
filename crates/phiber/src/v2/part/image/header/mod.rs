//! The Image editor's header, right of the menus — `IMAGE_HT_header`. Two
//! parts, because Blender puts a spacer between them: the image data-block
//! in the centre (`template_ID`), and at the right end the gizmo and overlay
//! toggles, the render slot, view layer and pass (`template_image_layers`)
//! and the display channels.

pub mod set;

use crate::v2::host::BHostType;
use crate::v2::scene::render_job;
use crate::v2::state::{BKey as _, BState, FileKey, ImageKey, MenuKey, ModalKey, OpKey, val::V_MODAL_NEW_IMAGE};
use crate::v2::view::prim::{BAction, BControl, BPick};
use crate::v2::view::{BView, BViewItem};

/// `template_ID(sima, "image", new="image.new", open="image.open")`: the
/// image shown — the Render Result once there is one, or one of the
/// session's (`shown::BShownImage`) — with New and Open beside it.
#[derive(Clone, Copy, Debug)]
pub struct BImageBlock {
    pub host: BHostType,
}

impl BImageBlock {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }

    /// The Render Result exists as a data-block once a render has been
    /// asked for — in any of its slots; the UV editor never shows it.
    pub fn shows_render(&self) -> bool {
        self.host == BHostType::Image && crate::v2::scene::render_slot::BRenderSlot::any_filled()
    }
}

impl BViewItem for BImageBlock {
    fn build(&self, s: &BState) -> BView {
        let shown = super::shown::BShownImage::now(s);
        let names = super::shown::BShownImage::name_list();
        let mut row = Vec::new();
        // The browse dropdown and the name: the Render Result while a slot
        // holds a render, and the session's images (`template_ID`).
        if self.host == BHostType::Image && !names.is_empty() {
            let pick = BPick::new(MenuKey::Top(crate::v2::area::current::menu_scope(s, self.host), "image").path(), BControl::new().with_icon("image").with_label(shown.name()).with_tooltip("Image"));
            row.push(names.iter().fold(pick, |pick, name| pick.item(name, *name == shown.name(), BAction::Set(ImageKey::Shown.path(), name.clone()))).view(s));
            // While the render runs, its progress, a Pause and a Cancel
            // beside the image it is drawing — Blender keeps the job's bar
            // and X in the status bar (`WM_jobs_stop`), which stays too;
            // here it is also where the eye is. The pause is Phiber's own
            // (`render_job::pause`): held, the bar says so and the button
            // reads Resume.
            if shown == super::shown::BShownImage::RenderResult
                && render_job::rendering()
                && let Some(shot) = render_job::shot().0
            {
                let held = render_job::paused();
                row.push(BView::progress_bar(shot.fraction(), if held { set::PAUSED } else { set::RENDERING }));
                row.push(BView::Button(
                    BControl::new()
                        .with_icon(if held { "play" } else { "pause" })
                        .with_label(if held { set::RESUME } else { set::PAUSE })
                        .with_tooltip(if held { "Let the render go on from the sample it stopped at" } else { "Hold the render after this sample; the image stays up as it stands" })
                        .with_action(BAction::Set(OpKey::Render.path(), "pause".to_string())),
                ));
                row.push(BView::Button(
                    BControl::new().with_icon("x").with_label(set::CANCEL).with_tooltip("Stop the render and keep what it has drawn (Esc)").with_action(BAction::Set(OpKey::Render.path(), "cancel".to_string())),
                ));
            }
        }
        // `image.new` (its dialog) and `image.open` (the file browser).
        row.push(BView::Button(BControl::new().with_icon("file_new").with_label(set::NEW).with_tooltip("New Image").with_action(BAction::Set(ModalKey::Open.path(), V_MODAL_NEW_IMAGE.to_string()))));
        row.push(BView::Button(
            BControl::new().with_icon("folder_open").with_label(set::OPEN).with_tooltip("Open Image").with_action(BAction::Set(FileKey::Purpose.path(), crate::v2::part::file::purpose::BFilePurpose::OpenImage.slug().to_string())),
        ));
        BView::row(row)
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
        if BImageBlock::new(self.host).shows_render() && super::shown::BShownImage::now(s) == super::shown::BShownImage::RenderResult {
            let (shot, _) = render_job::shot();
            let passes = set::passes(shot.as_ref());
            list.push(set::dropdown(s, self.host, "slot", "render", ImageKey::Slot, &crate::v2::scene::render_slot::BRenderSlot::NAME_LIST, crate::v2::scene::render_slot::BRenderSlot::NAME_LIST[0]));
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
