//! Headless pose 26: the file read by phiphx (Phiber's own reader), the
//! render cast by Blender's rules (camera icon on the object, camera icon
//! or checkbox on every collection holding it), traced by phiray through the
//! file's camera. Each object gets its own flat colour, so a face drawn from
//! the wrong object — the streaks — shows as a colour where it does not
//! belong. Writes the image and the depth as PNGs.
//!
//! `cargo run --release -p phiber --example ray_probe -- <file.blend> <out-prefix> [width] [samples] [slices]`
//!
//! What it isolates: the walk, the instancing and the dispatch slicing,
//! against the final render's other parts. `PROBE_ADAPT=1` turns on
//! Adaptive Sampling as the Render panel's defaults do (threshold 0.05,
//! min 64, every 16); `PROBE_DOF=1` gives the camera its depth of field.
//! A clean probe and a streaked F12 put the fault in what the probe leaves
//! out (materials, textures, lamps); a streaked probe puts it in phiray.

use glam::Mat4;
use phiphx::scene::{SceneCamera, SceneCollection, SceneModel};
use phiray::{AREA_DISK, AREA_ELLIPSE, AREA_RECT, AREA_SQUARE, Camera, LIGHTS, Light, Mat, Mode, Ray, Scene, Tracer, Tri};
use std::collections::HashSet;

fn camera_of(cam: &SceneCamera, w: u32, h: u32) -> Camera {
    let m = Mat4::from_cols_array_2d(&cam.matrix_world);
    let right = m.x_axis.truncate().normalize_or_zero();
    let up = m.y_axis.truncate().normalize_or_zero();
    let back = m.z_axis.truncate().normalize_or_zero();
    let eye = m.w_axis.truncate();
    let aspect = w as f32 / h as f32;
    let wide = w >= h;
    let half = (cam.sensor_width.max(1e-6) * 0.5) / cam.focal_length_mm.max(1e-6);
    let s = 0.5 / half;
    Camera {
        right: right.to_array(),
        up: up.to_array(),
        back: back.to_array(),
        pivot: (eye - back).to_array(),
        units: 1.0,
        persp: 1,
        aspect,
        scale: if wide { s } else { s / aspect },
        dist: 1.0,
        sky: [0.0, 0.0, 0.0, 0.0],
        env: [0.6, 0.6, 0.6, 1.0],
        sun: (back * 0.8 + up * 0.5).normalize().to_array(),
        aperture: if std::env::var_os("PROBE_DOF").is_some() { cam.dof.aperture(cam.focal_length_mm, false) } else { 0.0 },
        focus: cam.dof.focus_distance,
        blades: cam.dof.blades,
        ..Default::default()
    }
}

/// Blender's render cast: an object renders when its own camera is on and
/// at least one collection holding it (with every ancestor) renders.
fn cast(model: &SceneModel) -> HashSet<String> {
    fn walk(m: &SceneModel, c: &SceneCollection, hidden: bool, on: &mut HashSet<String>, off: &mut HashSet<String>) {
        let hides = hidden || c.visibility.hide_render || c.exclude;
        for &i in &c.object_indices {
            if let Some(o) = m.objects.get(i) {
                (if hides { &mut *off } else { &mut *on }).insert(o.name.clone());
            }
        }
        for k in &c.children {
            walk(m, k, hides, on, off);
        }
    }
    let (mut on, mut off) = (HashSet::new(), HashSet::new());
    for c in &model.collections {
        walk(model, c, false, &mut on, &mut off);
    }
    let own: HashSet<&String> = model.selection.render_disabled.iter().collect();
    model.objects.iter().map(|o| o.name.clone()).filter(|n| (on.contains(n) || !off.contains(n)) && !own.contains(n)).collect()
}

/// The file's lamps as Phiber hands them to phiray (`rendered/lamp.rs`).
fn lamps(model: &SceneModel) -> Vec<Light> {
    model
        .lights
        .iter()
        .map(|l| {
            let m = Mat4::from_cols_array_2d(&l.matrix_world);
            let kind = match l.kind.as_str() { "SUN" => 0, "SPOT" => 2, "AREA" => 3, _ => 1 };
            let (cone, blend, spot_scale) = if kind == 2 {
                let cone = (l.spot_size * 0.5).cos();
                let z = m.z_axis.truncate().length();
                (cone, ((1.0 - cone) * l.spot_blend).max(1e-8), [m.x_axis, m.y_axis].map(|a| z / a.truncate().length().max(1e-8)))
            } else {
                (0.0, 0.0, [0.0; 2])
            };
            let area = kind == 3;
            let (right, size, size_y) = match (area, kind == 2) {
                (true, _) => (m.x_axis.truncate().normalize_or_zero().to_array(), l.size * m.x_axis.truncate().length(), l.size_y * m.y_axis.truncate().length()),
                (_, true) => (m.x_axis.truncate().normalize_or_zero().to_array(), l.size, 0.0),
                _ => ([0.0; 3], l.size, 0.0),
            };
            let energy = if area { l.energy / phiphx::blend::AREA_ENERGY_SCALE } else { l.energy };
            let shape = match l.shape.as_str() { "RECTANGLE" => AREA_RECT, "DISK" => AREA_DISK, "ELLIPSE" => AREA_ELLIPSE, _ => AREA_SQUARE };
            Light {
                pos: m.w_axis.truncate().to_array(),
                kind,
                color: l.color.map(|c| c * energy.max(0.0)),
                size,
                dir: (-m.z_axis.truncate()).normalize_or_zero().to_array(),
                cone,
                blend,
                right,
                size_y,
                shape: if area { shape } else { AREA_SQUARE },
                spot_scale,
            }
        })
        .filter(|l| l.color.iter().any(|c| *c > 0.0))
        .take(LIGHTS)
        .collect()
}

