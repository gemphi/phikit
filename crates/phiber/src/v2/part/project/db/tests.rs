use super::*;

/// The demo court as a project: every kind it has is a folder, a
/// material knows who draws with it and what image it reads, and a
/// mesh shared by many objects is one asset.
#[test]
fn the_demo_is_a_project_of_linked_assets() {
    let model = crate::v2::scene::play::demo().expect("the embedded demo");
    let db = BProjectDb::of(&model);
    for kind in [BAssetKind::Scene, BAssetKind::Object, BAssetKind::Mesh, BAssetKind::Material, BAssetKind::Image, BAssetKind::Camera, BAssetKind::Light, BAssetKind::Collection] {
        assert!(!db.assets(kind).is_empty(), "{kind:?}");
    }
    assert_eq!(db.assets(BAssetKind::Object).len(), model.objects.len());
    let skin = db.find(BAssetKind::Material, "Skin").expect("the players' skin");
    let users: Vec<String> = db.used_by(skin).iter().filter_map(|&w| db.asset(w)).map(|a| a.name.clone()).collect();
    assert!(users.iter().any(|n| n == "Ayanda Body"), "{users:?}");
    let court = db.find(BAssetKind::Object, "Court").expect("the court");
    let images: Vec<Word> = db.uses_of(court).into_iter().flat_map(|m| db.uses_of(m)).collect();
    assert!(images.iter().any(|&w| db.asset(w).is_some_and(|a| a.kind == BAssetKind::Image)), "the court's material reads its image");
}
