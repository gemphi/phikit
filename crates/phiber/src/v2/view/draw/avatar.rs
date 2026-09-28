//! The avatar — a person's picture, round, as Discord's profile popout
//! leads with it. The image at `path` (PNG, JPEG, WebP… whatever `image`
//! reads), decoded once and kept; with no picture, the user glyph in a
//! ring of the card shade, so the place for the photo is always drawn.

use super::atom::icon_path;
use super::color::to_hsla;
use super::ctx::BRenderCtx;
use super::flex::{BAlign, BFlex, BJustify, BLen, BOverflow, BRadius, BFill};
use crate::v2::theme::BSurfaceTone;
use gpui::prelude::*;
use gpui::{AnyElement, ImageSource, RenderImage, img, px, svg};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

thread_local! {
    static PICTURES: RefCell<HashMap<String, Option<Arc<RenderImage>>>> = RefCell::new(HashMap::new());
}

/// The decoded picture at `path`, once; `None` when it cannot be read.
fn picture(path: &str) -> Option<Arc<RenderImage>> {
    PICTURES.with(|cache| {
        if let Some(hit) = cache.borrow().get(path) {
            return hit.clone();
        }
        let image = image::open(path).ok().map(|decoded| {
            let mut rgba = decoded.to_rgba8();
            // gpui's frames are BGRA.
            for px in rgba.chunks_exact_mut(4) {
                px.swap(0, 2);
            }
            Arc::new(RenderImage::new(smallvec::smallvec![image::Frame::new(rgba)]))
        });
        cache.borrow_mut().insert(path.to_string(), image.clone());
        image
    })
}

pub fn view(path: &str, size: f32, ctx: &BRenderCtx) -> AnyElement {
    let ring = BFlex::cell()
        .width(BLen::Measured(size))
        .height(BLen::Measured(size))
        .keeps_size()
        .align(BAlign::Center)
        .justify(BJustify::Center)
        .radius(BRadius::All(size / 2.0))
        .overflow(BOverflow::Clip)
        .fill(BFill::Tone(BSurfaceTone::Card))
        .div(ctx)
        .id(gpui::SharedString::from("avatar"))
        .debug_selector(|| "avatar".to_string());
    match (!path.is_empty()).then(|| picture(path)).flatten() {
        Some(image) => ring.child(img(ImageSource::Render(image)).size(px(size))).into_any_element(),
        None => ring
            .child(svg().path(icon_path("user")).size(px(size * 0.5)).text_color(to_hsla(ctx.style.colors.text_dim)))
            .into_any_element(),
    }
}
