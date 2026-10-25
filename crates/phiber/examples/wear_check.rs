//! Every sketch check the Project Browser shows — the Cut-Off Series'
//! drawings on their centimetre plane with the model's outline over the
//! drawing's — written as PNGs, each with its overlap.
//!
//!   cargo run --release -p phiber --example wear_check -- OUT_DIR
use phiber::v2::part::project::check::{checks, image, key, overlap};

fn main() {
    let out = std::path::PathBuf::from(std::env::args().nth(1).expect("out dir"));
    std::fs::create_dir_all(&out).expect("out dir");
    let mut seen = std::collections::HashSet::new();
    for d in phiplay::kit::footwear::DESIGNS.iter() {
        // One design per set: the drawing is the set's, not the colourway's.
        if !seen.insert(d.set) {
            continue;
        }
        for (i, check) in checks(d).iter().enumerate() {
            let k = key(d, i);
            let (w, h, rgba) = image(&k).expect("overlay");
            let name = format!("{}_{i}.png", d.slug);
            image::save_buffer(out.join(&name), &rgba, w, h, image::ColorType::Rgba8).expect("png");
            println!("{:20} {:14} overlap {:.2}  {name}", d.set.spec().label, check.label, overlap(&k).unwrap_or(0.0));
        }
    }
}
