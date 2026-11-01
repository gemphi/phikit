//! The glass pipeline against real pixels, and through a real view tree. Set
//! `GLASS_PNG_DIR` to also save what the view tests drew.

use rust_appkit::glass::*;
use rust_appkit::render::RenderContext;
use rust_appkit::view::NSView;
use rust_appkit::{NSColor, NSRect};

const W: usize = 400;
const H: usize = 300;

fn red(argb: u32) -> f32 {
    ((argb >> 16) & 0xFF) as f32 / 255.0
}

fn green(argb: u32) -> f32 {
    ((argb >> 8) & 0xFF) as f32 / 255.0
}

fn blue(argb: u32) -> f32 {
    (argb & 0xFF) as f32 / 255.0
}

/// One-pixel black and white squares: anything sharp shows at once.
fn checker() -> Vec<u32> {
    (0..W * H).map(|i| if (i % W + i / W) % 2 == 0 { 0xFF000000 } else { 0xFFFFFFFF }).collect()
}

fn plain(style: GlassStyle) -> GlassMaterial {
    GlassMaterial { tint_amount: 0.0, saturation: 1.0, edge_strength: 0.0, shadow_strength: 0.0, lens: 0.0, ..GlassMaterial::of(style) }
}

fn run(pixels: &mut [u32], shapes: &[GlassShape], spacing: f32, material: GlassMaterial, clip: Option<GlassBounds>) -> GlassStats {
    let mut canvas = BufferCanvas::new(pixels, W, H);
    GlassPipeline::new().render(&mut canvas, &GlassLayer { shapes, spacing, material }, clip)
}

fn panel() -> GlassShape {
    GlassShape::new(100.0, 75.0, 200.0, 150.0, 20.0)
}

#[test]
fn distance_is_negative_inside_and_zero_on_the_edge() {
    let s = panel();
    assert!(s.distance(200.0, 150.0) < -70.0);
    assert!(s.distance(100.0, 150.0).abs() < 1e-3);
    assert!(s.distance(90.0, 150.0) > 9.9);
    assert!(s.distance(101.0, 76.0) > 0.0, "the box's corner is outside the rounded corner");
}

#[test]
fn a_flat_backdrop_stays_exactly_itself() {
    for c in [0xFF000000u32, 0xFFFFFFFF, 0xFF123456, 0xFFABCDEF] {
        let mut pixels = vec![c; W * H];
        run(&mut pixels, &[panel()], 0.0, plain(GlassStyle::Regular), None);
        assert!(pixels.iter().all(|p| *p == c), "{c:#x} changed");
    }
}

#[test]
fn glass_replaces_the_sharp_image_so_nothing_ghosts_through() {
    let mut pixels = checker();
    run(&mut pixels, &[panel()], 0.0, plain(GlassStyle::Regular), None);
    for (x, y) in [(200, 150), (201, 150), (180, 140), (181, 140)] {
        let v = red(pixels[y * W + x]);
        assert!((v - 0.5).abs() < 0.02, "({x},{y}) is {v} and not grey");
    }
}

#[test]
fn blur_keeps_the_light_it_spreads() {
    let mut pixels = vec![0xFF000000u32; W * H];
    for y in 140..160 {
        for x in 190..210 {
            pixels[y * W + x] = 0xFFFFFFFF;
        }
    }
    run(&mut pixels, &[GlassShape::new(60.0, 40.0, 280.0, 220.0, 0.0)], 0.0, plain(GlassStyle::Regular), None);
    let total: f32 = pixels.iter().map(|p| red(*p)).sum();
    assert!((total - 400.0).abs() < 8.0, "400 pixels of light, now {total}");
    assert!(red(pixels[150 * W + 200]) < 0.5, "the peak spreads");
    assert!(red(pixels[150 * W + 225]) > 0.01, "to where there was none");
}

#[test]
fn nothing_changes_outside_the_shape_without_a_shadow() {
    let mut pixels = checker();
    let before = pixels.clone();
    run(&mut pixels, &[panel()], 0.0, plain(GlassStyle::Regular), None);
    for (x, y) in [(50, 150), (98, 150), (102, 77), (200, 40), (350, 280)] {
        assert_eq!(pixels[y * W + x], before[y * W + x], "({x},{y})");
    }
}

#[test]
fn clip_limits_what_is_written() {
    let mut pixels = checker();
    let before = pixels.clone();
    let clip = GlassBounds { x0: 150, y0: 100, x1: 250, y1: 200 };
    run(&mut pixels, &[panel()], 0.0, plain(GlassStyle::Regular), Some(clip));
    assert_eq!(pixels[110 * W + 120], before[110 * W + 120], "glass, but outside the clip");
    assert!((red(pixels[150 * W + 200]) - 0.5).abs() < 0.02, "inside the clip");
}

