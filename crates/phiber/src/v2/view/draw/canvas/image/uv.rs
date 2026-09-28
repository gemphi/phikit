//! The UV layout over the UV editor's square — `overlay_edit_uv.cc`: in
//! Edit Mode every face washed in the face colour, every edge light on a
//! dark outline (`edge_display_type` Outline) and every corner a vertex
//! dot; in Object Mode the edges alone, dim (`uv_shadow`). The maps are
//! the active and selected objects' (`part::image::uv::BUvShown`); `u`
//! runs right and `v` up the square.

use crate::phiphx::scene::SceneUvMap;
use crate::v2::part::image::uv::BUvShown;
use crate::v2::theme::BUvColors;
use crate::v2::view::draw::color::to_hsla;
use gpui::{Bounds, PathBuilder, Pixels, Point, Window, fill, point, px, size};

/// An edge's width, and its outline's either side (`edge_display_type` Outline).
const EDGE: f32 = 1.0;
const OUTLINE: f32 = 3.0;
/// `UI_GetThemeValuef(TH_VERTEX_SIZE)`: a vertex dot's side.
const VERTEX: f32 = 3.0;

/// The layout, placed on the square it is drawn over.
pub struct BUvPaint<'a> {
    pub shown: &'a BUvShown,
    pub colors: BUvColors,
    /// The 0..1 square on the canvas.
    pub square: Bounds<Pixels>,
}

impl BUvPaint<'_> {
    fn at(&self, [u, v]: [f32; 2]) -> Point<Pixels> {
        let (w, h) = (f32::from(self.square.size.width), f32::from(self.square.size.height));
        point(self.square.origin.x + px(u * w), self.square.origin.y + px((1.0 - v) * h))
    }

    /// Every face's outline as one path, `stroked` or filled.
    fn faces(&self, maps: &[std::sync::Arc<SceneUvMap>], mut builder: PathBuilder) -> Option<gpui::Path<Pixels>> {
        for face in maps.iter().flat_map(|map| map.faces()).filter(|face| face.len() >= 2) {
            builder.move_to(self.at(face[0]));
            for corner in &face[1..] {
                builder.line_to(self.at(*corner));
            }
            builder.close();
        }
        builder.build().ok()
    }

    pub fn paint(&self, window: &mut Window) {
        let maps = &self.shown.maps;
        if maps.is_empty() {
            return;
        }
        if !self.shown.edit {
            if let Some(path) = self.faces(maps, PathBuilder::stroke(px(EDGE))) {
                window.paint_path(path, to_hsla(self.colors.shadow));
            }
            return;
        }
        if let Some(path) = self.faces(maps, PathBuilder::fill()) {
            window.paint_path(path, to_hsla(self.colors.face));
        }
        for (width, color) in [(OUTLINE, self.colors.outline), (EDGE, self.colors.edge)] {
            if let Some(path) = self.faces(maps, PathBuilder::stroke(px(width))) {
                window.paint_path(path, to_hsla(color));
            }
        }
        let dot = to_hsla(self.colors.vertex);
        for corner in maps.iter().flat_map(|map| map.uvs.iter()) {
            let at = self.at(*corner);
            window.paint_quad(fill(Bounds::new(point(at.x - px(VERTEX / 2.0), at.y - px(VERTEX / 2.0)), size(px(VERTEX), px(VERTEX))), dot));
        }
    }
}
