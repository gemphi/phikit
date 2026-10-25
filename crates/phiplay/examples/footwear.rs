//! Builds every footwear design and writes each card's picture and its six
//! views, and a preview dump of one set for `tools/preview.py`.
//!
//! `cargo run -p phiplay --example footwear -- <out dir> [set slug]`

use phiplay::build::footwear::{pair, picture::{BShot, left, picture, sheet}};
use phiplay::kit::footwear::{BFootwearSet, DESIGNS};
use std::io::Write;

fn ppm(path: &std::path::Path, img: &phiplay::build::BImage) {
    let mut f = std::fs::File::create(path).unwrap();
    write!(f, "P6 {} {} 255\n", img.width, img.height).unwrap();
    // Over the card's grey, as the browser shows it.
    let px: Vec<u8> = img.rgba.chunks(4).flat_map(|p| { let a = p[3] as f32 / 255.0; [0, 1, 2].map(|i| (p[i] as f32 * a + 58.0 * (1.0 - a)) as u8) }).collect();
    f.write_all(&px).unwrap();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dir = std::path::Path::new(&args[1]);
    std::fs::create_dir_all(dir).unwrap();
    for d in DESIGNS.iter() {
        let parts = pair(d);
        ppm(&dir.join(format!("{}.ppm", d.slug)), &picture(&left(&parts), 256, BShot::SideOuter));
        ppm(&dir.join(format!("{}_views.ppm", d.slug)), &sheet(&left(&parts), 320));
        let tris: usize = parts.iter().map(|p| p.mesh.triangle_count()).sum();
        eprintln!("{:<16} {:<16} {:>3} parts {:>7} tris", d.slug, d.set.spec().label, parts.len(), tris);
    }
    if let Some(set) = args.get(2).and_then(|s| BFootwearSet::by_slug(s)) {
        let mut out = std::io::BufWriter::new(std::fs::File::create(dir.join(format!("{}.bin", set.spec().slug))).unwrap());
        let parts: Vec<_> = set.designs().enumerate().flat_map(|(i, d)| {
            let dx = (i as f32 - 1.5) * 0.5;
            pair(d).into_iter().map(move |p| (p, dx))
        }).collect();
        out.write_all(&(parts.len() as u32).to_le_bytes()).unwrap();
        for (p, dx) in &parts {
            out.write_all(&(p.name.len() as u32).to_le_bytes()).unwrap();
            out.write_all(p.name.as_bytes()).unwrap();
            out.write_all(&(p.mesh.positions.len() as u32).to_le_bytes()).unwrap();
            for (q, uv) in p.mesh.positions.iter().zip(&p.mesh.uvs) {
                for x in [q[0] + dx, q[1], q[2], uv[0], uv[1]] {
                    out.write_all(&x.to_le_bytes()).unwrap();
                }
            }
            out.write_all(&(p.mesh.indices.len() as u32).to_le_bytes()).unwrap();
            for i in &p.mesh.indices {
                out.write_all(&i.to_le_bytes()).unwrap();
            }
            let m = &p.material;
            for x in [m.base.0, m.base.1, m.base.2, m.roughness, m.metallic, 0.0, 0.0, 0.0, 0.0] {
                out.write_all(&x.to_le_bytes()).unwrap();
            }
            out.write_all(&0u32.to_le_bytes()).unwrap();
            out.write_all(&0u32.to_le_bytes()).unwrap();
        }
    }
}