#[test]
fn a_shape_past_the_canvas_edge_is_cut_and_one_off_it_draws_nothing() {
    let mut pixels = vec![0xFF404040u32; W * H];
    let stats = run(&mut pixels, &[GlassShape::new(-50.0, -50.0, 100.0, 100.0, 10.0)], 0.0, GlassMaterial::of(GlassStyle::Regular), None);
    assert!(stats.covered > 0);
    let none = run(&mut pixels, &[GlassShape::new(5000.0, 0.0, 10.0, 10.0, 0.0)], 0.0, plain(GlassStyle::Clear), None);
    assert_eq!(none, GlassStats::default());
}

#[test]
fn clear_glass_keeps_more_of_the_backdrop_than_regular() {
    let at = |style| {
        let mut px: Vec<u32> = (0..W * H).map(|i| if i % W < 200 { 0xFF000000 } else { 0xFFFFFFFF }).collect();
        run(&mut px, &[panel()], 0.0, plain(style), None);
        red(px[150 * W + 210])
    };
    let (regular, clear) = (at(GlassStyle::Regular), at(GlassStyle::Clear));
    assert!(clear > regular + 0.1, "10 px past a hard edge: clear {clear}, regular {regular}");
}

#[test]
fn the_lens_moves_what_the_rim_shows() {
    let rim = |lens: f32| {
        let mut px: Vec<u32> = (0..W * H).map(|i| if i % W < 106 { 0xFFFFFFFF } else { 0xFF000000 }).collect();
        let m = GlassMaterial { blur_radius: 2.0, lens, ..plain(GlassStyle::Clear) };
        run(&mut px, &[panel()], 0.0, m, None);
        red(px[150 * W + 102])
    };
    assert!(rim(0.0) > 0.8, "no lens: the white beneath, {}", rim(0.0));
    assert!(rim(30.0) < 0.2, "lens: the black further in, {}", rim(30.0));
}

#[test]
fn tint_and_saturation_change_colour_as_set() {
    let run_on = |m: GlassMaterial| {
        let mut px = vec![0xFF804040u32; W * H];
        run(&mut px, &[panel()], 0.0, m, None);
        px[150 * W + 200]
    };
    let base = plain(GlassStyle::Regular);
    let tinted = run_on(GlassMaterial { tint: [0.0, 0.0, 1.0], tint_amount: 0.5, ..base });
    assert!(blue(tinted) > 0.55, "half way to blue: {tinted:#x}");
    let flat = run_on(base);
    let vivid = run_on(GlassMaterial { saturation: 1.8, ..base });
    assert!(red(vivid) - green(vivid) > red(flat) - green(flat) + 0.05, "{vivid:#x} against {flat:#x}");
}

#[test]
fn the_shadow_falls_below_and_fades_with_distance() {
    let mut pixels = vec![0xFFC0C0C0u32; W * H];
    let m = GlassMaterial { shadow_strength: 0.42, ..plain(GlassStyle::Regular) };
    run(&mut pixels, &[panel()], 0.0, m, None);
    let grey = |x: usize, y: usize| red(pixels[y * W + x]);
    assert!(grey(200, 232) < grey(200, 68), "more below than above: {} and {}", grey(200, 232), grey(200, 68));
    assert!(grey(200, 295) > grey(200, 232) + 0.1, "and less further away: {}", grey(200, 295));
}

#[test]
fn shapes_merge_only_within_their_spacing() {
    let (a, b) = (GlassShape::new(100.0, 100.0, 60.0, 60.0, 0.0), GlassShape::new(180.0, 100.0, 60.0, 60.0, 0.0));
    let gap = |spacing: f32| {
        let mut px = checker();
        let before = px.clone();
        let stats = run(&mut px, &[a, b], spacing, plain(GlassStyle::Regular), None);
        assert_eq!(stats.captures, 1, "one layer, one capture");
        px[130 * W + 170] != before[130 * W + 170]
    };
    assert!(!gap(0.0), "apart at zero spacing");
    assert!(gap(60.0), "a bridge at 60");
}

/// A busy backdrop: coloured bars, fine lines and diagonals.
fn sample_scene() -> Vec<u32> {
    (0..W * H)
        .map(|i| {
            let (x, y) = (i % W, i / W);
            let bar = [0xFFE0456Cu32, 0xFF35C4B8, 0xFFF0B341, 0xFF5A6CF0][(x / 60) % 4];
            match (y % 24 < 2, (x + y) % 40 < 6) {
                (true, _) => 0xFFFFFFFF,
                (_, true) => 0xFF10131A,
                _ => bar,
            }
        })
        .collect()
}

