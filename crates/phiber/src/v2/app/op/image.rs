//! `image.clipboard_copy` — Image ▸ Copy (Ctrl C): the shown image, as the
//! Image editor paints it, onto the system clipboard as a PNG.

use super::super::BPhiberApp;
use crate::v2::part::image::shown::BShownImage;
use crate::v2::state::log;
use gpui::Context;

impl BPhiberApp {
    pub(super) fn image_copy(&mut self, cx: &mut Context<Self>) {
        let Some(picture) = BShownImage::now(&self.state).picture() else {
            log::push("No image to copy".to_string());
            return;
        };
        let bytes = picture.png();
        let id = bytes.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ u64::from(*b)).wrapping_mul(0x0100_0000_01b3));
        cx.write_to_clipboard(gpui::ClipboardItem::new_image(&gpui::Image { format: gpui::ImageFormat::Png, bytes, id }));
        log::push(format!("Copied '{}' ({} × {})", picture.name, picture.w, picture.h));
    }
}
