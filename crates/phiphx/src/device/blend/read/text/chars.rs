//! Characters for the layout: each style's font (`vfont_from_charinfo`),
//! its glyphs made once (`vfont_char_ensure_with_lock`), and the
//! placeholder box Blender draws for a character no font provides
//! (`vfont_placeholder_ensure`).

use super::font::{BFont, BFontMetrics};
use super::glyph::{char_curves, TChar};
use super::{BText, TBez, TCharInfo, CU_CHINFO_BOLD, CU_CHINFO_ITALIC, HD_VECT};
use glam::Vec3;
use std::collections::HashMap;
use std::sync::Arc;

pub struct TChars<'a> {
    fonts: &'a [Option<Arc<BFont>>; 4],
    cache: HashMap<(usize, u32), Option<TChar>>,
    space: TChar,
    boxed: TChar,
}

impl<'a> TChars<'a> {
    pub fn new(cu: &'a BText) -> Self {
        let metrics = Self::metrics_of(&cu.fonts);
        let (space, boxed) = placeholders(&metrics);
        Self { fonts: &cu.fonts, cache: HashMap::new(), space, boxed }
    }

    /// The metrics all text uses: the regular font's, or the defaults.
    pub fn metrics_of(fonts: &[Option<Arc<BFont>>; 4]) -> BFontMetrics {
        fonts[0].as_ref().map_or_else(BFontMetrics::default, |f| f.metrics)
    }

    /// `vfont_from_charinfo`: the style's font, else the regular one.
    fn font_index(&self, info: &TCharInfo) -> usize {
        let styled = match info.flag & (CU_CHINFO_BOLD | CU_CHINFO_ITALIC) {
            CU_CHINFO_BOLD => 1,
            CU_CHINFO_ITALIC => 2,
            f if f == CU_CHINFO_BOLD | CU_CHINFO_ITALIC => 3,
            _ => 0,
        };
        match self.fonts[styled] {
            Some(_) => styled,
            None => 0,
        }
    }

    /// The character, or the placeholder for one the font lacks.
    pub fn get(&mut self, info: &TCharInfo, code: char) -> &TChar {
        let k = self.font_index(info);
        let font = self.fonts[k].clone();
        let found = self
            .cache
            .entry((k, code as u32))
            .or_insert_with(|| font.as_ref().and_then(|f| f.glyph(code as u32).map(|g| char_curves(&g, f.metrics.scale))));
        match (found.as_ref(), code) {
            (Some(c), _) => c,
            (None, ' ') => &self.space,
            (None, _) => &self.boxed,
        }
    }
}

/// `vfont_placeholder_ensure`: an en-wide space, and an en-wide hollow
/// rectangle (outer outline, then the hole wound the other way).
fn placeholders(m: &BFontMetrics) -> (TChar, TChar) {
    let ascent = m.ascent();
    let line_width = 0.05 * m.em_ratio;
    let size_factor = [0.9f32, 0.9 - line_width * 2.0];
    let margin_y = (1.0 - size_factor[0]) / 2.0;
    let width = 0.5 * m.em_ratio;
    let mut nurbs = Vec::new();
    for nu_index in 0..2 {
        let (mut xmin, mut xmax, mut ymin, mut ymax) = (width * margin_y, width - margin_y, 0.0, ascent * size_factor[1]);
        if nu_index == 1 {
            (xmin, xmax, ymin, ymax) = (xmin + line_width, xmax - line_width, ymin + line_width, ymax - line_width);
        }
        let mut corners = [[xmin, ymin], [xmin, ymax], [xmax, ymax], [xmax, ymin]];
        if nu_index == 1 {
            corners.reverse();
        }
        let p = |c: [f32; 2]| TBez { vec: [Vec3::ZERO, Vec3::new(c[0], c[1], 0.0), Vec3::ZERO], h1: HD_VECT, h2: HD_VECT };
        nurbs.push(corners.map(p).to_vec());
    }
    (TChar { nurbs: Vec::new(), width }, TChar { nurbs, width })
}
