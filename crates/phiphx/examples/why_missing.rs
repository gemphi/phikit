//! Why an object does not reach the viewport.
//!
//! Walks a file and reports, per object, what it carries: geometry, the
//! evaluated result of its modifier stack, and whether anything hid it. An
//! object that is read but draws nothing shows up here as a mesh with no
//! faces, which is where a missing garment goes missing.

use phiphx::blend::{BlendFile, BlendScene};

fn main() {
    let path = std::env::args().nth(1).expect("usage: why_missing <file.blend>");
    let bytes = std::fs::read(&path).expect("read");
    let file = BlendFile::parse_bytes(&bytes).expect("parse");
    let scene: BlendScene = file.extract_scene().expect("scene");

    println!("{} objects\n", scene.objects.len());
    let (mut with_faces, mut no_faces, mut no_verts) = (0usize, Vec::new(), Vec::new());
    for o in &scene.objects {
        if o.object_type != "MESH" {
            continue;
        }
        let verts = o.vertices.len();
        let faces = o.faces.len();
        if verts == 0 {
            no_verts.push(o.name.clone());
        } else if faces == 0 {
            no_faces.push(o.name.clone());
        } else {
            with_faces += 1;
        }
        println!(
            "{:<42} verts={:<8} faces={:<8} uvs={:<8} slots={}",
            o.name,
            verts,
            faces,
            o.loop_uvs.len(),
            o.materials.len()
        );
    }
    println!("\n{with_faces} meshes have faces to draw");
    for (what, names) in [("no vertices", &no_verts), ("vertices but no faces", &no_faces)] {
        if !names.is_empty() {
            println!("\n{} mesh(es) with {what}:", names.len());
            for n in names {
                println!("  {n}");
            }
        }
    }
}
