//! Principled BSDF inputs must match Blender. `truth/bsdf.txt` was produced
//! by Blender 5.0.1 (`truth/blender_bsdf.py`, which folds linked inputs the
//! same way, with Blender evaluating the colour ramps); materials reached
//! through the opened scene's objects are compared. Missing files are skipped,
//! and so is a material whose Material Output is linked to another shader
//! than its Principled (a Glossy BSDF, a Mix Shader): since batch 140 its
//! `bsdf` is that surface folded (`surface_truth.rs`), where the script read
//! the Principled node it leaves unlinked.

use phiphx::blend::BlendFile;
use std::collections::BTreeMap;
use std::path::PathBuf;

fn f4(v: &[f32]) -> String {
    v.iter().map(|x| format!("{x:.4}")).collect::<Vec<_>>().join(",")
}

#[test]
fn principled_inputs_match_blender() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let assets = root.join("../phiber/assets/production");
    let files = [
        root.join("tests/fixtures/MATERIALS-BSDF.blend"),
        root.join("tests/fixtures/NORMAL-MAP.blend"),
        root.join("tests/fixtures/TEXTURED-PACKED.blend"),
        root.join("tests/fixtures/THYMUS-BOLDROID-WAVE-003.blend"),
        root.join("tests/fixtures/FLORIS_BOLDRIOD_RIG.blend"),
        assets.join("Mannequins-Female--Pose-05.blend"),
        assets.join("LAND-TYPE-15-01.blend"),
    ];
    let truth: BTreeMap<(String, String), String> = include_str!("truth/bsdf.txt")
        .lines()
        .filter_map(|l| {
            let mut it = l.splitn(3, '|');
            Some(((it.next()?.to_string(), it.next()?.to_string()), it.next()?.to_string()))
        })
        .collect();
    let (mut checked, mut skipped) = (0, 0);
    for path in files.iter().filter(|p| p.exists()) {
        let file = path.file_name().unwrap().to_string_lossy().to_string();
        let f = BlendFile::parse_bytes(&std::fs::read(path).unwrap()).unwrap();
        let scene = f.extract_scene().unwrap();
        let mut seen = BTreeMap::new();
        for m in scene.objects.iter().flat_map(|o| &o.materials) {
            // The script read the tree's first Principled; the output's
            // Surface link, when there is one, must come from that node for
            // the script's numbers to be the surface's.
            let output = m.graph.nodes.iter().filter(|n| n.idname.contains("OutputMaterial")).max_by_key(|n| n.flag & (1 << 6));
            let surface = output.and_then(|o| m.graph.links.iter().find(|l| l.to == o.name && l.to_socket == "Surface"));
            let first = m.graph.nodes.iter().find(|n| n.idname.contains("Principled")).map(|n| n.name.as_str());
            if surface.is_some_and(|l| Some(l.from.as_str()) != first) {
                skipped += 1;
                continue;
            }
            seen.insert(m.name.clone(), m.bsdf);
        }
        for (name, b) in seen {
            let Some(want) = truth.get(&(file.clone(), name.clone())) else { continue };
            let got = [
                f4(&b.base), f4(&[b.roughness]), f4(&[b.metallic]), f4(&[b.alpha]), f4(&[b.ior]),
                f4(&[b.transmission]), f4(&b.emission), f4(&[b.emission_strength]),
            ]
            .join("|");
            let nums = |t: &str| t.split(['|', ',']).map(|x| x.parse::<f32>().unwrap()).collect::<Vec<_>>();
            let close = nums(&got).iter().zip(nums(want)).all(|(a, b)| (a - b).abs() <= 2e-4);
            assert!(close, "{file} / {name}\n got  {got}\n want {want}");
            checked += 1;
        }
    }
    assert!(checked >= 5, "compared {checked} materials");
    eprintln!("{checked} materials match Blender, {skipped} surfaces are other shaders");
}
