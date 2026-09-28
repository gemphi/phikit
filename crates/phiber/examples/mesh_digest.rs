//! Per-object evaluated mesh digest — vertex / triangle counts and world
//! bounds — to set beside `phiphx/tests/truth/blender_mesh_digest.py`.
//!
//!   cargo run --release -p phiber --example mesh_digest -- FILE
use phiber::phiphx::Phx;
use phiber::v2::app::seed::seed_view_layer;
use phiber::v2::part::outliner::key::BHidden;
use phiber::v2::scene::{BObject3D, current_scene, source::set_loaded_scene};
use phiber::v2::state::BState;

fn main() {
    let path = std::env::args().nth(1).expect("file");
    let t = std::time::Instant::now();
    let model = Phx::open(std::path::Path::new(&path)).expect("open");
    eprintln!("open {:?}", t.elapsed());
    let mut s = BState::new();
    seed_view_layer(&mut s, &model);
    set_loaded_scene(Some(model.clone()));
    let hidden = BHidden::now(&s);
    for o in current_scene(&s, Some(&model)) {
        let BObject3D::Loaded { name, mesh, matrix_world, .. } = &o else { continue };
        if hidden.hides(&s, name) { continue; }
        let m = glam::Mat4::from_cols_array_2d(matrix_world);
        let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
        for v in &mesh.vertices {
            let p = m.transform_point3(glam::Vec3::from(*v)).to_array();
            for k in 0..3 { lo[k] = lo[k].min(p[k]); hi[k] = hi[k].max(p[k]); }
        }
        let r = |a: [f32; 3]| a.map(|x| (x * 1000.0).round() / 1000.0);
        println!("P {:40} v={:7} t={:7} lo={:?} hi={:?}", format!("{name:?}"), mesh.vertices.len(), mesh.indices.len() / 3, r(lo), r(hi));
    }
}
