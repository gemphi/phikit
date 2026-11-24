//! Write every thing of a collection's tables as `<name>.usda`:
//! `cargo run -p phiphx --example thing_usd -- <tables> <folder>`
//! (`<tables>` holds `thing.tsv`, `look.tsv` and `source.tsv`).
//!
//! Prints one line per thing: its name, its triangles and its file.
fn main() {
    let mut arg = std::env::args().skip(1);
    let (Some(tables), Some(folder)) = (arg.next(), arg.next()) else {
        eprintln!("usage: thing_usd <tables> <folder>");
        std::process::exit(2);
    };
    let set = phiphx::geom::thing::ThingSet::read(std::path::Path::new(&tables));
    let folder = std::path::Path::new(&folder);
    std::fs::create_dir_all(folder).expect("make the folder");
    for thing in &set.thing {
        let Some(tree) = set.build(&thing.name) else { continue };
        let file = folder.join(format!("{}.usda", thing.name));
        std::fs::write(&file, tree.to_usda()).expect("write the thing");
        println!("{}\t{} triangles\t{}", thing.name, tree.triangle_count(), file.display());
    }
}
