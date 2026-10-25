//! An image as the generators paint it: sRGB, 8 bits a channel, rows top
//! down — what Phiber's textures hold, so it goes across without a copy.

use crate::colour::Rgb;

#[derive(Clone, Debug, PartialEq)]
pub struct BImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl BImage {
    /// `width` × `height` of one colour.
    pub fn filled(width: u32, height: u32, colour: Rgb) -> Self {
        let px = to_bytes(colour);
        BImage { width, height, rgba: px.repeat((width * height) as usize) }
    }

    /// Painted pixel by pixel: `paint(u, v)` with u, v the pixel's centre
    /// in 0–1, v = 0 at the top row.
    pub fn painted(width: u32, height: u32, paint: impl Fn(f32, f32) -> Rgb) -> Self {
        let mut rgba = Vec::with_capacity((width * height * 4) as usize);
        for y in 0..height {
            for x in 0..width {
                let (u, v) = ((x as f32 + 0.5) / width as f32, (y as f32 + 0.5) / height as f32);
                rgba.extend(to_bytes(paint(u, v)));
            }
        }
        BImage { width, height, rgba }
    }

    /// The pixel at (`x`, `y`).
    pub fn at(&self, x: u32, y: u32) -> Rgb {
        let i = ((y * self.width + x) * 4) as usize;
        Rgb(self.rgba[i] as f32 / 255.0, self.rgba[i + 1] as f32 / 255.0, self.rgba[i + 2] as f32 / 255.0)
    }
}

fn to_bytes(c: Rgb) -> [u8; 4] {
    let b = |x: f32| (x.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
    [b(c.0), b(c.1), b(c.2), 255]
}
