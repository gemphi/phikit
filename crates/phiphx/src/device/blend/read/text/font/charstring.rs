//! The Type 1 charstring interpreter (FreeType `t1_decoder_parse_charstrings`,
//! `psaux/t1decode.c`) at `FT_LOAD_NO_SCALE`: the outline in font units
//! as `FT_Outline` points tagged on-curve or cubic control, hints ignored.
//!
//! Each contour is closed the way `ps_builder_close_contour` closes it:
//! its last point dropped when it lands on the first and is on-curve.
//! Flex (`othersubr` 0 – 2) becomes its two curves; hint replacement
//! (`othersubr` 3) hands its argument back to `pop`. An accented
//! composite (`seac`, `t1operator_seac`) is its base glyph's outline then
//! its accent's, moved by the accent's offset.

use super::type1::BType1;
use super::{BGlyph, BTag};

mod othersubr;

/// The interpreter's state over one glyph (`T1_DecoderRec`).
struct Run<'a> {
    font: &'a BType1,
    stack: Vec<f64>,
    /// Results of `callothersubr` for `pop`.
    ps: Vec<f64>,
    x: f64,
    y: f64,
    advance: f64,
    contours: Vec<Vec<(f64, f64, BTag)>>,
    /// A `moveto` waits for the first segment to open its contour.
    open: bool,
    /// Points collected between flex start and end.
    flex: Option<Vec<(f64, f64)>>,
    flex_from: (f64, f64),
    /// `builder.pos_x / pos_y`: where `hsbw` puts the origin (a `seac`
    /// accent's offset), and `left_bearing.x`, the glyph's own `sbx`.
    origin: (f64, f64),
    lsb: f64,
    done: bool,
}

/// Decode one glyph's charstring.
pub fn run(font: &BType1, cs: &[u8]) -> Option<BGlyph> {
    let mut r = Run { font, stack: Vec::new(), ps: Vec::new(), x: 0.0, y: 0.0, advance: 0.0, contours: Vec::new(), open: false, flex: None, flex_from: (0.0, 0.0), origin: (0.0, 0.0), lsb: 0.0, done: false };
    r.exec(cs, 0)?;
    r.close();
    Some(BGlyph { contours: r.contours, advance: r.advance })
}

impl Run<'_> {
    fn pop(&mut self) -> f64 {
        self.stack.pop().unwrap_or(0.0)
    }

    /// The operands of the current operator, bottom first.
    fn args<const N: usize>(&mut self) -> [f64; N] {
        let n = self.stack.len();
        let out = std::array::from_fn(|i| (n + i).checked_sub(N).and_then(|k| self.stack.get(k)).copied().unwrap_or(0.0));
        self.stack.truncate(n.saturating_sub(N));
        out
    }

    /// `ps_builder_start_point`: open a contour at the current point.
    fn start(&mut self) {
        if !self.open {
            self.contours.push(vec![(self.x, self.y, BTag::On)]);
            self.open = true;
        }
    }

    fn point(&mut self, dx: f64, dy: f64, tag: BTag) {
        self.x += dx;
        self.y += dy;
        let (x, y) = (self.x, self.y);
        if let Some(c) = self.contours.last_mut() {
            c.push((x, y, tag));
        }
    }

    fn line(&mut self, dx: f64, dy: f64) {
        self.start();
        self.point(dx, dy, BTag::On);
    }

    fn curve(&mut self, d: [f64; 6]) {
        self.start();
        self.point(d[0], d[1], BTag::Cubic);
        self.point(d[2], d[3], BTag::Cubic);
        self.point(d[4], d[5], BTag::On);
    }

    fn moveto(&mut self, dx: f64, dy: f64) {
        match self.flex.as_mut() {
            Some(points) => {
                self.x += dx;
                self.y += dy;
                points.push((self.x, self.y));
            }
            None => {
                self.close();
                self.x += dx;
                self.y += dy;
            }
        }
    }

    /// `ps_builder_close_contour`.
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

    fn exec(&mut self, cs: &[u8], depth: usize) -> Option<()> {
        if depth > 10 {
            return None;
        }
        let mut i = 0;
        while i < cs.len() && !self.done {
            let v = cs[i] as i32;
            i += 1;
            match v {
                32..=246 => self.stack.push((v - 139) as f64),
                247..=250 => {
                    self.stack.push(((v - 247) * 256 + *cs.get(i)? as i32 + 108) as f64);
                    i += 1;
                }
                251..=254 => {
                    self.stack.push((-(v - 251) * 256 - *cs.get(i)? as i32 - 108) as f64);
                    i += 1;
                }
                255 => {
                    self.stack.push(i32::from_be_bytes(cs.get(i..i + 4)?.try_into().ok()?) as f64);
                    i += 4;
                }
                12 => {
                    let op = *cs.get(i)?;
                    i += 1;
                    self.escape(op);
                }
                10 => {
                    let n = self.pop() as usize;
                    let subr = self.font.subrs.get(n)?.clone();
                    self.exec(&subr, depth + 1)?;
                }
                11 => return Some(()),
                _ => self.op(v),
            }
        }
        Some(())
    }

    fn op(&mut self, v: i32) {
        match v {
            4 => {
                let [dy] = self.args();
                self.moveto(0.0, dy);
            }
            21 => {
                let [dx, dy] = self.args();
                self.moveto(dx, dy);
            }
            22 => {
                let [dx] = self.args();
                self.moveto(dx, 0.0);
            }
            5 => {
                let [dx, dy] = self.args();
                self.line(dx, dy);
            }
            6 => {
                let [dx] = self.args();
                self.line(dx, 0.0);
            }
            7 => {
                let [dy] = self.args();
                self.line(0.0, dy);
            }
            8 => {
                let d = self.args::<6>();
                self.curve(d);
            }
            30 => {
                let [dy1, dx2, dy2, dx3] = self.args();
                self.curve([0.0, dy1, dx2, dy2, dx3, 0.0]);
            }
            31 => {
                let [dx1, dx2, dy2, dy3] = self.args();
                self.curve([dx1, 0.0, dx2, dy2, 0.0, dy3]);
            }
            9 => self.close(),
            13 => {
                let [sbx, wx] = self.args();
                self.lsb += sbx;
                (self.x, self.y, self.advance) = (self.origin.0 + sbx, self.origin.1, wx);
            }
            14 => {
                self.close();
                self.done = true;
            }
            _ => self.stack.clear(),
        }
    }

    fn escape(&mut self, op: u8) {
        match op {
            7 => {
                let [sbx, sby, wx, _wy] = self.args();
                self.lsb += sbx;
                (self.x, self.y, self.advance) = (self.origin.0 + sbx, self.origin.1 + sby, wx);
            }
            12 => {
                let [a, b] = self.args();
                self.stack.push(if b == 0.0 { 0.0 } else { a / b });
            }
            16 => self.othersubr(),
            17 => {
                let v = self.ps.pop().unwrap_or(0.0);
                self.stack.push(v);
            }
            33 => {
                let [x, y] = self.args();
                (self.x, self.y) = (x, y);
            }
            6 => {
                let [asb, adx, ady, bchar, achar] = self.args();
                self.seac(asb, adx, ady, bchar as u8, achar as u8);
                self.done = true;
            }
            _ => self.stack.clear(),
        }
    }
}
