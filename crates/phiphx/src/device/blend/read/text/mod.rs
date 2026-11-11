//! Text objects (`OB_FONT`) as geometry: Blender's own path from a string
//! to a mesh, translated.
//!
//! 1. The font ([`font`]): the `VFont`'s face, builtin `bfont.pfb` or a
//!    packed / on-disk TrueType, and its metrics (`BLF_get_vfont_metrics`).
//! 2. Each character's outline as Bézier splines ([`glyph`],
//!    `blf_glyph_to_curves`).
//! 3. The layout ([`layout`], `vfont_to_curve`): size, spacing, word and
//!    line spacing, shear, offsets, alignment, underline, small caps.
//! 4. The splines evaluated ([`bevlist`], `BKE_curve_bevelList_make`), then
//!    the display list ([`displist`], `evaluate_curve_type_object`):
//!    extrusion and bevel swept round each outline, caps, and the 2D fill
//!    ([`fill`], standing in for `BKE_displist_fill`'s scanfill).
//! 5. The mesh ([`mesh`], `mesh_nurbs_displist_to_mesh`), on which the
//!    object's modifier stack then runs like any mesh's.
//!
//! Steps 4 and 5 are the legacy curve display-list pipeline itself, which
//! curve objects share (`blend/curve/displist`): [`sweep`] is
//! `evaluate_curve_type_object`'s body for any [`TCurveShape`].
//!
//! Not done: text boxes that wrap or overflow (a box's offset is read,
//! its width is not), text on a curve, bevel objects and taper, the
//! bevel profile mode, combining marks, fallback fonts.

pub mod bevlist;
mod chars;
pub mod displist;
pub mod fill;
pub mod font;
pub mod glyph;
pub mod layout;
pub mod mesh;
pub mod profile;
pub(crate) mod read;
pub mod sweep;

use glam::{Vec2, Vec3};
use std::sync::Arc;

/// Handle types (`HD_*`).
pub const HD_FREE: u8 = 0;
pub const HD_VECT: u8 = 2;
pub const HD_ALIGN: u8 = 3;

/// `Curve.flag` bits text reads.
pub const CU_3D: u32 = 1 << 0;
pub const CU_FRONT: u32 = 1 << 1;
pub const CU_BACK: u32 = 1 << 2;
pub const CU_FILL_CAPS: u32 = 1 << 14;
pub const CU_MAP_TAPER: u32 = 1 << 15;

/// `CharInfo.flag` bits.
pub const CU_CHINFO_BOLD: u8 = 1 << 0;
pub const CU_CHINFO_ITALIC: u8 = 1 << 1;
pub const CU_CHINFO_UNDERLINE: u8 = 1 << 2;
pub const CU_CHINFO_SMALLCAPS: u8 = 1 << 4;

/// One Bézier point (`BezTriple`) of a text spline; radius 1, no tilt.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TBez {
    pub vec: [Vec3; 3],
    pub h1: u8,
    pub h2: u8,
}

/// A text spline (`Nurb`): a glyph contour (cyclic Bézier) or an
/// underline (a cyclic poly of four points, `build_underline`).
#[derive(Clone, Debug, PartialEq)]
pub enum TSpline {
    Bezier(Vec<TBez>),
    Poly(Vec<Vec3>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct TNurb {
    pub spline: TSpline,
    pub resolu: usize,
    /// `charidx`: the character it belongs to (underlines `+ 1000`).
    pub charidx: i32,
    pub mat_nr: i16,
}

/// `CharInfo`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TCharInfo {
    pub kern: f32,
    pub mat_nr: i16,
    pub flag: u8,
}

/// A text `Curve` datablock: what geometry reads of it.
#[derive(Clone, Debug, Default)]
pub struct BText {
    pub flag: u32,
    pub fsize: f32,
    pub spacing: f32,
    pub linedist: f32,
    pub wordspace: f32,
    pub shear: f32,
    pub xof: f32,
    pub yof: f32,
    pub ulpos: f32,
    pub ulheight: f32,
    pub smallcaps_scale: f32,
    /// `spacemode` (`CU_ALIGN_X_*`) and `align_y` (`CU_ALIGN_Y_*`).
    pub align_x: u8,
    pub align_y: u8,
    pub resolu: usize,
    pub extrude: f32,
    pub bevel_radius: f32,
    /// `Curve.offset` (RNA's Offset plus one).
    pub offset: f32,
    pub bevresol: usize,
    /// `bevel_mode`: 0 round, 1 object, 2 profile.
    pub bevel_mode: u8,
    pub text: Vec<char>,
    pub info: Vec<TCharInfo>,
    /// The first text box (`tb[0]`): x, y, width, height.
    pub textbox: [f32; 4],
    /// `vfont`, `vfontb`, `vfonti`, `vfontbi`; `None` when the font
    /// cannot be loaded (Blender then draws placeholder boxes).
    pub fonts: [Option<Arc<font::BFont>>; 4],
}

/// The `Curve` fields the display list reads (`BKE_curve_bevel_make`,
/// `evaluate_curve_type_object`), a text's or a curve's.
#[derive(Clone, Debug, Default)]
pub struct TCurveShape {
    pub flag: u32,
    pub extrude: f32,
    pub bevel_radius: f32,
    /// `Curve.offset` (RNA's Offset plus one).
    pub offset: f32,
    pub bevresol: usize,
    /// `bevel_mode`: 0 round, 1 object, 2 profile.
    pub bevel_mode: u8,
    /// `bevel_profile->segments`, `bevresol + 1` samples (profile mode).
    pub profile: Option<Vec<Vec2>>,
    /// `curve_bevel_make_from_object`'s lists: the bevel object's outlines
    /// turned into the profile plane (object mode).
    pub bevel_object: Vec<displist::TDispList>,
    /// The taper object's first display list's points (`taperobj`; empty
    /// when it has none, which tapers by 1).
    pub taper: Option<Vec<Vec3>>,
    /// `taper_radius_mode`: 0 override, 1 multiply, 2 add.
    pub taper_radius_mode: u8,
}

impl BText {
    /// The text's bevel and extrusion (no profile, bevel object or taper
    /// is read for a text).
    pub fn shape(&self) -> TCurveShape {
        TCurveShape {
            flag: self.flag,
            extrude: self.extrude,
            bevel_radius: self.bevel_radius,
            offset: self.offset,
            bevresol: self.bevresol,
            bevel_mode: self.bevel_mode,
            ..Default::default()
        }
    }
}

/// The text object's mesh before its modifiers.
#[derive(Clone, Debug, Default)]
pub struct TMesh {
    pub vertices: Vec<[f32; 3]>,
    pub faces: Vec<Vec<u32>>,
    pub face_materials: Vec<u16>,
    pub face_smooth: Vec<bool>,
    pub loose_edges: Vec<(u32, u32)>,
}

/// String → splines → display list → mesh (`evaluate_curve_type_object`
/// then `BKE_mesh_new_nomain_from_curve_displist`).
pub fn text_mesh(text: &BText) -> TMesh {
    let nurbs = layout::vfont_to_curve(text);
    let bev = bevlist::bevel_lists(text, &nurbs);
    let disp = displist::evaluate(text, &nurbs, &bev);
    mesh::from_displist(text.flag, &disp)
}
