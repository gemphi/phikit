//! Render a range of an animation the way Ctrl F12 does — the scene posed at
//! each frame, through the scene's own camera, at the file's resolution —
//! and write one PNG per frame beside a `times.txt` of per-frame timings.
//! The Blender side of the same comparison is
//! `phiphx/tests/truth/blender_animation.py`; `render_compare` reads a pair
//! of directories.
//!
//!   cargo run --release -p phiber --example animation_render -- FILE OUTDIR [start] [end] [step] [samples]
//!
//! `PHIBER_SCALE=<n>` divides the resolution, for a quick pass.
use phiber::phiphx::Phx;
use phiber::v2::app::seed::{seed_render_props, seed_view_layer};
use phiber::v2::part::outliner::key::BHidden;
use phiber::v2::scene::{BObject3D, current_scene, plan_at, render_set, source::set_loaded_scene};
use phiber::v2::state::{BState, CtxKey};
use phiber::v2::view::draw::canvas::view3d::rendered::{BMatStand, BRayLook, environment, still, view_words};

fn main() {
    let mut args = std::env::args().skip(1);
    let (path, dir) = (args.next().expect("file"), args.next().expect("outdir"));
    let mut num = |d: f32| -> f32 { args.next().and_then(|s| s.parse().ok()).unwrap_or(d) };
    let (start, end, step) = (num(1.0), num(0.0), num(1.0));
    drop(num);
    let samples: Option<u32> = args.next().and_then(|s| s.parse().ok());
    std::fs::create_dir_all(&dir).expect("outdir");

    let mut model = Phx::open(std::path::Path::new(&path)).expect("open");
    // Isolating where the light comes from, the same switches
    // `truth/blender_animation.py` takes (`rendering.md` §53):
    //   PHIBER_NO_WORLD=1   the world contributes nothing (lamps only)
    //   PHIBER_NO_LIGHTS=1  every lamp is dropped (the world only)
    //   PHIBER_VIEW=Standard|Filmic|AgX   overrides the file's transform
    //   PHIBER_OVERRIDE=<albedo>  View Layer > Override > Material: one grey
    //                   Lambertian surface in place of every material, so
    //                   the render measures the light and not the materials
    if let Ok(a) = std::env::var("PHIBER_OVERRIDE") {
        BMatStand::set(Some(BMatStand::grey(a.parse().unwrap_or(1.0))));
    }
    if std::env::var("PHIBER_NO_WORLD").is_ok() {
        model.world.background.strength = 0.0;
    }
    if std::env::var("PHIBER_NO_LIGHTS").is_ok() {
        model.lights.clear();
    }
    if let Ok(view) = std::env::var("PHIBER_VIEW") {
        model.render.view_transform = view;
    }
    let mut s = BState::new();
    seed_view_layer(&mut s, &model);
    seed_render_props(&mut s, &model);
    // Film ▸ Transparent and the output settings live in the other seed;
    // without it every render here painted a background Blender leaves out.
    phiber::v2::app::seed::seed_output_props(&mut s, &model);
    set_loaded_scene(Some(model.clone()));
    let hidden = BHidden::now(&s);
    let mut set = render_set(&s);
    if let Some(n) = samples {
        set.samples = n;
    }
    // The file's range when none is given, as Ctrl F12 takes it. `end`
    // equal to `start` is one frame, not "unset" — the Blender script takes
    // it that way too, and reading it as unset quietly rendered a hundred
    // frames when one was asked for.
    let end = if end >= start { end } else { model.timeline.frame_end as f32 };
    let scale: u32 = std::env::var("PHIBER_SCALE").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
    let plain = gpui::hsla(0.0, 0.0, 0.8, 1.0);
    let mut times = String::new();
    println!("{} — frames {start}..={end} step {step}, {} samples", model.render.camera, set.samples);

    let mut frame = start;
    while frame <= end + 1e-3 {
        let t = std::time::Instant::now();
        // The frame the scene stands at: `current_scene` poses the rig and
        // the objects from it, and `plan_at` puts the camera where it is.
        s.set(CtxKey::Frame, format!("{frame}"));
        let objects: Vec<BObject3D> = current_scene(&s, Some(&model))
            .into_iter()
            .filter(|o| !matches!(o, BObject3D::Loaded { name, .. } if hidden.hides(&s, name)))
            .collect();
        let posed = t.elapsed();
        // A camera render stands in the file's own world — its lamps at this
        // frame, its Background node, its environment image.
        let look = BRayLook {
            sky: gpui::hsla(0.0, 0.0, 0.14, 1.0),
            grid: gpui::hsla(0.0, 0.0, 0.3, 1.0),
            world: Some(model.world.background.radiance()),
            lights: phiber::v2::scene::motion::lights_at(&model, Some(frame)),
            wire: None,
            view: view_words(&s),
            grid_hide: 0,
            environment: environment().map(|(_, rot)| rot),
            set,
            studio: false,
        };
        let mut p = plan_at(&model, set, &look, plain, Some(frame)).expect("the scene names a camera");
        // `PHIRAY_DENOISE=0` turns the filter off, which is what a
        // *measurement* wants: the denoiser smooths frame-to-frame noise,
        // so the motion column of `animation_compare` reads low with it on
        // (`rendering.md` §59).
        if std::env::var("PHIRAY_DENOISE").map(|v| v == "0").unwrap_or(false) {
            p.denoise = false;
        }
        p.w = (p.w / scale.max(1)).max(1);
        p.h = (p.h / scale.max(1)).max(1);
        let shot = std::time::Instant::now();
        let (px, taken) = still(&objects, p.look, p.camera, p.w, p.h, p.samples, p.denoise).expect("gpu adapter and a scene that fits");
        let drew = shot.elapsed();
        let out = format!("{dir}/f{:04}.png", frame as i64);
        image::RgbaImage::from_raw(p.w, p.h, px).expect("image").save(&out).expect("save");
        let line = format!("{frame} {:.0} {:.0} {taken}\n", posed.as_secs_f64() * 1000.0, drew.as_secs_f64() * 1000.0);
        print!("{line}");
        times.push_str(&line);
        frame += step;
    }
    std::fs::write(format!("{dir}/times.txt"), times).expect("times");
    println!("-> {dir}");
}
