use super::super::uv_test::grid;
use super::super::{BUnwrap, BUvPin, unwrap};
use super::*;

/// Two islands come back inside the square and not on top of each
/// other.
#[test]
fn the_islands_land_in_the_square_apart() {
    let (pos, corner, starts) = grid(2);
    let mesh = BUvMesh { pos: &pos, corner: &corner, starts: &starts };
    let uv = unwrap(mesh, BUnwrap::Conformal, &[(1, 4), (4, 7)], &[]);
    assert!(uv.iter().all(|c| (0.0..=1.0).contains(&c[0]) && (0.0..=1.0).contains(&c[1])), "inside the square: {uv:?}");
    let islands = super::super::island::split(mesh, &[(1, 4), (4, 7)]);
    assert_eq!(islands.len(), 2);
    let a = box_of(mesh, &islands[0], &uv);
    let b = box_of(mesh, &islands[1], &uv);
    let apart = a.1[0] <= b.0[0] + 1e-4 || b.1[0] <= a.0[0] + 1e-4 || a.1[1] <= b.0[1] + 1e-4 || b.1[1] <= a.0[1] + 1e-4;
    assert!(apart, "the two islands do not overlap: {a:?} {b:?}");
}

/// An island holding a pin is left exactly where the solver put it —
/// the packer moving it would undo the pin.
#[test]
fn a_pinned_island_is_not_moved_by_the_packer() {
    let (pos, corner, starts) = grid(2);
    let mesh = BUvMesh { pos: &pos, corner: &corner, starts: &starts };
    let seam = [(1u32, 4u32), (4, 7)];
    let pins = [BUvPin { corner: 0, uv: [0.3, 0.4] }, BUvPin { corner: 1, uv: [0.7, 0.4] }];
    let uv = unwrap(mesh, BUnwrap::Conformal, &seam, &pins);
    for p in pins {
        assert!((uv[p.corner][0] - p.uv[0]).abs() < 1e-4 && (uv[p.corner][1] - p.uv[1]).abs() < 1e-4, "pin {p:?} held: {:?}", uv[p.corner]);
    }
}
