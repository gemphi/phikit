use crate::device::blend::shade::vector::*;
use std::f32::consts::FRAC_PI_2;

fn close(a: V3, b: V3) -> bool {
    a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-5)
}

#[test]
fn vector_math_follows_svm_vector_math() {
    assert_eq!(vector_math(7, [1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [0.0; 3], 0.0), Some(([0.0; 3], 32.0)), "dot");
    assert!(close(vector_math(4, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0; 3], 0.0).unwrap().0, [0.0, 0.0, 1.0]), "cross");
    assert!(close(vector_math(10, [1.0, 2.0, 3.0], [0.0; 3], [0.0; 3], 0.5).unwrap().0, [0.5, 1.0, 1.5]), "scale");
    assert!(close(vector_math(3, [1.0, 2.0, 3.0], [2.0, 0.0, 3.0], [0.0; 3], 0.0).unwrap().0, [0.5, 0.0, 1.0]), "safe divide");
    assert!(close(vector_math(20, [1.5, -0.5, 2.5], [1.0; 3], [0.0; 3], 0.0).unwrap().0, [0.5, 0.5, 0.5]), "wrap");
    assert!(close(vector_math(6, [1.0, -1.0, 0.0], [0.0, 2.0, 0.0], [0.0; 3], 0.0).unwrap().0, [1.0, 1.0, 0.0]), "reflect");
    assert_eq!(vector_math(99, [0.0; 3], [0.0; 3], [0.0; 3], 0.0), None);
}

/// A quarter turn about Z takes X to Y; inverted, to −Y; the Euler
/// form agrees with the axis form; a zero axis leaves the vector.
#[test]
fn vector_rotate_follows_svm_node_vector_rotate() {
    let x = [1.0, 0.0, 0.0];
    assert!(close(vector_rotate(3, false, x, [0.0; 3], [0.0; 3], FRAC_PI_2, [0.0; 3]), [0.0, 1.0, 0.0]));
    assert!(close(vector_rotate(3, true, x, [0.0; 3], [0.0; 3], FRAC_PI_2, [0.0; 3]), [0.0, -1.0, 0.0]));
    assert!(close(vector_rotate(ROTATE_EULER, false, x, [0.0; 3], [0.0; 3], 0.0, [0.0, 0.0, FRAC_PI_2]), [0.0, 1.0, 0.0]));
    assert!(close(vector_rotate(0, false, x, [0.0; 3], [0.0, 0.0, 2.0], FRAC_PI_2, [0.0; 3]), [0.0, 1.0, 0.0]), "an axis of any length");
    assert!(close(vector_rotate(0, false, x, [0.0; 3], [0.0; 3], FRAC_PI_2, [0.0; 3]), x), "no axis, no turn");
    assert!(close(vector_rotate(3, false, [2.0, 1.0, 0.0], [1.0, 1.0, 0.0], [0.0; 3], FRAC_PI_2, [0.0; 3]), [1.0, 2.0, 0.0]), "about a centre");
}
