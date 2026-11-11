use crate::device::blend::shade::surface::*;

#[test]
fn mixing_lerps_and_adding_sums() {
    let glass = BlendBsdf { base: [1.0, 1.0, 1.0, 1.0], roughness: 0.0, ior: 1.45, transmission: 1.0, ..Default::default() };
    let clear = BlendBsdf { alpha: 0.0, ..Default::default() };
    let half = clear.mixed(&glass, 0.5);
    assert_eq!((half.alpha, half.transmission, half.roughness), (0.5, 0.5, 0.25));
    let whole = clear.mixed(&glass, 1.0);
    assert_eq!((whole.transmission, whole.ior, whole.alpha, whole.emitted()), (1.0, 1.45, 1.0, [0.0; 3]), "factor 1 is the second shader (its emission as radiance)");
    let lamp = BlendBsdf { base: [0.0, 0.0, 0.0, 1.0], emission: [1.0, 0.5, 0.0], emission_strength: 4.0, ..Default::default() };
    let lit = glass.added(&lamp);
    assert_eq!(lit.emission, [4.0, 2.0, 0.0], "emission as radiance, summed");
    assert_eq!(lit.emission_strength, 1.0);
    assert_eq!(lit.alpha, 1.0);
    assert_eq!(lit.transmission, 0.5);
    assert_eq!(clear.added(&clear).alpha, 0.0, "two transparents are transparent");
}
