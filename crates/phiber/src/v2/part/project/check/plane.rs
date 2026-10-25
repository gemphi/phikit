//! The sketch check's plane: a drawing ruled in centimetres with its
//! silhouette and the model's laid over it as two clean outlines, and how
//! much the two share.

use super::{BCheck, BStand, EDGE, PREFIX, checks};
use phiplay::build::footwear::pair;
use phiplay::build::footwear::picture::{left, picture, span};
use phiplay::kit::footwear::BFootwearSpec;

/// The plane's margin round the drawing, where the axes' ticks stand.
pub(super) const MARGIN: u32 = 48;
pub(super) const PLANE: [u8; 3] = [12, 20, 50];
pub(super) const MINOR: [u8; 3] = [26, 38, 78];
pub(super) const MAJOR: [u8; 3] = [44, 60, 112];
pub(super) const AXIS: [u8; 3] = [120, 136, 190];
pub(super) const SKETCH: [u8; 3] = [255, 196, 92];
pub(super) const MODEL: [u8; 3] = [60, 230, 255];

/// The drawing's silhouette: everything its lines enclose. The lines are
/// whatever stands off the sheet's navy; thickened a little to close their
/// gaps, the navy reached from the border is outside, the rest inside.
fn enclosed(sketch: &image::RgbaImage) -> Vec<bool> {
    let (w, h) = sketch.dimensions();
    let paper = sketch.get_pixel(2, 2).0;
    let ink: Vec<bool> = sketch.pixels().map(|p| (0..3).map(|c| (p.0[c] as i32 - paper[c] as i32).abs()).sum::<i32>() > 60).collect();
    let at = |x: i32, y: i32| x >= 0 && y >= 0 && x < w as i32 && y < h as i32;
    let grow = 3;
    let thick: Vec<bool> = (0..h as i32)
        .flat_map(|y| (0..w as i32).map(move |x| (x, y)))
        .map(|(x, y)| (-grow..=grow).any(|dy| (-grow..=grow).any(|dx| at(x + dx, y + dy) && ink[((y + dy) as u32 * w + (x + dx) as u32) as usize])))
        .collect();
    let mut outside = vec![false; (w * h) as usize];
    let mut todo: Vec<(i32, i32)> = (0..w as i32).flat_map(|x| [(x, 0), (x, h as i32 - 1)]).chain((0..h as i32).flat_map(|y| [(0, y), (w as i32 - 1, y)])).collect();
    while let Some((x, y)) = todo.pop() {
        let i = (y as u32 * w + x as u32) as usize;
        if outside[i] || thick[i] {
            continue;
        }
        outside[i] = true;
        todo.extend([(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)].into_iter().filter(|&(x, y)| at(x, y)));
    }
    // Undo the thickening: inside is what lies more than `grow` from outside.
    (0..h as i32)
        .flat_map(|y| (0..w as i32).map(move |x| (x, y)))
        .map(|(x, y)| !(-grow..=grow).any(|dy| (-grow..=grow).any(|dx| !at(x + dx, y + dy) || outside[((y + dy) as u32 * w + (x + dx) as u32) as usize])))
        .collect()
}

/// The drawing's mask laid on the plane, its margin empty.
fn on_plane(mask: &[bool], w: u32, h: u32) -> Vec<bool> {
    let (cw, ch) = (w + 2 * MARGIN, h + 2 * MARGIN);
    (0..ch)
        .flat_map(|y| (0..cw).map(move |x| (x, y)))
        .map(|(x, y)| (MARGIN..w + MARGIN).contains(&x) && (MARGIN..h + MARGIN).contains(&y) && mask[((y - MARGIN) * w + x - MARGIN) as usize])
        .collect()
}

/// Whether pixel (`x`, `y`) of a `w`×`h` mask is inside it with a neighbour out.
fn rim(mask: &[bool], w: u32, h: u32, x: u32, y: u32) -> bool {
    mask[(y * w + x) as usize] && [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)].iter().any(|(dx, dy)| {
        let (nx, ny) = (x as i32 + dx, y as i32 + dy);
        nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 || !mask[(ny as u32 * w + nx as u32) as usize]
    })
}

/// The model's silhouette over the plane's pixels — the drawing and the
/// margin round it, so whatever of the model reaches past the drawing shows
/// — its box scaled to the drawing's height and stood on its floor
/// (`BStand`). With it, the drawing's pixels per centimetre and where the
/// plane's origin falls in the drawing.
fn laid(d: &BFootwearSpec, check: &BCheck, w: u32, h: u32) -> Option<(Vec<bool>, f32, (f32, f32))> {
    let parts = left(&pair(d));
    let model = picture(&parts, EDGE, check.shot);
    let covered = |x: u32, y: u32| {
        let x = if check.flip { EDGE - 1 - x } else { x };
        model.rgba[((y * EDGE + x) * 4 + 3) as usize] > 0
    };
    let (mut lo, mut hi) = ((u32::MAX, u32::MAX), (0, 0));
    for y in 0..EDGE {
        for x in 0..EDGE {
            if covered(x, y) {
                lo = (lo.0.min(x), lo.1.min(y));
                hi = (hi.0.max(x), hi.1.max(y));
            }
        }
    }
    if hi.1 <= lo.1 {
        return None;
    }
    let [bx0, by0, bx1, by1] = check.bounds;
    let scale = (by1 - by0) as f32 / (hi.1 - lo.1) as f32;
    let width = (hi.0 - lo.0) as f32 * scale;
    let left_edge = match check.stand {
        BStand::Left => bx0 as f32,
        BStand::Right => bx1 as f32 - width,
        BStand::Middle => (bx0 + bx1) as f32 / 2.0 - width / 2.0,
    };
    let m = MARGIN as f32;
    let mask = (0..h + 2 * MARGIN)
        .flat_map(|y| (0..w + 2 * MARGIN).map(move |x| (x, y)))
        .map(|(x, y)| {
            let (x, y) = (x as f32 - m, y as f32 - m);
            let (mx, my) = ((x + 0.5 - left_edge) / scale + lo.0 as f32, (y + 0.5 - by0 as f32) / scale + lo.1 as f32);
            mx >= 0.0 && my >= 0.0 && (mx as u32) < EDGE && (my as u32) < EDGE && covered(mx as u32, my as u32)
        })
        .collect();
    let per_cm = scale * EDGE as f32 / (span(&parts) * 100.0);
    let origin_x = if check.stand == BStand::Right { left_edge + width } else { left_edge };
    Some((mask, per_cm, (origin_x, by1 as f32)))
}