fn save(name: &str, pixels: &[u32]) {
    let Ok(dir) = std::env::var("GLASS_PNG_DIR") else { return };
    let rgba: Vec<u8> = pixels.iter().flat_map(|p| [(p >> 16) as u8, (p >> 8) as u8, *p as u8, 255]).collect();
    image::save_buffer(format!("{dir}/{name}"), &rgba, W as u32, H as u32, image::ColorType::Rgba8).expect("png");
}

fn draw(view: &NSView, pixels: &mut [u32]) {
    let mut ctx = RenderContext::new(W, H, pixels);
    view.draw(&mut ctx);
}

#[test]
fn a_view_tree_draws_glass_under_sharp_subviews() {
    let tree = |style: GlassStyle| {
        let mut root = NSView::new(NSRect::new(0.0, 0.0, W as f32, H as f32));
        let mut panel = NSView::new(NSRect::new(100.0, 75.0, 200.0, 150.0));
        panel.set_background_color(Some(NSColor::new(0.1, 0.1, 0.15, 1.0)));
        panel.set_corner_radius(20.0);
        panel.set_glass(true);
        panel.set_glass_style(style);
        let mut label = NSView::new(NSRect::new(40.0, 50.0, 40.0, 20.0));
        label.set_background_color(Some(NSColor::white()));
        panel.add_subview(label);
        root.add_subview(panel);
        root
    };
    for (style, name) in [(GlassStyle::Regular, "glass_regular.png"), (GlassStyle::Clear, "glass_clear.png")] {
        let mut pixels = sample_scene();
        let before = pixels.clone();
        draw(&tree(style), &mut pixels);
        save(name, &pixels);
        assert_eq!(pixels[135 * W + 155], 0xFFFFFFFF, "the child paints over the glass, unblurred");
        assert_ne!(pixels[190 * W + 200], before[190 * W + 200], "glass covers the panel");
        assert_eq!(pixels[3 * W + 3], before[3 * W + 3], "and leaves far pixels alone");
    }
}

#[test]
fn a_container_is_one_layer_of_all_its_glass() {
    let tint = NSColor::new(0.2, 0.2, 0.3, 1.0);
    let mut container = NSView::new(NSRect::new(0.0, 0.0, W as f32, H as f32));
    let spots = [40.0f32, 120.0, 200.0];
    for x in spots {
        let mut v = NSView::new(NSRect::new(x, 100.0, 60.0, 60.0));
        v.set_corner_radius(30.0);
        v.set_background_color(Some(tint));
        v.set_glass(true);
        container.add_subview(v);
    }
    container.set_glass_container(Some(40.0));
    let mut drawn = sample_scene();
    draw(&container, &mut drawn);
    save("glass_container.png", &drawn);

    // Exactly one render of the three shapes, with no view painting again.
    let shapes = spots.map(|x| GlassShape::new(x, 100.0, 60.0, 60.0, 30.0));
    let mut direct = sample_scene();
    let stats = run(&mut direct, &shapes, 40.0, GlassMaterial::themed(GlassStyle::Regular, Some(tint)), None);
    assert_eq!(stats.captures, 1);
    assert!(drawn == direct, "a container draws what one layer draws");
}

/// Reports what a sidebar costs at 1080p: `--ignored --nocapture`. Run it
/// optimized, as a debug build says little about a frame.
#[test]
#[ignore]
fn report_the_cost_of_a_sidebar() {
    let (w, h) = (1920, 1080);
    let mut pixels: Vec<u32> = (0..w * h).map(|i| 0xFF000000 | (i as u32).wrapping_mul(2654435761) >> 8).collect();
    let shape = GlassShape::new(12.0, 60.0, 320.0, 1000.0, 12.0);
    let layer = GlassLayer { shapes: &[shape], spacing: 0.0, material: GlassMaterial::of(GlassStyle::Regular) };
    let mut pipeline = GlassPipeline::new();
    let start = std::time::Instant::now();
    let mut stats = GlassStats::default();
    for _ in 0..20 {
        let mut canvas = BufferCanvas::new(&mut pixels, w, h);
        stats = pipeline.render(&mut canvas, &layer, None);
    }
    println!("{:.2} ms a frame, {stats:?}", start.elapsed().as_secs_f64() * 1000.0 / 20.0);
}

#[test]
fn a_hidden_view_adds_no_glass() {
    let mut pixels = sample_scene();
    let before = pixels.clone();
    let mut root = NSView::new(NSRect::new(0.0, 0.0, W as f32, H as f32));
    let mut panel = NSView::new(NSRect::new(100.0, 75.0, 200.0, 150.0));
    panel.set_glass(true);
    panel.set_hidden(true);
    root.add_subview(panel);
    draw(&root, &mut pixels);
    assert!(pixels == before);
}
