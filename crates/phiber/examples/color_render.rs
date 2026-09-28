//! Render a `.blend` the way the viewport draws Material Preview (or Solid /
//! Rendered) — full modifier stack through phiphx, textures, phiray — from the
//! fixed orthographic front view of `blender_color.py`, and time each stage.
//!
//!   cargo run --release -p phiber --example color_render -- FILE OUT.png [material|solid|rendered] [samples]
use phiber::phiphx::Phx;
use phiber::v2::app::seed::seed_view_layer;
use phiber::v2::part::outliner::key::BHidden;
use phiber::v2::scene::{BObject3D, BShading, current_scene, source::set_loaded_scene};
use phiber::v2::state::BState;
use phiber::v2::view::draw::canvas::view3d::rendered::{ENV_CDF_SLOT, ENV_SLOT, environment, light, still};
use phiray::{Camera, LIGHTS, Light};
use std::time::Instant;

const W: u32 = 720;
const H: u32 = 1080;

fn main() {
    let mut args = std::env::args().skip(1);
    let (path, out) = (args.next().expect("file"), args.next().expect("out.png"));
    let mode = args.next();
    let samples: u32 = args.next().and_then(|s| s.parse().ok()).unwrap_or(32);
    // `PHIRAY_DENOISE=0` turns the filter off; the Render panel's own default
    // (`BRenderSet`) is on, like Blender's.
    let denoise = std::env::var("PHIRAY_DENOISE").map(|v| v != "0").unwrap_or(true);
    let shading = match mode.as_deref() {
        Some("solid") => BShading::Solid,
        Some("rendered") => BShading::Rendered,
        _ => BShading::Material,
    };
    let all = Instant::now();
    let t = Instant::now();
    let model = Phx::open(std::path::Path::new(&path)).expect("open");
    let read = t.elapsed();

    let t = Instant::now();
    let mut s = BState::new();
    seed_view_layer(&mut s, &model);
    phiber::v2::app::seed::seed_render_props(&mut s, &model);
    set_loaded_scene(Some(model.clone()));
    let hidden = BHidden::now(&s);
    let objects: Vec<BObject3D> = current_scene(&s, Some(&model))
        .into_iter()
        .filter(|o| !matches!(o, BObject3D::Loaded { name, .. } if hidden.hides(&s, name)))


        .collect();
    let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
    for o in &objects {
        let BObject3D::Loaded { mesh, matrix_world, .. } = o else { continue };
        let m = glam::Mat4::from_cols_array_2d(matrix_world);
        for v in &mesh.vertices {
            let p = m.transform_point3(glam::Vec3::from(*v)).to_array();
            for k in 0..3 { lo[k] = lo[k].min(p[k]); hi[k] = hi[k].max(p[k]); }
        }
    }
    // PHIBER_ONLY=<substring> keeps only the objects whose name matches,
    // PHIBER_DROP=<substring> removes them. Both are applied after the bounds,
    // so the frame does not move when an object is taken out to see what it
    // was contributing.
    let objects: Vec<BObject3D> = objects
        .into_iter()
        .filter(|o| match (std::env::var("PHIBER_ONLY").ok(), o) {
            (Some(k), BObject3D::Loaded { name, .. }) => name.contains(&k),
            _ => true,
        })
        .filter(|o| match (std::env::var("PHIBER_DROP").ok(), o) {
            (Some(k), BObject3D::Loaded { name, .. }) => !name.contains(&k),
            _ => true,
        })
        .collect();
    let scene = t.elapsed();

    // Orthographic front view (numpad 1): +X right, +Z up, looking down +Y —
    // the same frame `blender_color.py` sets up.
    let height = hi[2] - lo[2];
    let right = glam::Vec3::X;
    let up = glam::Vec3::Z;
    let back = glam::Vec3::NEG_Y;
    let sun = (right * 0.35 + up * 0.45 + back * 0.82).normalize();
    // The file's own lights and world, the way Blender renders it. Without
    // them the studio rig takes over and the image is flat.
    let studio = shading == phiber::v2::scene::BShading::Material;
    let mut lights = [Light::default(); LIGHTS];
    // Material Preview is lit by the studio HDRI alone: Blender switches the
    // scene's lamps off there (`blender_color.py` hides them for the render),
    // so they are left out here too — with them in, every Material
    // comparison read brighter than Cycles for a reason that was not the
    // tracer's.
    let usable: Vec<Light> = if studio {
        Vec::new()
    } else {
        phiber::v2::scene::scene_lights().iter().map(light).filter(|l| l.color.iter().any(|c| *c > 0.0)).take(LIGHTS).collect()
    };
    lights[..usable.len()].copy_from_slice(&usable);
    let env_image = if studio { None } else { environment() };
    let world = model.world.background.radiance();
    let ambient = if env_image.is_some() { world } else { world.map(|c| c.max(0.05)) };
    println!(
        "world: {:?} x{:.3} env={} rot={:.3} rad | lights: {}",
        model.world.background.color, model.world.background.strength,
        model.world.background.texture.as_deref().unwrap_or("-"),
        model.world.background.rotation, usable.len()
    );
    let cam = Camera {
        right: right.to_array(), up: up.to_array(), back: back.to_array(),
        pivot: [0.0, 0.0, (lo[2] + hi[2]) * 0.5], units: 1.0, persp: 0, aspect: W as f32 / H as f32,
        // `blender_color.py` sets `ortho_scale = height * 1.1 / aspect²`, and
        // for a portrait frame that spans the *height*. The kernel's vertical
        // extent is `1 / (aspect * scale)`, so matching it means
        // `scale = aspect / (height * 1.1)`. Measured, not derived: with the
        // factor the other way round the figure came out 2.3× too large
        // (1011 px against Blender's 436).
        scale: (W as f32 / H as f32) / (height * 1.1), dist: 10.0,
        sun: sun.to_array(),
        sky: [0.24, 0.24, 0.24, 1.0],
        env: [ambient[0], ambient[1], ambient[2], 1.0],
        // Material Preview is lit by Blender's studio HDRI (`rendered/studio.rs`),
        // which `scene::build` puts in the atlas for that shading; the file's
        // world image only lights a Rendered view. The studio light is *not
        // shown*: the background stays the viewport grey, as in Blender.
        envmap: if studio || env_image.is_some() { ENV_SLOT + 1 } else { 0 },
        env_cdf: if studio || env_image.is_some() { ENV_CDF_SLOT + 1 } else { 0 },
        env_rot: if studio { 0.0 } else { env_image.as_ref().map_or(0.0, |(_, rot)| *rot) },
        env_hidden: u32::from(studio),
        // PHIRAY_BOUNCES sweeps the diffuse bounce depth (0 = the default 4).
        bounces: std::env::var("PHIRAY_BOUNCES").ok().and_then(|v| v.parse().ok()).unwrap_or(0),
        count: usable.len() as u32,
        lights,
        grid: 0.0,
        // The file's view transform (Filmic, AgX …), as Blender shows it.
        // PHIBER_VIEW=Standard|Filmic|AgX overrides the file's view transform
        // (the reference script honours the same variable).
        view: phiray::View::from_blender(&std::env::var("PHIBER_VIEW").unwrap_or_else(|_| model.render.view_transform.clone())).words(model.render.exposure, model.render.gamma),
        ..Default::default()
    };
    let t = Instant::now();
    let (px, samples) = still(&objects, shading, gpui::hsla(0.0, 0.0, 0.8, 1.0), cam, W, H, samples, denoise).expect("gpu adapter and a scene that fits");
    let render = t.elapsed();
    image::RgbaImage::from_raw(W, H, px).expect("image").save(&out).expect("save");
    let ms = |d: std::time::Duration| d.as_secs_f64() * 1000.0;
    println!(
        "phiber {:?}: read {:.0} ms, modifiers+scene {:.0} ms, render {:.0} ms ({samples} samples), total {:.0} ms -> {out}",
        shading, ms(read), ms(scene), ms(render), ms(all.elapsed())
    );
}
