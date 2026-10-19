//! The titles on the top faces.
//!
//! A card's title is a div in Flat; here the cards are paths, so the name is
//! shaped and painted over the picture, as the socket labels are in
//! `paint/dot.rs`. A title is a BILLBOARD: upright and centred over its
//! solid's top, never laid into the face's perspective — GPUI shapes text in
//! the screen plane, and an upright name over a tilted slab reads better than
//! a skewed one would anyway. It follows the perspective in SIZE
//! (`TITLE_SCALE` of the header's projected height, held to `TITLE_PX`), and
//! is dropped where Flat drops its own — a top narrower than `TITLE_FLOOR` of
//! the canvas has no room for a name.
//!
//! Painted last, over every face and tube, because a name hidden behind a
//! nearer solid is a name you cannot read, and the reader is what the title
//! is for.

use super::camera::BSpatialCamera;
use super::paint::BSpatialPainting;
use super::set::{TITLE_FLOOR, TITLE_PX, TITLE_SCALE};
use crate::v2::view::draw::color::to_hsla;
use gpui::{App, Bounds, Pixels, SharedString, TextRun, Window, point, px};

impl BSpatialPainting {
    /// Every solid's title, centred over its projected top.
    pub fn titles(&self, camera: &BSpatialCamera, at: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
        let region_width = f32::from(at.size.width);
        for solid in &self.layout.solids {
            if solid.title.is_empty() {
                continue;
            }
            // The top face's corners on the canvas, and the box round them.
            let top: Option<Vec<(f32, f32, f32)>> =
                solid.footprint().iter().map(|&(x, y)| camera.project([x, y, solid.height])).collect();
            let Some(top) = top else { continue };
            let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
            for &(x, y, _) in &top {
                (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
            }
            let width = x1 - x0;
            if width < TITLE_FLOOR * region_width {
                continue;
            }
            let depth = top.iter().map(|p| p.2).sum::<f32>() / top.len() as f32;
            let header_px = self.header * camera.zoom * camera.dist / depth;
            let size = px((header_px * TITLE_SCALE).clamp(TITLE_PX.0, TITLE_PX.1));
            let mut color = self.ink.title;
            if solid.muted {
                color.3 *= self.ink.title_muted_alpha;
            }
            let run = TextRun {
                len: solid.title.len(),
                font: window.text_style().font(),
                color: to_hsla(color),
                background_color: None,
                underline: None,
                strikethrough: None,
            };
            let line = window.text_system().shape_line(SharedString::from(solid.title.clone()), size, &[run], Some(px(width - f32::from(size) * 0.5)));
            let line_height = size * 1.3;
            let origin = point(
                at.origin.x + px((x0 + x1) / 2.0) - line.width() / 2.0,
                at.origin.y + px((y0 + y1) / 2.0) - line_height / 2.0,
            );
            let _ = line.paint(origin, line_height, gpui::TextAlign::Left, None, window, cx);
        }
    }
}
