//! Print a `.blend`'s quick info (version, REND scenes, thumbnail size) and
//! optionally dump the thumbnail as raw RGBA: `peek_blend file.blend [out.rgba]`.
fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("path to .blend");
    let t = std::time::Instant::now();
    let info = phiphx::blend::BlendFile::read_quick_info(std::path::Path::new(&path)).expect("peek");
    println!("{path}: Blender {} in {:.2?}", info.version, t.elapsed());
    for s in &info.scenes {
        println!("  scene {:?} frames {}–{}", s.scene, s.frame_start, s.frame_end);
    }
    match &info.thumbnail {
        Some(th) => {
            println!("  thumbnail {}x{}", th.width, th.height);
            if let Some(out) = args.next() {
                std::fs::write(out, &th.rgba).expect("write");
            }
        }
        None => println!("  no thumbnail"),
    }
}
