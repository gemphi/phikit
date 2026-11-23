//! The Subsurface Scale chain read from its images (`BlendFile::scale_bake`).
//!
//! CW-POSES' skin plugs a `Subsurface` image into the Principled's
//! Subsurface Scale: red where light should travel far under the skin. A
//! linked float input the fold cannot answer exactly keeps its socket's
//! default, so every such skin scattered at 0.05 whatever its image said.
//!
//! Two things come out of the chain. The material's own scale is its mean
//! over the texels the images cover (a tile's empty margin is no part of
//! the surface): what a ray that reads no texture scatters by. And the
//! value at each texel, as a share of that mean, goes into the ALPHA of
//! the roughness map, which the kernel reads raw and used only the red
//! of: one image carrying two of a surface's numbers, as a packed
//! occlusion-roughness-metal image does. The kernel scales the scattering
//! distance by it at the camera's hit (`shade.wgsl`).
//!
//! The share is stored over [`REACH`], and never as 255: an image plugged
//! straight into Roughness is opaque, alpha 255, and that value says
//! "nothing here" to the kernel.

use super::material::{SceneMap, SceneMaterial};
use super::texture::{SceneTexture, TextureCache, tile_scale};
use crate::blend::shade::bake::{self, BlendBake};
use std::sync::Arc;

/// Texels across one UV square for the mean, and for a map made here.
const SIDE: u32 = 256;

/// Covered at all, for a texel to count.
const COVERED: f32 = 0.5;

/// The largest share of the mean a texel can hold: alpha 254 is this.
pub const REACH: f32 = 4.0;

/// What marks a roughness map as baked here, over the mesh's own UVs
/// (`texture_rough.rs` names it so).
const BAKED: &str = "(Roughness, baked)";

/// The chain's images, ready to sample at a UV in tile squares.
struct Chain<'a> {
    program: &'a BlendBake,
    source_list: Vec<Option<SceneTexture>>,
    tile_list: Vec<[u32; 2]>,
    grid: [u32; 2],
}

impl<'a> Chain<'a> {
    fn of(program: &'a BlendBake, cache: &mut TextureCache) -> Option<Self> {
        let source_list: Vec<Option<SceneTexture>> = program.images.iter().map(|i| cache.get(&i.key).map(|t| if i.straight { t } else { t.associated() })).collect();
        source_list.iter().any(Option::is_some).then_some(())?;
        let tile_list: Vec<[u32; 2]> = program.images.iter().map(|i| cache.grid(&i.key)).collect();
        let grid = tile_list.iter().fold([1u32, 1], |most, g| [most[0].max(g[0]), most[1].max(g[1])]);
        Some(Self { program, source_list, tile_list, grid })
    }

    fn sample(&self, i: u16, uv: [f32; 2]) -> Option<[f32; 4]> {
        let t = self.source_list.get(i as usize).and_then(Option::as_ref)?;
        Some(t.sample_smooth(bake::apply(tile_scale(self.tile_list[i as usize]), bake::apply(self.program.images[i as usize].affine, uv))))
    }

    /// The chain's value at `uv`, or `None` where no image covers it.
    fn at(&self, uv: [f32; 2]) -> Option<f32> {
        let covered = (0..self.program.images.len() as u16).any(|i| self.sample(i, uv).is_some_and(|c| c[3] > COVERED));
        covered.then(|| {
            // A missing source reads as nothing: the magenta of a missing
            // image is a colour to show, not a distance to scatter.
            let [r, g, b, _] = bake::run(&self.program.ops, &|i| self.sample(i, uv).unwrap_or([0.0, 0.0, 0.0, 1.0]));
            0.2126 * r + 0.7152 * g + 0.0722 * b
        })
    }

    /// The mean over the covered texels of the UV squares the images run
    /// over. `None` when none covers anything.
    fn mean(&self) -> Option<f32> {
        let (mut sum, mut count) = (0.0f64, 0u32);
        for y in 0..SIDE * self.grid[1] {
            for x in 0..SIDE * self.grid[0] {
                if let Some(v) = self.at([(x as f32 + 0.5) / SIDE as f32, (y as f32 + 0.5) / SIDE as f32]) {
                    sum += f64::from(v);
                    count += 1;
                }
            }
        }
        (count > 0).then(|| (sum / f64::from(count)) as f32)
    }
}

