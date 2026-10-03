//! Base-colour textures for Material Preview (#26).
//!
//! Resolution follows Blender's `BKE_image_load` order: packed bytes first,
//! then `Image.filepath` with `//` resolved against the `.blend`'s folder
//! (`BLI_path_abs`), then the bare file name next to the `.blend`. Images
//! are decoded once per load and reduced to at most [`MAX_TEXTURE`] px per
//! side.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub const MAX_TEXTURE: u32 = 256;

/// Widest environment map kept, in pixels. A world image lights every ray, so
/// it keeps more detail than a surface texture — but not 4K, which would cost
/// 32 MB on the GPU for light that is mostly low frequency.
pub const MAX_ENVIRONMENT: u32 = 1024;

/// A decoded environment map: linear RGB, equirectangular, rows top-down.
/// Kept as floats because an HDRI's sun is far brighter than white.
#[derive(Clone, Debug, PartialEq)]
pub struct SceneEnvironment {
    pub width: u32,
    pub height: u32,
    pub rgb: std::sync::Arc<Vec<f32>>,
}

impl SceneEnvironment {
    /// Decode EXR / Radiance HDR (already linear) or an LDR image (sRGB).
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let img = image::load_from_memory(bytes).ok()?;
        let img = if img.width() > MAX_ENVIRONMENT {
            img.resize(MAX_ENVIRONMENT, MAX_ENVIRONMENT / 2, image::imageops::FilterType::Triangle)
        } else {
            img
        };
        let rgb = img.to_rgb32f();
        Some(Self { width: rgb.width(), height: rgb.height(), rgb: std::sync::Arc::new(rgb.into_raw()) })
    }

    /// Mean radiance over the whole sphere — what a surface facing nowhere in
    /// particular receives, and the fallback when the image cannot be sampled.
    pub fn average(&self) -> [f32; 3] {
        let n = (self.rgb.len() / 3).max(1);
        let mut sum = [0.0f64; 3];
        for px in self.rgb.chunks_exact(3) {
            for k in 0..3 {
                sum[k] += px[k] as f64;
            }
        }
        [0, 1, 2].map(|k| (sum[k] / n as f64) as f32)
    }
}

/// Decoded sRGB RGBA8 pixels, rows top-down.
#[derive(Clone, Debug, PartialEq)]
pub struct SceneTexture {
    pub width: u32,
    pub height: u32,
    pub rgba: Arc<Vec<u8>>,
}

