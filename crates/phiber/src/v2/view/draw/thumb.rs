//! Thumbnail tile + wrapping grid renderer. `.blend` previews (and the Play
//! host's painted cards) are decoded
//! once per file (`part::file::preview`) and uploaded as a gpui
//! `RenderImage` (BGRA), cached per path for the life of the process.

use super::color::to_hsla;
use super::ctx::BRenderCtx;
use super::walk;
use super::flex::{BAlign, BEdge, BFill, BFlex, BJustify, BLen, BRadius};
use crate::v2::theme::BSurfaceTone;
use super::atom::icon_path;
use crate::v2::part::file::preview::quick_info;
use crate::v2::theme::BStyleRole;
use crate::v2::view::{BThumb, BView};
use gpui::prelude::*;
use gpui::{AnyElement, ClickEvent, ImageSource, RenderImage, SharedString, div, img, px, svg};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

thread_local! {
    static IMAGES: RefCell<HashMap<String, Option<Arc<RenderImage>>>> = RefCell::new(HashMap::new());
}

fn render_image(path: &str) -> Option<Arc<RenderImage>> {
    IMAGES.with(|cache| {
        if let Some(hit) = cache.borrow().get(path) {
            return hit.clone();
        }
        // The Play host's cards are painted (`part::play::thumb`); a
        // file's preview is read from the file.
        let pixels = match crate::v2::part::play::thumb::image(path) {
            Some(card) => Some((card.width, card.height, card.rgba)),
            None if path.starts_with("asset:") => crate::v2::part::project::thumb::image(path),
            // The Library's designs are drawn from the figure.
            None if path.starts_with("wear-check:") => crate::v2::part::project::check::image(path),
            None if path.starts_with("wear") => crate::v2::part::project::library::image(path),
            // A picture file is its own preview, read once and shrunk to the
            // largest card it is shown on.
            None if crate::v2::part::file::kind::BFileKind::of(path, false) == crate::v2::part::file::kind::BFileKind::Image => image::open(path).ok().map(|picture| {
                let small = picture.thumbnail(256, 256).to_rgba8();
                (small.width(), small.height(), small.into_raw())
            }),
            None => quick_info(path).and_then(|info| info.thumbnail.as_ref().map(|t| (t.width, t.height, t.rgba.to_vec()))),
        };
        let image = pixels.and_then(|(width, height, mut bgra)| {
            for px in bgra.chunks_exact_mut(4) {
                px.swap(0, 2);
            }
            let buf = image::RgbaImage::from_raw(width, height, bgra)?;
            let frame = image::Frame::new(buf);
            Some(Arc::new(RenderImage::new(smallvec::smallvec![frame])))
        });
        cache.borrow_mut().insert(path.to_string(), image.clone());
        image
    })
}

pub fn view(t: &BThumb, ctx: &BRenderCtx) -> AnyElement {
    let radius = px(ctx.style.radius_for(BStyleRole::Card));
    let size = px(t.size);
    let picture: AnyElement = match (!t.preview_path.is_empty())
        .then(|| render_image(&t.preview_path))
        .flatten()
    {
        Some(image) => img(ImageSource::Render(image)).size(size).rounded(radius).into_any_element(),
        None => svg()
            .path(icon_path(&t.icon))
            .size(px(t.size * 0.5))
            .text_color(to_hsla(ctx.style.colors.text_dim))
            .into_any_element(),
    };
    // The picture's card, edged so it reads on a ground as light as it is.
    let frame = BFlex::row()
        .width(BLen::Measured(t.size))
        .height(BLen::Measured(t.size))
        .align(BAlign::Center)
        .justify(BJustify::Center)
        .radius(BRadius::Role(BStyleRole::Card))
        .fill(BFill::Tone(BSurfaceTone::Card))
        .border(BEdge::All)
        .div(ctx)
        .child(picture);
    let selector = format!("thumb-{}", t.label);
    let mut tile = div()
        .id(SharedString::from(selector.clone()))
        .debug_selector(move || selector)
        .w(px(t.size + 2.0 * ctx.style.spacing.pad_sm))
        // Tiles in a row line up by their tops: a name that wraps makes its
        // tile taller, and centred, its picture rode up out of the row.
        .self_start()
        .p(px(ctx.style.spacing.pad_sm))
        .flex()
        .flex_col()
        .items_center()
        .gap(px(ctx.style.spacing.gap_xs))
        .rounded(radius)
        .cursor_pointer()
        .hover(|s| s.bg(to_hsla(ctx.style.host_item_hover(ctx.host))))
        .child(frame)
        .child(
            div()
                .w_full()
                .overflow_hidden()
                .text_center()
                .text_size(px(ctx.style.typography.body))
                .text_color(to_hsla(ctx.style.colors.text))
                .child(SharedString::from(t.label.clone())),
        );
    if !t.subtitle.is_empty() {
        tile = tile.child(
            div()
                .w_full()
                .overflow_hidden()
                .text_center()
                .text_size(px(ctx.style.typography.caption))
                .text_color(to_hsla(ctx.style.colors.text_dim))
                .child(SharedString::from(t.subtitle.clone())),
        );
    }
    if t.selected {
        tile = tile.bg(to_hsla(ctx.style.colors.row_selected));
    }
    if let Some(action) = t.action.clone() {
        let handler = ctx.handler.clone();
        tile = tile.on_click(move |_: &ClickEvent, window, app| {
            (Rc::clone(&handler))(&action, window, app);
        });
    }
    tile.into_any_element()
}

pub fn grid(list: &[BView], ctx: &BRenderCtx) -> AnyElement {
    let mut element = div()
        .id("thumb-grid")
        .flex()
        .flex_row()
        .flex_wrap()
        .content_start()
        .w_full()
        .h_full()
        .overflow_y_scroll()
        .gap(px(ctx.style.spacing.gap_sm))
        .p(px(ctx.style.spacing.pad_sm));
    for child in list {
        element = element.child(walk(child, ctx));
    }
    element.into_any_element()
}
