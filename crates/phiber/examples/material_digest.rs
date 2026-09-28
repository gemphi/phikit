//! What each visible object's materials fold to — base colour, metallic,
//! roughness, textures — to check against Blender's node graphs.
//!
//!   cargo run --release -p phiber --example material_digest -- FILE [name filter]
use phiber::phiphx::Phx;

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("file");
    let filter = args.next().unwrap_or_default();
    let model = Phx::open(std::path::Path::new(&path)).expect("open");
    for o in &model.objects {
        if !o.name.contains(&filter) { continue; }
        for m in &o.materials {
            let r = |v: [f32; 4]| v.map(|x| (x * 1000.0).round() / 1000.0);
            println!(
                "{:32} {:28} base={:?} metal={:.3} rough={:.3} sss={:.3} radius={:?} sheen={:.3} coat={:.3} aniso={:.3} tex={} maps={:?}",
                o.name, m.name, r(m.base_color), m.metallic, m.roughness,
                m.bsdf.subsurface, m.bsdf.scatter().map(|x| (x * 1000.0).round() / 1000.0),
                m.bsdf.sheen, m.bsdf.coat, m.bsdf.anisotropic,
                m.texture.as_ref().map_or("-".into(), |t| format!("{}x{}", t.width, t.height)),
                m.maps.images().iter().map(Option::is_some).collect::<Vec<_>>()
            );
        }
    }
}
