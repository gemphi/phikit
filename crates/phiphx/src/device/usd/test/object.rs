//! The stage and its prims.
use crate::device::usd::UsdValue;
use crate::device::usd::prim::UsdPrim;
use crate::device::usd::stage::UsdStage;


#[test]
fn traverse_and_lookup() {
    let mut stage = UsdStage::new();
    let mut root = UsdPrim::new("/Root", "Xform");
    let mut mesh = UsdPrim::new("/Root/Box", "Mesh");
    mesh.set_attribute("points", UsdValue::Float3Array(vec![[0.0, 0.0, 0.0]]));
    root.add_child(mesh);
    stage.root.add_child(root);

    assert_eq!(stage.traverse().len(), 2);
    assert!(stage.prim_at_path("/Root/Box").is_some());
    assert_eq!(stage.meshes().len(), 1);
    assert_eq!(stage.prim_at_path("/Root/Box").unwrap().name(), "Box");
}
