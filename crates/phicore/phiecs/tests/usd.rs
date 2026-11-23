//! The USD text stage: a layer read into prims, references composed,
//! and the prims put into the world with their poses and components.

use std::path::PathBuf;

use phiecs::comp::{Placed, children, place};
use phiecs::prelude::*;
use phiecs::usd::{ComponentSet, Kind, Layer, Stage, Value, intake};

/// A planter as Reality Composer Pro writes one: a pose, a plant
/// component, a collider with a shape inside it.
const PLANTER: &str = r#"#usda 1.0
(
    defaultPrim = "Root"
    metersPerUnit = 1
    upAxis = "Y"
)

def Xform "Root"
{
    reorder nameChildren = ["pot", "MI_soil"]
    def Xform "pot" (
        active = true
        prepend apiSchemas = ["MaterialBindingAPI"]
        customData = {
            float3 rotationEulerHint = (0, 1.6471374, 0)
        }
    )
    {
        rel material:binding = </Root/MI_soil> (
            bindMaterialAs = "weakerThanDescendants"
        )
        float3 xformOp:translate = (0, 0.5, 0)
        uniform token[] xformOpOrder = ["xformOp:translate", "xformOp:orient", "xformOp:scale"]
    }
}
"#;

const GREENHOUSE: &str = r#"#usda 1.0
(
    defaultPrim = "Root"
)

def Xform "Root"
{
    float3 xformOp:scale = (0.05, 0.05, 0.05)

    def "planter" (
        active = true
        references = @planter.usda@
    )
    {
        float3 xformOp:translate = (-3.5, 0, -0.7)
        quatf xformOp:orient = (1, 0, 0, 0)

        over "pot"
        {
            double size = 2
        }

        def RealityKitCustomComponent "PlantComponent"
        {
            uniform token info:id = "Garden.PlantComponent"
            bool interactedWith = 0
            token plantType = "poppy"
        }

        def RealityKitComponent "Collider"
        {
            uniform token info:id = "RealityKit.Collider"
            def RealityKitStruct "Shape"
            {
                float3 extent = (1.6, 1.8, 1.6)
                token shapeType = "Box"
            }
        }
    }

    def "gone" (
        active = false
    )
    {
    }

    def "mesh" (
        references = @robot.usdz@
    )
    {
    }
}
"#;

#[test]
fn a_layer_reads_into_prims_with_their_metadata_and_properties() {
    let layer = Layer::read(PLANTER);
    assert_eq!(layer.default_prim(), "Root");
    let pot = layer.find("/Root/pot").expect("the pot");
    assert_eq!((pot.kind.as_str(), pot.meta("active")), ("Xform", Some(&Value::Bool(true))));
    assert_eq!(pot.get("xformOp:translate").map(Value::numbers), Some(vec![0.0, 0.5, 0.0]));
    assert_eq!(pot.get("material:binding"), Some(&Value::Path("/Root/MI_soil".to_string())));
    match pot.meta("customData") {
        Some(Value::Dict(entry)) => assert_eq!(entry[0].0, "rotationEulerHint"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_stage_composes_references_and_puts_prims_into_the_world() {
    let dir = std::env::temp_dir().join(format!("phiecs-usd-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("planter.usda"), PLANTER).unwrap();
    std::fs::write(dir.join("greenhouse.usda"), GREENHOUSE).unwrap();
    let stage = Stage::open(&dir.join("greenhouse.usda")).expect("the stage");
    let mut world = World::new();
    let made = intake(&mut world, &stage, None);
    // The planter took the referenced root's pot, and its own opinion
    // on the pot stands over it; an inactive prim is not made.
    let planter = world.lexicon.get("/Root/planter").expect("the planter");
    let pot = world.lexicon.get("/Root/planter/pot").expect("the referenced pot");
    assert!(world.lexicon.get("/Root/gone").is_none() && made.contains(&pot));
    assert_eq!(world.read::<Kind>(pot).and_then(|k| k.get("size")).map(Value::numbers), Some(vec![2.0]));
    assert_eq!(children(&world, planter), [pot]);
    // Its components are data on it, not things of their own.
    let set = world.read::<ComponentSet>(planter).expect("its components");
    assert_eq!(set.find("Garden.PlantComponent").and_then(|c| c.get("plantType")).map(Value::text), Some("poppy"));
    assert_eq!(set.find("RealityKit.Collider").and_then(|c| c.get("Shape.extent")).map(Value::numbers), Some(vec![1.6, 1.8, 1.6]));
    // Where it stands: the root's size, then its own move.
    place(&mut world);
    let at = world.read::<Placed>(pot).map(|p| p.0.origin()).unwrap();
    assert!((at[0] - -0.175).abs() < 1e-5 && (at[1] - 0.025).abs() < 1e-5 && (at[2] - -0.035).abs() < 1e-5, "{at:?}");
    // A binary asset is kept for whoever draws meshes, not opened.
    let mesh = world.lexicon.get("/Root/mesh").expect("the mesh");
    let asset = world.read::<Kind>(mesh).and_then(|k| k.get(phiecs::usd::stage::ASSET)).map(Value::text).unwrap_or_default().to_string();
    assert!(asset.ends_with("robot.usdz"), "{asset}");
    _ = std::fs::remove_dir_all(&dir);
}

/// Apple's own greenhouse, where it is on this disk: its three planters
/// come in with their plant types, as the BOT-anist sample authors them.
#[test]
fn apples_greenhouse_comes_in_with_its_three_planters() {
    let file: PathBuf = [env!("CARGO_MANIFEST_DIR"), "..", "..", "..", "..", "phi-old", "apple-old", "crates", "apple", "apps", "botanist", "BOTAnist", "BOTAnist", "Packages", "BOTanistAssets", "Sources", "BOTanistAssets", "BOTanistAssets.rkassets", "scenes", "volume.usda"].iter().collect();
    let Some(stage) = Stage::open(&file) else {
        eprintln!("not on this disk: {}", file.display());
        return;
    };
    let mut world = World::new();
    intake(&mut world, &stage, None);
    let mut plant: Vec<String> = world.each::<ComponentSet>().filter_map(|(_, set)| set.find("BOTanistAssets.PlantComponent")).filter_map(|c| c.get("plantType")).map(|v| v.text().to_string()).collect();
    plant.sort();
    assert_eq!(plant, ["coffeeBerry", "poppy", "yucca"]);
}
