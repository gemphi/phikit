//! The render region — `RenderData.border` (an `rctf`, fractions of the
//! frame: `xmin`, `xmax`, `ymin`, `ymax`, y up from the bottom as Blender's
//! buffers run) with `R_BORDER` and `R_CROP` in `RenderData.mode`
//! (Properties → Output → Format: Render Region, Crop to Render Region).
//! `render_init_from_main` turns it into the pixel rectangle a render
//! fills: the whole frame without the flag, the rectangle alone with Crop,
//! else the rectangle inside a frame left black. A 3D view not looking
//! through the camera has a border of its own, `View3D.render_border`
//! with `V3D_RENDER_BORDER` in `View3D.flag2`, which its Rendered shading
//! fills (`ED_view3d_calc_render_border`) — [`BlendViewBorder`].

/// `Scene.r.border` and its two mode bits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlendRenderBorder {
    /// `R_BORDER`: the region is rendered.
    pub on: bool,
    /// `R_CROP`: the result is the region's size.
    pub crop: bool,
    /// `[xmin, xmax, ymin, ymax]`, 0 ..= 1.
    pub rect: [f32; 4],
}

impl Default for BlendRenderBorder {
    /// `DNA_scene_defaults.h`: the whole frame, off.
    fn default() -> Self {
        Self { on: false, crop: false, rect: [0.0, 1.0, 0.0, 1.0] }
    }
}

impl BlendRenderBorder {
    pub const R_BORDER: u32 = 1 << 9;
    pub const R_CROP: u32 = 1 << 11;

    pub fn of(mode: u32, rect: [f32; 4]) -> Self {
        Self { on: mode & Self::R_BORDER != 0, crop: mode & Self::R_CROP != 0, rect }
    }

    /// `mode` with these two bits as this says.
    pub fn mode_over(&self, mode: u32) -> u32 {
        let with = |m: u32, bit: u32, on: bool| if on { m | bit } else { m & !bit };
        with(with(mode, Self::R_BORDER, self.on), Self::R_CROP, self.crop)
    }

    /// `render_init_from_main`'s `disprect` for a `w × h` frame: `(x, y, w,
    /// h)` in pixels, y down from the top as a picture's rows run; `None`
    /// when off or empty. Blender truncates the fractions to pixels and
    /// clamps to the frame.
    pub fn pixels(&self, w: u32, h: u32) -> Option<(u32, u32, u32, u32)> {
        if !self.on {
            return None;
        }
        let (fw, fh) = (w as f32, h as f32);
        let x0 = (self.rect[0] * fw).max(0.0) as u32;
        let x1 = ((self.rect[1] * fw) as u32).min(w);
        let y_top = ((1.0 - self.rect[3]) * fh).max(0.0) as u32;
        let y_bottom = (((1.0 - self.rect[2]) * fh) as u32).min(h);
        (x1 > x0 && y_bottom > y_top).then_some((x0, y_top, x1 - x0, y_bottom - y_top))
    }
}

/// `View3D.render_border` and its flag: the viewport's own region, as
/// fractions of the region, y up; `view3d.render_border` outside the
/// camera view sets it, `view3d.clear_render_border` puts the whole
/// region back.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlendViewBorder {
    /// `V3D_RENDER_BORDER`.
    pub on: bool,
    /// `[xmin, xmax, ymin, ymax]`, 0 ..= 1.
    pub rect: [f32; 4],
}

impl Default for BlendViewBorder {
    /// `DNA_view3d_defaults.h`: the whole region, off.
    fn default() -> Self {
        Self { on: false, rect: [0.0, 1.0, 0.0, 1.0] }
    }
}

impl BlendViewBorder {
    pub const V3D_RENDER_BORDER: u32 = 1 << 11;

    /// A view made since 2.6 keeps an all-zero `rctf` until a box is set
    /// (`DNA_view3d_types.h`: `render_border = {}`); older files were
    /// versioned to the whole region (`versioning_260.cc`), which is what
    /// the zeros mean here too.
    pub fn of(flag2: u32, rect: [f32; 4]) -> Self {
        let rect = if rect == [0.0; 4] { Self::default().rect } else { rect };
        Self { on: flag2 & Self::V3D_RENDER_BORDER != 0, rect }
    }

    /// The region's pixels of a `w × h` viewport, y down from the top, as
    /// `ED_view3d_calc_render_border` cuts them (`None` when off or empty).
    pub fn pixels(&self, w: u32, h: u32) -> Option<(u32, u32, u32, u32)> {
        BlendRenderBorder { on: self.on, crop: false, rect: self.rect }.pixels(w, h)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_view_s_own_border_reads_its_flag() {
        let v = BlendViewBorder::of(BlendViewBorder::V3D_RENDER_BORDER | 4, [0.5, 1.0, 0.0, 0.5]);
        assert_eq!(v, BlendViewBorder { on: true, rect: [0.5, 1.0, 0.0, 0.5] });
        // The lower right quarter of a 200 × 100 region: rows 50 ..< 100.
        assert_eq!(v.pixels(200, 100), Some((100, 50, 100, 50)));
        assert_eq!(BlendViewBorder::of(4, v.rect).pixels(200, 100), None);
        assert_eq!(BlendViewBorder::of(0, [0.0; 4]).rect, [0.0, 1.0, 0.0, 1.0], "a view that never had a box");
        assert_eq!(BlendViewBorder::default(), BlendViewBorder { on: false, rect: [0.0, 1.0, 0.0, 1.0] });
    }

    #[test]
    fn the_region_becomes_a_pixel_rectangle_from_the_top() {
        let b = BlendRenderBorder { on: true, crop: false, rect: [0.25, 0.75, 0.5, 1.0] };
        // The top half's middle of a 200 × 100 frame: x 50 ..< 150, rows 0 ..< 50.
        assert_eq!(b.pixels(200, 100), Some((50, 0, 100, 50)));
        assert_eq!(BlendRenderBorder { on: false, ..b }.pixels(200, 100), None);
        assert_eq!(BlendRenderBorder { rect: [0.5, 0.5, 0.0, 1.0], ..b }.pixels(200, 100), None, "empty");
        assert_eq!(BlendRenderBorder::of(BlendRenderBorder::R_BORDER | BlendRenderBorder::R_CROP, [0.0, 1.0, 0.0, 1.0]), BlendRenderBorder { on: true, crop: true, rect: [0.0, 1.0, 0.0, 1.0] });
        assert_eq!(b.mode_over(BlendRenderBorder::R_CROP | 1), BlendRenderBorder::R_BORDER | 1, "the bits follow, the rest stays");
    }
}
