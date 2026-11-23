//! What glass costs on the CPU. `cargo test -p glass --release -- --ignored --nocapture`.

use crate::*;

/// Reports what a sidebar costs at 1080p: `--ignored --nocapture`. Run it
/// optimized, as a debug build says little about a frame.
#[test]
#[ignore]
fn report_the_cost_of_a_sidebar() {
    let (w, h) = (1920, 1080);
    let mut pixels: Vec<u32> = (0..w * h).map(|i| 0xFF000000 | (i as u32).wrapping_mul(2654435761) >> 8).collect();
    let shape = GlassShape::new(12.0, 60.0, 320.0, 1000.0, 12.0);
    let layer = GlassLayer { shape: &[shape], spacing: 0.0, material: GlassMaterial::of(GlassStyle::Regular) };
    let mut pipeline = GlassPipeline::new();
    let start = std::time::Instant::now();
    let mut stats = GlassStat::default();
    for _ in 0..20 {
        let mut canvas = BufferCanvas::new(&mut pixels, w, h);
        stats = pipeline.render(&mut canvas, &layer, None);
    }
    println!("{:.2} ms a frame, {stats:?}", start.elapsed().as_secs_f64() * 1000.0 / 20.0);
}
