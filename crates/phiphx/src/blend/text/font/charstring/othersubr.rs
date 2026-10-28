//! The charstring operators that reach outside the glyph: `callothersubr`
//! (flex, hint replacement) and `seac` (accented composites), for
//! [`super::Run`].

use super::super::names::standard_name;
use super::super::BTag;
use super::Run;

impl Run<'_> {
    /// `callothersubr`: flex and hint replacement; any other result goes
    /// back to `pop` as its arguments.
    pub(super) fn othersubr(&mut self) {
        let index = self.pop() as i32;
        let n = self.pop().max(0.0) as usize;
        let args: Vec<f64> = (0..n).map(|_| self.pop()).collect();
        match index {
            1 => {
                self.flex = Some(Vec::new());
                self.flex_from = (self.x, self.y);
            }
            0 => {
                let points = self.flex.take().unwrap_or_default();
                if points.len() == 7 {
                    // The flex starts where the current point was when it
                    // began; its reference point (`points[0]`) is not drawn.
                    let end = (self.x, self.y);
                    (self.x, self.y) = self.flex_from;
                    self.start();
                    (self.x, self.y) = end;
                    for (k, &(px, py)) in points[1..].iter().enumerate() {
                        let tag = if k % 3 == 2 { BTag::On } else { BTag::Cubic };
                        if let Some(c) = self.contours.last_mut() {
                            c.push((px, py, tag));
                        }
                    }
                }
                // `pop pop setcurrentpoint` reads the end point back.
                self.ps = vec![self.y, self.x];
            }
            _ => self.ps = args.into_iter().rev().collect(),
        }
    }

    /// `t1operator_seac`: the base glyph at the origin, then the accent
    /// with its origin at `adx - asb` past the composite's side bearing;
    /// the advance stays the base glyph's.
    pub(super) fn seac(&mut self, asb: f64, adx: f64, ady: f64, bchar: u8, achar: u8) {
        let adx = adx + self.lsb;
        let glyph = |code: u8| standard_name(code).and_then(|n| self.font.charstrings.get(n)).cloned();
        let (Some(base), Some(accent)) = (glyph(bchar), glyph(achar)) else { return };
        self.close();
        (self.lsb, self.origin) = (0.0, (0.0, 0.0));
        let _ = self.exec(&base, 1);
        self.close();
        let (lsb, advance) = (self.lsb, self.advance);
        (self.lsb, self.origin, self.done) = (0.0, (adx - asb, ady), false);
        let _ = self.exec(&accent, 1);
        self.close();
        (self.lsb, self.advance, self.origin) = (lsb, advance, (0.0, 0.0));
    }
}
