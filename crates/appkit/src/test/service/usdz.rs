//! A USD file as one package: what goes in, in what order, where each
//! file's bytes start, the names in its layers, and its root turned.

use std::path::PathBuf;

use crate::service::{usda, usdz};

/// The files of a zip as it keeps them: each name, where its bytes start,
/// and its bytes.
fn read(zip: &[u8]) -> Vec<(String, usize, Vec<u8>)> {
    let word = |at: usize| u16::from_le_bytes([zip[at], zip[at + 1]]) as usize;
    let long = |at: usize| u32::from_le_bytes([zip[at], zip[at + 1], zip[at + 2], zip[at + 3]]) as usize;
    let mut out = Vec::new();
    let mut at = 0;
    while long(at) == 0x0403_4b50 {
        let (size, name, extra) = (long(at + 22), word(at + 26), word(at + 28));
        let start = at + 30 + name + extra;
        out.push((String::from_utf8_lossy(&zip[at + 30..at + 30 + name]).to_string(), start, zip[start..start + size].to_vec()));
        at = start + size;
    }
    out
}

/// A folder of its own for one test, empty.
fn folder(name: &str) -> PathBuf {
    let folder = std::env::temp_dir().join(format!("appkit-usdz-{name}"));
    _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir_all(folder.join("part")).expect("a folder");
    folder
}

const ROOT: &str = "#usda 1.0\n(\n    defaultPrim = \"couch\"\n)\n\ndef Xform \"couch\"\n{\n    def Xform \"leg\" (\n        prepend references = @./part/leg.usda@\n    )\n    {\n    }\n}\n";

#[test]
fn a_package_holds_the_layer_first_and_what_it_names_side_by_side() {
    let folder = folder("holds");
    std::fs::write(folder.join("couch.usda"), ROOT).unwrap();
    std::fs::write(folder.join("part/leg.usda"), "#usda 1.0\ndef Mesh \"leg\"\n{\n    asset inputs:file = @../wood.png@\n}\n").unwrap();
    std::fs::write(folder.join("wood.png"), [1u8, 2, 3]).unwrap();
    let file = read(&usdz::package(&folder.join("couch.usda"), 0.0).expect("packed"));
    let name: Vec<&str> = file.iter().map(|(n, _, _)| n.as_str()).collect();
    assert_eq!(name, ["couch.usda", "leg.usda", "wood.png"]);
    // Every file's bytes on a 64-byte boundary, as Quick Look reads them.
    assert!(file.iter().all(|(_, start, _)| start % 64 == 0), "{file:?}");
    // The names in the layers are the package's own.
    let root = String::from_utf8_lossy(&file[0].2).to_string();
    let leg = String::from_utf8_lossy(&file[1].2).to_string();
    assert!(root.contains("@./leg.usda@") && leg.contains("@./wood.png@"), "{root}\n{leg}");
    assert_eq!(file[2].2, [1, 2, 3]);
}

#[test]
fn a_root_turned_faces_where_it_was_seen() {
    let text = usda::turned(ROOT, std::f32::consts::FRAC_PI_2);
    // A quarter turn: what faced +x (the eye at the right) now faces +z.
    assert!(text.contains("matrix4d xformOp:transform:turn = ( (") && text.contains(", 0, 1, 0), (0, 1, 0, 0), (-1, 0, "), "{text}");
    assert!(text.contains("uniform token[] xformOpOrder = [\"xformOp:transform:turn\"]"), "{text}");
    // Its own transforms are kept, the turn taken after them.
    let own = ROOT.replace("def Xform \"couch\"\n{\n", "def Xform \"couch\"\n{\n    matrix4d xformOp:transform = ( (1, 0, 0, 0), (0, 1, 0, 0), (0, 0, 1, 0), (0, 0, 0, 1) )\n    uniform token[] xformOpOrder = [\"xformOp:transform\"]\n");
    let text = usda::turned(&own, 1.0);
    assert!(text.contains("xformOpOrder = [\"xformOp:transform:turn\", \"xformOp:transform\"]"), "{text}");
    // A child's order is the child's.
    assert_eq!(text.matches("xformOp:transform:turn =").count(), 1);
}

#[test]
fn a_zip_keeps_each_file_whole_with_its_check() {
    let zip = usdz::zip(&[("a.usda".to_string(), b"#usda 1.0\n".to_vec()), ("b.png".to_string(), vec![0; 100])]);
    let file = read(&zip);
    assert_eq!(file.len(), 2);
    assert_eq!(file[0].2, b"#usda 1.0\n");
    // The end of the zip says two files.
    let end = zip.len() - 22;
    assert_eq!(&zip[end..end + 4], &0x0605_4b50u32.to_le_bytes());
    assert_eq!(u16::from_le_bytes([zip[end + 10], zip[end + 11]]), 2);
    // The check of "123456789" is the one every zip uses.
    let check = usdz::zip(&[("c".to_string(), b"123456789".to_vec())]);
    assert_eq!(u32::from_le_bytes([check[14], check[15], check[16], check[17]]), 0xCBF4_3926);
}
