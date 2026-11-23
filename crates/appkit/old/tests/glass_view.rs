//! Glass through a real view tree. Set `GLASS_PNG_DIR` to also save what the
//! tests drew. The stages themselves are tested in the `glass` crate.

use glass::*;
use rust_appkit::glass_view::themed;
use rust_appkit::render::RenderContext;
use rust_appkit::view::NSView;
use rust_appkit::{NSColor, NSRect};

const W: usize = 400;
const H: usize = 300;

fn run(pixels: &mut [u32], shape: &[GlassShape], spacing: f32, material: GlassMaterial, clip: Option<GlassBounds>) -> GlassStat {
    let mut canvas = BufferCanvas::new(pixels, W, H);
    GlassPipeline::new().render(&mut canvas, &GlassLayer { shape, spacing, material }, clip)
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
    let stats = run(&mut direct, &shapes, 40.0, themed(GlassStyle::Regular, Some(tint)), None);
    assert_eq!(stats.capture, 1);
    assert!(drawn == direct, "a container draws what one layer draws");
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
