//! The render region a still fills — Blender's `Scene.r.border`
//! (`render_init_from_main`'s `disprect`): the pixels of the frame to
//! draw, and whether the result is cropped to them (`R_CROP`) or laid into
//! a frame left black with alpha 0, as Blender leaves the rest of a
//! bordered `RenderResult`.

use super::still::BStill;

/// The pixels of the frame to fill — none for the whole frame — and the crop.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BStillRegion {
    pub pixels: Option<(u32, u32, u32, u32)>,
    pub crop: bool,
}

impl BStillRegion {
    /// The size of the picture that comes out of a `w × h` frame.
    pub fn size(&self, w: u32, h: u32) -> (u32, u32) {
        match self.pixels {
            Some((_, _, rw, rh)) if self.crop => (rw, rh),
            _ => (w, h),
        }
    }

    /// Blender draws the border's pixels alone; the effects and the
    /// anti-aliasing gather read a pixel's neighbours, so the region is
    /// drawn this much wider each way (within the frame) and cut back.
    /// The rectangle actually drawn of a `w × h` frame: the region with
    /// a margin of `margin` pixels (`Tracer::seam_rows`: two, or SMAA's
    /// reach under the single pass), or the whole frame.
    pub fn drawn(&self, w: u32, h: u32, margin: u32) -> (u32, u32, u32, u32) {
        let Some((x, y, rw, rh)) = self.pixels else { return (0, 0, w, h) };
        let (x0, y0) = (x.saturating_sub(margin), y.saturating_sub(margin));
        let (x1, y1) = ((x + rw + margin).min(w), (y + rh + margin).min(h));
        (x0, y0, x1 - x0, y1 - y0)
    }

    /// The drawn pixels as the result: the frame itself when there is no
    /// region; else the region cut out of what was drawn, alone when
    /// cropped, or laid into the black frame.
    pub(super) fn framed(&self, drawn: BStill, w: u32, h: u32, margin: u32) -> BStill {
        let Some((x, y, rw, rh)) = self.pixels else { return drawn };
        let (dx, dy, dw, _) = self.drawn(w, h, margin);
        let mut cut = BStill::empty(rw, rh);
        cut.lay_from(&drawn, (x - dx) as usize, (y - dy) as usize, rw as usize, rh as usize, dw as usize);
        if self.crop {
            return cut;
        }
        let mut frame = BStill::empty(w, h);
        frame.lay_at(&cut, x as usize, y as usize, rw as usize, rh as usize, w as usize);
        frame
    }

    /// `framed` for the bytes alone (a `BStill::peek`).
    pub(super) fn framed_rgba(&self, drawn: Vec<u8>, w: u32, h: u32, margin: u32) -> Vec<u8> {
        let Some((x, y, rw, rh)) = self.pixels else { return drawn };
        let (dx, dy, dw, _) = self.drawn(w, h, margin);
        let (x, y, rw, rh, dw, w, h) = (x as usize, y as usize, rw as usize, rh as usize, dw as usize, w as usize, h as usize);
        let mut cut = vec![0u8; rw * rh * 4];
        for row in 0..rh {
            let (at, from) = (row * rw, (y - dy as usize + row) * dw + (x - dx as usize));
            cut[at * 4..(at + rw) * 4].copy_from_slice(&drawn[from * 4..(from + rw) * 4]);
        }
        if self.crop {
            return cut;
        }
        let mut frame = vec![0u8; w * h * 4];
        for row in 0..rh {
            let (at, from) = ((y + row) * w + x, row * rw);
            frame[at * 4..(at + rw) * 4].copy_from_slice(&cut[from * 4..(from + rw) * 4]);
        }
        frame
    }
}


impl BStill {
    /// This `rw × rh` picture filled from `piece` (`stride` wide) at `(x, y)`.
    pub(super) fn lay_from(&mut self, piece: &Self, x: usize, y: usize, rw: usize, rh: usize, stride: usize) {
        for row in 0..rh {
            let (at, from) = (row * rw, (y + row) * stride + x);
            self.rgba[at * 4..(at + rw) * 4].copy_from_slice(&piece.rgba[from * 4..(from + rw) * 4]);
            self.display[at..at + rw].copy_from_slice(&piece.display[from..from + rw]);
            self.linear[at..at + rw].copy_from_slice(&piece.linear[from..from + rw]);
            self.albedo[at..at + rw].copy_from_slice(&piece.albedo[from..from + rw]);
            self.normal[at..at + rw].copy_from_slice(&piece.normal[from..from + rw]);
            self.depth[at..at + rw].copy_from_slice(&piece.depth[from..from + rw]);
        }
    }

