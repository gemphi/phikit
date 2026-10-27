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

pub const MAX_TEXTURE: u32 = 1024;

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

pub(crate) fn srgb_to_linear(c: u8) -> f32 {
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
    Self::candidates_in(key, blend.and_then(Path::parent))
}

/// The same, given the `.blend`'s folder: the path as written (`//`
/// resolved against the folder), the file name beside the `.blend`, and —
/// Find Missing Files — the file name anywhere under the folder's tree or
/// its `textures/` (`texture_find.rs`).
///
/// A tiled image's path carries a `<UDIM>` token (`BKE_image_get_tile_*`);
/// the first tile, 1001, is the one UV square the kernel samples, so the
/// token resolves to it here.
pub fn candidates_in(key: &str, dir: Option<&Path>) -> Vec<PathBuf> {
    let key = key.replace("<UDIM>", "1001");
    let mut out = Vec::new();
    match (key.strip_prefix("//"), dir) {
        (Some(rel), Some(d)) => out.push(d.join(rel.replace('\\', "/"))),
        (None, _) => out.push(PathBuf::from(&key)),
        _ => {}
    }
    let name = key.rsplit(['/', '\\']).next().unwrap_or(&key);
    if let (Some(d), false) = (dir, name.is_empty()) {
        out.push(d.join(name));
        if let Some(found) = super::texture_find::find(d, name) {
            out.push(found);
        }
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
    // A Sky Texture is an environment image too: Blender bakes it rather
    // than evaluating it per ray (`node_shader_tex_sky.cc`), so baking it
    // here hands the sky to the one path that already carries a dome
    // correctly — the CDF, the prefiltered chain and the MIS weight
    // (`rendering.md` §59, §62).
    if let Some(sky) = p.world.sky {
        return Some(SceneEnvironment {
            width: crate::sky::SKY_WIDTH,
            height: crate::sky::SKY_HEIGHT,
            rgb: std::sync::Arc::new(sky.bake()),
        });
    }
    let key = p.world.texture.as_ref()?;
    let blend = Path::new(source_path);
    let bytes = match p.packed_images.iter().find(|(k, _)| k == key) {
        Some((_, b)) => b.as_ref().clone(),
        None => SceneTexture::candidates(key, Some(blend)).iter().find_map(|c| std::fs::read(c).ok())?,
    };
    SceneEnvironment::decode(&bytes)
}

/// Every image a material asked for, and what to do about the ones that
/// were neither packed nor on disk — base colour, normal map and the other
/// maps alike.
///
/// Blender prints its `Cannot read '<path>'` per image; a file asking for
/// three hundred of another machine's textures turns that into a wall with
/// the answer nowhere in it, so the tally leads with the folders owed and
/// keeps Blender's lines as evidence (`texture_miss.rs`, which carries the
/// reasoning and the divergence).
pub(super) fn attach(objects: &mut [super::SceneObject], p: &crate::blend::BlendSceneProps, source_path: &str) -> Vec<String> {
    let blend = std::path::Path::new(source_path);
    let mut cache = TextureCache::new(&p.packed_images, Some(blend));
    let mut missing = super::texture_miss::BMissingTexture::default();
    // A bake belongs to the material, not to the object wearing it: every
    // object in its slot carries the same program over the same images. It
    // was run again per object, on one core at the images' full size — the
    // crystal caves city ran 3,101 bakes for 89 distinct ones, ten minutes
    // of its open, and kept every copy. A list, not a map: `BlendBake` holds
    // floats, so it is `PartialEq` and no more, and there are few of them.
    let mut baked: Vec<(crate::blend::shade::bake::BlendBake, Option<(SceneTexture, Option<[[f32; 3]; 2]>)>)> = Vec::new();
    for m in objects.iter_mut().flat_map(|o| o.materials.iter_mut()) {
        if let Some((key, _)) = m.normal_texture.clone() {
            m.normal_map = cache.get(&key);
            missing.note(&key, m.normal_map.is_some());
        }
        for slot in m.maps.each_mut_pub() {
            if let Some(map) = slot {
                map.image = cache.get(&map.key);
                missing.note(&map.key, map.image.is_some());
            }
        }
        // The Base Color chain baked over its decoded images
        // (`blend::shade::bake`): the result carries the whole base colour,
        // so a pattern that only drives a Mix's factor still shows.
        if let Some(bake) = m.bake.clone() {
            for img in &bake.images {
                missing.note(&img.key, cache.get(&img.key).is_some());
            }
            let done = match baked.iter().find(|(seen, _)| *seen == bake) {
                Some((_, done)) => done.clone(),
                None => {
                    let done = run_bake(&bake, &mut cache);
                    baked.push((bake, done.clone()));
                    done
                }
            };
            if let Some((tex, affine)) = done {
                m.texture = Some(tex);
                m.uv_affine = affine;
                m.baked = true;
                continue;
            }
        }
        let Some(key) = m.base_color_texture.clone() else { continue };
        m.texture = cache.get(&key);
        missing.note(&key, m.texture.is_some());
    }
    missing.lines()
}

/// Linear back to the sRGB bytes `SceneTexture` stores.
fn linear_to_srgb(v: f32) -> u8 {
    let v = v.clamp(0.0, 1.0);
    let s = if v <= 0.003_130_8 { v * 12.92 } else { 1.055 * v.powf(1.0 / 2.4) - 0.055 };
    (s * 255.0 + 0.5) as u8
}

/// How wide a several-image bake's grid is at most.
const BAKE_GRID: u32 = 512;

/// Run a compiled Base Color bake over its decoded images.
///
/// One source image: every texel of that image through the program — a
/// recolour at the image's own resolution — with its Mapping answered
/// separately for the mesh's UVs. Several: the program over the UV unit
/// square, each image sampled through its own affine. `None` when no source
/// image resolved, so the caller falls back to the folded tint.
fn run_bake(bake: &crate::blend::shade::bake::BlendBake, cache: &mut TextureCache) -> Option<(SceneTexture, Option<[[f32; 3]; 2]>)> {
    use crate::blend::shade::bake as b;
    let srcs: Vec<Option<SceneTexture>> = bake.images.iter().map(|i| cache.get(&i.key)).collect();
    srcs.iter().any(Option::is_some).then_some(())?;
    if let (Some(one), Some(Some(src))) = (bake.single(), srcs.first()) {
        let (w, h) = (src.width.max(1), src.height.max(1));
        let mut out = Vec::with_capacity((w * h * 4) as usize);
        for i in 0..(w * h) as usize {
            let p = src.rgba.get(i * 4..i * 4 + 4)?;
            let texel = [srgb_to_linear(p[0]), srgb_to_linear(p[1]), srgb_to_linear(p[2]), p[3] as f32 / 255.0];
            let c = b::run(&bake.ops, &|_| texel);
            out.extend([linear_to_srgb(c[0]), linear_to_srgb(c[1]), linear_to_srgb(c[2]), (c[3].clamp(0.0, 1.0) * 255.0 + 0.5) as u8]);
        }
        return Some((SceneTexture { width: w, height: h, rgba: Arc::new(out) }, Some(one.affine)));
    }
    let side = srcs.iter().flatten().map(|t| t.width.max(t.height)).max().unwrap_or(BAKE_GRID).clamp(64, BAKE_GRID);
    let mut out = Vec::with_capacity((side * side * 4) as usize);
    for y in 0..side {
        for x in 0..side {
            let uv = [(x as f32 + 0.5) / side as f32, 1.0 - (y as f32 + 0.5) / side as f32];
            let c = b::run(&bake.ops, &|i| match srcs.get(i as usize).and_then(Option::as_ref) {
                Some(t) => t.sample_smooth(b::apply(bake.images[i as usize].affine, uv)),
                // A missing source draws Blender's missing-image magenta.
                None => [1.0, 0.0, 1.0, 1.0],
            });
            out.extend([linear_to_srgb(c[0]), linear_to_srgb(c[1]), linear_to_srgb(c[2]), (c[3].clamp(0.0, 1.0) * 255.0 + 0.5) as u8]);
        }
    }
    Some((SceneTexture { width: side, height: side, rgba: Arc::new(out) }, None))
}