fn srgb_to_linear(c: u8) -> f32 {
    let c = c as f32 / 255.0;
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

impl SceneTexture {
    /// Bilinear sample with repeat wrapping, as linear RGBA.
    ///
    /// The viewport shades per pixel, so a face is not one texel any more:
    /// neighbouring texels are blended by the fractional part of the sample
    /// position, which is what stops a texture from reading as blocks when a
    /// face is drawn larger than the image it carries.
    pub fn sample_smooth(&self, uv: [f32; 2]) -> [f32; 4] {
        let (w, h) = (self.width.max(1), self.height.max(1));
        // Half-texel offset: a sample at a texel's centre must return that
        // texel exactly, not a blend of it with its neighbour.
        let x = uv[0].rem_euclid(1.0) * w as f32 - 0.5;
        let y = (1.0 - uv[1].rem_euclid(1.0)) * h as f32 - 0.5;
        let (x0, y0) = (x.floor(), y.floor());
        let (fx, fy) = (x - x0, y - y0);
        let wrap = |v: f32, n: u32| v.rem_euclid(n as f32) as u32 % n;
        let (x0i, y0i) = (wrap(x0, w), wrap(y0, h));
        let (x1i, y1i) = (wrap(x0 + 1.0, w), wrap(y0 + 1.0, h));
        let texel = |x: u32, y: u32| -> [f32; 4] {
            let i = ((y * w + x) * 4) as usize;
            match self.rgba.get(i..i + 4) {
                Some(p) => [
                    srgb_to_linear(p[0]),
                    srgb_to_linear(p[1]),
                    srgb_to_linear(p[2]),
                    p[3] as f32 / 255.0,
                ],
                None => [1.0, 0.0, 1.0, 1.0],
            }
        };
        let (a, b, c, d) = (
            texel(x0i, y0i),
            texel(x1i, y0i),
            texel(x0i, y1i),
            texel(x1i, y1i),
        );
        std::array::from_fn(|k| {
            let top = a[k] + (b[k] - a[k]) * fx;
            let bottom = c[k] + (d[k] - c[k]) * fx;
            top + (bottom - top) * fy
        })
    }

    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let img = image::load_from_memory(bytes).ok()?;
        let img = if img.width() > MAX_TEXTURE || img.height() > MAX_TEXTURE {
            img.resize(MAX_TEXTURE, MAX_TEXTURE, image::imageops::FilterType::Triangle)
        } else {
            img
        };
        let rgba = img.to_rgba8();
        Some(Self { width: rgba.width(), height: rgba.height(), rgba: Arc::new(rgba.into_raw()) })
    }

    /// Closest-pixel sample with repeat wrapping, as linear RGBA. UV origin
    /// is bottom-left (Blender), pixel rows are top-down.
    pub fn sample(&self, uv: [f32; 2]) -> [f32; 4] {
        let (w, h) = (self.width.max(1), self.height.max(1));
        let u = uv[0].rem_euclid(1.0);
        let v = 1.0 - uv[1].rem_euclid(1.0);
        let x = ((u * w as f32) as u32).min(w - 1);
        let y = ((v * h as f32) as u32).min(h - 1);
        let i = ((y * w + x) * 4) as usize;
        match self.rgba.get(i..i + 4) {
            Some(p) => [srgb_to_linear(p[0]), srgb_to_linear(p[1]), srgb_to_linear(p[2]), p[3] as f32 / 255.0],
            None => [1.0, 0.0, 1.0, 1.0],
        }
    }
}

impl SceneTexture {
/// Candidate on-disk locations for an image key.
pub fn candidates(key: &str, blend: Option<&Path>) -> Vec<PathBuf> {
    let dir = blend.and_then(Path::parent);
    let mut out = Vec::new();
    match (key.strip_prefix("//"), dir) {
        (Some(rel), Some(d)) => out.push(d.join(rel.replace('\\', "/"))),
        (None, _) => out.push(PathBuf::from(key)),
        _ => {}
    }
    let name = key.rsplit(['/', '\\']).next().unwrap_or(key);
    if let (Some(d), false) = (dir, name.is_empty()) {
        out.push(d.join(name));
    }
    out
}
}

/// Decodes each image key once per load.
#[derive(Default)]
pub struct TextureCache {
    packed: HashMap<String, Arc<Vec<u8>>>,
    blend: Option<PathBuf>,
    done: HashMap<String, Option<SceneTexture>>,
}

impl TextureCache {
    pub fn new(packed: &[(String, Arc<Vec<u8>>)], blend: Option<&Path>) -> Self {
        Self { packed: packed.iter().cloned().collect(), blend: blend.map(Path::to_path_buf), done: HashMap::new() }
    }