/// The overlay a key names, as (width, height, sRGB RGBA).
pub fn image(key: &str) -> Option<(u32, u32, Vec<u8>)> {
    let (slug, i) = key.strip_prefix(PREFIX)?.rsplit_once(':')?;
    let d = phiplay::kit::footwear::by_slug(slug)?;
    let check = checks(d).get(i.parse::<usize>().ok()?)?;
    let sketch = image::load_from_memory(check.png).ok()?.to_rgba8();
    let (w, h) = sketch.dimensions();
    let drawn = on_plane(&enclosed(&sketch), w, h);
    let (model, per_cm, (ox, oy)) = laid(d, check, w, h)?;
    let (cw, ch) = (w + 2 * MARGIN, h + 2 * MARGIN);
    let mut out = image::RgbaImage::from_pixel(cw, ch, image::Rgba([PLANE[0], PLANE[1], PLANE[2], 255]));
    // `c` laid at (`x`, `y`) with weight `a` over what is there.
    let mut lay = |x: i32, y: i32, c: [u8; 3], a: f32| {
        if x >= 0 && y >= 0 && (x as u32) < cw && (y as u32) < ch {
            let p = out.get_pixel_mut(x as u32, y as u32);
            for k in 0..3 {
                p.0[k] = (p.0[k] as f32 * (1.0 - a) + c[k] as f32 * a) as u8;
            }
        }
    };
    // The plane, ruled every centimetre and stronger every five, its axes
    // through the origin, ticked into the margin.
    let (ox, oy) = (ox + MARGIN as f32, oy + MARGIN as f32);
    let step = per_cm.max(2.0);
    for (along_x, from, to) in [(true, ox, cw as f32), (true, ox, 0.0), (false, oy, ch as f32), (false, oy, 0.0)] {
        let mut k = 0;
        loop {
            let at = from + (to - from).signum() * step * k as f32;
            if (to - from).signum() * (to - at) < 0.0 {
                break;
            }
            let colour = if k % 5 == 0 { MAJOR } else { MINOR };
            let tick = if k % 5 == 0 { 8 } else { 4 };
            for t in 0..(if along_x { ch } else { cw }) as i32 {
                if along_x {
                    lay(at as i32, t, colour, 1.0);
                } else {
                    lay(t, at as i32, colour, 1.0);
                }
            }
            for t in 0..tick {
                if along_x {
                    lay(at as i32, oy as i32 + t, AXIS, 1.0);
                } else {
                    lay(ox as i32 - t, at as i32, AXIS, 1.0);
                }
            }
            k += 1;
        }
    }
    for t in 0..cw.max(ch) as i32 {
        lay(t, oy as i32, AXIS, 1.0);
        lay(ox as i32, t, AXIS, 1.0);
    }
    // The drawing, faint, and the two outlines over it.
    let paper = sketch.get_pixel(2, 2).0;
    for y in 0..h {
        for x in 0..w {
            let s = sketch.get_pixel(x, y).0;
            if (0..3).map(|c| (s[c] as i32 - paper[c] as i32).abs()).sum::<i32>() > 60 {
                lay((x + MARGIN) as i32, (y + MARGIN) as i32, [s[0], s[1], s[2]], 0.4);
            }
        }
    }
    for y in 0..ch {
        for x in 0..cw {
            // The model's outline over the drawing's where both run.
            let colour = [(&model, MODEL), (&drawn, SKETCH)].into_iter().find(|(mask, _)| rim(mask, cw, ch, x, y)).map(|(_, c)| c);
            if let Some(c) = colour {
                lay(x as i32, y as i32, c, 1.0);
            }
        }
    }
    Some((cw, ch, out.into_raw()))
}

/// How much the model's silhouette and the drawing's share: what both cover
/// over what either does.
pub fn overlap(key: &str) -> Option<f32> {
    let (slug, i) = key.strip_prefix(PREFIX)?.rsplit_once(':')?;
    let d = phiplay::kit::footwear::by_slug(slug)?;
    let check = checks(d).get(i.parse::<usize>().ok()?)?;
    let sketch = image::load_from_memory(check.png).ok()?.to_rgba8();
    let (w, h) = sketch.dimensions();
    let drawn = on_plane(&enclosed(&sketch), w, h);
    let (model, _, _) = laid(d, check, w, h)?;
    let both = drawn.iter().zip(&model).filter(|(a, b)| **a && **b).count();
    let either = drawn.iter().zip(&model).filter(|(a, b)| **a || **b).count();
    Some(both as f32 / either.max(1) as f32)
}
