//! The drop box's contract, and the part of it that cannot be seen from
//! outside: that the prefix is applied once rather than twice, whichever
//! consumer gets there first.

use super::prefix::{BPrefixBox, BPrefixKey};
use super::BlendEvalMesh;

fn mesh(vertices: usize) -> BlendEvalMesh {
    BlendEvalMesh { vertices: vec![[1.0, 2.0, 3.0]; vertices], ..Default::default() }
}

fn key(object: usize, applied: usize) -> BPrefixKey {
    BPrefixKey { object, applied, weights: true }
}

#[test]
fn the_second_reader_gets_what_the_first_left() {
    let b = BPrefixBox::default();
    b.put(key(7, 2), &mesh(41));
    assert_eq!(b.waiting(), 1);
    let got = b.take(key(7, 2)).expect("the mesh the first consumer left");
    assert_eq!(got.vertices.len(), 41);
}

#[test]
fn taking_frees_it_so_a_subdivided_mesh_is_not_held() {
    // Each prefix has exactly two readers. Once the second has it, holding
    // the mesh would be a leak the size of a subdivided garment, and nothing
    // runs afterwards to clear the box.
    let b = BPrefixBox::default();
    b.put(key(1, 1), &mesh(9));
    assert!(b.take(key(1, 1)).is_some());
    assert_eq!(b.waiting(), 0, "the entry is gone once its second reader has it");
    assert!(b.take(key(1, 1)).is_none(), "and a third take finds nothing");
}

#[test]
fn a_different_prefix_length_is_a_different_mesh() {
    // `PHIPHX_MODIFIERS` truncates one consumer's stack and not the other's,
    // so the lengths disagree — which must miss, not hand over a mesh with
    // the wrong modifiers applied.
    let b = BPrefixBox::default();
    b.put(key(3, 2), &mesh(64));
    assert!(b.take(key(3, 1)).is_none(), "one modifier applied is not two");
    assert!(b.take(key(4, 2)).is_none(), "and another object is another mesh");
    assert!(b.take(key(3, 2)).is_some());
}

#[test]
fn a_mesh_without_weights_is_never_handed_to_the_rig() {
    // The rig skins `base` by its vertex groups. A mesh whose groups were
    // never loaded cannot be skinned, so the keys must not match across that
    // difference even for the same object and the same modifiers.
    let b = BPrefixBox::default();
    b.put(BPrefixKey { object: 2, applied: 1, weights: false }, &mesh(8));
    assert!(b.take(BPrefixKey { object: 2, applied: 1, weights: true }).is_none());
    assert!(b.take(BPrefixKey { object: 2, applied: 1, weights: false }).is_some());
}

#[test]
fn a_second_put_keeps_the_one_its_partner_is_waiting_for() {
    // Two puts under one key would mean a third consumer appeared. Replacing
    // the entry would hand the waiting reader a mesh it did not ask for, so
    // the first is kept.
    let b = BPrefixBox::default();
    b.put(key(5, 1), &mesh(2));
    b.put(key(5, 1), &mesh(900));
    assert_eq!(b.take(key(5, 1)).expect("the first").vertices.len(), 2);
}

#[test]
fn the_box_itself_has_no_switch() {
    // `PHIPHX_NOSHARE` gates the two *consumers* (`prefix::sharing()`), not
    // the box, so these tests describe the box's contract whatever is in the
    // environment running them. Asserting it here keeps the switch from
    // drifting back inside `take`, where it made five of these fail for
    // anyone who had the variable exported.
    let b = BPrefixBox::default();
    b.put(key(0, 1), &mesh(3));
    assert!(b.take(key(0, 1)).is_some(), "the box hands it over regardless of the environment");
}
