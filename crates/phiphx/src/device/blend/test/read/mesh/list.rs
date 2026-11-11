use crate::device::blend::read::mesh::list::*;

#[test]
fn keeps_faces_and_costs_one_array() {
    let f: BFaceList = vec![vec![0, 1, 2], vec![2, 3, 4, 5]].into();
    assert_eq!((f.len(), f.corners()), (2, 7));
    assert_eq!(f.get(0), [0, 1, 2]);
    assert_eq!(&f[1], [2, 3, 4, 5]);
    assert_eq!(f.start(1), 3, "face 1's first loop");
    assert_eq!(f.get(2), [] as [u32; 0], "past the end");
    assert_eq!(f.iter().map(<[u32]>::len).sum::<usize>(), 7);
    let mut g = BFaceList::default();
    g.push(&[0, 1, 2]);
    g.push(&[2, 3, 4, 5]);
    assert_eq!(g, f);
}
