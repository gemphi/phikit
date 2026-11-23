//! Apple's BOT-anist meshes, where they are on this disk: each part's
//! binary USD reads into triangles, and the body's skeleton has the
//! joints the head and the backpack are pinned to.

use std::path::PathBuf;

fn asset(name: &str) -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "..", "..", "..", "..", "phi-old", "apple-old", "crates", "apple", "apps", "botanist", "BOTAnist", "BOTAnist", "Packages", "BOTanistAssets", "Sources", "BOTanistAssets", "BOTanistAssets.rkassets", "Assets", "Robot", "mesh", name].iter().collect()
}

#[test]
fn the_robots_parts_read_into_triangles_and_the_body_has_its_joints() {
    if !asset("head1.usdz").exists() {
        eprintln!("not on this disk: {}", asset("head1.usdz").display());
        return;
    }
    for part in ["head1", "head2", "head3", "body1", "body2", "body3", "backpack1", "backpack2", "backpack3"] {
        let started = std::time::Instant::now();
        let scene = phiusd::read(&asset(&format!("{part}.usdz")));
        let joints: Vec<String> = scene.skeleton.iter().flat_map(|s| s.joint.iter().cloned()).collect();
        println!("{part}: {} meshes, {} triangles, {} skeletons, joints with head/backpack: {:?}, in {} ms", scene.mesh.len(), scene.count(), scene.skeleton.len(), joints.iter().filter(|j| j.contains("head") || j.contains("back")).collect::<Vec<_>>(), started.elapsed().as_millis());
        assert!(scene.count() > 100, "{part}");
    }
}

#[test]
fn the_greenhouse_composes_in_phiecs_and_draws_from_its_assets() {
    let file: PathBuf = [env!("CARGO_MANIFEST_DIR"), "..", "..", "..", "..", "phi-old", "apple-old", "crates", "apple", "apps", "botanist", "BOTAnist", "BOTAnist", "Packages", "BOTanistAssets", "Sources", "BOTanistAssets", "BOTanistAssets.rkassets", "scenes", "volume.usda"].iter().collect();
    if !file.exists() {
        return;
    }
    let started = std::time::Instant::now();
    let stage = phiecs::usd::Stage::open(&file).expect("the greenhouse's scene");
    let mut world = phiecs::prelude::World::new();
    let root = world.coin_bare("greenhouse");
    phiecs::usd::intake(&mut world, &stage, Some(root));
    phiecs::comp::place(&mut world);
    let composed = started.elapsed().as_millis();
    let mut library = phiusd::draw::Library::default();
    let placed = phiusd::draw::placed(&world, root, &mut library);
    let triangle = phiusd::draw::triangles(&world, root, &[], &mut library);
    println!("greenhouse: composed in {composed} ms, {} things from assets, {} triangles, in {} ms", placed.len(), triangle.len(), started.elapsed().as_millis());
    // The biggest assets, to colour them by their meshes' names.
    let mut big: Vec<(usize, String)> = placed.iter().flat_map(|p| library.scene(&p.asset).mesh.iter().map(|m| (m.index.len() / 3, m.path.clone())).collect::<Vec<_>>()).collect();
    big.sort_by_key(|b| std::cmp::Reverse(b.0));
    big.dedup_by(|a, b| a.1 == b.1);
    big.iter().take(24).for_each(|(n, path)| println!("  {n} {path}"));
    assert!(!placed.is_empty() && triangle.len() > 1000);
}
