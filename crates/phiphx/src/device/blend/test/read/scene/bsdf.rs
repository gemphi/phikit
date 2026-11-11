use crate::device::blend::read::scene::bsdf::BlendBsdf;

fn f0(eta: f32) -> f32 {
    ((eta - 1.0) / (eta + 1.0)).powi(2)
}

/// Cycles' IOR adjustment (`svm/closure.h`): F0 scales with twice the
/// level, 0.5 leaves the IOR as it is, 0 takes the reflection away.
#[test]
fn specular_ior_level_scales_the_reflection_as_cycles_does() {
    let at = |level: f32| BlendBsdf { ior: 1.45, specular_level: level, ..Default::default() }.specular_eta();
    assert_eq!(at(0.5), 1.45);
    assert_eq!(at(0.0), 1.0, "no reflection at all");
    // Pose 23_2's floor: 0.115 is 23% of the plain IOR's reflection.
    assert!((f0(at(0.115)) / f0(1.45) - 0.23).abs() < 1e-4, "{}", f0(at(0.115)) / f0(1.45));
    assert!((f0(at(1.0)) / f0(1.45) - 2.0).abs() < 1e-4);
    // An IOR under 1 stays under 1 (the inside of a bubble).
    assert!(BlendBsdf { ior: 0.75, specular_level: 0.25, ..Default::default() }.specular_eta() < 1.0);
}