    /// `piece`, `rw × rh`, into this `w`-wide picture at `(x, y)`.
    pub(super) fn lay_at(&mut self, piece: &Self, x: usize, y: usize, rw: usize, rh: usize, w: usize) {
        for row in 0..rh {
            let (at, from) = ((y + row) * w + x, row * rw);
            self.rgba[at * 4..(at + rw) * 4].copy_from_slice(&piece.rgba[from * 4..(from + rw) * 4]);
            self.display[at..at + rw].copy_from_slice(&piece.display[from..from + rw]);
            self.linear[at..at + rw].copy_from_slice(&piece.linear[from..from + rw]);
            self.albedo[at..at + rw].copy_from_slice(&piece.albedo[from..from + rw]);
            self.normal[at..at + rw].copy_from_slice(&piece.normal[from..from + rw]);
            self.depth[at..at + rw].copy_from_slice(&piece.depth[from..from + rw]);
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::scene::{BObject3D, BShading, BShape};

    /// A region of the frame renders the same pixels the whole frame has
    /// there — laid into black without Crop, alone with it.
    #[test]
    fn a_region_is_the_frames_pixels_there_cropped_or_in_black() {
        if phiray::Ray::gpu().is_none() {
            return;
        }
        let objects = vec![BObject3D::Primitive { name: "Cube".into(), shape: BShape::Cube, x: 0.0, y: 0.0 }];
        let look = super::super::world::BRayLook { sky: gpui::black(), grid: gpui::white(), grid_hide: 0, world: None, lights: Vec::new(), wire: None, view: [0.0, 0.0, 1.0, 0.0], environment: None, set: Default::default(), studio: false };
        let setup = crate::v2::scene::camera::setup_from_state(&crate::v2::state::BState::default());
        let cam = super::super::scene::camera(20.0, 30.0, (0.0, 0.0), 1.0, setup, 1.0, &look);
        let render = |region: BStillRegion| {
            let mut out = None;
            super::super::still_stepwise(&objects, super::super::BStillLook::new(BShading::Solid, gpui::white()), cam, 40, 40, 1, false, [0.0; 4], 0.0, region, |_, at| {
                if out.is_none() && at.settled() { out = at.image(); }
                true
            });
            out.expect("a still")
        };
        let whole = render(BStillRegion::default());
        let pixels = Some((10, 8, 16, 12));
        let framed = render(BStillRegion { pixels, crop: false });
        assert_eq!(framed.rgba.len(), whole.rgba.len(), "uncropped: the frame's size");
        fn at(px: &[u8], x: usize, y: usize) -> &[u8] {
            &px[4 * (y * 40 + x)..4 * (y * 40 + x) + 4]
        }
        let differ = (8..20).flat_map(|y| (10..26).map(move |x| (x, y))).filter(|&(x, y)| at(&framed.rgba, x, y) != at(&whole.rgba, x, y)).count();
        assert_eq!(differ, 0, "every pixel of the region is the frame's there");
        assert_eq!(at(&framed.rgba, 12, 10), at(&whole.rgba, 12, 10), "inside the region: the frame's pixel");
        assert_eq!(at(&framed.rgba, 25, 19), at(&whole.rgba, 25, 19));
        assert_eq!(at(&framed.rgba, 2, 2), &[0, 0, 0, 0], "outside: black, alpha 0");
        assert_eq!(at(&framed.rgba, 30, 30), &[0, 0, 0, 0]);
        let cropped = render(BStillRegion { pixels, crop: true });
        assert_eq!(cropped.rgba.len(), 16 * 12 * 4, "cropped: the region's size");
        assert_eq!(&cropped.rgba[4 * (2 * 16 + 2)..4 * (2 * 16 + 2) + 4], at(&whole.rgba, 12, 10));
        assert_eq!((BStillRegion { pixels, crop: true }.size(40, 40), BStillRegion { pixels, crop: false }.size(40, 40)), ((16, 12), (40, 40)));
    }
}
