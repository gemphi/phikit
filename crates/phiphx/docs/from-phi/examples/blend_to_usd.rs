//! blend_to_usd — Native Rust .blend → .usda/.usdz converter
//!
//! Reads a .blend file, extracts all mesh geometry via SDNA,
//! and writes a single combined USDA scene (or USDZ package).
//!
//! Usage:
//!   cargo run --example blend_to_usd

use std::path::Path;
use std::fs;
use std::fmt::Write as FmtWrite;
use std::io::Write;

fn main() {
    let imports_dir = Path::new("phi/ext/phx/imports");
    let exports_dir = Path::new("phi/ext/phx/exports");
    fs::create_dir_all(exports_dir).unwrap();

    // Collect all .blend files
    let blend_files: Vec<_> = fs::read_dir(imports_dir)
        .expect("Cannot read imports/")
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().map(|x| x == "blend").unwrap_or(false)
        })
        .collect();

    println!("Found {} .blend files to convert to USD\n", blend_files.len());

    for entry in &blend_files {
        let path = entry.path();
        let stem = path.file_stem().unwrap().to_string_lossy().to_string();

        println!("━━━ Converting: {} ━━━", stem);

        // 1. Parse the .blend file natively
        let mut file = match fs::File::open(&path) {
            Ok(f) => f,
            Err(e) => { println!("  ✗ Cannot open: {}\n", e); continue; }
        };
        let blend = match phiphx::importers::blend_native::BlendFile::parse(&mut file) {
            Ok(b) => b,
            Err(e) => { println!("  ✗ Parse failed: {}\n", e); continue; }
        };
        let scene = match blend.extract_scene() {
            Ok(s) => s,
            Err(e) => { println!("  ✗ Scene extraction failed: {}\n", e); continue; }
        };

        // 2. Build a single combined USDA with all objects as Xform > Mesh prims
        let mut usda = String::new();
        writeln!(usda, "#usda 1.0").unwrap();
        writeln!(usda, "(").unwrap();
        writeln!(usda, "    defaultPrim = \"Root\"").unwrap();
        writeln!(usda, "    metersPerUnit = 1.0").unwrap();
        writeln!(usda, "    upAxis = \"Z\"").unwrap();
        writeln!(usda, ")").unwrap();
        writeln!(usda).unwrap();
        writeln!(usda, "def Xform \"Root\"").unwrap();
        writeln!(usda, "{{").unwrap();

        let mut obj_count = 0usize;
        let mut vert_count = 0usize;
        let mut face_count = 0usize;

        for obj in &scene.objects {
            // Sanitise name for USD prim path (alphanumeric + underscore only)
            let prim_name: String = obj.name.chars().map(|c| {
                if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' }
            }).collect();
            let prim_name = if prim_name.is_empty() { format!("Object_{}", obj_count) } else { prim_name };

            // Skip objects with no geometry
            if obj.vertices.is_empty() {
                // Still emit an Xform so the hierarchy is complete
                writeln!(usda, "    def Xform \"{}\"", prim_name).unwrap();
                writeln!(usda, "    {{").unwrap();
                write_matrix(&mut usda, &obj.matrix_world, 8);
                writeln!(usda, "    }}").unwrap();
                writeln!(usda).unwrap();
                obj_count += 1;
                continue;
            }

            writeln!(usda, "    def Xform \"{}\"", prim_name).unwrap();
            writeln!(usda, "    {{").unwrap();
            write_matrix(&mut usda, &obj.matrix_world, 8);

            // Mesh sub-prim
            writeln!(usda, "        def Mesh \"mesh\"").unwrap();
            writeln!(usda, "        {{").unwrap();

            // Points
            let pts: Vec<String> = obj.vertices.iter()
                .map(|v| format!("({:.6}, {:.6}, {:.6})", v[0], v[1], v[2]))
                .collect();
            writeln!(usda, "            point3f[] points = [{}]", pts.join(", ")).unwrap();

            // Faces → faceVertexCounts + faceVertexIndices
            let mut fvc = Vec::new();
            let mut fvi = Vec::new();
            for f in &obj.faces {
                fvc.push(f.len().to_string());
                for &idx in f {
                    fvi.push(idx.to_string());
                }
                face_count += 1;
            }
            writeln!(usda, "            int[] faceVertexCounts = [{}]", fvc.join(", ")).unwrap();
            writeln!(usda, "            int[] faceVertexIndices = [{}]", fvi.join(", ")).unwrap();
            writeln!(usda, "            uniform token subdivisionScheme = \"none\"").unwrap();

            writeln!(usda, "        }}").unwrap(); // close Mesh
            writeln!(usda, "    }}").unwrap(); // close Xform
            writeln!(usda).unwrap();

            vert_count += obj.vertices.len();
            obj_count += 1;
        }

        writeln!(usda, "}}").unwrap(); // close Root

        // 3. Write .usda (plain text)
        let usda_path = exports_dir.join(format!("{}.usda", stem));
        fs::write(&usda_path, &usda).unwrap();

        // 4. Write .usdz (uncompressed zip containing the .usda)
        let usdz_path = exports_dir.join(format!("{}.usdz", stem));
        let zip_file = fs::File::create(&usdz_path).unwrap();
        let mut zip = zip::ZipWriter::new(zip_file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip.start_file(format!("{}.usda", stem), options).unwrap();
        zip.write_all(usda.as_bytes()).unwrap();
        zip.finish().unwrap();

        println!("  ✓ {} objects, {} vertices, {} faces", obj_count, vert_count, face_count);
        println!("  → {}", usda_path.display());
        println!("  → {}\n", usdz_path.display());
    }
}

/// Write a 4×4 transform matrix as xformOp:transform
fn write_matrix(out: &mut String, mat: &[[f32; 4]; 4], indent: usize) {
    let pad = " ".repeat(indent);
    writeln!(out, "{}matrix4d xformOp:transform = (", pad).unwrap();
    for row in 0..4 {
        let comma = if row < 3 { "," } else { "" };
        writeln!(out, "{}    ({:.6}, {:.6}, {:.6}, {:.6}){}",
            pad, mat[row][0], mat[row][1], mat[row][2], mat[row][3], comma).unwrap();
    }
    writeln!(out, "{})", pad).unwrap();
    writeln!(out, "{}uniform token[] xformOpOrder = [\"xformOp:transform\"]", pad).unwrap();
}
