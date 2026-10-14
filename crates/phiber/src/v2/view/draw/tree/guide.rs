//! The hierarchy lines in a tree row: one per indent unit of the row's depth,
//! before its arrow slot, in the colour of the *other* zebra band.
//!
//! Each row draws its own run (`BTreePart::Indent`), so a line exists only
//! where a row is nested — a root row has none, a child one, a grandchild
//! two — and ends with the row it belongs to. The lines used to be one
//! backdrop of the deepest row's depth behind the whole column, in the accent
//! and the text tone, which put the same loud columns through every row and
//! told nothing about any one of them.
//!
//! The colour is the opposite band composited over the editor's body — the
//! alternate row's background on a flat row, the flat row's on an alternate
//! one — so a line is subtle against the row it crosses and is never a third
//! colour in the list.

use super::super::color::to_hsla;
use super::super::ctx::BRenderCtx;
use super::super::flex::BFill;
use crate::v2::view::prim::BRowShade;

/// The fill of the hierarchy lines in a row of `shade`: the opposite band's
/// tint over the editor's body, as that band's rows actually paint.
pub(super) fn guide_fill(shade: BRowShade, ctx: &BRenderCtx) -> BFill {
    let body = to_hsla(ctx.style.host_body_surface(ctx.host).top).to_rgb();
    let Some(band) = shade.opposite_band().fill(ctx.style) else { return BFill::Clear };
    let tint = to_hsla(band).to_rgb();
    let a = band.3;
    let over = |b: f32, t: f32| b * (1.0 - a) + t * a;
    BFill::Swatch([over(body.r, tint.r), over(body.g, tint.g), over(body.b, tint.b), 1.0])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::state::BState;
    use crate::v2::theme::{BPaletteName, BThemeMode, resolve_ui_style, ui_style_defaults_in};
    use crate::v2::host::BHostType;
    use crate::v2::view::draw::ctx::{BActionHandler, BRenderCtx, BResizeHandler};
    use std::rc::Rc;

    fn ctx<R>(f: impl FnOnce(&BRenderCtx) -> R) -> R {
        let style = resolve_ui_style(ui_style_defaults_in(BPaletteName::Sphere, BThemeMode::Dark), &BState::new());
        let s = BState::new();
        let handler: BActionHandler = Rc::new(|_, _, _| {});
        let resize: BResizeHandler = Rc::new(|_, _, _, _, _, _, _| {});
        f(&BRenderCtx::new(&style, &s, handler, resize).with_host(BHostType::Outliner))
    }

    fn rgb(fill: BFill) -> [f32; 3] {
        match fill {
            BFill::Swatch([r, g, b, _]) => [r, g, b],
            other => panic!("a swatch, got {other:?}"),
        }
    }

    #[test]
    fn a_rows_lines_are_the_other_bands_colour_and_differ_from_its_own() {
        ctx(|c| {
            let on_flat = rgb(guide_fill(BRowShade::Flat, c));
            let on_alt = rgb(guide_fill(BRowShade::Alt, c));
            assert_ne!(on_flat, on_alt, "the two bands' lines differ");
            // The line on a flat row IS the alternate row's painted colour, and
            // vice versa: no third colour.
            let body = to_hsla(c.style.host_body_surface(c.host).top).to_rgb();
            let painted = |shade: BRowShade| {
                let band = shade.fill(c.style).unwrap();
                let t = to_hsla(band).to_rgb();
                let a = band.3;
                [body.r * (1.0 - a) + t.r * a, body.g * (1.0 - a) + t.g * a, body.b * (1.0 - a) + t.b * a]
            };
            let close = |x: [f32; 3], y: [f32; 3]| x.iter().zip(y).all(|(p, q)| (p - q).abs() < 1e-4);
            assert!(close(on_flat, painted(BRowShade::Alt)));
            assert!(close(on_alt, painted(BRowShade::Flat)));
            // Selection keeps the flat band's line.
            assert!(close(rgb(guide_fill(BRowShade::Selected, c)), painted(BRowShade::Flat)));
        });
    }
}
