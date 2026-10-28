//! The text layout (`vfont_to_curve` in `FO_EDIT` mode, `vfont_curve.cc`):
//! every character's offset along its line, the lines stacked by the line
//! distance, aligned in X and Y, then each glyph copied there with shear,
//! small caps and the font size (`vfont_char_build_impl`), and underlines
//! (`build_underline`).
//!
//! The text box is used for its offset; wrapping inside a box's width,
//! overflow and scale-to-fit, text on a curve and combining-mark placement
//! are not done (a zero-width glyph simply does not advance).

use super::chars::TChars;
use super::{BText, TCharInfo, TNurb, TSpline, CU_CHINFO_SMALLCAPS, CU_CHINFO_UNDERLINE};
use glam::{Vec2, Vec3};

/// `CharTrans`.
#[derive(Clone, Copy, Default)]
struct TCharTrans {
    offset: Vec2,
    linenr: usize,
    is_smallcaps: bool,
}

fn safe_divide(a: f32, b: f32) -> f32 {
    if b == 0.0 { 0.0 } else { a / b }
}

/// The text's splines, in the curve's own space.
pub fn vfont_to_curve(cu: &BText) -> Vec<TNurb> {
    let slen = cu.text.len();
    let mem = |i: usize| cu.text.get(i).copied().unwrap_or('\0');
    let info = |i: usize| cu.info.get(i).copied().unwrap_or_default();
    let mut chars = TChars::new(cu);
    let metrics = TChars::metrics_of(&cu.fonts);
    let font_size = cu.fsize;
    let off_scale = Vec2::new(safe_divide(cu.xof, font_size), safe_divide(cu.yof, font_size));
    let tb = cu.textbox.map(|v| v * safe_divide(1.0, font_size));
    let margin = Vec2::new(off_scale.x + tb[0], off_scale.y + tb[1]);
    let linedist = cu.linedist;
    // `xtrax`: extra space is added, less space multiplied.
    let contract = cu.spacing < 1.0;
    let xtrax = if contract { cu.spacing - 1.0 } else { 0.5 * cu.spacing - 0.5 };
    let xtrax_w = |twidth: f32| if contract { xtrax * twidth } else { xtrax };

    let mut ct = vec![TCharTrans::default(); slen + 1];
    // Per line: the left margin before alignment.
    let mut line_x_min: Vec<f32> = Vec::new();
    let mut offset = margin;
    let mut lnr = 0;
    for i in 0..=slen {
        let mut code = mem(i);
        let inf = info(i);
        if inf.flag & CU_CHINFO_SMALLCAPS != 0 {
            let upper = code.to_uppercase().next().unwrap_or(code);
            ct[i].is_smallcaps = upper != code;
            code = upper;
        }
        let twidth = match code {
            '\n' | '\0' => 0.0,
            _ => width(chars.get(&inf, code).width, ct[i].is_smallcaps, cu),
        };
        ct[i].offset = offset;
        ct[i].linenr = lnr;
        match code {
            '\n' | '\0' => {
                offset.y -= linedist;
                line_x_min.push((offset.x - xtrax_w(twidth)) - tb[0]);
                offset.x = margin.x;
                lnr += 1;
            }
            '\t' => {
                let tabfac = 2.0 * ((offset.x - margin.x + 0.01) / 2.0).ceil();
                offset.x = margin.x + tabfac;
            }
            _ => {
                let wsfac = if code == ' ' { cu.wordspace } else { 1.0 };
                offset.x += (twidth * wsfac * (1.0 + (inf.kern / 40.0))) + xtrax_w(twidth);
            }
        }
    }

    // Alignment in X: the line's free width (`x_max` is the box width,
    // zero without a box) shifts the whole line, or half of it.
    let shift = |x_min: f32, div: f32| ((tb[2] - x_min) + off_scale.x) / div;
    let line_shift: Vec<f32> = match cu.align_x {
        1 => line_x_min.iter().map(|&m| shift(m, 2.0)).collect(),
        2 => line_x_min.iter().map(|&m| shift(m, 1.0)).collect(),
        _ => vec![0.0; line_x_min.len()],
    };
    for c in ct.iter_mut() {
        c.offset.x += line_shift.get(c.linenr).copied().unwrap_or(0.0);
    }
    let yoff = align_y(cu, &metrics, lnr, tb[3]);
    for c in ct.iter_mut() {
        c.offset.y += yoff;
    }

    let mut nurbs = Vec::new();
    let mut ul_prev: Option<(usize, usize)> = None;
    for i in 0..slen {
        let inf = info(i);
        let code = match ct[i].is_smallcaps {
            true => mem(i).to_uppercase().next().unwrap_or(mem(i)),
            false => mem(i),
        };
        if code == '\n' {
            continue;
        }
        let che = chars.get(&inf, code).clone();
        build_char(cu, &mut nurbs, &che.nurbs, &inf, ct[i].is_smallcaps, ct[i].offset, i as i32, font_size);
        if inf.flag & CU_CHINFO_UNDERLINE == 0 {
            continue;
        }
        let twidth = width(che.width, ct[i].is_smallcaps, cu);
        let next = mem(i + 1);
        let overlap = match i + 1 < slen && next != '\n' && (next != ' ' || info(i + 1).flag & CU_CHINFO_UNDERLINE != 0) {
            true => xtrax_w(twidth),
            false => 0.0,
        };
        let ulwidth = (twidth * (1.0 + (inf.kern / 40.0))) + overlap;
        let (xmin, ymin) = (ct[i].offset.x, ct[i].offset.y);
        let (xmax, ymax) = (xmin + ulwidth, ymin - cu.ulheight);
        let yofs = cu.ulpos - 0.05;
        let mut bp = [[xmin, ymax + yofs], [xmax, ymax + yofs], [xmax, ymin + yofs], [xmin, ymin + yofs]].map(|p| Vec3::new(p[0] * font_size, p[1] * font_size, 0.0));
        // Weld with the previous, adjacent underline on the same line.
        if let Some((_, prev_nu)) = ul_prev.filter(|&(pi, _)| pi + 1 == i && ct[pi].linenr == ct[i].linenr) {
            if let TSpline::Poly(prev) = &mut nurbs[prev_nu].spline {
                let mid = |a: Vec3, b: Vec3| Vec3::new(a.x * 0.5 + b.x * 0.5, a.y * 0.5 + b.y * 0.5, a.z);
                (prev[1], bp[0]) = (mid(prev[1], bp[0]), mid(bp[0], prev[1]));
                (prev[2], bp[3]) = (mid(prev[2], bp[3]), mid(bp[3], prev[2]));
            }
        }
        nurbs.push(TNurb { spline: TSpline::Poly(bp.to_vec()), resolu: cu.resolu, charidx: i as i32 + 1000, mat_nr: inf.mat_nr.max(0) });
        ul_prev = Some((i, nurbs.len() - 1));
    }
    nurbs
}

