//! Normal maps (Principled `Normal` ← Normal Map ← Image Texture) must match
//! Blender: `truth/normal_map.txt` from `truth/blender_normal.py` (5.0.1).

use phiphx::phx::Phx;
use std::path::PathBuf;

#[test]
fn normal_maps_match_blender() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut checked = 0;
    for line in include_str!("truth/normal_map.txt").lines() {
        let f: Vec<&str> = line.split('|').collect();
        let path = root.join(f[0]);
        if !path.exists() {
            continue;
        }
        let model = Phx::open(&path).expect("open");
        let m = model.objects.iter().flat_map(|o| &o.materials).find(|m| m.name == f[1]).expect("material");
        let (key, strength) = m.normal_texture.clone().expect("normal map found");
        assert!(key.contains(f[2]), "image key {key} names {}", f[2]);
        assert!((strength - f[3].parse::<f32>().unwrap()).abs() < 1e-4, "{} strength {strength}", f[1]);
        let tex = m.normal_map.as_ref().expect("packed normal map decoded");
        assert_eq!((tex.width, tex.height), (64, 64));
        checked += 1;
    }
    assert!(checked >= 1);
    // A material without a Normal link has none.
    let model = Phx::open(&root.join("NORMAL-MAP.blend")).unwrap();
    let flat = model.objects.iter().flat_map(|o| &o.materials).find(|m| m.name == "Flat").unwrap();
    assert!(flat.normal_texture.is_none());
    // The normal map image is not a colour texture (Base Color is unlinked).
    let bumpy = model.objects.iter().flat_map(|o| &o.materials).find(|m| m.name == "Bumpy").unwrap();
    assert!(bumpy.base_color_texture.is_none(), "{:?}", bumpy.base_color_texture);
    // Blender: only TEXTURED-PACKED links images to Base Color (keys are the image paths).
    let model = Phx::open(&root.join("TEXTURED-PACKED.blend")).unwrap();
    for (name, image) in [("External", "magenta.png"), ("Packed", "quad_src.png")] {
        let m = model.objects.iter().flat_map(|o| &o.materials).find(|m| m.name == name).unwrap();
        let key = m.base_color_texture.clone().unwrap_or_default();
        assert!(key.contains(image), "{name}: {key}");
    }
}
