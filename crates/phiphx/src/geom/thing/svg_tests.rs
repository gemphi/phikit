//! The svg reader on paths of its own.

use super::svg::rings;

#[test]
fn a_square_path_is_one_ring_turned_up() {
    let ring = rings(r#"<path d="M0 0L10 0L10 10L0 10Z"/>"#, 4);
    assert_eq!(ring, vec![vec![[0.0, 0.0], [10.0, 0.0], [10.0, -10.0], [0.0, -10.0]]]);
}

#[test]
fn relative_moves_and_a_hole_are_two_rings() {
    let ring = rings(r#"<path d="M0 0h10v10h-10z m2 2h6v6h-6z"/>"#, 4);
    assert_eq!(ring.len(), 2);
    assert_eq!(ring[1][0], [2.0, -2.0]);
}

#[test]
fn a_half_turn_arc_keeps_to_its_circle() {
    let ring = rings(r#"<path d="M0 0A5 5 0 0 1 10 0Z"/>"#, 4);
    let r = &ring[0];
    assert!(r.len() > 4, "{r:?}");
    for p in &r[1..] {
        let d = ((p[0] - 5.0).powi(2) + p[1].powi(2)).sqrt();
        assert!((d - 5.0).abs() < 1e-3, "{p:?}");
    }
}
