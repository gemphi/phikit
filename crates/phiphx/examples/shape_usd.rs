//! Write every shape of `data/shape.tsv` as `<name>.usda` into a folder:
//! `cargo run -p phiphx --example shape_usd -- <folder>`
//!
//! Prints one line per shape: its name, its parts and their triangles.
fn main() {
    let Some(folder) = std::env::args().nth(1) else {
        eprintln!("usage: shape_usd <folder>");
        std::process::exit(2);
    };
    let folder = std::path::Path::new(&folder);
    std::fs::create_dir_all(folder).expect("make the folder");
    for shape in phiphx::geom::shape::Shape::all() {
        let file = folder.join(format!("{}.usda", shape.name));
        std::fs::write(&file, shape.to_usda()).expect("write the shape");
        let part: Vec<String> = shape.part().iter().map(|p| format!("{} {}", p.name, p.triangle_count())).collect();
        let total: usize = shape.part().iter().map(|p| p.triangle_count()).sum();
        println!("{}\t{} triangles\t({})\t{}", shape.name, total, part.join(", "), file.display());
    }
}
