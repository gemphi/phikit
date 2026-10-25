//! The brand: the Phiber mark and the PHIBER wordmark.
//!
//! These are the only icons that are not one square glyph in the text colour.
//! The mark is Phi-Pill Bold's P with the i's dot laid flat under its bowl:
//! the dot starts where the bowl's terminal starts and ends where the bowl
//! ends, so it closes the bottom-right corner and the P reads as a B. It is
//! drawn as the 3D logo is — side, face and pill, three one-colour layers
//! stacked — because GPUI flattens an SVG to an alpha mask painted in one
//! colour.
//!
//! Every brand slug is resolved here so a view says `BView::icon("phiber")`
//! like any other icon and never knows about layers or aspect ratios.

use super::color::to_hsla;
use super::ctx::BRenderCtx;
use gpui::prelude::*;
use gpui::{AnyElement, div, px, svg};

/// The mark at header scale (menubar anchor, modal title).
pub const MARK: &str = "phiber";
/// The mark at splash/about scale.
pub const MARK_LARGE: &str = "phiber_splash";
/// The PHIBER letterforms, one colour, sized by height.
pub const WORDMARK: &str = "phiber_wordmark";
/// The wordmark at splash/about scale, set to the large mark's letter height.
pub const WORDMARK_LARGE: &str = "phiber_wordmark_large";

/// The mark's layers, bottom to top, in the logo's colours (the same three
/// as the 3D wordmark's). Each is a 700 × 700 square cropped to the art.
const MARK_LAYERS: [(&str, u32); 3] = [
    ("brand/phiber_mark_side.svg", 0x1E3A8A),
    ("brand/phiber_mark_body.svg", 0x3B82F6),
    ("brand/phiber_mark_pill.svg", 0xBFDBFE),
];

/// The PHIBER logo in the 3D font (Phi-Pill Bold): its deep side, its face
/// and the light pill in each upright, three layers stacked as the mark's
/// two tones are. The Welcome screen's name.
pub const WORDMARK_3D: &str = "phiber_wordmark_3d";
/// The three layers, bottom to top, each with its own colour — the logo's,
/// Phi-Pill Blue, kept in every palette as the mark keeps its tones.
const WORDMARK_3D_LAYERS: [(&str, u32); 3] = [
    ("brand/phiber_logo_side.svg", 0x1E3A8A),
    ("brand/phiber_logo_body.svg", 0x3B82F6),
    ("brand/phiber_logo_pill.svg", 0xBFDBFE),
];
/// The layers' shared viewBox is 3884 × 680, cropped to the art.
const WORDMARK_3D_ASPECT: f32 = 3884.0 / 680.0;
/// The Welcome window's hero: four brand units tall, the first thing the
/// window shows.
const WORDMARK_3D_SCALE: f32 = 4.0;

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

/// The Welcome backdrop's ground, by ladder STEP, not by hex — as every
/// shade of it is, so it follows the theme and its mode: the sky behind the
/// ridges (`backdrop`).
const BACKDROP_GROUND: u8 = 3;
/// The ridges' rungs, far to near: the far ridge a step off the ground,
/// the nearest five — the land deepening toward you, as the frames it was
/// traced from (`modal/splash/terrain`) haze toward the horizon.
const BACKDROP_RIDGE_STEPS: (u8, u8) = (4, 8);
/// How far the nearest ridge leans toward the accent: the land takes the
/// theme's own hue, a little more the nearer it is.
const BACKDROP_ACCENT_LEAN: f32 = 0.22;

/// `a` moved `t` of the way to `b`, in HSL — the hue the short way round.
fn lean(a: gpui::Hsla, b: gpui::Hsla, t: f32) -> gpui::Hsla {
    // The short way round the hue circle: a step in (-0.5, 0.5].
    let dh = (b.h - a.h + 0.5).rem_euclid(1.0) - 0.5;
    gpui::Hsla { h: (a.h + dh * t).rem_euclid(1.0), s: a.s + (b.s - a.s) * t, l: a.l + (b.l - a.l) * t, a: a.a }
}

