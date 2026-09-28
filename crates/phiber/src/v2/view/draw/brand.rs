//! The brand: the Phiber mark and the PHIBER wordmark.
//!
//! These are the only icons that are not one square glyph in the text colour.
//! The mark is two tones, and GPUI flattens an SVG to an alpha mask painted in
//! one colour, so it is four files stacked, bottom to top: `phiber_cool`
//! (left bracket, right inner arc), `phiber_warm` (right bracket, left inner
//! hex, stem), `phiber_join` and `phiber_line` (the stem's core). The split
//! follows the colourway, not the geometry.
//!
//! Gaps at the stem: each bracket's outer line touches the stem at one end
//! and merges into it (the arc at its lower end, the hex at its upper end —
//! the mark's half-turn symmetry), while its inner line keeps a 0.4 gap
//! there. A merging end runs through the stem's outline to its core: the
//! arc is the outline's own colour; the hex's end is `phiber_join`, painted
//! over the outline and under the core, so no border shows between them.
//!
//! Every brand slug is resolved here so a view says `BView::icon("phiber")`
//! like any other icon and never knows about layers or aspect ratios.

use super::color::to_hsla;
use super::ctx::BRenderCtx;
use super::atom::icon_path;
use gpui::prelude::*;
use gpui::{AnyElement, div, px, svg};

/// The mark at header scale (menubar anchor, modal title).
pub const MARK: &str = "phiber";
/// The mark at splash/about scale.
pub const MARK_LARGE: &str = "phiber_splash";
/// The mark and wordmark on the Welcome banner, sized to fit it.
pub const MARK_BANNER: &str = "phiber_banner";
pub const WORDMARK_BANNER: &str = "phiber_wordmark_banner";
/// The PHIBER letterforms, one colour, sized by height.
pub const WORDMARK: &str = "phiber_wordmark";
/// The wordmark at splash/about scale, set to the large mark's letter height.
pub const WORDMARK_LARGE: &str = "phiber_wordmark_large";

/// The wordmark is wide, so it lives outside the square icon set.
pub const WORDMARK_PATH: &str = "brand/phiber_wordmark.svg";
/// `WORDMARK_PATH` viewBox is 42.832 × 15.6: cropped to the letters, so the
/// element's height is the letters' height.
const WORDMARK_ASPECT: f32 = 42.832 / 15.6;
/// The Welcome / About mark, as a multiple of the header mark: 64 px against
/// the header's 22.4. The banner is a greeting, not a poster — at 84 px it
/// dominated the screen it was introducing.
const LARGE_SCALE: f32 = 64.0 / 22.4;
/// Beside the large mark the wordmark is set to this share of its height, so
/// the letters stand exactly as tall as the mark's silhouette — the lockup on
/// the brand page. Both assets are cropped to their art: the mark's art is
/// 0.950 of its box, the letters 0.987 of theirs, and 0.950 / 0.987 = 0.96.
const WORDMARK_LARGE_SCALE: f32 = 0.96;

/// The Welcome wordmark stands at two thirds of the mark's height, so the
/// name reads as the mark's caption rather than its twin.
const WORDMARK_BANNER_SCALE: f32 = 2.0 / 3.0;

/// The Welcome backdrop: the mark itself, laid flat (stem level, arc on top
/// — `brand/welcome_bg_*`, the top-bar layers turned from 30° to −90° at the
/// 1.25 mark scale so nothing is clipped), scaled so its full width spans
/// the window's width — the stem runs edge to edge — and centred; on a wide
/// window the outer arc and hex run off the top and bottom.
/// Where a bracket touches the stem it merges in: the arc's right end and
/// the hex's left end run to the stem, and their inner partners keep a gap;
/// the touching ends pass through the stem's outline to its core (the arc
/// is the outline's own shade; the hex's join is its own layer, painted
/// over the outline and under the core), so no border shows between them.
/// Layers are shades of the sphere colour #3B4657 over black — cool 40%,
/// warm 60%, join 40%, line 30% — on a 30% ground, so it reads as texture
/// behind the form. A canvas measures the window, so the mark keeps
/// its proportions at every size.
const BACKDROP_BG: u32 = 0x12151A;
/// The flat mark's art width in its 24-unit box (it is centred in the box).
const BACKDROP_ART_WIDTH: f32 = 21.76;
const BACKDROP_LAYERS: [(&str, u32); 4] = [
    ("brand/welcome_bg_cool.svg", 0x232A34),
    ("brand/welcome_bg_warm.svg", 0x2F3846),
    ("brand/welcome_bg_join.svg", 0x232A34),
    ("brand/welcome_bg_line.svg", 0x1E232C),
];

