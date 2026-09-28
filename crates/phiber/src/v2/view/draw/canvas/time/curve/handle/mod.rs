//! The handles of one F-curve — `draw_fcurve_handles` and
//! `draw_fcurve_handle_vertices` in `graph_draw.cc`: a line from each
//! Bézier key to each of its handles, coloured by the handle's type
//! (`TH_HANDLE_*`, selected or not), and a ring on each handle's end. A
//! key's first handle shows only when the segment before it is Bézier, its
//! second only when its own segment is; with Only Selected Keyframes
//! Handles on, only keys with something selected show theirs. Structure:
//! the segments and the rings. Painting them is `set.rs`.

pub mod set;

use crate::phiphx::blend::{BFCurve, BKey, IPO_BEZ};

/// One handle line, from the handle's end to its key.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BHandleSegment {
    pub handle: [f32; 2],
    pub key: [f32; 2],
    /// `eBezTriple_Handle`: which `TH_HANDLE_*` colour.
    pub kind: u8,
    pub selected: bool,
}

/// One handle's end, `TH_HANDLE_VERTEX` or `_SELECT`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BHandleRing {
    pub at: [f32; 2],
    pub selected: bool,
}

/// The handles of a curve across a view, unselected before selected as
/// Blender draws them (`for (sel = 0; sel < 2; sel++)`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BCurveHandles {
    pub segments: Vec<BHandleSegment>,
    pub rings: Vec<BHandleRing>,
}

impl BCurveHandles {
    /// `get_bounding_bezt_index_range`: the keys in view and one either side.
    fn in_view(curve: &BFCurve, first: f32, last: f32) -> &[BKey] {
        let keys = &curve.keys;
        if keys.is_empty() {
            return keys;
        }
        let from = keys.partition_point(|key| key.co[0] < first).saturating_sub(1);
        let to = keys.partition_point(|key| key.co[0] <= last).min(keys.len() - 1);
        &keys[from..=to]
    }

    /// Whether `key`'s first handle has a Bézier segment before it: the
    /// segment from the previous key, or its own when it is the first.
    fn first_handle_shows(previous: Option<&BKey>, key: &BKey) -> bool {
        previous.map_or(key.ipo == IPO_BEZ, |before| before.ipo == IPO_BEZ)
    }

    pub fn of(curve: &BFCurve, first: f32, last: f32, selected_keys_only: bool) -> Self {
        let keys = Self::in_view(curve, first, last);
        let mut out = Self::default();
        for selected in [false, true] {
            let mut previous: Option<&BKey> = None;
            for key in keys {
                if selected_keys_only && !key.any_selected() {
                    previous = Some(key);
                    continue;
                }
                let first_shows = Self::first_handle_shows(previous, key);
                let second_shows = key.ipo == IPO_BEZ;
                // `draw_fcurve_handles`: a selected key's handles take its
                // pass whatever their own selection; an unselected key's
                // take the pass of their own selection.
                let (first_pass, second_pass) = if key.selected == selected { (true, true) } else { (key.h1_selected == selected, key.h2_selected == selected) };
                if first_shows && first_pass {
                    out.segments.push(BHandleSegment { handle: key.h1, key: key.co, kind: key.h1_type, selected });
                }
                if second_shows && second_pass {
                    out.segments.push(BHandleSegment { handle: key.h2, key: key.co, kind: key.h2_type, selected });
                }
                // `draw_fcurve_selected_handle_vertices`: a ring on each
                // handle whose own selection matches the pass.
                if first_shows && key.h1_selected == selected {
                    out.rings.push(BHandleRing { at: key.h1, selected });
                }
                if second_shows && key.h2_selected == selected {
                    out.rings.push(BHandleRing { at: key.h2, selected });
                }
                previous = Some(key);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phiphx::blend::{HANDLE_ALIGN, HANDLE_FREE, HANDLE_VECT};

    fn key(frame: f32, ipo: u8) -> BKey {
        BKey::new([frame - 2.0, 0.0], [frame, 0.0], [frame + 2.0, 0.0], ipo)
    }

    #[test]
    fn handles_follow_the_bezier_segments_and_the_selection_passes() {
        let mut second = key(10.0, IPO_BEZ);
        second.selected = true;
        second.h1_type = HANDLE_FREE;
        second.h2_type = HANDLE_VECT;
        let mut third = key(20.0, 1);
        third.h2_selected = true;
        third.h1_type = HANDLE_ALIGN;
        let curve = BFCurve::new("location", 0, vec![key(0.0, IPO_BEZ), second, third, key(30.0, IPO_BEZ)]);
        let handles = BCurveHandles::of(&curve, 0.0, 30.0, false);
        // Key 1 (Bézier, first): both handles, unselected pass. Key 2
        // (selected, handles not): both handles in the unselected pass by
        // their own selection, then again in the selected pass by the
        // key's — as Blender draws them, twice. Key 3 (linear after a
        // Bézier segment): first handle only — its second handle, though
        // selected, has no Bézier segment. Key 4 (Bézier after linear): its
        // second handle only.
        let unselected: Vec<_> = handles.segments.iter().filter(|s| !s.selected).collect();
        let selected: Vec<_> = handles.segments.iter().filter(|s| s.selected).collect();
        assert_eq!(unselected.len(), 6, "{unselected:?}");
        assert_eq!(unselected[0].handle, [-2.0, 0.0]);
        assert_eq!(unselected[4].key, [20.0, 0.0]);
        assert_eq!(unselected[4].kind, HANDLE_ALIGN);
        assert_eq!(unselected[5].handle, [32.0, 0.0]);
        assert_eq!(selected.iter().map(|s| s.kind).collect::<Vec<_>>(), [HANDLE_FREE, HANDLE_VECT]);
        assert!(handles.segments.iter().position(|s| s.selected).unwrap() > handles.segments.iter().rposition(|s| !s.selected).unwrap(), "unselected first");
        // Rings follow each handle's own selection: the selected key's
        // handles are not themselves selected, so its rings are plain.
        assert_eq!(handles.rings.iter().filter(|r| r.selected).count(), 0);
        assert_eq!(handles.rings.len(), 6);
        // Only Selected Keyframes Handles: keys 2 and 3 have something
        // selected — key 2's two handles twice, key 3's first; key 3's
        // second handle still has no segment to show.
        let only = BCurveHandles::of(&curve, 0.0, 30.0, true);
        assert_eq!(only.segments.len(), 5, "{:?}", only.segments);
        assert_eq!(only.rings.len(), 3);
        // A step or a straight run has no handles.
        assert_eq!(BCurveHandles::of(&BFCurve::new("v", 0, vec![key(0.0, 0), key(10.0, 1)]), 0.0, 10.0, false), BCurveHandles::default());
    }

    #[test]
    fn only_the_keys_in_view_and_their_neighbours_count() {
        let curve = BFCurve::new("v", 0, (0..6).map(|i| key(i as f32 * 10.0, IPO_BEZ)).collect());
        assert_eq!(BCurveHandles::in_view(&curve, 15.0, 35.0).len(), 4, "keys 10, 20, 30, 40");
        assert_eq!(BCurveHandles::in_view(&curve, -5.0, 100.0).len(), 6);
        assert!(BCurveHandles::in_view(&BFCurve::new("v", 0, vec![]), 0.0, 1.0).is_empty());
    }
}
