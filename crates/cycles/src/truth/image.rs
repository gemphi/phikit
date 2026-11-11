// SPDX-License-Identifier: Apache-2.0
//! The images the judge compares. Blender's runner hands them to `oiiotool`
//! (`tests/python/modules/render_report.py:138-144`), which reads a PNG as
//! floats, a byte over 255, without a colour transform, and an EXR as it is.
//! The same is done here with the workspace's `image` crate.
use std::path::Path;

use super::TruthError;

/// A picture as floats: `channels` of them a pixel, rows top to bottom.
#[derive(Debug, Clone, PartialEq)]
pub struct Image {
    pub width: usize,
    pub height: usize,
    /// 1 (grey), 2 (grey, alpha), 3 (RGB) or 4 (RGBA), as the file had.
    pub channels: usize,
    pub pixel: Vec<f32>,
}

impl Image {
    /// A black image (alpha 0 when it has one).
    pub fn new(width: usize, height: usize, channels: usize) -> Self {
        Self { width, height, channels, pixel: vec![0.0; width * height * channels] }
    }
    /// Channel `channel` of pixel `index`; 0 for a channel the image does
    /// not have, as OpenImageIO's `compare` reads a missing channel.
    pub fn channel(&self, index: usize, channel: usize) -> f32 {
        if channel < self.channels { self.pixel[index * self.channels + channel] } else { 0.0 }
    }
    /// Read a PNG, EXR or any format the `image` crate reads.
    pub fn read(path: &Path) -> Result<Self, TruthError> {
        let decoded = image::open(path).map_err(|error| TruthError::Image { path: path.display().to_string(), message: error.to_string() })?;
        let channels = decoded.color().channel_count() as usize;
        let rgba = decoded.to_rgba32f();
        let (width, height) = (rgba.width() as usize, rgba.height() as usize);
        let mut image = Self::new(width, height, channels);
        for (index, source) in rgba.pixels().enumerate() {
            let [r, g, b, a] = source.0;
            let wanted: &[f32] = match channels {
                1 => &[r],
                2 => &[r, a],
                3 => &[r, g, b],
                _ => &[r, g, b, a],
            };
            image.pixel[index * channels..(index + 1) * channels].copy_from_slice(wanted);
        }
        Ok(image)
    }
    /// Write as an 8-bit PNG (values clamped to 0..1 and rounded), the
    /// format of `oiiotool -o diff.png`. One or two channels are written
    /// as grey, three as RGB.
    pub fn write_png(&self, path: &Path) -> Result<(), TruthError> {
        let fail = |message: String| TruthError::Image { path: path.display().to_string(), message };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| fail(error.to_string()))?;
        }
        let byte = |value: f32| (value.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
        let (width, height) = (self.width as u32, self.height as u32);
        let at = |index: usize, channel: usize| byte(self.channel(index, channel));
        let saved = match self.channels {
            1 | 2 => {
                let grey = (0..self.width * self.height).map(|index| at(index, 0)).collect();
                image::GrayImage::from_raw(width, height, grey).map(|image| image.save(path))
            }
            3 => {
                let rgb = (0..self.width * self.height).flat_map(|index| [at(index, 0), at(index, 1), at(index, 2)]).collect();
                image::RgbImage::from_raw(width, height, rgb).map(|image| image.save(path))
            }
            _ => {
                let rgba = (0..self.width * self.height).flat_map(|index| [at(index, 0), at(index, 1), at(index, 2), at(index, 3)]).collect();
                image::RgbaImage::from_raw(width, height, rgba).map(|image| image.save(path))
            }
        };
        saved.ok_or_else(|| fail("the pixels do not fit the size".into()))?.map_err(|error| fail(error.to_string()))
    }
}