/// The welcome window's ground: a landscape, ridge behind ridge — one of the
/// six traced off the welcome video's frames (`modal/splash/terrain`) — in
/// the theme's own shades. It replaced a single φ blown up behind the
/// window, which read as a letter left on the wall rather than a place.
pub fn backdrop(ctx: &BRenderCtx) -> AnyElement {
    // Resolved here, out of the paint closure: a shade is a theme question and
    // the closure's job is geometry.
    let ladder = ctx.style.colors.ladder;
    let accent = to_hsla(ctx.style.colors.accent);
    let terrain = &crate::v2::modal::splash::terrain::TERRAIN[0];
    let n = terrain.ridges.len().max(2) as f32 - 1.0;
    let (far, near) = BACKDROP_RIDGE_STEPS;
    let ridges: Vec<(&'static [f32], gpui::Hsla)> = terrain
        .ridges
        .iter()
        .enumerate()
        .map(|(k, ridge)| {
            let t = k as f32 / n;
            let step = far + ((near - far) as f32 * t).round() as u8;
            (&ridge.curve[..], lean(to_hsla(ladder.color(step)), accent, BACKDROP_ACCENT_LEAN * t))
        })
        .collect();
    let paint = move |bounds: gpui::Bounds<gpui::Pixels>, _: (), window: &mut gpui::Window, _: &mut gpui::App| {
        let (x0, y0) = (bounds.origin.x, bounds.origin.y);
        let (w, h) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
        for (curve, shade) in &ridges {
            let last = curve.len().max(2) as f32 - 1.0;
            let mut path = gpui::PathBuilder::fill();
            path.move_to(gpui::point(x0, y0 + px(h)));
            for (i, y) in curve.iter().enumerate() {
                path.line_to(gpui::point(x0 + px(w * i as f32 / last), y0 + px(h * y)));
            }
            path.line_to(gpui::point(x0 + px(w), y0 + px(h)));
            path.close();
            if let Ok(path) = path.build() {
                window.paint_path(path, *shade);
            }
        }
    };
    div()
        .id("welcome-backdrop")
        .debug_selector(|| "welcome-backdrop".to_string())
        .absolute()
        .inset_0()
        .overflow_hidden()
        .bg(to_hsla(ladder.color(BACKDROP_GROUND)))
        .child(gpui::canvas(|_, _, _| (), paint).size_full())
        .into_any_element()
}

/// True for the slugs this module draws itself (they have no square icon).
pub fn is_brand(name: &str) -> bool {
    matches!(name, MARK | MARK_LARGE | WORDMARK | WORDMARK_LARGE | WORDMARK_3D)
}

/// Renders a brand slug, or `None` when `name` is an ordinary icon.
pub fn view(name: &str, ctx: &BRenderCtx) -> Option<AnyElement> {
    let brand = ctx.style.dimensions.brand_size;
    match name {
        MARK => Some(mark(brand, "brand-mark", ctx)),
        MARK_LARGE => Some(mark(brand * LARGE_SCALE, "brand-mark-large", ctx)),
        WORDMARK => Some(wordmark(brand, ctx)),
        WORDMARK_LARGE => Some(wordmark(brand * LARGE_SCALE * WORDMARK_LARGE_SCALE, ctx)),
        WORDMARK_3D => Some(wordmark_3d(brand * WORDMARK_3D_SCALE, ctx)),
        _ => None,
    }
}

/// The mark, `size` px square, tagged with `selector` so each place it is
/// drawn can be measured on its own.
pub fn mark(size: f32, selector: &'static str, ctx: &BRenderCtx) -> AnyElement {
    layered(size, size, selector, tinted(MARK_LAYERS), ctx)
}

/// Brand layers as (asset path, colour) pairs.
fn tinted(layers: [(&str, u32); 3]) -> [(String, gpui::Hsla); 3] {
    layers.map(|(path, colour)| (path.to_string(), gpui::Hsla::from(gpui::rgb(colour))))
}

/// A brand drawing made of stacked one-colour svg layers, `w` × `h` px —
/// the mark and the 3D logo alike, so the two are drawn one way.
fn layered(w: f32, h: f32, selector: &'static str, layers: impl IntoIterator<Item = (String, gpui::Hsla)>, ctx: &BRenderCtx) -> AnyElement {
    use super::flex::{BFlex, BLen, BPlace};
    let size = |flex: BFlex| flex.width(BLen::Measured(w)).height(BLen::Measured(h));
    let base = size(BFlex::row()).keeps_size().place(BPlace::Anchor).div(ctx).id(selector);
    layers
        .into_iter()
        .fold(base.debug_selector(move || selector.to_string()), |base, (path, colour)| {
            base.child(size(BFlex::cell()).place(BPlace::Overlay).styled(svg().path(path), ctx).text_color(colour))
        })
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

/// The 3D PHIBER logo, `height` px tall: its three layers stacked.
pub fn wordmark_3d(height: f32, ctx: &BRenderCtx) -> AnyElement {
    layered(height * WORDMARK_3D_ASPECT, height, "brand-wordmark-3d", tinted(WORDMARK_3D_LAYERS), ctx)
}