fn hue(i: usize) -> [f32; 4] {
    let h = (i as f32 * 0.618034).fract() * 6.0;
    let x = 1.0 - (h % 2.0 - 1.0).abs();
    let (r, g, b) = match h as u32 { 0 => (1.0, x, 0.0), 1 => (x, 1.0, 0.0), 2 => (0.0, 1.0, x), 3 => (0.0, x, 1.0), 4 => (x, 0.0, 1.0), _ => (1.0, 0.0, x) };
    [0.1 + 0.8 * r, 0.1 + 0.8 * g, 0.1 + 0.8 * b, 1.0]
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let (file, out) = (&a[1], &a[2]);
    let w: u32 = a.get(3).and_then(|v| v.parse().ok()).unwrap_or(480);
    let samples: u32 = a.get(4).and_then(|v| v.parse().ok()).unwrap_or(4);
    let slices: u32 = a.get(5).and_then(|v| v.parse().ok()).unwrap_or(32);
    let t = std::time::Instant::now();
    let model = phiphx::phx::Phx::open(std::path::Path::new(file)).expect("open");
    let keep = cast(&model);
    let cam = model.cameras.iter().find(|c| c.name == model.render.camera).or(model.cameras.first()).expect("a camera");
    let (rx, ry) = (model.render.resolution_x.max(1) as u32, model.render.resolution_y.max(1) as u32);
    let h = (w as u64 * ry as u64 / rx as u64) as u32;
    println!("opened in {:.1}s: {} objects, {} in the render cast, camera {}, {}x{} -> {}x{}", t.elapsed().as_secs_f32(), model.objects.len(), keep.len(), cam.name, rx, ry, w, h);
    let mut scene = Scene::default();
    let mut tris = 0usize;
    for (i, o) in model.objects.iter().enumerate() {
        if !keep.contains(&o.name) || o.mesh.indices.is_empty() {
            continue;
        }
        scene.instance(o.matrix_world);
        let m = scene.material(Mat::of(&Tri::new([[0.0; 3]; 3], [[0.0, 0.0, 1.0]; 3], hue(i))));
        let base: Vec<u32> = o.mesh.vertices.iter().map(|p| scene.vertex(*p, [0.0; 3], [0.0; 2])).collect();
        for f in o.mesh.indices.chunks_exact(3) {
            if f.iter().all(|&k| (k as usize) < base.len()) {
                scene.face([base[f[0] as usize], base[f[1] as usize], base[f[2] as usize]], m);
                tris += 1;
            }
        }
    }
    scene.seal();
    println!("{} instances, {tris} triangles", scene.insts.len());
    // What the render still needs from the file, then the file's model is
    // let go: it and the scene built from it are each gigabytes on pose 26.
    let cam = cam.clone();
    let lamps_now = lamps(&model);
    drop(keep);
    drop(model);
    let gpu = Ray::gpu().expect("a GPU adapter");
    println!("GPU: {}", gpu.adapter.name);
    let limit: u64 = std::env::var("LIMIT_MB").ok().and_then(|v| v.parse().ok()).map_or(gpu.max_binding(), |mb: u64| mb << 20);
    let t = std::time::Instant::now();
    let mut tracer = Tracer::with_limit(&gpu, scene, limit).expect("the scene fits");
    println!("tracer built in {:.1}s (binding limit {} MB)", t.elapsed().as_secs_f32(), limit >> 20);
    tracer.set_mode(Mode::Rendered);
    tracer.set_samples(samples);
    if std::env::var_os("PROBE_ADAPT").is_some() {
        tracer.set_adaptive([0.05, 64.0, 16.0, 1.0]);
    }
    println!("adaptive {}, depth of field {} (aperture {:.4})", std::env::var_os("PROBE_ADAPT").is_some(), std::env::var_os("PROBE_DOF").is_some(), cam.dof.aperture(cam.focal_length_mm, false));
    let mut c = camera_of(&cam, w, h);
    if std::env::var_os("PROBE_LAMPS").is_some() {
        let ls = lamps_now;
        println!("lamps: {}", ls.iter().map(|l| format!("kind {} colour {:?} size {:.2}", l.kind, l.color, l.size)).collect::<Vec<_>>().join(" | "));
        c.count = ls.len() as u32;
        c.lights[..ls.len()].copy_from_slice(&ls);
    }
    let t = std::time::Instant::now();
    for _ in 0..samples {
        tracer.sample_in(&gpu, c, w, h, slices, || true);
    }
    let px = tracer.pixels(&gpu);
    println!("{samples} samples in {:.1}s", t.elapsed().as_secs_f32());
    let rgb: Vec<u8> = px.chunks_exact(4).flat_map(|p| [p[2], p[1], p[0]]).collect();
    image::save_buffer(format!("{out}_color.png"), &rgb, w, h, image::ColorType::Rgb8).unwrap();
    let depth = tracer.depth(&gpu);
    let hit: Vec<f32> = depth.iter().copied().filter(|d| *d < 1e9).collect();
    let (lo, hi) = hit.iter().fold((f32::MAX, 0f32), |(a, b), d| (a.min(*d), b.max(*d)));
    let grey: Vec<u8> = depth.iter().map(|d| if *d >= 1e9 { 0 } else { (255.0 - 230.0 * (d - lo) / (hi - lo).max(1e-6)) as u8 }).collect();
    image::save_buffer(format!("{out}_depth.png"), &grey, w, h, image::ColorType::L8).unwrap();
    println!("depth {lo:.2}..{hi:.2}, {} of {} pixels hit", hit.len(), depth.len());
}
