//! `BUvEdit`'s own tests: a pick against Blender's `uv_select_pick`, a pin
//! held through an unwrap, an unwrap confined to its own map, and a move
//! folded into the offset already there.

use super::*;
use crate::v2::state::BKey as _;

#[test]
fn a_pick_replaces_the_selection_and_shift_toggles_within_it() {
    let mut s = BState::new();
    let edit = BUvEdit::now(&s);
    assert!(edit.selected.is_empty() && edit.moved.is_empty(), "a fresh session has edited nothing");
    // A plain pick is the selection.
    assert_eq!(edit.picked((0, 7), false), [(0, 7)]);
    s.set(UvKey::Selected.path(), BUvEdit::write_selected(&[(0, 7), (1, 2)]));
    let edit = BUvEdit::now(&s);
    assert_eq!(edit.selected, [(0, 7), (1, 2)], "the two corners, in the order picked");
    assert!(edit.holds((1, 2)) && !edit.holds((1, 3)));
    // Shift adds, and Shift on a selected corner drops it.
    assert_eq!(edit.picked((0, 1), true), [(0, 7), (1, 2), (0, 1)]);
    assert_eq!(edit.picked((0, 7), true), [(1, 2)]);
    // And a plain pick still replaces the lot.
    assert_eq!(edit.picked((0, 1), false), [(0, 1)]);
}

/// Pin, unpin, and the thing that makes a pin a pin: an unwrap holds
/// it where it was and solves the rest around it.
#[test]
fn a_pinned_corner_survives_an_unwrap() {
    use crate::phiphx::uv::{BUnwrap, BUvMesh};
    // A 2 x 2 grid of quads on the XY plane, as the reader gives one.
    let (mut pos, mut corner, mut starts) = (Vec::new(), Vec::new(), vec![0u32]);
    for j in 0..=2 {
        for i in 0..=2 {
            pos.push([i as f32, j as f32, 0.0]);
        }
    }
    for j in 0..2u32 {
        for i in 0..2u32 {
            let a = j * 3 + i;
            corner.extend_from_slice(&[a, a + 1, a + 4, a + 3]);
            starts.push(corner.len() as u32);
        }
    }
    let mesh = BUvMesh { pos: &pos, corner: &corner, starts: &starts };
    let file_uv = vec![[0.0f32; 2]; corner.len()];

    let mut s = BState::new();
    s.set(UvKey::Selected.path(), BUvEdit::write_selected(&[(0, 0), (0, 1)]));
    let edit = BUvEdit::now(&s);
    assert!(edit.pinned.is_empty() && !edit.stuck((0, 0)));
    // P pins the selection; Alt P takes it back off.
    let pinned = edit.pinning(false);
    assert_eq!(pinned, [(0, 0), (0, 1)]);
    s.set(UvKey::Pinned.path(), BUvEdit::write_pinned(&pinned));
    let edit = BUvEdit::now(&s);
    assert!(edit.stuck((0, 0)) && edit.stuck((0, 1)) && !edit.stuck((0, 2)));
    assert_eq!(edit.pinning(true), Vec::<BUvCorner>::new(), "Alt P clears them");

    // Move the two pinned corners somewhere of their own, then unwrap.
    let moved = edit.nudged([0.3, 0.6]);
    s.set(UvKey::Moved.path(), BUvEdit::write_moved(&moved));
    let edit = BUvEdit::now(&s);
    let before = [edit.uv_of((0, 0), file_uv[0]), edit.uv_of((0, 1), file_uv[1])];
    let out = edit.unwrapped(0, mesh, &file_uv, BUnwrap::Conformal, &[]);
    s.set(UvKey::Moved.path(), BUvEdit::write_moved(&out));
    let edit = BUvEdit::now(&s);
    for (k, c) in [(0usize, (0usize, 0usize)), (1, (0, 1))] {
        let now = edit.uv_of(c, file_uv[k]);
        assert!((now[0] - before[k][0]).abs() < 1e-4 && (now[1] - before[k][1]).abs() < 1e-4, "pin {c:?} held: {now:?} vs {:?}", before[k]);
    }
    // And the rest actually moved: an unwrap that changes nothing is
    // not an unwrap.
    assert!(edit.moved.len() > 2, "the island was solved around the pins: {:?}", edit.moved.len());
}

/// An unwrap of one map leaves another map's offsets alone.
#[test]
fn an_unwrap_touches_only_its_own_map() {
    use crate::phiphx::uv::{BUnwrap, BUvMesh};
    let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]];
    let (corner, starts) = (vec![0u32, 1, 2, 3], vec![0u32, 4]);
    let mesh = BUvMesh { pos: &pos, corner: &corner, starts: &starts };
    let file_uv = vec![[0.0f32; 2]; 4];
    let mut s = BState::new();
    s.set(UvKey::Moved.path(), BUvEdit::write_moved(&[((1, 5), [0.2, 0.3])]));
    let edit = BUvEdit::now(&s);
    let out = edit.unwrapped(0, mesh, &file_uv, BUnwrap::Conformal, &[]);
    assert!(out.contains(&((1, 5), [0.2, 0.3])), "the other map's offset is still there: {out:?}");
    assert!(out.iter().any(|((m, _), _)| *m == 0), "and this map was unwrapped");
}

#[test]
fn a_move_folds_into_the_offset_already_there_and_reads_back_whole() {
    let mut s = BState::new();
    s.set(UvKey::Selected.path(), BUvEdit::write_selected(&[(0, 3), (0, 4)]));
    let edit = BUvEdit::now(&s);
    let once = edit.nudged([0.25, -0.5]);
    s.set(UvKey::Moved.path(), BUvEdit::write_moved(&once));
    let edit = BUvEdit::now(&s);
    assert_eq!(edit.offset((0, 3)), [0.25, -0.5]);
    assert_eq!(edit.offset((0, 9)), [0.0, 0.0], "a corner never moved has no offset");
    // The file's UV, moved.
    assert_eq!(edit.uv_of((0, 3), [0.5, 0.5]), [0.75, 0.0]);
    assert_eq!(edit.uv_of((0, 9), [0.5, 0.5]), [0.5, 0.5]);
    // A second drag continues the first rather than replacing it.
    let twice = edit.nudged([0.1, 0.1]);
    s.set(UvKey::Moved.path(), BUvEdit::write_moved(&twice));
    let edit = BUvEdit::now(&s);
    assert_eq!(edit.offset((0, 4)), [0.35, -0.4]);
    assert_eq!(edit.moved.len(), 2, "two corners moved, not four entries");
}
