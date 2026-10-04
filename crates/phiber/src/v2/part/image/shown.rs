//! Which image the Image editor shows (`sima->image`): the Render Result,
//! or one of the session's images by name (`scene::image_list`) — what the
//! header's `template_ID` picks and the canvas paints — and the Image
//! menu's operators run on it: `image.invert`, `image.flip`,
//! `image.rotate_orthogonal` (`BPixelOp`), `image.reload`. On the Render
//! Result they change the shown slot's picture, as Blender's change its
//! display buffer.

use crate::v2::scene::image_list::{BImage, BImageList, BPixelOp};
use crate::v2::scene::render_job;
use crate::v2::state::{BKey as _, BPath, BState, ImageKey, log, rule::BRule};

/// `IMAGE_HT_header`'s data-block name for the render.
pub const RENDER_RESULT: &str = "Render Result";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BShownImage {
    RenderResult,
    Named(String),
}

impl BShownImage {
    pub fn now(s: &BState) -> Self {
        match s.get(ImageKey::Shown).filter(|n| !n.is_empty() && *n != RENDER_RESULT) {
            Some(name) => Self::Named(name.to_string()),
            None => Self::RenderResult,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::RenderResult => RENDER_RESULT,
            Self::Named(name) => name,
        }
    }

    /// The names the header's dropdown offers: the Render Result while any
    /// slot holds a render, then the session's images.
    pub fn name_list() -> Vec<String> {
        let mut out = Vec::new();
        if crate::v2::scene::render_slot::BRenderSlot::any_filled() {
            out.push(RENDER_RESULT.to_string());
        }
        out.extend(BImageList::name_list());
        out
    }

    /// The shown picture as an image: the slot's render — finished or as
    /// far as it got — or the named image; `None` with nothing to show.
    pub fn picture(&self) -> Option<BImage> {
        match self {
            Self::RenderResult => {
                let shot = render_job::shot().0.filter(|s| s.has_image())?;
                Some(BImage { name: RENDER_RESULT.into(), path: None, w: shot.w, h: shot.h, bgra: shot.bgra })
            }
            Self::Named(name) => BImageList::get(name),
        }
    }

    /// A change counter for the painter: the render's sequence, or the list's.
    pub fn seq(&self) -> u32 {
        match self {
            Self::RenderResult => render_job::shot().1,
            Self::Named(_) => BImageList::seq(),
        }
    }

    /// Run `op` over the shown picture; `false` when there is none, or a
    /// render is under way in the shown slot.
    pub fn run(&self, op: BPixelOp) -> bool {
        match self {
            Self::RenderResult => {
                let Some(mut shot) = render_job::shot().0.filter(|s| s.has_image() && s.done) else { return false };
                (shot.w, shot.h) = op.apply(shot.w, shot.h, &mut shot.bgra);
                // The floats and passes describe the picture before; the
                // display pixels are what the operator changed.
                shot.display.clear();
                shot.linear.clear();
                render_job::show(shot)
            }
            Self::Named(name) => BImageList::with(name, |image| image.run(op)),
        }
    }
}

/// `image.op`: an Image-menu operator over the shown image — a pixel
/// operator's slug, or `reload` — taken and cleared.
pub struct BImageOpRule;

impl BRule for BImageOpRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == ImageKey::Op.path()
    }
    fn after(&self, path: &BPath, value: &str, s: &mut BState) {
        s.remove(path);
        let shown = BShownImage::now(s);
        if value == "reload" {
            let mut outcome = Ok(());
            let found = match &shown {
                BShownImage::Named(name) => BImageList::with(name, |image| outcome = image.reload()),
                BShownImage::RenderResult => false,
            };
            match (found, outcome) {
                (false, _) => log::push("Reload: the Render Result has no file to read".to_string()),
                (true, Err(why)) => log::push(format!("Cannot reload: {why}")),
                (true, Ok(())) => {}
            }
            return;
        }
        let Some(op) = BPixelOp::of_slug(value) else { return };
        if !shown.run(op) {
            log::push("No image to change".to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shown_image_is_the_render_result_unless_named() {
        let mut s = BState::new();
        assert_eq!(BShownImage::now(&s), BShownImage::RenderResult);
        s.set(ImageKey::Shown, RENDER_RESULT);
        assert_eq!(BShownImage::now(&s), BShownImage::RenderResult);
        s.set(ImageKey::Shown, "photo.png");
        assert_eq!(BShownImage::now(&s).name(), "photo.png");
        assert!(BShownImage::Named("nobody".into()).picture().is_none());
        assert!(!BShownImage::Named("nobody".into()).run(BPixelOp::Flip(true, false)));
    }
}
