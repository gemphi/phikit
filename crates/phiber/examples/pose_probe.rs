//! Diagnose a file's render path: what the file asks for, what the render
//! set resolves to, and what a render produces.
//!   cargo run -p phiber --example pose_probe -- FILE [OUT.png]
use phiber::phiphx::Phx;
use phiber::v2::app::seed::{seed_output_props, seed_render_props, seed_view_layer};
use phiber::v2::part::outliner::key::BHidden;
use phiber::v2::scene::{current_scene, render_plan, render_set, source::set_loaded_scene};
use phiber::v2::state::BState;
use phiber::v2::view::draw::canvas::view3d::rendered::{BRayLook, environment, still, view_words};

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("file");
    let out = args.next();
    let t = std::time::Instant::now();
    let model = Phx::open(std::path::Path::new(&path)).expect("open");
    println!("== OPEN in {:.2?}", t.elapsed());
    let r = &model.render;
    println!("== FILE SAYS");
    println!("  camera            {:?}", r.camera);
    println!("  resolution        {} x {} @ {}%", r.resolution_x, r.resolution_y, r.resolution_percent);
    println!("  engine            {:?}", r.engine);
    println!("  view_transform    {:?} exposure {} gamma {}", r.view_transform, r.exposure, r.gamma);
    println!("  film_transparent  {}", r.film_transparent);
    println!("  output path       {:?} format {:?}", r.output.path, r.output.format);
    
    for k in ["samples", "preview_samples", "diffuse_bounces", "use_denoising", "max_bounces", "use_adaptive_sampling"] {
        println!("    cycles.{k:<22} {:?}", r.cycles.get(k));
    }
    println!("== SCENE");
    println!("  objects {} cameras {} lights {}", model.objects.len(), model.cameras.len(), model.lights.len());
    let verts: usize = model.objects.iter().map(|o| o.mesh.vertices.len()).sum();
    println!("  verts {verts}");
    for c in &model.cameras {
        println!("  camera obj {:?}", c.name);
    }
    for l in model.lights.iter().take(8) {
        println!("  light {:?} kind {:?} energy {}", l.name, l.kind, l.energy);
    }
    println!("  world background {:?}", model.world.background.radiance());

    let mut s = BState::new();
    seed_view_layer(&mut s, &model);
    seed_render_props(&mut s, &model);
    seed_output_props(&mut s, &model);
    set_loaded_scene(Some(model.clone()));
    let set = render_set(&s);
    println!("== RENDER SET RESOLVES TO");
    println!("  samples {} preview {} bounces {} denoise {}", set.samples, set.preview_samples, set.diffuse_bounces, set.denoise);
    println!("  resolution {:?}", set.resolution);
    println!("  film_transparent {} engine {:?}", set.film_transparent, set.engine);

    let hidden = BHidden::now(&s);
    let objects: Vec<_> = current_scene(&s, Some(&model))
        .into_iter()
        .filter(|o| !matches!(o, phiber::v2::scene::BObject3D::Loaded { name, .. } if hidden.hides(&s, name)))
        .collect();
    println!("  objects reaching the render {}", objects.len());

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
        studio: false,
    };
    let plain = gpui::hsla(0.0, 0.0, 0.8, 1.0);
    let Some(mut p) = render_plan(&model, set, &look, plain) else {
        println!("!! render_plan returned None — the scene names no camera");
        return;
    };
    println!("== PLAN {} x {}", p.w, p.h);
    if let Ok(n) = std::env::var("PHIBER_SCALE").map(|v| v.parse::<u32>().unwrap_or(1)) {
        p.w = (p.w / n.max(1)).max(1);
        p.h = (p.h / n.max(1)).max(1);
        println!("   scaled to {} x {}", p.w, p.h);
    }
    if let Ok(n) = std::env::var("PHIBER_SAMPLES").map(|v| v.parse::<u32>().unwrap_or(0)) {
        if n > 0 { p.samples = n; println!("   samples overridden to {n}"); }
    }
    println!("== RENDERING {} x {} at {} samples, denoise {}", p.w, p.h, p.samples, p.denoise);
    let t = std::time::Instant::now();
    let shot = still(&objects, p.look, p.camera, p.w, p.h, p.samples, p.denoise, p.adapt, p.time_limit, |_| {});
    println!("== RENDER in {:.2?}", t.elapsed());
    match shot {
        Some((px, taken)) => {
            println!("  took {taken} samples, bytes {}", px.len());
            let opaque = px.chunks_exact(4).filter(|p| p[3] > 0).count();
            let nonblack = px.chunks_exact(4).filter(|p| p[0] > 4 || p[1] > 4 || p[2] > 4).count();
            println!("  pixels with alpha>0 {opaque}  non-black {nonblack}  of {}", px.len() / 4);
            if let Some(out) = out {
                image::RgbaImage::from_raw(p.w, p.h, px).expect("buffer").save(&out).expect("save");
                println!("  wrote {out}");
            }
        }
        None => println!("!! still() returned None — NOTHING RENDERED"),
    }
}