pub fn backdrop() -> AnyElement {
    let paint = |bounds: gpui::Bounds<gpui::Pixels>, _: (), window: &mut gpui::Window, cx: &mut gpui::App| {
        let w = f32::from(bounds.size.width);
        let side = px(w * 24.0 / BACKDROP_ART_WIDTH);
        let square = gpui::Bounds::centered_at(bounds.center(), gpui::size(side, side));
        for (name, shade) in BACKDROP_LAYERS {
            // A missing asset leaves the ground bare; nothing else to do.
            let _ = window
                .paint_svg(
                    square,
                    name.into(),
                    None,
                    gpui::TransformationMatrix::unit(),
                    gpui::rgb(shade).into(),
                    cx,
                );
        }
    };
    div()
        .id("welcome-backdrop")
        .debug_selector(|| "welcome-backdrop".to_string())
        .absolute()
        .inset_0()
        .overflow_hidden()
        .bg(gpui::rgb(BACKDROP_BG))
        .child(gpui::canvas(|_, _, _| (), paint).size_full())
        .into_any_element()
}

/// True for the slugs this module draws itself (they have no square icon).
pub fn is_brand(name: &str) -> bool {
    matches!(name, MARK | MARK_LARGE | WORDMARK | WORDMARK_LARGE | MARK_BANNER | WORDMARK_BANNER)
}

/// Renders a brand slug, or `None` when `name` is an ordinary icon.
pub fn view(name: &str, ctx: &BRenderCtx) -> Option<AnyElement> {
    let brand = ctx.style.dimensions.brand_size;
    match name {
        MARK => Some(mark(brand, "brand-mark", ctx)),
        MARK_LARGE => Some(mark(brand * LARGE_SCALE, "brand-mark-large", ctx)),
        MARK_BANNER => Some(mark(banner_size(ctx), "brand-mark-banner", ctx)),
        WORDMARK_BANNER => Some(wordmark(banner_size(ctx) * WORDMARK_BANNER_SCALE, ctx)),
        WORDMARK => Some(wordmark(brand, ctx)),
        WORDMARK_LARGE => Some(wordmark(brand * LARGE_SCALE * WORDMARK_LARGE_SCALE, ctx)),
        _ => None,
    }
}

/// The Welcome banner is an ordinary header strip, so its mark and wordmark
/// are the header's own size: the greeting sits at the height of every other
/// heading rather than above it.
fn banner_size(ctx: &BRenderCtx) -> f32 {
    ctx.style.dimensions.brand_size
}

/// The two-tone mark, `size` px square, tagged with `selector` so each place
/// it is drawn can be measured on its own.
pub fn mark(size: f32, selector: &'static str, ctx: &BRenderCtx) -> AnyElement {
    let layer = |name: &str, tint| {
        svg()
            .path(icon_path(name))
            .absolute()
            .inset_0()
            .size(px(size))
            .text_color(to_hsla(tint))
    };
    div()
        .id(selector)
        .debug_selector(move || selector.to_string())
        .relative()
        .size(px(size))
        .flex_shrink_0()
        .child(layer("phiber_cool", ctx.style.colors.brand_cool))
        .child(layer("phiber_warm", ctx.style.colors.brand_warm))
        .child(layer("phiber_join", ctx.style.colors.brand_cool))
        .child(layer("phiber_line", ctx.style.colors.brand_cool))
        .into_any_element()
}

/// The wordmark, `height` px tall, in the text colour.
pub fn wordmark(height: f32, ctx: &BRenderCtx) -> AnyElement {
    svg()
        .path(WORDMARK_PATH)
        .h(px(height))
        .w(px(height * WORDMARK_ASPECT))
        .flex_shrink_0()
        .text_color(to_hsla(ctx.style.colors.text))
        .into_any_element()
}
