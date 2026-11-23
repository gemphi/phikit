//! What the pipeline draws on: pixels it can read and write.

/// A surface holding everything drawn so far, as `0xAARRGGBB`. Glass reads
/// what is below it from here and writes its result back.
pub trait GlassCanvas {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn pixel(&self, x: usize, y: usize) -> u32;
    fn set_pixel(&mut self, x: usize, y: usize, argb: u32);
}

/// The UI's software buffer.
pub struct BufferCanvas<'a> {
    pixels: &'a mut [u32],
    width: usize,
    height: usize,
}

impl<'a> BufferCanvas<'a> {
    pub fn new(pixels: &'a mut [u32], width: usize, height: usize) -> Self {
        assert_eq!(pixels.len(), width * height, "a buffer is width x height pixels");
        Self { pixels, width, height }
    }
}

impl GlassCanvas for BufferCanvas<'_> {
    fn width(&self) -> usize {
        self.width
    }
    fn height(&self) -> usize {
        self.height
    }
    fn pixel(&self, x: usize, y: usize) -> u32 {
        self.pixels[y * self.width + x]
    }
    fn set_pixel(&mut self, x: usize, y: usize, argb: u32) {
        self.pixels[y * self.width + x] = argb;
    }
}

/// `0xAARRGGBB` as red, green and blue in 0..=1. Colour stays gamma encoded
/// throughout, as a browser's `backdrop-filter` keeps it.
pub(super) fn to_rgb(argb: u32) -> [f32; 3] {
    let unit = |shift: u32| ((argb >> shift) & 0xFF) as f32 / 255.0;
    [unit(16), unit(8), unit(0)]
}

/// Back to an opaque `0xFFRRGGBB`.
pub(super) fn from_rgb(rgb: [f32; 3]) -> u32 {
    let byte = |c: f32| (c.clamp(0.0, 1.0) * 255.0 + 0.5) as u32;
    (0xFF << 24) | (byte(rgb[0]) << 16) | (byte(rgb[1]) << 8) | byte(rgb[2])
}
