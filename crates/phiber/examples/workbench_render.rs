//! Render a `.blend` the way the viewport does in Solid mode — full modifier
//! stack through phiphx, view-layer visibility, phiray's Workbench kernel —
//! from a fixed orthographic front view, to a PNG. Pair it with
//! `blender_front.py` for the same view out of Blender itself.
//!
//!   cargo run --release -p phiber --example workbench_render -- FILE OUT.png
use phiber::phiphx::Phx;
use phiber::v2::app::seed::seed_view_layer;
use phiber::v2::part::outliner::key::BHidden;
use phiber::v2::scene::{BShading, current_scene, source::set_loaded_scene};
use phiber::v2::state::BState;
use phiray::{Atlas, Camera, Mode, Ray, Tracer, Tri};

const W: u32 = 720;
const H: u32 = 1080;

fn main() {
    let mut args = std::env::args().skip(1);
    let (path, out) = (args.next().expect("file"), args.next().expect("out.png"));
    let model = Phx::open(std::path::Path::new(&path)).expect("open");
    let mut s = BState::new();
    seed_view_layer(&mut s, &model);
    set_loaded_scene(Some(model.clone()));
    let hidden = BHidden::now(&s);
    let objects = current_scene(&s, Some(&model));

    let mut tris = Vec::new();
    let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
    for o in &objects {
        let phiber::v2::scene::BObject3D::Loaded { name, mesh, matrix_world, .. } = o else { continue };
        if hidden.hides(&s, name) { continue; }
        let m = glam::Mat4::from_cols_array_2d(matrix_world);
        let nm = glam::Mat3::from_mat4(m).inverse().transpose();
        let pos: Vec<[f32; 3]> = mesh.vertices.iter().map(|v| m.transform_point3(glam::Vec3::from(*v)).to_array()).collect();
        for p in &pos { for k in 0..3 { lo[k] = lo[k].min(p[k]); hi[k] = hi[k].max(p[k]); } }
        for (f, t) in mesh.indices.chunks_exact(3).enumerate() {
            let i = [t[0] as usize, t[1] as usize, t[2] as usize];
            if i.iter().any(|&k| k >= pos.len()) { continue; }
            let n = |c: usize, v: usize| -> [f32; 3] {
                let raw = match &mesh.normals {
                    _ if !mesh.corners.is_empty() => mesh.corners.normal(c),
                    phiber::phiphx::scene::SceneNormals::PerVertex(n) => n.get(v).copied().unwrap_or_default(),
                    phiber::phiphx::scene::SceneNormals::PerFace(_) => [0.0; 3],
                };
                if raw == [0.0; 3] { raw } else { (nm * glam::Vec3::from(raw)).normalize_or_zero().to_array() }
            };
            let color = o.triangle_color(f, BShading::Solid).unwrap_or([0.8, 0.8, 0.8, 1.0]);
            let lin = |c: f32| if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) };
            tris.push(Tri::new(
                [pos[i[0]], pos[i[1]], pos[i[2]]],
                [n(f * 3, i[0]), n(f * 3 + 1, i[1]), n(f * 3 + 2, i[2])],
                [lin(color[0]), lin(color[1]), lin(color[2]), 1.0],
            ));
        }
    }
    eprintln!("{} triangles, bounds {lo:?}..{hi:?}", tris.len());
    let gpu = Ray::gpu().expect("gpu");
    let mut tr = Tracer::new(&gpu, &tris, &Atlas::default()).expect("scene fits");
    tr.set_mode(Mode::Solid);
    let centre = [0.0, 0.0, (lo[2] + hi[2]) * 0.5];
    let height = hi[2] - lo[2];
    // Orthographic front view (Blender numpad 1): +X right, +Z up, looking down +Y.
    let cam = Camera {
        right: [1.0, 0.0, 0.0], up: [0.0, 0.0, 1.0], back: [0.0, -1.0, 0.0],
        pivot: centre, units: 1.0, persp: 0, aspect: W as f32 / H as f32,
        // scale: world units per full frame height → 1 / (height * 1.1) on the horizontal axis
        scale: 1.0 / (height * 1.1) * (W as f32 / H as f32), dist: 10.0,
        sky: [0.24, 0.24, 0.24, 1.0], env: [0.24, 0.24, 0.24, 1.0], grid: 0.0,
        ..Default::default()
    };
    let (px, _) = tr.render(&gpu, cam, W, H);
    image::RgbaImage::from_raw(W, H, px).expect("image").save(&out).expect("save");
    eprintln!("wrote {out}");
}
