//! Dump what Phiber will show for a `.blend`: header, active scene, the
//! View Layer collection tree with flags, and every object with type, parent
//! and world position. Usage: `cargo run -p phiphx --example inspect_blend -- file.blend`
use phiphx::blend::{BlendCollection, BlendFile, BlendScene};

fn tree(c: &BlendCollection, scene: &BlendScene, depth: usize) {
    let pad = "  ".repeat(depth);
    let flags = format!(
        "{}{}",
        if c.exclude { " [excluded]" } else { "" },
        if c.hide_viewport { " [hidden]" } else { "" }
    );
    println!("{pad}▸ {}{flags}", c.name);
    for &i in &c.object_indices {
        if let Some(o) = scene.objects.get(i) {
            let sel = if scene.selected.contains(&i) { "*" } else { " " };
            let act = if scene.active_object == Some(i) { "A" } else { " " };
            let t = &o.matrix_world[3];
            println!(
                "{pad}  {sel}{act} {:<10} {:<28} verts={:<7} slots={} world=({:.2}, {:.2}, {:.2})",
                o.object_type, o.name, o.vertices.len(), o.materials.len(), t[0], t[1], t[2]
            );
        }
    }
    for k in &c.children {
        tree(k, scene, depth + 1);
    }
}

fn main() {
    let path = std::env::args().nth(1).expect("path to .blend");
    let data = std::fs::read(&path).expect("read");
    let file = BlendFile::parse_bytes(&data).expect("parse");
    let h = &file.header;
    println!(
        "Blender {} (format v{}, {}-bit, {} blocks)",
        h.version_label(), h.file_format_version, h.pointer_size * 8, file.blocks.len()
    );
    let scene = file.extract_scene().expect("scene");
    let p = &scene.props;
    println!(
        "Scene '{}' · view layer '{}' · frames {}–{} @ {} fps (current {}) · {}x{} {}% · {}",
        scene.name, p.view_layer_name, p.frame_start, p.frame_end, p.fps, p.frame_current,
        p.resolution_x, p.resolution_y, p.resolution_percent, p.engine
    );
    for c in &scene.collections {
        tree(c, &scene, 0);
    }
    println!(
        "{} objects, {} selected, {} hidden",
        scene.objects.len(), scene.selected.len(), scene.hidden.len()
    );
}
