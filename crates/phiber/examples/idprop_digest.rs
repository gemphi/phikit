//! What a `.blend` stores as ID properties on its scene — `scene["cycles"]`
//! and anything else an add-on left there. Set beside `bpy`'s
//! `scene.get("cycles").keys()` to check the reader.
//!
//!   cargo run --release -p phiber --example idprop_digest -- FILE
use phiber::phiphx::Phx;

fn main() {
    let path = std::env::args().nth(1).expect("file");
    let model = Phx::open(std::path::Path::new(&path)).expect("open");
    let cycles = &model.render.cycles;
    println!("scene[\"cycles\"]: {} entries", cycles.names().len());
    for name in cycles.names() {
        println!("  {name} = {:?}", cycles.get(name).expect("listed"));
    }
}
