//! Playing a sheet's marks into a gpui window: a fill is a quad, a blend
//! a quad with a gradient, glass a backdrop, a line
//! of text is shaped and painted, an icon is an svg from the app's
//! assets, a picture is one of the album's, and what is drawn between a clip and its end is kept inside
//! the clip's rectangle. The calls are those of gpui's `painting` and
//! `input` examples (`gpu/examples`).

use gpui::{App, BackdropStyle, Bounds, ContentMask, Corners, Font, FontWeight, Hsla, Pixels, Rgba, SharedString, TextAlign, TextRun, TransformationMatrix, Window, font, linear_color_stop, linear_gradient, point, px, quad, size, transparent_black};

use super::album::Album;
use super::{FACE, warn};
use crate::pui::core::{PColor, PIcon, PMark, PRect, PSize};
use crate::theme::PTheme;

/// The theme's tokens text is set with.
pub const SIZE: &str = "text.size";
pub const LINE: &str = "text.line";
pub const WEIGHT: &str = "text.weight";
pub const HEAD: &str = "head.size";
pub const HEAD_LINE: &str = "head.line";

/// The type text is set in: a face, a size and a line height, as the
/// theme gives them.
#[derive(Clone)]
pub struct Type {
    font: Font,
    size: Pixels,
    line: Pixels,
    /// A heading's size and line height.
    head: (Pixels, Pixels),
}

impl Type {
    pub fn of(theme: &dyn PTheme) -> Type {
        let weight = theme.size(WEIGHT).map_or(FontWeight::default(), FontWeight);
        let font = Font { weight, ..font(theme.word(FACE).unwrap_or_default().to_string()) };
        let size = |token: &str| px(theme.size(token).unwrap_or_default());
        Type { font, size: size(SIZE), line: size(LINE), head: (size(HEAD), size(HEAD_LINE)) }
    }

    /// What sets this type apart from another: its face, its weight and
    /// its size, as a key for what is kept of it.
    pub fn key(&self) -> String {
        format!("{}/{}/{}", self.font.family, self.font.weight.0, self.size.as_f32())
    }

    fn run(&self, text: &str, color: Hsla) -> TextRun {
        TextRun { len: text.len(), font: self.font.clone(), color, background_color: None, underline: None, strikethrough: None }
    }

    /// How wide a line of text is set.
    pub fn width(&self, text: &str, window: &Window) -> f32 {
        match text.is_empty() {
            true => 0.0,
            false => window.text_system().shape_line(SharedString::from(text.to_string()), self.size, &[self.run(text, transparent_black())], None).width.as_f32(),
        }
    }
}

fn tint(color: &PColor) -> Hsla {
    Rgba { r: color.r, g: color.g, b: color.b, a: color.a }.into()
}

fn bound(rect: &PRect) -> Bounds<Pixels> {
    Bounds::new(point(px(rect.origin.x), px(rect.origin.y)), size(px(rect.size.width), px(rect.size.height)))
}

/// How many marks come before the end of a clip that has just begun,
/// counting the clips begun and ended inside it. All of them, when its
/// end is missing.
pub fn inside(mark: &[PMark]) -> usize {
    let mut depth = 0usize;
    for (index, mark) in mark.iter().enumerate() {
        match (mark, depth) {
            (PMark::Unclip, 0) => return index,
            (PMark::Unclip, _) => depth -= 1,
            (PMark::Clip(_), _) => depth += 1,
            _ => {}
        }
    }
    mark.len()
}

/// Paint every mark, the first the lowest.
pub fn play(mark: &[PMark], face: &Type, album: &Album, window: &mut Window, cx: &mut App) {
    let mut next = 0;
    while let Some(now) = mark.get(next) {
        next += 1;
        match now {
            PMark::Fill { rect, radius, color } => window.paint_quad(quad(bound(rect), px(*radius), tint(color), px(0.), transparent_black(), Default::default())),
            PMark::Blend { rect, radius, from, to, turn } => {
                let ground = linear_gradient(*turn, linear_color_stop(tint(from), 0.0), linear_color_stop(tint(to), 1.0));
                window.paint_quad(quad(bound(rect), px(*radius), ground, px(0.), transparent_black(), Default::default()));
            }
            PMark::Glass { rect, radius, glass } => {
                let style = BackdropStyle { blur_radius: glass.blur, saturation: glass.saturation, tint: tint(&glass.tint), tint_amount: glass.share, lens: glass.lens, edge: tint(&glass.edge), edge_strength: glass.rim };
                window.paint_backdrop(bound(rect), Corners::all(px(*radius)), style);
            }
            PMark::Text { text, .. } | PMark::Head { text, .. } if text.is_empty() => {}
            PMark::Text { at, text, color } | PMark::Head { at, text, color } => {
                let (size, high) = match now {
                    PMark::Head { .. } => face.head,
                    _ => (face.size, face.line),
                };
                let line = window.text_system().shape_line(SharedString::from(text.clone()), size, &[face.run(text, tint(color))], None);
                let drawn = line.paint(point(px(at.x), px(at.y)), high, TextAlign::Left, None, window, cx);
                drawn.err().into_iter().for_each(|error| warn(&format!("a line of text could not be painted: {error:#}")));
            }
            PMark::Icon { name, rect, color } => {
                let path = PIcon::new(name, PSize::default()).path();
                let drawn = window.paint_svg(bound(rect), path.into(), None, TransformationMatrix::unit(), tint(color), cx);
                drawn.err().into_iter().for_each(|error| warn(&format!("the icon {name} could not be painted: {error:#}")));
            }
            // A picture that is not ready yet is drawn once it is.
            PMark::Picture { name, rect, radius, fit } => {
                if let Some(picture) = album.get(name, cx) {
                    let natural = picture.size(0);
                    let (shown, lie) = fit.place(*rect, PSize { width: natural.width.0 as f32, height: natural.height.0 as f32 });
                    let drawn = window.paint_image(bound(&shown), bound(&lie), Corners::all(px(*radius)), picture, 0, false);
                    drawn.err().into_iter().for_each(|error| warn(&format!("the picture {name} could not be painted: {error:#}")));
                }
            }
            PMark::Clip(rect) => {
                let held = &mark[next..next + inside(&mark[next..])];
                window.with_content_mask(Some(ContentMask { bounds: bound(rect) }), |window| play(held, face, album, window, cx));
                next += held.len() + 1;
            }
            PMark::Unclip => {}
        }
    }
}
