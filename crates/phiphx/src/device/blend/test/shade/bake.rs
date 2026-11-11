use crate::device::blend::shade::bake::*;

/// The floor's shape: `lerp(dark red, orange, image)` — a black texel
/// takes A, a white one B, exactly as the Mix node reads its factor.
#[test]
fn a_factor_driven_mix_recolors_texels() {
    let a = [0.3, 0.0, 0.0, 1.0];
    let b = [1.0, 0.32, 0.0, 1.0];
    let ops = vec![
        BBakeOp::Image(0),
        BBakeOp::Const(a),
        BBakeOp::Const(b),
        BBakeOp::Mix { blend: 0, clamp_fac: true, clamp_res: false },
    ];
    let black = run(&ops, &|_| [0.0, 0.0, 0.0, 1.0]);
    let white = run(&ops, &|_| [1.0, 1.0, 1.0, 1.0]);
    assert_eq!(black, a, "a black texel is colour A");
    assert_eq!(white, b, "a white texel is colour B");
    let grey = run(&ops, &|_| [0.5, 0.5, 0.5, 1.0]);
    assert!((grey[0] - 0.65).abs() < 1e-3, "half-way: {grey:?}");
}

/// Layered mixes (the swimsuit's shape): each image masks its own flat
/// colour over the stack below.
#[test]
fn layered_factor_mixes_stack() {
    let ops = vec![
        BBakeOp::Image(0),
        BBakeOp::Const([0.1, 0.1, 0.1, 1.0]),
        BBakeOp::Const([0.9, 0.0, 0.0, 1.0]),
        BBakeOp::Mix { blend: 0, clamp_fac: true, clamp_res: false },
        // Second layer: previous result is A, factor from image 1.
    ];
    let out = run(&ops, &|i| if i == 0 { [1.0; 4] } else { [0.0; 4] });
    assert_eq!(out, [0.9, 0.0, 0.0, 1.0]);
}

/// The grille's shape: the image's coverage through a ramp is the alpha,
/// the colour program is left alone.
#[test]
fn an_alpha_program_cuts_without_touching_the_colour() {
    let bake = BlendBake {
        images: vec![BBakeImage { key: "grille".into(), affine: IDENTITY, straight: false }],
        ops: vec![BBakeOp::Const([0.5, 0.25, 0.0, 1.0])],
        alpha: Some(vec![BBakeOp::Image(0)]),
    };
    assert_eq!(texel(&bake, &|_| [0.0, 0.0, 0.0, 0.0]), [0.5, 0.25, 0.0, 0.0], "a hole");
    let solid = texel(&bake, &|_| [1.0; 4]);
    assert_eq!(&solid[..3], &[0.5, 0.25, 0.0]);
    assert!((solid[3] - 1.0).abs() < 1e-6, "the strand: {solid:?}");
}

#[test]
fn a_mapping_scales_and_moves_uvs() {
    let m = mapping_affine([0.25, 0.5, 0.0], 0.0, [2.0, 3.0, 1.0]);
    assert_eq!(apply(m, [1.0, 1.0]), [2.25, 3.5]);
    let spin = mapping_affine([0.0; 3], std::f32::consts::FRAC_PI_2, [1.0, 1.0, 1.0]);
    let [u, v] = apply(spin, [1.0, 0.0]);
    assert!(u.abs() < 1e-6 && (v - 1.0).abs() < 1e-6, "{u} {v}");
    let two = compose(m, spin);
    let [u2, v2] = apply(two, [1.0, 0.0]);
    let direct = apply(m, apply(spin, [1.0, 0.0]));
    assert!((u2 - direct[0]).abs() < 1e-6 && (v2 - direct[1]).abs() < 1e-6);
}
