//! Why a scene does or does not reach the GPU: what it asks for, against a
//! device's binding limit (`rendered::scene_fit`).
//!   cargo run --release -p phiber --example scene_sizes -- FILE
use phiber::phiphx::Phx;
use phiber::v2::app::seed::{seed_output_props, seed_render_props, seed_view_layer};
use phiber::v2::part::outliner::key::BHidden;
use phiber::v2::scene::{BShading, current_scene, source::set_loaded_scene};
use phiber::v2::state::BState;
use phiber::v2::view::draw::canvas::view3d::rendered::scene_fit;

fn main() {
    let path = std::env::args().nth(1).expect("file");
    let model = Phx::open(std::path::Path::new(&path)).expect("open");
    let mut s = BState::new();
    seed_view_layer(&mut s, &model);
    seed_render_props(&mut s, &model);
    seed_output_props(&mut s, &model);
    set_loaded_scene(Some(model.clone()));
    let hidden = BHidden::now(&s);
    let objects: Vec<_> = current_scene(&s, Some(&model))
        .into_iter()
        .filter(|o| !matches!(o, phiber::v2::scene::BObject3D::Loaded { name, .. } if hidden.hides(&s, name)))
        .collect();
    for (name, limit) in [
        ("GTX 1660 Ti / wgpu default", 128u64 * 1048576),
        ("256 MiB device", 256 * 1048576),
        ("2 GiB device", 2048 * 1048576),
    ] {
        println!("\n== {name}: {} MiB per binding, 4 chunks", limit / 1048576);
        match scene_fit(&objects, BShading::Rendered, limit, 4) {
            Ok(sizes) => println!("   fits — {sizes}"),
            Err(why) => println!("   {why}"),
        }
    }
}
