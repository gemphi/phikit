use super::*;

#[test]
fn each_property_carries_its_own_hard_range() {
    // The one unbounded property: a driver may make a lamp negative.
    let (lo, hi) = BLightProp::Energy.range();
    assert!(lo < -1e30 && hi > 1e30, "energy is unbounded, not 0..1");
    // The one with a non-zero minimum.
    assert_eq!(BLightProp::SpotSize.range().0, SPOT_SIZE_MIN);
    assert_eq!(BLightProp::SpotSize.range().1, std::f32::consts::PI);
    assert_eq!(BLightProp::SpotBlend.range(), (0.0, 1.0));
    assert_eq!(BLightProp::SunAngle.range(), (0.0, std::f32::consts::PI));
    for prop in [BLightProp::Color(0), BLightProp::Radius, BLightProp::AreaSize, BLightProp::AreaSizeY] {
        assert_eq!(prop.range(), (0.0, f32::MAX), "{prop:?} clamps at 0 and is free above");
    }
}

#[test]
fn a_path_names_the_property_it_writes() {
    assert_eq!(BLightProp::of("energy", 0), Some(BLightProp::Energy));
    assert_eq!(BLightProp::of("color", 2), Some(BLightProp::Color(2)));
    assert_eq!(BLightProp::of("color", 3), None, "a light's colour is three components");
    assert_eq!(BLightProp::of("shadow_soft_size", 0), Some(BLightProp::Radius));
    assert_eq!(BLightProp::of("angle", 0), Some(BLightProp::SunAngle));
    assert_eq!(BLightProp::of("use_shadow", 0), None, "not a float this models");
}

#[test]
fn driving_a_spot_closed_leaves_it_one_degree_open() {
    let (lo, _) = BLightProp::SpotSize.range();
    assert_eq!((-2.0f32).clamp(lo, std::f32::consts::PI), SPOT_SIZE_MIN);
    assert_eq!(0.0f32.clamp(lo, std::f32::consts::PI), SPOT_SIZE_MIN);
}
