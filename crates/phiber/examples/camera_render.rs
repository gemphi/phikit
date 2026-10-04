//! Render a `.blend` the way F12 does — through the scene's own camera, at
//! the file's resolution, with the Render panel's settings.
//!
//!   cargo run --release -p phiber --example camera_render -- FILE OUT.png [samples]
use phiber::phiphx::Phx;
use phiber::v2::app::seed::{seed_render_props, seed_view_layer};
use phiber::v2::part::outliner::key::BHidden;
use phiber::v2::scene::{current_scene, render_plan, render_set, source::set_loaded_scene};
use phiber::v2::state::BState;
use phiber::v2::view::draw::canvas::view3d::rendered::{BRayLook, environment, still, view_words};

fn main() {
    let mut args = std::env::args().skip(1);
    let (path, out) = (args.next().expect("file"), args.next().expect("out.png"));
    let samples: Option<u32> = args.next().and_then(|s| s.parse().ok());
    let model = Phx::open(std::path::Path::new(&path)).expect("open");
    let mut s = BState::new();
    seed_view_layer(&mut s, &model);
    seed_render_props(&mut s, &model);
    set_loaded_scene(Some(model.clone()));
    let hidden = BHidden::now(&s);
    let objects: Vec<_> = current_scene(&s, Some(&model))
        .into_iter()
        .filter(|o| !matches!(o, phiber::v2::scene::BObject3D::Loaded { name, .. } if hidden.hides(&s, name)))
        .collect();
    let mut set = render_set(&s);
    if let Some(n) = samples {
        set.samples = n;
    }
    // The same world the viewport stands in: the file's lamps, its Background
    // node and the environment image behind it.
    let look = BRayLook {
        sky: gpui::hsla(0.0, 0.0, 0.14, 1.0),
        grid: gpui::hsla(0.0, 0.0, 0.3, 1.0),
        world: Some(model.world.background.radiance()),
        lights: phiber::v2::scene::scene_lights(),
        wire: None,
        view: view_words(&s),
        grid_hide: 0,
        environment: environment().map(|(_, rot)| rot),
        set,
        // A camera render stands in the file's OWN world — its lamps, its
        // Background node, its environment image. Material Preview's studio
        // HDRI would replace all three, so it stays off here.
        studio: false,
    };
    let plain = gpui::hsla(0.0, 0.0, 0.8, 1.0);
    let mut p = render_plan(&model, set, &look, plain).expect("the scene names a camera");
    // `PHIBER_SCALE=<n>` divides the resolution, for quick checks against a
    // Blender render at the same fraction.
    if let Ok(n) = std::env::var("PHIBER_SCALE").map(|v| v.parse::<u32>().unwrap_or(1)) {
        p.w = (p.w / n.max(1)).max(1);
        p.h = (p.h / n.max(1)).max(1);
    }
    // `PHIBER_RES=WxH` renders a different frame shape, to check the sensor
    // fit against Blender at the same shape.
    if let Ok(res) = std::env::var("PHIBER_RES") {
        if let Some((w, h)) = res.split_once('x') {
            p.w = w.parse().unwrap_or(p.w);
            p.h = h.parse().unwrap_or(p.h);
            p.camera = phiber::v2::scene::camera_of(
                phiber::v2::scene::active_camera(&model).expect("camera"),
                p.w,
                p.h,
            );
            phiber::v2::view::draw::canvas::view3d::rendered::lit(&mut p.camera, &look);
        }
    }
    println!(
        "camera {:?} {}x{} {} samples, denoise {}",
        model.render.camera, p.w, p.h, p.samples, p.denoise
    );
    let t = std::time::Instant::now();
    // The engine's look: Workbench renders Solid with `Scene.display.shading`.
    let (px, taken) = still(&objects, p.look, p.camera, p.w, p.h, p.samples, p.denoise)
        .expect("gpu adapter and a scene that fits");
    image::RgbaImage::from_raw(p.w, p.h, px).expect("image").save(&out).expect("save");
    println!("rendered {taken} samples in {:.0} ms -> {out}", t.elapsed().as_secs_f64() * 1000.0);
}