/// `vfont_char_width`.
fn width(w: f32, smallcaps: bool, cu: &BText) -> f32 {
    if smallcaps { w * cu.smallcaps_scale } else { w }
}

/// The vertical alignment's offset, with or without a box height.
fn align_y(cu: &BText, m: &super::font::BFontMetrics, lines: usize, box_h: f32) -> f32 {
    let lines_extra = (lines as f32 - 1.0) * cu.linedist;
    match box_h != 0.0 {
        false => match cu.align_y {
            1 => -m.ascent(),
            2 => ((m.em_ratio + lines_extra) * 0.5) - m.ascent(),
            3 => lines_extra,
            4 => lines_extra + m.descent(),
            _ => 0.0,
        },
        // The box's origin is hard-coded at one text size up.
        true => match cu.align_y {
            1 => 1.0 - m.ascent(),
            2 => (((m.em_ratio + lines_extra) * 0.5) - m.ascent()) - (box_h * 0.5) + 1.0,
            3 => 1.0 + lines_extra - box_h,
            4 => 1.0 + lines_extra - box_h + m.descent(),
            _ => 0.0,
        },
    }
}

/// `vfont_char_build_impl`: the glyph's splines sheared, small-capped and
/// moved to `offset`, all scaled by the font size.
fn build_char(cu: &BText, out: &mut Vec<TNurb>, splines: &[Vec<super::TBez>], info: &TCharInfo, smallcaps: bool, offset: Vec2, charidx: i32, fsize: f32) {
    for bezt in splines {
        let mut b = bezt.clone();
        for p in b.iter_mut().flat_map(|b| b.vec.iter_mut()) {
            if cu.shear != 0.0 {
                p.x += cu.shear * p.y;
            }
            if smallcaps {
                p.x *= cu.smallcaps_scale;
                p.y *= cu.smallcaps_scale;
            }
            p.x = (p.x + offset.x) * fsize;
            p.y = (p.y + offset.y) * fsize;
        }
        out.push(TNurb { spline: TSpline::Bezier(b), resolu: cu.resolu, charidx, mat_nr: info.mat_nr.max(0) });
    }
}
