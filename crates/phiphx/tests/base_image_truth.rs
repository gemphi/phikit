//! The colour texture of each material is the first image upstream of
//! Principled Base Color, as `truth/blender_base_image.py` (Blender 5.0.1) finds it.

use phiphx::blend::BlendFile;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[test]
fn base_color_images_match_blender() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let assets = root.join("../phiber/assets/production");
    let mut want: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    for l in include_str!("truth/base_image.txt").lines() {
        let f: Vec<&str> = l.splitn(3, '|').collect();
        want.entry(f[0].to_string()).or_default().insert(f[1].to_string(), f[2].to_string());
    }
    let mut checked = 0;
    for (file, mats) in &want {
        let path = [root.join("tests/fixtures").join(file), assets.join(file)].into_iter().find(|p| p.exists());
        let Some(path) = path else { continue };
        let scene = BlendFile::parse_bytes(&std::fs::read(&path).unwrap()).unwrap().extract_scene().unwrap();
        let mut got = BTreeMap::new();
        for m in scene.objects.iter().flat_map(|o| &o.materials) {
            got.insert(m.name.clone(), m.base_color_texture.clone().unwrap_or_default());
        }
        for (name, key) in &got {
            let Some(expect) = mats.get(name) else { continue };
            // Blender shows backslash separators as `/` on Linux; the key keeps the file's spelling.
            assert_eq!(&key.replace('\\', "/"), expect, "{file} / {name}");
            checked += 1;
        }
    }
    assert!(checked >= 5, "compared {checked}");
    eprintln!("{checked} base-colour images match Blender");
}
