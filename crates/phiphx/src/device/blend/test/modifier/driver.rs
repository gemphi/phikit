//! The two rules a modifier setting adds: truncation toward zero for an
//! integer property, and a table keyed by modifier *type* because the
//! property name alone is ambiguous.

use super::*;

#[test]
fn an_integer_setting_truncates_toward_zero_and_does_not_round_or_floor() {
    let int = BModKind::Int { range: (-32768, 32767) };
    // Positive: never rounds up.
    assert_eq!(int.take(0.2), BModValue::Int(0));
    assert_eq!(int.take(0.5), BModValue::Int(0), "a half does not round up");
    assert_eq!(int.take(0.7), BModValue::Int(0), "nor does seven tenths");
    assert_eq!(int.take(2.5), BModValue::Int(2));
    assert_eq!(int.take(11.9), BModValue::Int(11));
    // Negative: toward zero, which is where floor would differ.
    assert_eq!(int.take(-0.7), BModValue::Int(0), "floor would give -1");
    assert_eq!(int.take(-1.5), BModValue::Int(-1), "floor would give -2");
    assert_eq!(int.take(-2.5), BModValue::Int(-2), "floor would give -3");
}

#[test]
fn an_integer_setting_clamps_after_truncating_and_its_minimum_is_not_always_zero() {
    let levels = setting("SUBSURF", "levels", 0).expect("levels");
    assert_eq!(levels.kind.take(-3.0), BModValue::Int(0));
    assert_eq!(levels.kind.take(99.0), BModValue::Int(11), "eleven levels is the most Blender allows");

    let quality = setting("SUBSURF", "quality", 0).expect("quality");
    assert_eq!(quality.kind.take(0.5), BModValue::Int(1), "quality starts at one, not zero");
    assert_eq!(quality.kind.take(-5.0), BModValue::Int(1));

    let min_vertices = setting("TRIANGULATE", "min_vertices", 0).expect("min_vertices");
    assert_eq!(min_vertices.kind.take(0.0), BModValue::Int(4), "a face needs four corners");
    assert_eq!(min_vertices.kind.take(11.9), BModValue::Int(11));
}

#[test]
fn a_float_setting_keeps_what_it_is_given_unless_its_range_says_otherwise() {
    let strength = setting("DISPLACE", "strength", 0).expect("strength");
    assert_eq!(strength.kind.take(-0.7), BModValue::Float(-0.7), "unbounded both ways");
    assert_eq!(strength.kind.take(1e6), BModValue::Float(1e6));

    let threshold = setting("WELD", "merge_threshold", 0).expect("merge_threshold");
    assert_eq!(threshold.kind.take(-0.7), BModValue::Float(0.0), "a distance cannot be negative");
    assert_eq!(threshold.kind.take(2.5), BModValue::Float(2.5));
}

#[test]
fn the_same_property_name_means_different_things_on_different_modifiers() {
    // `strength` is unbounded on Displace and on Lattice …
    assert_eq!(setting("DISPLACE", "strength", 0).map(|s| s.kind), Some(BModKind::Float { range: (f32::MIN, f32::MAX) }));
    assert_eq!(setting("LATTICE", "strength", 0).map(|s| s.kind), Some(BModKind::Float { range: (f32::MIN, f32::MAX) }));
    // … and `merge_threshold` is a distance on Weld and on Mirror, while
    // `offset` is a free float on Solidify and has no entry on Mirror.
    assert!(setting("MIRROR", "merge_threshold", 0).is_some());
    assert!(setting("SOLIDIFY", "offset", 0).is_some());
    assert!(setting("MIRROR", "offset", 0).is_none(), "Mirror has no `offset`");
    // A type with no table, and a property that is not drivable here.
    assert!(setting("ARMATURE", "strength", 0).is_none());
    assert!(setting("SUBSURF", "use_creases", 0).is_none(), "a boolean is not modelled");
}

#[test]
fn a_vector_setting_takes_its_components_and_no_more() {
    for c in 0..3 {
        assert!(setting("ARRAY", "constant_offset_displace", c).is_some(), "component {c}");
    }
    assert!(setting("ARRAY", "constant_offset_displace", 3).is_none(), "three components, not four");
    assert!(setting("ARRAY", "count", 1).is_none(), "`count` is a scalar");
}

#[test]
fn a_value_reads_back_as_a_float_whichever_it_is() {
    assert_eq!(BModValue::Int(-2).as_f32(), -2.0);
    assert_eq!(BModValue::Float(0.25).as_f32(), 0.25);
}
