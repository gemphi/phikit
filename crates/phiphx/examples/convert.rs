//! Convert any file phiphx opens to any scene format it writes, the
//! formats from the extensions: `.blend`, `.usd/.usda/.usdc/.usdz`,
//! `.gltf/.glb`, `.obj`, `.ply`, `.phx` in; `.usda/.usd`, `.phx`, `.gltf`,
//! `.ply` out. USD to USD is copied as it is.
//!
//! `cargo run -p phiphx --release --example convert -- <in> <out>`
//!
//! This is the command a server runs on every uploaded file. Prints the
//! object, mesh and triangle counts.
fn main() {
    let arg: Vec<String> = std::env::args().skip(1).collect();
    let [input, output] = arg.as_slice() else {
        eprintln!("usage: convert <in> <out>");
        std::process::exit(2);
    };
    let started = std::time::Instant::now();
    match phiphx::Phx::convert(std::path::Path::new(input), std::path::Path::new(output)) {
        Ok(c) => {
            let how = match c.copied {
                true => "copied",
                false => "written",
            };
            println!(
                "{output}: {how}, {} objects, {} meshes, {} triangles, {} bytes, {:?}",
                c.object_count, c.mesh_count, c.triangle_count, c.byte_count, started.elapsed()
            );
        }
        Err(e) => {
            eprintln!("{input}: {e:?}");
            std::process::exit(1);
        }
    }
}
