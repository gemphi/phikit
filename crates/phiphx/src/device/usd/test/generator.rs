//! Proptest generators for arbitrary `UsdStage` values.

use proptest::prelude::*;

use crate::device::usd::attribute::{UsdAttribute, UsdValue};
use crate::device::usd::prim::UsdPrim;
use crate::device::usd::stage::{UsdStage, UsdUpAxis};

/// Generate a valid USDA identifier (alphanumeric + underscore, starts with letter).
pub fn arb_identifier() -> impl Strategy<Value = String> {
    "[A-Za-z][A-Za-z0-9_]{0,7}".prop_map(|s| s)
}

/// Generate a token string (no quotes, no newlines, no backslashes).
pub fn arb_token_str() -> impl Strategy<Value = String> {
    "[A-Za-z][A-Za-z0-9_.:]{0,15}".prop_map(|s| s)
}

/// Generate an f32 that round-trips cleanly through text serialization.
fn arb_f32() -> impl Strategy<Value = f32> {
    (-1000i32..1000).prop_map(|i| i as f32 * 0.25)
}

/// Generate an f64 that round-trips cleanly through text serialization.
fn arb_f64() -> impl Strategy<Value = f64> {
    (-1000i32..1000).prop_map(|i| i as f64 * 0.01)
}

/// Generate an arbitrary `UsdValue`.
pub fn arb_value() -> impl Strategy<Value = UsdValue> {
    prop_oneof![
        any::<bool>().prop_map(UsdValue::Bool),
        (-1000i64..1000).prop_map(UsdValue::Int),
        arb_f64().prop_map(UsdValue::Float),
        arb_token_str().prop_map(UsdValue::Token),
        arb_token_str().prop_map(UsdValue::String),
        (arb_f32(), arb_f32(), arb_f32()).prop_map(|(a, b, c)| UsdValue::Float3([a, b, c])),
        prop::collection::vec((arb_f32(), arb_f32(), arb_f32()), 0..3)
            .prop_map(|v| UsdValue::Float3Array(v.into_iter().map(|(a, b, c)| [a, b, c]).collect())),
        prop::collection::vec((arb_f32(), arb_f32()), 0..3)
            .prop_map(|v| UsdValue::Float2Array(v.into_iter().map(|(a, b)| [a, b]).collect())),
        prop::collection::vec(-100i32..100, 0..4).prop_map(UsdValue::IntArray),
        prop::collection::vec(arb_token_str(), 0..4).prop_map(UsdValue::StringArray),
    ]
}

/// Generate an arbitrary `UsdAttribute` with a valid name.
pub fn arb_attribute() -> impl Strategy<Value = UsdAttribute> {
    (arb_identifier(), arb_value()).prop_map(|(name, value)| UsdAttribute::new(name, value))
}


/// Generate an arbitrary `UsdStage` with metadata and a small prim tree.
pub fn arb_stage() -> impl Strategy<Value = UsdStage> {
    let metadata = (
        prop::option::of(arb_identifier()),
        prop_oneof![Just(UsdUpAxis::Y), Just(UsdUpAxis::Z)],
        arb_f64(),
    );
    let prims = prop::collection::vec(arb_leaf_prim_with_children(), 0..3);

    (metadata, prims).prop_map(|((default_prim, up_axis, meters_per_unit), prims)| {
        let mut stage = UsdStage::new();
        stage.default_prim = default_prim;
        stage.up_axis = up_axis;
        stage.meters_per_unit = meters_per_unit;
        for prim in prims {
            stage.root.add_child(prim);
        }
        stage
    })
}

/// Generate a prim that may have one level of children.
fn arb_leaf_prim_with_children() -> impl Strategy<Value = UsdPrim> {
    (
        arb_identifier(),
        arb_identifier(),
        prop::collection::vec(arb_attribute(), 0..3),
        prop::collection::vec(
            (arb_identifier(), arb_identifier(), prop::collection::vec(arb_attribute(), 0..2)),
            0..2,
        ),
    )
        .prop_map(|(name, type_name, attrs, children_data)| {
            let path = format!("/{}", name);
            let mut prim = UsdPrim::new(&path, type_name);
            for attr in attrs {
                prim.set_attribute(attr.name, attr.value);
            }
            for (child_name, child_type, child_attrs) in children_data {
                let child_path = format!("{}/{}", path, child_name);
                let mut child = UsdPrim::new(child_path, child_type);
                for attr in child_attrs {
                    child.set_attribute(attr.name, attr.value);
                }
                prim.add_child(child);
            }
            prim
        })
}
