//! What the materials in a file actually use: `cargo run --release --example mat_survey -- f.blend`
fn main() {
    for p in std::env::args().skip(1) {
        let Ok(m) = phiphx::phx::Phx::open(std::path::Path::new(&p)) else {
            eprintln!("{p}: could not open");
            continue;
        };
        let mats: Vec<_> = m.objects.iter().flat_map(|o| &o.materials).collect();
        let n = mats.len().max(1);
        let base_img = mats.iter().filter(|x| x.base_color_texture.is_some()).count();
        let normal = mats.iter().filter(|x| x.normal_texture.is_some()).count();
        let decoded = mats.iter().filter(|x| x.texture.is_some()).count();
        let missing: Vec<&str> = mats
            .iter()
            .filter(|x| x.base_color_texture.is_some() && x.texture.is_none())
            .filter_map(|x| x.base_color_texture.as_deref())
            .take(3)
            .collect();
        let maps = mats.iter().filter(|x| x.maps.images().iter().any(Option::is_some)).count();
        let emit = mats.iter().filter(|x| x.bsdf.emitted().iter().any(|c| *c > 0.0)).count();
        let bright = mats.iter().filter(|x| x.bsdf.emitted().iter().any(|c| *c > 1.0)).count();
        let glass = mats.iter().filter(|x| x.bsdf.transmission > 0.0).count();
        let alpha = mats.iter().filter(|x| x.bsdf.alpha < 1.0).count();
        // A material whose colour came out as the procedural fold's flat 0.5
        // grey looks wrong next to Blender.
        let grey = mats
            .iter()
            .filter(|x| x.base_color[..3].iter().all(|c| (c - 0.5).abs() < 1e-6))
            .count();
        let distinct: std::collections::HashSet<[u32; 3]> = mats
            .iter()
            .map(|x| [0, 1, 2].map(|i| x.base_color[i].to_bits()))
            .collect();
        println!("  flat-0.5 colours: {grey} | distinct colours: {}", distinct.len());
        let name = std::path::Path::new(&p).file_name().unwrap().to_string_lossy();
        if !missing.is_empty() {
            println!("  missing images e.g. {missing:?}");
        }
        println!("{name}: {n} materials | base image {base_img} (decoded {decoded}) | normal {normal} | rough/metal/emit maps {maps} | emitting {emit} (over 1.0: {bright}) | glass {glass} | alpha {alpha}");
    }
}
