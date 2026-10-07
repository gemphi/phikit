//! The UV layout over the UV editor's square — `overlay_edit_uv.cc`: in
//! Edit Mode every face washed in the face colour, every edge light on a
//! dark outline (`edge_display_type` Outline) and every corner a vertex
//! dot; in Object Mode the edges alone, dim (`uv_shadow`). The maps are
//! the active and selected objects' (`part::image::uv::BUvShown`); `u`
//! runs right and `v` up the square.

use crate::v2::part::image::uv::BUvShown;
use crate::v2::scene::uv_edit::BUvEdit;
use crate::v2::theme::BUvColors;
use crate::v2::view::draw::color::to_hsla;
use gpui::{Bounds, PathBuilder, Pixels, Point, Window, fill, point, px, size};

/// An edge's width, and its outline's either side (`edge_display_type` Outline).
const EDGE: f32 = 1.0;
const OUTLINE: f32 = 3.0;
/// `UI_GetThemeValuef(TH_VERTEX_SIZE)`: a vertex dot's side.
const VERTEX: f32 = 3.0;

/// The layout, placed on the square it is drawn over, with the session's
/// own edits over it (`scene::uv_edit`): a moved corner is drawn where it
/// has been moved to and a selected one in the accent.
pub struct BUvPaint<'a> {
    pub shown: &'a BUvShown,
    pub colors: BUvColors,
    /// The 0..1 square on the canvas.
    pub square: Bounds<Pixels>,
    pub edit: &'a BUvEdit,
}

impl BUvPaint<'_> {
    fn at(&self, [u, v]: [f32; 2]) -> Point<Pixels> {
        let (w, h) = (f32::from(self.square.size.width), f32::from(self.square.size.height));
        point(self.square.origin.x + px(u * w), self.square.origin.y + px((1.0 - v) * h))
    }

    /// Corner `(map, corner)` where it now stands.
    fn corner(&self, map: usize, corner: usize) -> Option<Point<Pixels>> {
        let uv = self.shown.maps.get(map)?.uvs.get(corner)?;
        Some(self.at(self.edit.uv_of((map, corner), *uv)))
    }

    /// Every face's outline as one path, `stroked` or filled. The corners
    /// are walked by index rather than by value, because a face has to be
    /// drawn with its moved corners where they were moved to.
    fn faces(&self, mut builder: PathBuilder) -> Option<gpui::Path<Pixels>> {
        for (m, map) in self.shown.maps.iter().enumerate() {
            for face in 0..map.starts.len().saturating_sub(1) {
                let (from, to) = (map.starts[face] as usize, map.starts[face + 1] as usize);
                if to - from < 2 {
                    continue;
                }
                let Some(first) = self.corner(m, from) else { continue };
                builder.move_to(first);
                for c in from + 1..to {
                    if let Some(at) = self.corner(m, c) {
                        builder.line_to(at);
                    }
                }
                builder.close();
            }
        }
        builder.build().ok()
    }

    pub fn paint(&self, window: &mut Window) {
        let maps = &self.shown.maps;
        if maps.is_empty() {
            return;
        }
        if !self.shown.edit {
            if let Some(path) = self.faces(PathBuilder::stroke(px(EDGE))) {
                window.paint_path(path, to_hsla(self.colors.shadow));
            }
            return;
        }
        if let Some(path) = self.faces(PathBuilder::fill()) {
            window.paint_path(path, to_hsla(self.colors.face));
        }
        for (width, color) in [(OUTLINE, self.colors.outline), (EDGE, self.colors.edge)] {
            if let Some(path) = self.faces(PathBuilder::stroke(px(width))) {
                window.paint_path(path, to_hsla(color));
            }
        }
        // `TH_VERTEX_SELECT`: a selected corner is the accent, and bigger,
        // as Blender draws a selected UV vertex larger than an unselected
        // one.
        let (dot, picked) = (to_hsla(self.colors.vertex), to_hsla(self.colors.selected));
        // A pin outranks a selection in the drawing, as Blender's does: a
        // pinned corner is what an unwrap will hold, and that matters more
        // to the eye than whether it also happens to be selected.
        let stuck = to_hsla(self.colors.pinned);
        for (m, map) in self.shown.maps.iter().enumerate() {
            for c in 0..map.uvs.len() {
                let Some(at) = self.corner(m, c) else { continue };
                let on = self.edit.holds((m, c));
                let pin = self.edit.stuck((m, c));
                let side = if on || pin { VERTEX + 2.0 } else { VERTEX };
                let color = if pin { stuck } else if on { picked } else { dot };
                window.paint_quad(fill(Bounds::new(point(at.x - px(side / 2.0), at.y - px(side / 2.0)), size(px(side), px(side))), color));
            }
        }
    }
}
