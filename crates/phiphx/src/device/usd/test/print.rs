//! The USDA printer.
use crate::device::usd::write::UsdaPrinter;
use crate::device::usd::{UsdPrim, UsdStage, UsdValue};
use crate::device::usd::stage::UsdUpAxis;


#[test]
fn prints_empty_stage() {
    let stage = UsdStage::new();
    let output = UsdaPrinter::print(&stage);
    assert!(output.starts_with("#usda 1.0\n"));
}

#[test]
fn prints_stage_metadata() {
    let mut stage = UsdStage::new();
    stage.default_prim = Some("Root".to_string());
    stage.up_axis = UsdUpAxis::Z;
    stage.meters_per_unit = 0.01;
    let output = UsdaPrinter::print(&stage);
    assert!(output.contains("defaultPrim = \"Root\""));
    assert!(output.contains("upAxis = \"Z\""));
    assert!(output.contains("metersPerUnit = 0.01"));
}

#[test]
fn prints_prim_with_attributes() {
    let mut stage = UsdStage::new();
    let mut prim = UsdPrim::new("/Root", "Xform");
    prim.set_attribute("visibility", UsdValue::Token("inherited".to_string()));
    stage.root.add_child(prim);
    let output = UsdaPrinter::print(&stage);
    assert!(output.contains("def Xform \"Root\""));
    assert!(output.contains("token visibility = \"inherited\""));
}

#[test]
fn prints_nested_prims() {
    let mut stage = UsdStage::new();
    let mut root = UsdPrim::new("/Root", "Xform");
    let child = UsdPrim::new("/Root/Child", "Mesh");
    root.add_child(child);
    stage.root.add_child(root);
    let output = UsdaPrinter::print(&stage);
    assert!(output.contains("def Xform \"Root\""));
    assert!(output.contains("    def Mesh \"Child\""));
}

#[test]
fn prints_all_value_types() {
    let mut stage = UsdStage::new();
    let mut prim = UsdPrim::new("/Test", "Scope");
    prim.set_attribute("flag", UsdValue::Bool(true));
    prim.set_attribute("count", UsdValue::Int(42));
    prim.set_attribute("scale", UsdValue::Float(2.5));
    prim.set_attribute("kind", UsdValue::Token("component".to_string()));
    prim.set_attribute("label", UsdValue::String("hello".to_string()));
    prim.set_attribute("pos", UsdValue::Float3([1.0, 2.0, 3.0]));
    prim.set_attribute("points", UsdValue::Float3Array(vec![[0.0, 1.0, 2.0]]));
    prim.set_attribute("uvs", UsdValue::Float2Array(vec![[0.5, 0.5]]));
    prim.set_attribute("indices", UsdValue::IntArray(vec![0, 1, 2]));
    stage.root.add_child(prim);
    let output = UsdaPrinter::print(&stage);
    assert!(output.contains("bool flag = true"));
    assert!(output.contains("int count = 42"));
    assert!(output.contains("double scale = 2.5"));
    assert!(output.contains("token kind = \"component\""));
    assert!(output.contains("string label = \"hello\""));
    assert!(output.contains("float3 pos = (1.0, 2.0, 3.0)"));
    assert!(output.contains("point3f[] points = [(0.0, 1.0, 2.0)]"));
    assert!(output.contains("texCoord2f[] uvs = [(0.5, 0.5)]"));
    assert!(output.contains("int[] indices = [0, 1, 2]"));
}

#[test]
fn matrix_prints_and_reads_back() {
    use crate::device::usd::UsdaParser;
    let mut m = [0.0f64; 16];
    for i in 0..4 {
        m[i * 5] = 1.0;
    }
    m[12] = 1.5;
    m[13] = -2.0;
    let mut stage = UsdStage::new();
    let mut prim = UsdPrim::new("/T", "Xform");
    prim.set_attribute("xformOp:transform", UsdValue::Matrix(m));
    stage.root.add_child(prim);
    let text = UsdaPrinter::print(&stage);
    assert!(text.contains("matrix4d xformOp:transform = ((1.0, 0.0, 0.0, 0.0)"));
    let back = UsdaParser::parse(&text).unwrap();
    assert_eq!(back.prim_at_path("/T").unwrap().attribute("xformOp:transform"), Some(&UsdValue::Matrix(m)));
}
