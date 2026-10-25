//! A colour as the sets write it: sRGB, from the hex a designer reads.
//!
//! Sets are `static`, so a colour has to be built in a `const` context;
//! [`Rgb::hex`] is a `const fn` for that reason. Renderers want scene-linear
//! light, which [`Rgb::linear`] gives.

/// An sRGB colour, each channel 0–1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgb(pub f32, pub f32, pub f32);

impl Rgb {
    /// From `0xRRGGBB`.
    pub const fn hex(rgb: u32) -> Self {
        Self(((rgb >> 16) & 0xff) as f32 / 255.0, ((rgb >> 8) & 0xff) as f32 / 255.0, (rgb & 0xff) as f32 / 255.0)
    }

    /// Scene-linear, for a material or a light.
    pub fn linear(self) -> [f32; 3] {
        let f = |c: f32| if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) };
        [f(self.0), f(self.1), f(self.2)]
    }

    /// The sRGB channels as an array.
    pub const fn srgb(self) -> [f32; 3] {
        [self.0, self.1, self.2]
    }

    /// `self` moved toward `other` by `t`, in sRGB (how a designer mixes).
    pub fn mix(self, other: Rgb, t: f32) -> Rgb {
        Rgb(self.0 + (other.0 - self.0) * t, self.1 + (other.1 - self.1) * t, self.2 + (other.2 - self.2) * t)
    }

    /// Relative luminance of the linear colour (Rec. 709 weights): what
    /// decides whether a number on it is printed light or dark.
    pub fn luminance(self) -> f32 {
        let [r, g, b] = self.linear();
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }
}

/// White and ink, the two inks every kit's numbers choose between.
pub const WHITE: Rgb = Rgb::hex(0xF7F5F0);
pub const INK: Rgb = Rgb::hex(0x16171C);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_reads_its_channels_and_linear_darkens_the_mids() {
        assert_eq!(Rgb::hex(0xFF8000).srgb(), [1.0, 128.0 / 255.0, 0.0]);
        let [r, g, _] = Rgb::hex(0xFF8000).linear();
        assert!((r - 1.0).abs() < 1e-6 && (g - 0.2158).abs() < 1e-3, "sRGB 0.5 is linear 0.216");
        assert!(WHITE.luminance() > 0.5 && INK.luminance() < 0.02);
    }
}
