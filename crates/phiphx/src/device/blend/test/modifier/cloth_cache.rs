use super::*;

#[test]
fn the_hermite_passes_through_both_ends() {
    let (x1, x2) = (Vec3::new(0.0, 1.0, 2.0), Vec3::new(3.0, -1.0, 0.5));
    let (v1, v2) = (Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 2.0, 0.0));
    assert!((interp_cubic(x1, v1, x2, v2, 0.0) - x1).length() < 1e-6);
    assert!((interp_cubic(x1, v1, x2, v2, 1.0) - x2).length() < 1e-6);
    // Straight-line motion at constant velocity is reproduced exactly.
    let v = x2 - x1;
    assert!((interp_cubic(x1, v, x2, v, 0.25) - (x1 + v * 0.25)).length() < 1e-6);
}

#[test]
fn an_index_array_places_points_by_index() {
    let fr = BCacheFrame::from_arrays(3, 2, Some(&[2, 0]), &[[1.0; 3], [2.0; 3]], None);
    assert_eq!(fr.positions, vec![[2.0; 3], [0.0; 3], [1.0; 3]]);
}
