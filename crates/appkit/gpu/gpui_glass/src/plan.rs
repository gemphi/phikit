//! What a GPU renderer does for one piece of glass, worked out once for every
//! backend: what to copy, how far to shrink it, and the block of numbers each
//! of the four draws reads. `glass.hlsl` and `glass.wgsl` declare the
//! same block, field for field, so DirectX and wgpu draw the same glass.
//!
//! A renderer copies [`GlassPlan::capture`] out of its window, then draws
//! [`GlassPlan::stage`] in order: shrink, blur across, blur down into two
//! layers [`GlassPlan::small`] wide, then the shape back onto the window.

/// How far in from the edge the rim bends light and catches it, in logical px.
pub const RIM_BAND: f32 = 38.0;
/// A Gaussian is flat past three sigmas, and the sigma is half the blur.
pub const BLUR_REACH: f32 = 1.5;

/// The shader body that shrinks the copy.
pub const STAGE_SHRINK: u32 = 0;
/// The shader body that blurs one way.
pub const STAGE_BLUR: u32 = 1;
/// The shader body that draws the shape.
pub const STAGE_GLASS: u32 = 2;

/// The block the shader reads: 144 bytes, laid out the same in HLSL and WGSL.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct GlassParam {
    pub rect: [f32; 4],
    pub target_size: [f32; 2],
    pub pixel_to_uv: [f32; 2],
    pub uv_limit: [f32; 4],
    pub step: [f32; 2],
    pub sigma: f32,
    pub stage: u32,
    pub bounds: [f32; 4],
    pub radii: [f32; 4],
    pub tint: [f32; 4],
    pub edge: [f32; 4],
    pub saturation: f32,
    pub lens: f32,
    pub opacity: f32,
    pub rim_band: f32,
}

impl GlassParam {
    /// The block as the bytes a uniform buffer takes.
    pub fn bytes(&self) -> [u8; 144] {
        // The stage is a whole number in the block; its bits ride as a float.
        let middle = [self.sigma, f32::from_bits(self.stage)];
        let tail = [self.saturation, self.lens, self.opacity, self.rim_band];
        let number: [&[f32]; 11] = [
            &self.rect,
            &self.target_size,
            &self.pixel_to_uv,
            &self.uv_limit,
            &self.step,
            &middle,
            &self.bounds,
            &self.radii,
            &self.tint,
            &self.edge,
            &tail,
        ];
        let mut out = [0u8; 144];
        for (slot, value) in out.chunks_exact_mut(4).zip(number.iter().flat_map(|part| part.iter())) {
            slot.copy_from_slice(&value.to_ne_bytes());
        }
        out
    }
}

/// One piece of glass on a window. Positions are in the window's pixels; the
/// material's lengths are logical and `scale_factor` turns them into pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlassRequest {
    /// Left, top, width, height.
    pub bounds: [f32; 4],
    /// What of it shows: left, top, right, bottom.
    pub visible: [f32; 4],
    /// Top left, top right, bottom right, bottom left.
    pub radii: [f32; 4],
    pub blur_radius: f32,
    pub saturation: f32,
    pub lens: f32,
    /// The tint, its alpha already times how far colour moves toward it.
    pub tint: [f32; 4],
    /// The rim's light, its alpha already times its strength.
    pub edge: [f32; 4],
    pub opacity: f32,
    pub scale_factor: f32,
}

impl GlassRequest {
    /// The request for `material` over `bounds`, of which `visible` shows.
    pub fn of(material: &crate::GlassMaterial, bounds: [f32; 4], visible: [f32; 4], radii: [f32; 4], opacity: f32, scale_factor: f32) -> Self {
        let [tint_r, tint_g, tint_b] = material.tint;
        let [edge_r, edge_g, edge_b] = material.edge;
        Self {
            bounds,
            visible,
            radii,
            blur_radius: material.blur_radius,
            saturation: material.saturation,
            lens: material.lens,
            tint: [tint_r, tint_g, tint_b, material.tint_amount],
            edge: [edge_r, edge_g, edge_b, material.edge_strength],
            opacity,
            scale_factor,
        }
    }
}

/// The copy and the four draws for one [`GlassRequest`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlassPlan {
    /// The window region to copy first: left, top, right, bottom.
    pub capture: [u32; 4],
    /// The size of the two layers the middle draws go between.
    pub small: [f32; 2],
    /// Shrink, blur across, blur down, the shape.
    pub stage: [GlassParam; 4],
}

