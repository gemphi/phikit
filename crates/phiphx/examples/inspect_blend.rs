use std::fs::File;
use phiphx::importers::blend_native::{BlendFile, BlendCollection};

fn print_collection(collection: &BlendCollection, depth: usize, out: &mut String) {
    if collection.name.contains("WGT") { return; }
    
    let indent = "  ".repeat(depth);
    out.push_str(&format!("{}📁 {}\n", indent, collection.name));
    
    for child in &collection.children {
        print_collection(child, depth + 1, out);
    }
    
    for obj_name in &collection.object_names {
        if !obj_name.contains("WGT") {
            let category = if obj_name.to_lowercase().contains("rig") || obj_name.contains("ctrl") { "🦴" } else { "🔷" };
            out.push_str(&format!("{}  {} {}\n", indent, category, obj_name));
        }
    }
}

fn main() {
    let path = std::path::Path::new("phi/ext/phx/assets/Mannequins-Female--Pose-05.blend");
    let mut file = File::open(path).expect("Failed to open blend file");
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📦 BLENDER SCENE HIERARCHY (NATIVE API)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    let blend = BlendFile::parse(&mut file).expect("Failed to parse blend file");
    
    let scene = blend.extract_scene().expect("Failed to extract scene");
    
    let mut output = String::new();
    for col in &scene.collections {
        print_collection(col, 0, &mut output);
    }
    
    println!("{}", output);
}
