//! The Subsurface Scale chain read from its images (`BlendFile::scale_bake`).
//!
//! CW-POSES' skin plugs a `Subsurface` image into the Principled's
//! Subsurface Scale: red where light should travel far under the skin. A
//! linked float input the fold cannot answer exactly keeps its socket's
//! default, so every such skin scattered at 0.05 whatever its image said.
//!
//! The kernel takes one scattering distance for a material, not a map, so
//! the chain is run over the UV squares its images cover and the material
//! takes the mean over the texels the images cover (a tile's empty margin
//! is no part of the surface). What this loses is the variation across
//! the surface; a map slot in the kernel is the whole answer.

use super::material::SceneMaterial;
use super::texture::{SceneTexture, TextureCache, tile_scale};
use crate::blend::shade::bake::{self, BlendBake};

/// Texels across one UV square: a mean needs few.
const SIDE: u32 = 128;

/// Covered at all, for a texel to count.
const COVERED: f32 = 0.5;

/// Give `m` the Subsurface Scale its chain says, when it has one and an
/// image of it was found; the folded value stands otherwise.
pub(super) fn attach(m: &mut SceneMaterial, cache: &mut TextureCache) {
    let Some(program) = m.scale_bake.clone() else { return };
    if let Some(mean) = mean(&program, cache) {
        m.bsdf.subsurface_scale = mean;
    }
}

/// The chain's mean over the covered texels of the UV squares its images
/// run over. `None` when no image resolved or none covers anything.
fn mean(program: &BlendBake, cache: &mut TextureCache) -> Option<f32> {
    let source_list: Vec<Option<SceneTexture>> = program.images.iter().map(|i| cache.get(&i.key).map(|t| if i.straight { t } else { t.associated() })).collect();
    source_list.iter().any(Option::is_some).then_some(())?;
    let tile_list: Vec<[u32; 2]> = program.images.iter().map(|i| cache.grid(&i.key)).collect();
    let grid = tile_list.iter().fold([1u32, 1], |most, g| [most[0].max(g[0]), most[1].max(g[1])]);
    let at = |i: u16, uv: [f32; 2]| {
        let t = source_list.get(i as usize).and_then(Option::as_ref)?;
        Some(t.sample_smooth(bake::apply(tile_scale(tile_list[i as usize]), bake::apply(program.images[i as usize].affine, uv))))
    };
    let (mut sum, mut count) = (0.0f64, 0u32);
    for y in 0..SIDE * grid[1] {
        for x in 0..SIDE * grid[0] {
            let uv = [(x as f32 + 0.5) / SIDE as f32, (y as f32 + 0.5) / SIDE as f32];
            let covered = (0..program.images.len() as u16).any(|i| at(i, uv).is_some_and(|c| c[3] > COVERED));
            if !covered {
                continue;
            }
            // A missing source reads as nothing here: the magenta of a
            // missing image is a colour to show, not a distance to scatter.
            let [r, g, b, _] = bake::run(&program.ops, &|i| at(i, uv).unwrap_or([0.0, 0.0, 0.0, 1.0]));
            sum += f64::from(0.2126 * r + 0.7152 * g + 0.0722 * b);
            count += 1;
        }
    }
    (count > 0).then(|| (sum / f64::from(count)) as f32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blend::shade::bake::{BBakeImage, BBakeOp, IDENTITY};

    #[test]
    fn a_chain_whose_image_is_absent_leaves_the_scale_alone() {
        let program = BlendBake { images: vec![BBakeImage { key: "absent.png".into(), affine: IDENTITY, straight: false }], ops: vec![BBakeOp::Image(0)], alpha: None };
        let mut cache = TextureCache::new(&[], None);
        assert!(mean(&program, &mut cache).is_none());
    }

    /// Half the image covered and white, half a hole: the mean is the
    /// covered half's value, not half of it.
    #[test]
    fn the_mean_is_over_what_the_image_covers() {
        let mut png = Vec::new();
        let pixels: Vec<u8> = (0..4 * 4).flat_map(|i| if i % 4 < 2 { [255u8, 255, 255, 255] } else { [0, 0, 0, 0] }).collect();
        image::RgbaImage::from_raw(4, 4, pixels).expect("pixels").write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png).expect("png");
        let program = BlendBake { images: vec![BBakeImage { key: "half.png".into(), affine: IDENTITY, straight: false }], ops: vec![BBakeOp::Image(0)], alpha: None };
        let mut cache = TextureCache::new(&[("half.png".to_string(), std::sync::Arc::new(png))], None);
        let mean = mean(&program, &mut cache).expect("an image was found");
        assert!(mean > 0.9, "{mean}");
    }
}
