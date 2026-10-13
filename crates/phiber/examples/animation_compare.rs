//! Compare two *sequences* — Blender's render of an animation and Phiber's —
//! frame by frame, and over time.
//!
//!   cargo run --release -p phiber --example animation_compare -- BLENDER_DIR PHIBER_DIR [contact.png]
//!
//! `render_compare` answers "does this frame look the same". An animation
//! asks two more questions that a still cannot:
//!
//!   - **Does it move the same?** The mean absolute change from one frame to
//!     the next, in each sequence. A rig that does not pose, or poses at the
//!     wrong frame, matches well on frame 1 and has near-zero motion after
//!     it — which a per-frame difference alone would report as a small
//!     constant error, not as the whole animation missing.
//!   - **Is it steady?** A path tracer's noise changes every frame even when
//!     the subject does not, so motion measured where Blender's is ~0 is
//!     this render's flicker.
//!
//! Both directories hold `fNNNN.png` named by frame, as
//! `examples/animation_render.rs` and `truth/blender_animation.py` write
//! them, and a `times.txt` of `frame posed_ms render_ms samples`.
use image::GenericImageView;

fn luma(p: image::Rgba<u8>) -> f32 {
    0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32
}

/// The frames a directory holds, by number.
fn frames(dir: &str) -> Vec<(i64, std::path::PathBuf)> {
    let mut out: Vec<(i64, std::path::PathBuf)> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{dir}: {e}"))
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "png"))
        .filter_map(|p| {
            let stem = p.file_stem()?.to_string_lossy().to_string();
            stem.strip_prefix('f')?.parse().ok().map(|n| (n, p))
        })
        .collect();
    out.sort_by_key(|(n, _)| *n);
    out
}

/// The per-frame mean of `times.txt`'s posed and render milliseconds.
fn timing(dir: &str) -> Option<(f64, f64)> {
    let text = std::fs::read_to_string(format!("{dir}/times.txt")).ok()?;
    let rows: Vec<Vec<f64>> = text.lines().map(|l| l.split_whitespace().filter_map(|v| v.parse().ok()).collect()).filter(|r: &Vec<f64>| r.len() >= 3).collect();
    let n = rows.len().max(1) as f64;
    Some((rows.iter().map(|r| r[1]).sum::<f64>() / n, rows.iter().map(|r| r[2]).sum::<f64>() / n))
}

fn main() {
    let mut a = std::env::args().skip(1);
    let (da, db) = (a.next().expect("blender dir"), a.next().expect("phiber dir"));
    let contact = a.next();
    let (fa, fb) = (frames(&da), frames(&db));
    let shared: Vec<i64> = fa.iter().map(|(n, _)| *n).filter(|n| fb.iter().any(|(m, _)| m == n)).collect();
    assert!(!shared.is_empty(), "no frame numbers in common");

    let load = |list: &[(i64, std::path::PathBuf)], n: i64| image::open(list.iter().find(|(m, _)| *m == n).map(|(_, p)| p).unwrap()).expect("open");
    let first = load(&fa, shared[0]);
    let (w, h) = first.dimensions();
    assert_eq!(load(&fb, shared[0]).dimensions(), (w, h), "the two sequences must be the same size");

    // Per frame: the figure both agree on, the mean difference over it, and
    // each render's own mean brightness.
    println!("frame   figure px   mean|B-P|   bright B   bright P   move B   move P");
    let (mut prev_a, mut prev_b): (Option<image::DynamicImage>, Option<image::DynamicImage>) = (None, None);
    let (mut tot_diff, mut tot_move_a, mut tot_move_b, mut counted, mut moves) = (0.0f64, 0.0f64, 0.0f64, 0usize, 0usize);
    for &n in &shared {
        let (ia, ib) = (load(&fa, n), load(&fb, n));
        let (bga, bgb) = (luma(ia.get_pixel(0, 0)), luma(ib.get_pixel(0, 0)));
        let figure = |img: &image::DynamicImage, bg: f32, x: u32, y: u32| (luma(img.get_pixel(x, y)) - bg).abs() > 12.0;
        let (mut k, mut diff, mut sa, mut sb) = (0usize, 0f32, 0f32, 0f32);
        for y in 0..h {
            for x in 0..w {
                if figure(&ia, bga, x, y) && figure(&ib, bgb, x, y) {
                    let (la, lb) = (luma(ia.get_pixel(x, y)), luma(ib.get_pixel(x, y)));
                    k += 1;
                    diff += (la - lb).abs();
                    sa += la;
                    sb += lb;
                }
            }
        }
        let k1 = k.max(1) as f32;
        // How far this frame moved from the last one, over the whole frame:
        // the measure that says whether the animation is animating.
        let moved = |now: &image::DynamicImage, was: &Option<image::DynamicImage>| -> f32 {
            let Some(was) = was else { return f32::NAN };
            let mut d = 0f32;
            for y in 0..h {
                for x in 0..w {
                    d += (luma(now.get_pixel(x, y)) - luma(was.get_pixel(x, y))).abs();
                }
            }
            d / (w * h) as f32
        };
        let (ma, mb) = (moved(&ia, &prev_a), moved(&ib, &prev_b));
        println!("{n:>5}   {k:>9}   {:>9.1}   {:>8.1}   {:>8.1}   {:>6.2}   {:>6.2}", diff / k1, sa / k1, sb / k1, ma, mb);
        tot_diff += (diff / k1) as f64;
        counted += 1;
        if ma.is_finite() {
            tot_move_a += ma as f64;
            tot_move_b += mb as f64;
            moves += 1;
        }
        prev_a = Some(ia);
        prev_b = Some(ib);
    }
    let (c1, m1) = (counted.max(1) as f64, moves.max(1) as f64);
    println!("\nover {counted} frames: mean |Blender − Phiber| {:.1} / 255", tot_diff / c1);
    println!("motion per frame: Blender {:.2}   Phiber {:.2}   (ratio {:.2})", tot_move_a / m1, tot_move_b / m1, (tot_move_b / m1) / (tot_move_a / m1).max(1e-6));
    for (name, dir) in [("Blender", &da), ("Phiber", &db)] {
        if let Some((pose, draw)) = timing(dir) {
            println!("{name:>8}: {pose:.0} ms posing, {draw:.0} ms rendering, per frame");
        }
    }
    // A contact sheet of the frames down two rows, Blender over Phiber.
    if let Some(out) = contact {
        let cols = shared.len() as u32;
        let (tw, th) = (w.min(320), h.min(320) * h.min(320) / h.min(320));
        let th = (h * tw / w).max(1).min(th.max(1));
        let mut sheet = image::RgbaImage::from_pixel(cols * tw, th * 2 + 8, image::Rgba([24, 24, 24, 255]));
        for (i, &n) in shared.iter().enumerate() {
            let small = |img: image::DynamicImage| image::imageops::resize(&img.to_rgba8(), tw, th, image::imageops::FilterType::Triangle);
            image::imageops::overlay(&mut sheet, &small(load(&fa, n)), i as i64 * tw as i64, 0);
            image::imageops::overlay(&mut sheet, &small(load(&fb, n)), i as i64 * tw as i64, th as i64 + 8);
        }
        sheet.save(&out).expect("save");
        println!("wrote {out}  (Blender on top, Phiber below)");
    }
}
