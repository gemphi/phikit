//! Compare two renders of the same view — Blender's and Phiber's — for
//! agreement and crispness.
//!
//!   cargo run --release -p phiber --example render_compare -- blender.png phiber.png [side_by_side.png]
//!
//! Prints, over the pixels both images agree are *figure* (not background):
//!   - mean absolute difference (0..255) — how alike the shading is,
//!   - mean brightness of each,
//!   - edge softness: how many pixels along each silhouette hold a value
//!     between background and figure (anti-aliased / blurred transitions),
//!     per boundary pixel. 0 is a hard edge; Blender with AA off is the
//!     reference. Contrast is factored out, so a darker render does not
//!     read as softer.
use image::GenericImageView;

fn luma(p: image::Rgba<u8>) -> f32 {
    0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32
}

fn main() {
    let mut a = std::env::args().skip(1);
    let (pa, pb) = (a.next().expect("blender.png"), a.next().expect("phiber.png"));
    let out = a.next();
    let (ia, ib) = (image::open(&pa).expect("open a"), image::open(&pb).expect("open b"));
    assert_eq!(ia.dimensions(), ib.dimensions(), "renders must be the same size");
    let (w, h) = ia.dimensions();
    // Background = the top-left pixel of each image (both views leave the corner empty).
    let (bga, bgb) = (luma(ia.get_pixel(0, 0)), luma(ib.get_pixel(0, 0)));
    let figure = |img: &image::DynamicImage, bg: f32, x: u32, y: u32| (luma(img.get_pixel(x, y)) - bg).abs() > 12.0;
    let (mut n, mut diff, mut sa, mut sb) = (0usize, 0f32, 0f32, 0f32);
    for y in 0..h {
        for x in 0..w {
            if figure(&ia, bga, x, y) && figure(&ib, bgb, x, y) {
                let (la, lb) = (luma(ia.get_pixel(x, y)), luma(ib.get_pixel(x, y)));
                n += 1; diff += (la - lb).abs(); sa += la; sb += lb;
            }
        }
    }
    let n1 = n.max(1) as f32;
    // A transition pixel sits between the background and the figure's own
    // tone: more than the figure threshold above background, but under 60% of
    // the way to the figure's mean. A hard edge has none.
    let soft = |img: &image::DynamicImage, bg: f32, mean: f32| -> f32 {
        let (mut trans, mut k) = (0usize, 0usize);
        let hi = bg + 0.6 * (mean - bg);
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                let f = figure(img, bg, x, y);
                if f != figure(img, bg, x + 1, y) || f != figure(img, bg, x, y + 1) {
                    k += 1;
                    for (dx, dy) in [(0, 0), (1, 0), (0, 1)] {
                        let l = luma(img.get_pixel(x + dx, y + dy));
                        if (l - bg).abs() > 12.0 && (l - bg).abs() < (hi - bg).abs() { trans += 1; }
                    }
                }
            }
        }
        if k == 0 { 0.0 } else { trans as f32 / k as f32 }
    };
    println!("figure pixels in both: {n}");
    println!("mean |Blender − Phiber| on the figure: {:.1} / 255", diff / n1);
    println!("mean brightness  Blender {:.1}   Phiber {:.1}", sa / n1, sb / n1);
    println!("edge softness (transition px per boundary px, 0 = hard)  Blender {:.2}   Phiber {:.2}", soft(&ia, bga, sa / n1), soft(&ib, bgb, sb / n1));
    if let Some(out) = out {
        let mut side = image::RgbaImage::from_pixel(w * 2 + 16, h, image::Rgba([24, 24, 24, 255]));
        image::imageops::overlay(&mut side, &ia.to_rgba8(), 0, 0);
        image::imageops::overlay(&mut side, &ib.to_rgba8(), (w + 16) as i64, 0);
        side.save(&out).expect("save");
        println!("wrote {out}");
    }
}