impl GlassPlan {
    /// The plan for `request` on a window `width` by `height`, or nothing
    /// when none of it shows.
    pub fn of(request: &GlassRequest, width: u32, height: u32) -> Option<Self> {
        let shown = request.visible;
        let hidden = shown[2] <= shown[0] || shown[3] <= shown[1] || request.opacity <= 0.0;
        if hidden || width == 0 || height == 0 {
            return None;
        }
        let [x, y, w, h] = request.bounds;
        let half_side = w.min(h) * 0.5;
        let blur = request.blur_radius * request.scale_factor;
        let lens = (request.lens * request.scale_factor).min(half_side);
        let shrink = shrink_for(blur);
        let capture = capture_region(shown, blur * BLUR_REACH + lens, shrink, width, height);
        let [l, t, r, b] = capture.map(|side| side as f32);

        let (full_w, full_h) = (width as f32, height as f32);
        let (small_w, small_h) = (half_up(width), half_up(height));
        let scale = shrink as f32;
        let small_rect = [l / scale, t / scale, (r - l) / scale, (b - t) / scale];
        let small_limit = limit(small_rect, small_w, small_h);
        let base = GlassParam { rect: small_rect, target_size: [small_w, small_h], ..GlassParam::default() };
        // Four taps a quarter of the shrink out each cover a quarter of it.
        let shrinking = GlassParam {
            stage: STAGE_SHRINK,
            pixel_to_uv: [scale / full_w, scale / full_h],
            uv_limit: limit([l, t, r - l, b - t], full_w, full_h),
            step: [scale * 0.25 / full_w, scale * 0.25 / full_h],
            ..base
        };
        let blurring = GlassParam {
            stage: STAGE_BLUR,
            pixel_to_uv: [1.0 / small_w, 1.0 / small_h],
            uv_limit: small_limit,
            sigma: blur * 0.5 / scale,
            ..base
        };
        let shape = GlassParam {
            stage: STAGE_GLASS,
            rect: [shown[0], shown[1], shown[2] - shown[0], shown[3] - shown[1]],
            target_size: [full_w, full_h],
            pixel_to_uv: [1.0 / (scale * small_w), 1.0 / (scale * small_h)],
            uv_limit: small_limit,
            bounds: [x, y, w, h],
            radii: request.radii,
            tint: request.tint,
            edge: request.edge,
            saturation: request.saturation,
            lens,
            opacity: request.opacity,
            rim_band: (RIM_BAND * request.scale_factor).min(half_side).max(1.0),
            ..GlassParam::default()
        };
        let across = GlassParam { step: [1.0 / small_w, 0.0], ..blurring };
        let down = GlassParam { step: [0.0, 1.0 / small_h], ..blurring };
        Some(Self { capture, small: [small_w, small_h], stage: [shrinking, across, down, shape] })
    }
}

/// The blurred layers' side for a window side of this.
pub fn half_up(side: u32) -> f32 {
    side.div_ceil(2).max(1) as f32
}

/// How much smaller the copy is. A wide blur loses nothing at a quarter size
/// and costs a sixteenth as much.
pub fn shrink_for(blur: f32) -> u32 {
    match blur >= 12.0 {
        true => 4,
        false => 2,
    }
}

/// What to copy: `shown` and `margin` around it, inside the window, on whole
/// texels of the shrunk copy.
pub fn capture_region(shown: [f32; 4], margin: f32, shrink: u32, width: u32, height: u32) -> [u32; 4] {
    let down = |v: f32| (v - margin).max(0.0) as u32 / shrink * shrink;
    let up = |v: f32, side: u32| ((v + margin).ceil() as u32).div_ceil(shrink).saturating_mul(shrink).min(side);
    [down(shown[0]), down(shown[1]), up(shown[2], width), up(shown[3], height)]
}

/// The first and last texel centres of `rect`, as texture coordinates.
pub fn limit(rect: [f32; 4], width: f32, height: f32) -> [f32; 4] {
    [
        (rect[0] + 0.5) / width,
        (rect[1] + 0.5) / height,
        (rect[0] + rect[2] - 0.5) / width,
        (rect[1] + rect[3] - 0.5) / height,
    ]
}
