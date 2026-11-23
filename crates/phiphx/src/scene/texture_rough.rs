//! The Roughness chain baked to a map (`BlendFile::rough_bake`).
//!
//! An image plugged straight into Roughness is a map already
//! (`blend::shade::maps`). One that reaches it through other nodes was
//! folded to a single number with the image read as white: CW-POSES' skin
//! is `Mask → Invert → Color Ramp → Bright/Contrast → Roughness`, about
//! 0.53 over bare skin, and it folded to 0.15 — a wet shine on a matte
//! surface. The chain is the same kind of program the Base Color bake
//! runs, so it is run the same way, over the UV unit square, and the
//! result takes the roughness map's slot.
//!
//! The map holds the value itself in each byte, not an sRGB colour: the
//! kernel reads a roughness map raw (`sample_raw`).

use super::material::{SceneMap, SceneMaterial};
use super::texture::{SceneTexture, TextureCache};
use crate::blend::shade::bake::{self, BlendBake};
use std::sync::Arc;

/// How wide the baked map is at most.
const GRID: u32 = 512;

/// The maps baked so far this load: a bake belongs to the material, and
/// many objects wear one material.
#[derive(Default)]
pub(super) struct RoughBakeSet {
    done: Vec<(BlendBake, Option<SceneTexture>)>,
}

impl RoughBakeSet {
    /// Give `m` its baked roughness map, when it has a chain to bake and
    /// no map of its own. A base texture laid through its own affine
    /// (`uv_affine`) keeps the folded value: the maps share the base's
    /// UVs, and the bake is over the mesh's own.
    pub(super) fn attach(&mut self, m: &mut SceneMaterial, cache: &mut TextureCache) {
        let Some(program) = m.rough_bake.clone() else { return };
        // Tiled sources are baked over their whole grid, and that is the
        // one affine the base may carry: the UVs scaled onto the grid
        // (`texture::tile_scale`), which this map then shares.
        let grid = program.images.iter().fold([1u32, 1], |most, i| {
            let _ = cache.get(&i.key);
            let g = cache.grid(&i.key);
            [most[0].max(g[0]), most[1].max(g[1])]
        });
        let on_the_grid = grid != [1, 1] && m.uv_affine == Some(super::texture::tile_scale(grid));
        if m.maps.rough.is_some() || (m.uv_affine.is_some() && !on_the_grid) {
            return;
        }
        let image = match self.done.iter().find(|(seen, _)| *seen == program) {
            Some((_, image)) => image.clone(),
            None => {
                let image = run(&program, cache, grid);
                self.done.push((program, image.clone()));
                image
            }
        };
        if let Some(image) = image {
            m.maps.rough = Some(SceneMap { key: format!("{} (Roughness, baked)", m.name), image: Some(image) });
        }
    }
}

/// The program over the UV unit square, each image sampled through its own
/// affine. `None` when no source image resolved: the folded value stands.
fn run(program: &BlendBake, cache: &mut TextureCache, grid: [u32; 2]) -> Option<SceneTexture> {
    let source_list: Vec<Option<SceneTexture>> = program.images.iter().map(|i| cache.get(&i.key).map(|t| if i.straight { t } else { t.associated() })).collect();
    source_list.iter().any(Option::is_some).then_some(())?;
    let tile_list: Vec<[u32; 2]> = program.images.iter().map(|i| cache.grid(&i.key)).collect();
    // One UV square is `side` texels; `grid` of them are baked.
    let side = source_list.iter().zip(&tile_list).filter_map(|(t, g)| t.as_ref().map(|t| (t.width / g[0].max(1)).max(t.height / g[1].max(1)))).max().unwrap_or(GRID).clamp(64, GRID);
    let (w, h) = (side * grid[0].max(1), side * grid[1].max(1));
    let mut out = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let uv = [(x as f32 + 0.5) / side as f32, grid[1].max(1) as f32 - (y as f32 + 0.5) / side as f32];
            let [r, g, b, _] = bake::run(&program.ops, &|i| match source_list.get(i as usize).and_then(Option::as_ref) {
                Some(t) => t.sample_smooth(bake::apply(super::texture::tile_scale(tile_list[i as usize]), bake::apply(program.images[i as usize].affine, uv))),
                // A missing source reads as Blender's missing-image magenta.
                None => [1.0, 0.0, 1.0, 1.0],
            });
            out.extend([byte(0.2126 * r + 0.7152 * g + 0.0722 * b); 3]);
            out.push(255);
        }
    }
    Some(SceneTexture { width: w, height: h, rgba: Arc::new(out) })
}

/// A value in 0..1 as the byte the kernel reads back.
fn byte(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blend::shade::bake::{BBakeImage, BBakeOp, IDENTITY};

    /// The skin's chain with no image behind it: a constant program bakes
    /// nothing (no source resolved), so the folded value stands.
    #[test]
    fn a_chain_whose_image_is_absent_bakes_nothing() {
        let program = BlendBake { images: vec![BBakeImage { key: "absent.png".into(), affine: IDENTITY, straight: true }], ops: vec![BBakeOp::Image(0)], alpha: None };
        let mut cache = TextureCache::new(&[], None);
        assert!(run(&program, &mut cache, [1, 1]).is_none());
    }

    #[test]
    fn a_value_is_stored_as_itself_not_as_a_colour() {
        assert_eq!(byte(0.0), 0);
        assert_eq!(byte(0.528), 135, "0.528 × 255, where sRGB would say 190");
        assert_eq!(byte(1.5), 255);
    }
}
