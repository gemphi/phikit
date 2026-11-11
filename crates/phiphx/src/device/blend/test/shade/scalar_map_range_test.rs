use super::*;

/// 0.3 of 0..1 onto 0.2..0.8 is 0.5; stepped at 4 steps floors to
/// quarters; smoothstep eases; clamp holds an out-of-range value to
/// the target range, whichever way round it is.
#[test]
fn map_range_follows_svm_node_map_range() {
    assert!((map_range(0, 0.3, 0.0, 1.0, 0.2, 0.8, 4.0, false) - 0.38).abs() < 1e-6);
    assert!((map_range(1, 0.3, 0.0, 1.0, 0.0, 1.0, 4.0, false) - 0.25).abs() < 1e-6, "stepped");
    assert!((map_range(2, 0.25, 0.0, 1.0, 0.0, 1.0, 4.0, false) - 0.15625).abs() < 1e-6, "smoothstep");
    assert!((map_range(3, 0.5, 0.0, 1.0, 0.0, 1.0, 4.0, false) - 0.5).abs() < 1e-6, "smootherstep");
    assert_eq!(map_range(0, 2.0, 0.0, 1.0, 0.0, 1.0, 4.0, true), 1.0, "clamped");
    assert_eq!(map_range(0, 2.0, 0.0, 1.0, 0.0, 1.0, 4.0, false), 2.0, "unclamped");
    assert_eq!(map_range(0, 2.0, 0.0, 1.0, 1.0, 0.0, 4.0, true), 0.0, "clamped to a reversed target");
    assert_eq!(map_range(0, 0.5, 1.0, 1.0, 0.0, 1.0, 4.0, false), 0.0, "no source range");
    let op = BScalarOp::MapRange { ty: 0, from_min: 0.0, from_max: 1.0, to_min: 0.2, to_max: 0.8, steps: 4.0, clamp: true };
    assert!((op.apply(0.3) - 0.38).abs() < 1e-6);
    assert_eq!(BScalarOp::Math { op: 2, b: 3.0, c: 0.0, clamp: true }.apply(0.5), 1.0);
}