/// Give `m` the Subsurface Scale its chain says, when it has one and an
/// image of it was found; the folded value stands otherwise.
pub(super) fn attach(m: &mut SceneMaterial, cache: &mut TextureCache) {
    let Some(program) = m.scale_bake.clone() else { return };
    let Some(chain) = Chain::of(&program, cache) else { return };
    let Some(mean) = chain.mean().filter(|mean| *mean > 0.0) else { return };
    m.bsdf.subsurface_scale = mean;
    // The map shares the mesh's UVs, as the baked roughness does: plain,
    // or scaled onto the tile grid. Any other layout keeps the mean alone.
    let laid = match m.uv_affine {
        None => chain.grid == [1, 1],
        Some(affine) => chain.grid != [1, 1] && affine == tile_scale(chain.grid),
    };
    let own = m.maps.rough.as_ref().map(|map| (map.key.ends_with(BAKED), map.image.clone()));
    let base = match own {
        // An image plugged straight into Roughness is not ours to repaint.
        Some((false, _)) => None,
        Some((true, image)) => image,
        None => Some(white(SIDE * chain.grid[0], SIDE * chain.grid[1])),
    };
    let (Some(base), true) = (base, laid) else { return };
    m.maps.rough = Some(SceneMap { key: format!("{} {BAKED}", m.name), image: Some(with_share(&base, &chain, mean)) });
}

/// A roughness map that changes nothing: every value one.
fn white(w: u32, h: u32) -> SceneTexture {
    SceneTexture { width: w, height: h, rgba: Arc::new(vec![255u8; (w * h * 4) as usize]) }
}

/// `base` with each texel's share of `mean` in its alpha.
fn with_share(base: &SceneTexture, chain: &Chain, mean: f32) -> SceneTexture {
    let (w, h) = (base.width.max(1), base.height.max(1));
    let (cols, rows) = (chain.grid[0] as f32, chain.grid[1] as f32);
    let mut rgba = base.rgba.as_ref().clone();
    for y in 0..h {
        for x in 0..w {
            let uv = [(x as f32 + 0.5) / w as f32 * cols, rows - (y as f32 + 0.5) / h as f32 * rows];
            // Off the surface the mean stands, so a sample blended across
            // an island's edge is not pulled toward nothing.
            let share = chain.at(uv).map_or(1.0, |v| v / mean);
            if let Some(alpha) = rgba.get_mut(((y * w + x) * 4 + 3) as usize) {
                *alpha = share_byte(share);
            }
        }
    }
    SceneTexture { width: w, height: h, rgba: Arc::new(rgba) }
}

/// A share of the mean as the byte the kernel reads back: 254 is
/// [`REACH`], and 255 is never written.
fn share_byte(share: f32) -> u8 {
    ((share / REACH).clamp(0.0, 1.0) * 254.0 + 0.5) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blend::shade::bake::{BBakeImage, BBakeOp, IDENTITY};

    fn half_covered() -> (BlendBake, TextureCache) {
        let mut png = Vec::new();
        let pixels: Vec<u8> = (0..4 * 4).flat_map(|i| if i % 4 < 2 { [255u8, 255, 255, 255] } else { [0, 0, 0, 0] }).collect();
        image::RgbaImage::from_raw(4, 4, pixels).expect("pixels").write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png).expect("png");
        let program = BlendBake { images: vec![BBakeImage { key: "half.png".into(), affine: IDENTITY, straight: false }], ops: vec![BBakeOp::Image(0)], alpha: None };
        (program, TextureCache::new(&[("half.png".to_string(), Arc::new(png))], None))
    }

    #[test]
    fn a_chain_whose_image_is_absent_leaves_the_scale_alone() {
        let program = BlendBake { images: vec![BBakeImage { key: "absent.png".into(), affine: IDENTITY, straight: false }], ops: vec![BBakeOp::Image(0)], alpha: None };
        let mut cache = TextureCache::new(&[], None);
        assert!(Chain::of(&program, &mut cache).is_none());
    }

    /// Half the image covered and white, half a hole: the mean is the
    /// covered half's value, not half of it.
    #[test]
    fn the_mean_is_over_what_the_image_covers() {
        let (program, mut cache) = half_covered();
        let mean = Chain::of(&program, &mut cache).expect("an image was found").mean().expect("covered");
        assert!(mean > 0.9, "{mean}");
    }

    /// The mean itself is a share of one, a quarter of the reach; the byte
    /// an opaque image holds is never written.
    #[test]
    fn a_share_is_stored_over_the_reach_and_never_as_opaque() {
        assert_eq!(share_byte(1.0), 64);
        assert_eq!(share_byte(REACH), 254);
        assert_eq!(share_byte(100.0), 254);
        assert_eq!(share_byte(0.0), 0);
    }
}
