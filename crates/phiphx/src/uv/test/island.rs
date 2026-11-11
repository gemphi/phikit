use crate::uv::uv_test::grid;
use crate::uv::island::*;

#[test]
fn a_grid_with_no_seam_is_one_island() {
    let (pos, corner, starts) = grid(3);
    let mesh = BUvMesh { pos: &pos, corner: &corner, starts: &starts };
    let parts = split(mesh, &[]);
    assert_eq!(parts.len(), 1);
    assert_eq!(parts[0].face.len(), 9);
    assert!(open(mesh, &parts[0]), "a flat grid is all boundary");
}

/// A seam down the middle cuts the grid in two — which is the whole
/// job of a seam, and the thing a reader that ignores them gets wrong
/// in a way that looks like a solver bug.
#[test]
fn a_seam_cuts_the_grid_in_two() {
    let (pos, corner, starts) = grid(2);
    let mesh = BUvMesh { pos: &pos, corner: &corner, starts: &starts };
    // The vertical edges at i = 1: (1,4) and (4,7) on a 3 x 3 vertex grid.
    let parts = split(mesh, &[(1, 4), (4, 7)]);
    assert_eq!(parts.len(), 2, "{parts:?}");
    assert!(parts.iter().all(|p| p.face.len() == 2));
}