    pub fn get(&mut self, key: &str) -> Option<SceneTexture> {
        if let Some(t) = self.done.get(key) {
            return t.clone();
        }
        let tex = match self.packed.get(key) {
            Some(bytes) => SceneTexture::decode(bytes),
            None => SceneTexture::candidates(key, self.blend.as_deref())
                .iter()
                .find_map(|p| std::fs::read(p).ok())
                .and_then(|b| SceneTexture::decode(&b)),
        };
        self.done.insert(key.to_string(), tex.clone());
        tex
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_paths_resolve_next_to_the_blend() {
        let c = SceneTexture::candidates("//tex\\a.png", Some(Path::new("/w/scene.blend")));
        assert_eq!(c[0], PathBuf::from("/w/tex/a.png"));
        assert_eq!(c[1], PathBuf::from("/w/a.png"));
    }

    #[test]
    fn sample_is_bottom_up_and_wraps() {
        // 2×2, rows top-down: TL blue, TR yellow / BL red, BR green.
        let px = [0, 0, 255, 255, 255, 255, 0, 255, 255, 0, 0, 255, 0, 255, 0, 255];
        let t = SceneTexture { width: 2, height: 2, rgba: Arc::new(px.to_vec()) };
        assert_eq!(t.sample([0.25, 0.25]), [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(t.sample([0.75, 0.75]), [1.0, 1.0, 0.0, 1.0]);
        assert_eq!(t.sample([1.25, 0.25]), [1.0, 0.0, 0.0, 1.0]);
    }
    #[test]
    fn a_smooth_sample_at_a_texel_centre_is_that_texel() {
        // 2×2, rows top-down: TL blue, TR yellow / BL red, BR green.
        let px = [0, 0, 255, 255, 255, 255, 0, 255, 255, 0, 0, 255, 0, 255, 0, 255];
        let tex = SceneTexture { width: 2, height: 2, rgba: std::sync::Arc::new(px.to_vec()) };
        // Texel centres in UV space (origin bottom-left) for the bottom row.
        let left = tex.sample_smooth([0.25, 0.25]);
        assert!(left[0] > 0.9 && left[1] < 0.1, "bottom-left is red: {left:?}");
        let right = tex.sample_smooth([0.75, 0.25]);
        assert!(right[1] > 0.9 && right[0] < 0.1, "bottom-right is green: {right:?}");
    }

    #[test]
    fn a_smooth_sample_between_texels_blends_them() {
        let px = [0, 0, 0, 255, 255, 255, 255, 255, 0, 0, 0, 255, 255, 255, 255, 255];
        let tex = SceneTexture { width: 2, height: 2, rgba: std::sync::Arc::new(px.to_vec()) };
        // Halfway between a black and a white texel on the bottom row.
        let mid = tex.sample_smooth([0.5, 0.25]);
        let near = tex.sample([0.5, 0.25]);
        assert!(mid[0] > 0.05 && mid[0] < 0.95, "the blend is neither texel: {mid:?}");
        assert!(
            (mid[0] - near[0]).abs() > 0.05,
            "and differs from the nearest-texel answer"
        );
    }
}

/// Decode each material's base-colour image once; returns the keys that
/// could not be found (Blender logs these as missing images).
/// The world's environment image, decoded the way surface textures are.
pub(super) fn environment(p: &crate::blend::BlendSceneProps, source_path: &str) -> Option<SceneEnvironment> {
    let key = p.world.texture.as_ref()?;
    let blend = Path::new(source_path);
    let bytes = match p.packed_images.iter().find(|(k, _)| k == key) {
        Some((_, b)) => b.as_ref().clone(),
        None => SceneTexture::candidates(key, Some(blend)).iter().find_map(|c| std::fs::read(c).ok())?,
    };
    SceneEnvironment::decode(&bytes)
}

/// Every image a material asked for that was neither packed nor on disk —
/// base colour, normal map and the other maps alike — as the message
/// Blender prints for it: `Cannot read '<path>': No such file or directory`,
/// with the path the `//`-relative name resolved to beside the file.
pub(super) fn attach(objects: &mut [super::SceneObject], p: &crate::blend::BlendSceneProps, source_path: &str) -> Vec<String> {
    let blend = std::path::Path::new(source_path);
    let mut cache = TextureCache::new(&p.packed_images, Some(blend));
    let mut missing: Vec<String> = Vec::new();
    let note = |key: &str, found: bool, missing: &mut Vec<String>| {
        if found {
            return;
        }
        let path = SceneTexture::candidates(key, Some(blend)).into_iter().next().unwrap_or_else(|| PathBuf::from(key));
        let line = format!("Cannot read '{}': No such file or directory", path.display());
        if !missing.contains(&line) {
            missing.push(line);
        }
    };
    for m in objects.iter_mut().flat_map(|o| o.materials.iter_mut()) {
        if let Some((key, _)) = m.normal_texture.clone() {
            m.normal_map = cache.get(&key);
            note(&key, m.normal_map.is_some(), &mut missing);
        }
        for slot in m.maps.each_mut_pub() {
            if let Some(map) = slot {
                map.image = cache.get(&map.key);
                note(&map.key, map.image.is_some(), &mut missing);
            }
        }
        let Some(key) = m.base_color_texture.clone() else { continue };
        m.texture = cache.get(&key);
        note(&key, m.texture.is_some(), &mut missing);
    }
    missing
}
