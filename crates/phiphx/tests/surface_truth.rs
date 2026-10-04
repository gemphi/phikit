//! A material's surface is what its Material Output is linked to
//! (`shade/surface.rs`), not the first Principled node in the tree: a
//! Glossy BSDF on the output folds to a metal, a Mix Shader keyed on Light
//! Path ▸ Is Shadow Ray folds to what the camera sees, and a Principled
//! left unlinked beside a Glass node does not shade the glass — nor the
//! other way round. The production files are checked when present.

use phiphx::blend::BlendFile;
use phiphx::scene::SceneModel;
use std::path::PathBuf;

fn open(rel: &str) -> Option<SceneModel> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = [root.join("tests/fixtures").join(rel), root.join("../phiber/assets/production").join(rel)].into_iter().find(|p| p.exists())?;
    let scene = BlendFile::parse_bytes(&std::fs::read(&path).unwrap()).unwrap().extract_scene().unwrap();
    Some(SceneModel::from_blend_owned(scene, &path.to_string_lossy()))
}

fn material<'a>(m: &'a SceneModel, name: &str) -> &'a phiphx::scene::SceneMaterial {
    m.objects.iter().flat_map(|o| &o.materials).find(|ma| ma.name == name).unwrap_or_else(|| panic!("{name}"))
}

#[test]
fn the_output_decides_which_shader_is_the_surface() {
    // THYMUS: `mat_glass` holds a Principled and a Glass BSDF; the
    // Principled is the one on the output, with its own transmission.
    let thymus = open("THYMUS-BOLDROID-WAVE-003.blend").expect("the fixture");
    let glass = &material(&thymus, "mat_glass").bsdf;
    assert_eq!(glass.transmission, 1.0);
    assert!((glass.roughness - 0.235).abs() < 1e-3 && (glass.ior - 1.45).abs() < 1e-3, "the Principled's own numbers: {glass:?}");
    assert!((glass.base[0] - 0.8).abs() < 1e-3, "not the Glass node's white");
    assert!(material(&thymus, "mat_glass").mix.is_none(), "one closure, no lobes");

    let Some(land) = open("LAND-TYPE-15-01.blend") else { return };
    // A Glossy BSDF on the output: a metal at its roughness, the Principled
    // beside it (metallic 1, IOR 1.45) left out — the IOR is the default.
    let window = &material(&land, "HB04_Glass_Shader").bsdf;
    assert_eq!((window.metallic, window.roughness, window.transmission), (1.0, 0.0, 0.0));
    assert_eq!(window.ior, 1.5);

    let Some(city) = open("FLOWER-CITY-BUILDINGS-DRIJKE-BLUE-CAFFEINE-NEW-CITY.blend") else { return };
    // Mix Shader(Fac = Light Path ▸ Is Shadow Ray; Glass, Transparent): the
    // fake-caustics setup — the camera sees the glass, water at 1.33.
    let water = material(&city, "Material");
    assert_eq!((water.bsdf.transmission, water.bsdf.alpha, water.bsdf.roughness), (1.0, 1.0, 0.0));
    assert!((water.bsdf.ior - 1.33).abs() < 1e-3, "{}", water.bsdf.ior);
    // As two lobes: the glass and the transparent, the weight 0 — the
    // camera's — so the tracer shades the glass alone.
    let lobes = water.mix.as_ref().expect("a Mix Shader on the output");
    assert_eq!((lobes.first.transmission, lobes.second.alpha, lobes.fac, lobes.principled), (1.0, 0.0, 0.0, None));
    // A Principled on the output with a Glass node idle beside it.
    let mirror = &material(&city, "mat_mirror").bsdf;
    assert_eq!((mirror.metallic, mirror.transmission), (1.0, 0.0));
    // Mix Shader(Voronoi through a ramp; Diffuse, Principled): the lerp of
    // the two, and the Principled reached through the mix is the one whose
    // base image is read.
    let moss = material(&city, "MOSS-GRASS-TEXTURE");
    assert!(moss.base_color_texture.is_some(), "the texture through the Mix Shader");
    assert!(moss.bsdf.base[0] < 0.2 && moss.bsdf.roughness > 0.5, "{:?}", moss.bsdf);
    // Its two lobes, the Principled's side the one the texture belongs to,
    // the factor the ramp's value at the Voronoi's mean — between the two.
    let lobes = moss.mix.as_ref().expect("a Mix Shader on the output");
    assert!(lobes.principled.is_some(), "one side is the Principled: {lobes:?}");
    assert!(lobes.fac > 0.05 && lobes.fac < 0.95, "{}", lobes.fac);
}
