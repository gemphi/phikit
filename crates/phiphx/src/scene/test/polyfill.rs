use super::*;

fn area(p: &[[f32; 3]], tris: &[[usize; 3]]) -> f32 {
    tris.iter()
        .map(|t| {
            let (a, b, c) = (p[t[0]], p[t[1]], p[t[2]]);
            0.5 * ((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])).abs()
        })
        .sum()
}

#[test]
fn concave_l_shape_is_covered_exactly_once() {
    // L-shaped hexagon in XY, area 3; a fan from corner 0 would overlap.
    let l = [
        [0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [2.0, 1.0, 0.0],
        [1.0, 1.0, 0.0], [1.0, 2.0, 0.0], [0.0, 2.0, 0.0],
    ];
    let tris = triangulate(&l);
    assert_eq!(tris.len(), 4);
    assert!((area(&l, &tris) - 3.0).abs() < 1e-5);
}

#[test]
fn clockwise_and_vertical_faces_still_triangulate() {
    let cw: Vec<[f32; 3]> = [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [2.0, 0.5], [1.0, 0.0]]
        .iter()
        .map(|p| [p[0], 0.0, p[1]])
        .collect();
    assert_eq!(triangulate(&cw).len(), 3);
}

#[test]
fn quads_use_blenders_fixed_split() {
    let q = [[0.0; 3], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]];
    assert_eq!(triangulate(&q), vec![[0, 1, 2], [0, 2, 3]]);
}
