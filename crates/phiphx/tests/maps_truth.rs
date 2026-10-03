//! Images linked to Roughness / Metallic / Emission Color must match Blender:
//! `truth/shade_maps.txt` from `truth/blender_maps.py` (5.0.1).

use phiphx::phx::Phx;
use std::path::PathBuf;

/// The socket names of `SceneMaps::images()`, in order.
const SOCKETS: [&str; 3] = ["Roughness", "Metallic", "Emission Color"];

#[test]
fn shade_maps_match_blender() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut checked = 0;
    for line in include_str!("truth/shade_maps.txt").lines() {
        let f: Vec<&str> = line.split('|').collect();
        let path = root.join(f[0]);
        if !path.exists() {
            continue;
        }
        let model = Phx::open(&path).expect("open");
        let m = model.objects.iter().flat_map(|o| &o.materials).find(|m| m.name == f[1]).expect("material");
        let slot = SOCKETS.iter().position(|s| *s == f[2]).expect("known socket");
        let map = [&m.maps.rough, &m.maps.metal, &m.maps.emit][slot].as_ref().expect("map read");
        assert!(map.key.contains(f[3]), "{}: {} key {}", f[1], f[2], map.key);
        let image = map.image.as_ref().expect("packed image decoded");
        assert_eq!((image.width, image.height), (8, 8));
        checked += 1;
    }
    assert_eq!(checked, 3, "one map per socket");

    // A material with nothing linked has no maps at all.
    let model = Phx::open(&root.join("SHADE-MAPS.blend")).expect("open");
    let mats = |name: &str| {
        model.objects.iter().flat_map(|o| &o.materials).find(|m| m.name == name).expect("material").clone()
    };
    let plain = mats("Plain");
    assert!(plain.maps.images().iter().all(Option::is_none), "no maps on the plain material");
    assert!(plain.base_color_texture.is_none(), "a map is not a base-colour image");
    // Each mapped material carries exactly its own map.
    for (name, slot) in [("Rough", 0), ("Metal", 1), ("Glow", 2)] {
        let m = mats(name);
        let set: Vec<usize> = m.maps.images().iter().enumerate().filter(|(_, i)| i.is_some()).map(|(k, _)| k).collect();
        assert_eq!(set, vec![slot], "{name} maps only {}", SOCKETS[slot]);
        assert!(m.base_color_texture.is_none(), "{name}: base colour stays unlinked");
    }
}
