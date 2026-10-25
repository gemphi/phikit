//! A card's picture by key, `asset:<scene>:<spelling>` — the scene's
//! address, so a picture cached for one scene is not shown for the next,
//! and the asset's spelling in the project database. A material is a ball
//! in its colour (in its image's, when it has one), an image is itself, a
//! light a disc of its colour; everything else shows its kind's icon.

use super::db::{BAssetKind, BProjectDb};
use phiecs::prelude::Word;

const PREFIX: &str = "asset:";
const EDGE: u32 = 96;

fn scene_key() -> usize {
    crate::v2::scene::source::loaded_shared().map_or(0, |m| std::sync::Arc::as_ptr(&m) as usize)
}

pub fn key(db: &BProjectDb, word: Word) -> String {
    match db.asset(word).map(|a| a.kind) {
        Some(BAssetKind::Material | BAssetKind::Image | BAssetKind::Light) => format!("{PREFIX}{}:{}", scene_key(), db.world.spelling(word)),
        _ => String::new(),
    }
}

/// The picture a key names, as (width, height, sRGB RGBA).
pub fn image(key: &str) -> Option<(u32, u32, Vec<u8>)> {
    let rest = key.strip_prefix(PREFIX)?;
    let (_, spelling) = rest.split_once(':')?;
    let (kind, name) = spelling.split_once(':')?;
    let kind = BAssetKind::by_slug(kind)?;
    let model = crate::v2::scene::source::loaded_shared()?;
    let materials = || model.objects.iter().flat_map(|o| o.materials.iter());
    match kind {
        BAssetKind::Image => {
            let t = materials().filter_map(|m| m.texture.as_ref().filter(|_| m.base_color_texture.as_deref().unwrap_or(&m.name) == name)).next()?;
            Some(scaled(t.width, t.height, &t.rgba))
        }
        BAssetKind::Material => {
            let m = materials().find(|m| m.name == name)?;
            let [r, g, b, _] = m.base_color;
            let tex = m.texture.clone();
            Some(ball(|u, v| match &tex {
                Some(t) => {
                    let c = t.sample_smooth([u, v]);
                    [c[0] * r, c[1] * g, c[2] * b]
                }
                None => [r, g, b],
            }))
        }
        BAssetKind::Light => {
            let l = model.lights.iter().find(|l| l.name == name)?;
            Some(ball(|_, _| l.color))
        }
        _ => None,
    }
}

/// A lit ball, its surface coloured by `colour(u, v)` (linear).
fn ball(colour: impl Fn(f32, f32) -> [f32; 3]) -> (u32, u32, Vec<u8>) {
    let mut rgba = Vec::with_capacity((EDGE * EDGE * 4) as usize);
    let light = glam::Vec3::new(-0.45, 0.55, 0.7).normalize();
    for y in 0..EDGE {
        for x in 0..EDGE {
            let (px, py) = ((x as f32 + 0.5) / EDGE as f32 * 2.0 - 1.0, 1.0 - (y as f32 + 0.5) / EDGE as f32 * 2.0);
            let r2 = (px * px + py * py) / 0.7;
            if r2 > 1.0 {
                rgba.extend([0, 0, 0, 0]);
                continue;
            }
            let n = glam::Vec3::new(px, py, (1.0 - r2).max(0.0).sqrt()).normalize();
            let (u, v) = (n.x.atan2(n.z) / std::f32::consts::TAU + 0.5, 1.0 - n.y.acos() / std::f32::consts::PI);
            let shade = 0.25 + 0.75 * n.dot(light).max(0.0) + 0.25 * n.dot(light).max(0.0).powi(24);
            let c = colour(u, v).map(|c| encode(c * shade));
            rgba.extend([c[0], c[1], c[2], 255]);
        }
    }
    (EDGE, EDGE, rgba)
}

/// An image fitted into the card, nearest-sampled.
fn scaled(w: u32, h: u32, src: &[u8]) -> (u32, u32, Vec<u8>) {
    let scale = (EDGE as f32 / w.max(h) as f32).min(1.0);
    let (ow, oh) = (((w as f32 * scale) as u32).max(1), ((h as f32 * scale) as u32).max(1));
    let mut rgba = Vec::with_capacity((ow * oh * 4) as usize);
    for y in 0..oh {
        for x in 0..ow {
            let (sx, sy) = ((x as f32 / scale) as u32, (y as f32 / scale) as u32);
            let i = ((sy.min(h - 1) * w + sx.min(w - 1)) * 4) as usize;
            rgba.extend_from_slice(&src[i..i + 4]);
        }
    }
    (ow, oh, rgba)
}

/// Linear to an sRGB byte.
fn encode(c: f32) -> u8 {
    let c = c.clamp(0.0, 1.0);
    let s = if c <= 0.003_130_8 { c * 12.92 } else { 1.055 * c.powf(1.0 / 2.4) - 0.055 };
    (s * 255.0 + 0.5) as u8
}
