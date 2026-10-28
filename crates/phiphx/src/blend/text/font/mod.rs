//! A text object's font (`VFont`) as Blender loads it: the face's
//! metrics (`BLF_get_vfont_metrics`) and each character's outline in font
//! units (`blf_character_to_curves` at `FT_LOAD_NO_SCALE`), from the
//! builtin `bfont.pfb` (Type 1, [`type1`]) or a TrueType / OpenType face
//! ([`sfnt`], through `ttf-parser`).
//!
//! Not done: the fallback font stack Blender searches when the builtin
//! font lacks a character (`use_fallback`), and synthesized bold / italic.

mod charstring;
mod names;
mod sfnt;
mod type1;

use std::sync::{Arc, OnceLock};

/// Blender's builtin font (`release/datafiles/bfont.pfb`, registered by
/// `BKE_vfont_builtin_register`).
const BFONT: &[u8] = include_bytes!("../../../../assets/bfont.pfb");

/// `FT_Curve_Tag_*`: an outline point's kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BTag {
    On,
    Conic,
    Cubic,
}

/// One glyph at `FT_LOAD_NO_SCALE`: its contours (`FT_Outline`) and its
/// advance (`glyph->advance.x`), in font units.
#[derive(Clone, Debug, Default)]
pub struct BGlyph {
    pub contours: Vec<Vec<(f64, f64, BTag)>>,
    pub advance: f64,
}

/// `VFontData_Metrics`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BFontMetrics {
    /// Font units to Blender units: one over the face's bounding-box height.
    pub scale: f32,
    pub em_ratio: f32,
    pub ascend_ratio: f32,
}

impl Default for BFontMetrics {
    /// `BKE_vfontdata_metrics_get_defaults`.
    fn default() -> Self {
        Self { scale: 1.0 / 1000.0, em_ratio: 1.0, ascend_ratio: 0.8 }
    }
}

impl BFontMetrics {
    /// `BLF_get_vfont_metrics` from FreeType's face fields.
    pub fn from_face(ascender: i32, descender: i32, bbox_ymax: i32, bbox_ymin: i32) -> Self {
        let mut m = Self::default();
        // Blender default BFont is not "complete".
        let complete = ascender != 0 && descender != 0 && ascender != descender;
        if complete {
            m.ascend_ratio = ascender as f32 / (ascender - descender) as f32;
        }
        if bbox_ymax != bbox_ymin {
            m.scale = (1.0 / (bbox_ymax - bbox_ymin) as f64) as f32;
            if complete {
                m.em_ratio = (ascender - descender) as f32 / (bbox_ymax - bbox_ymin) as f32;
            }
        }
        m
    }

    /// `vfont_metrics_ascent`.
    pub fn ascent(&self) -> f32 {
        self.ascend_ratio * self.em_ratio
    }

    /// `vfont_metrics_descent`.
    pub fn descent(&self) -> f32 {
        self.em_ratio - self.ascent()
    }
}

/// A loaded face.
#[derive(Debug)]
enum BFace {
    Type1(type1::BType1),
    Sfnt(Vec<u8>),
}

/// A `VFont`'s data (`VFontData`): its metrics and a way to its glyphs.
#[derive(Debug)]
pub struct BFont {
    face: BFace,
    pub metrics: BFontMetrics,
}

impl BFont {
    /// The builtin font, parsed once.
    pub fn builtin() -> Option<Arc<BFont>> {
        static BUILTIN: OnceLock<Option<Arc<BFont>>> = OnceLock::new();
        BUILTIN.get_or_init(|| Self::from_bytes(BFONT.to_vec()).map(Arc::new)).clone()
    }

    /// A font file's bytes: Type 1 (`.pfb` / `.pfa`) or an sfnt face.
    pub fn from_bytes(data: Vec<u8>) -> Option<Self> {
        match data.first() {
            Some(0x80) | Some(b'%') => {
                let t1 = type1::BType1::parse(&data)?;
                let metrics = t1.metrics();
                Some(Self { face: BFace::Type1(t1), metrics })
            }
            _ => {
                let metrics = sfnt::metrics(&data)?;
                Some(Self { face: BFace::Sfnt(data), metrics })
            }
        }
    }

    /// The outline of `unicode` (`blf_glyphslot_ensure_outline`): control
    /// codes other than line breaks draw as a space; `None` when the face
    /// has no glyph for it.
    pub fn glyph(&self, unicode: u32) -> Option<BGlyph> {
        let code = match unicode {
            0x10 | 0x13 => return None,
            0..=31 => 32,
            c => c,
        };
        match &self.face {
            BFace::Type1(t1) => t1.glyph(code),
            BFace::Sfnt(data) => sfnt::glyph(data, code),
        }
    }
}
