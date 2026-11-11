//! A TrueType / OpenType face through `ttf-parser`, read the way FreeType
//! hands it to Blender: the face metrics FreeType fills (`ascender` and
//! `descender` from `hhea`, the `head` bounding box) and each glyph's
//! unscaled outline as on / conic / cubic points.
//!
//! `ttf-parser` reports a TrueType contour with its implied on-curve
//! midpoints made explicit; Blender makes the same midpoints itself (the
//! "virtual conic on-curve points" of `blf_glyph_to_curves`), with the
//! same handles, so the curves agree.

use super::{BFontMetrics, BGlyph, BTag};

/// `BLF_get_vfont_metrics` for an sfnt face.
pub fn metrics(data: &[u8]) -> Option<BFontMetrics> {
    let face = ttf_parser::Face::parse(data, 0).ok()?;
    let hhea = face.tables().hhea;
    // FreeType falls back to OS/2 when `hhea` has neither (`sfobjs.c`).
    let (asc, desc) = match (hhea.ascender, hhea.descender) {
        (0, 0) => (face.typographic_ascender().unwrap_or(0), face.typographic_descender().unwrap_or(0)),
        pair => pair,
    };
    let bbox = face.global_bounding_box();
    Some(BFontMetrics::from_face(asc as i32, desc as i32, bbox.y_max as i32, bbox.y_min as i32))
}

/// The glyph for `unicode`, `None` when the face maps no glyph to it.
pub fn glyph(data: &[u8], unicode: u32) -> Option<BGlyph> {
    let face = ttf_parser::Face::parse(data, 0).ok()?;
    let id = face.glyph_index(char::from_u32(unicode)?)?;
    if id.0 == 0 {
        return None;
    }
    let mut b = Builder::default();
    face.outline_glyph(id, &mut b);
    b.close();
    Some(BGlyph { contours: b.contours, advance: face.glyph_hor_advance(id).unwrap_or(0) as f64 })
}

#[derive(Default)]
struct Builder {
    contours: Vec<Vec<(f64, f64, BTag)>>,
    open: bool,
}

impl Builder {
    fn push(&mut self, x: f32, y: f32, tag: BTag) {
        if let Some(c) = self.contours.last_mut() {
            c.push((x as f64, y as f64, tag));
        }
    }

    /// The contour's closing point, when it repeats the first, is not a
    /// point of FreeType's outline (TrueType closes implicitly).
    fn close(&mut self) {
        if !self.open {
            return;
        }
        self.open = false;
        let Some(c) = self.contours.last_mut() else { return };
        if c.len() > 1 {
            let (first, last) = (c[0], c[c.len() - 1]);
            if first.0 == last.0 && first.1 == last.1 && last.2 == BTag::On {
                c.pop();
            }
        }
    }
}

impl ttf_parser::OutlineBuilder for Builder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.close();
        self.contours.push(Vec::new());
        self.open = true;
        self.push(x, y, BTag::On);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.push(x, y, BTag::On);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.push(x1, y1, BTag::Conic);
        self.push(x, y, BTag::On);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.push(x1, y1, BTag::Cubic);
        self.push(x2, y2, BTag::Cubic);
        self.push(x, y, BTag::On);
    }

    fn close(&mut self) {
        Builder::close(self);
    }
}
