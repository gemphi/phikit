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

#[test]
fn a_cubic_curve_starts_and_ends_where_it_says_and_bends_between() {
    let ring = rings(r#"<path d="M0 0C0 10 10 10 10 0Z"/>"#, 2);
    let r = &ring[0];
    assert_eq!(r.len(), 1 + 8);
    assert_eq!(r[r.len() - 1], [10.0, 0.0]);
    // Halfway it is three quarters of the way up (y turned up: -7.5).
    assert!((r[4][0] - 5.0).abs() < 1e-4 && (r[4][1] + 7.5).abs() < 1e-4, "{:?}", r[4]);
}
