//! Moving the selected markers with the pointer — Blender's `marker.move`
//! (G over the Timeline, `ed_marker_move_modal`): the markers follow the
//! pointer's frame from where it first stood, a click confirms, Escape or
//! the right button puts them back. The grab keeps what it started from in
//! `ctx.markers.grab` — `<start frame>#<selected>#<markers>` — so every
//! move is measured from the start, not from the last move, and a cancel
//! restores exactly. The whole grab is one undo step (`BHistory` holds the
//! document from before it while the grab is on).

use super::marker::BMarkerList;
use crate::v2::state::{BState, CtxKey, log};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BMarkerGrab {
    /// The pointer's frame when it first moved; `None` until then.
    pub from: Option<i32>,
    /// The selection and the markers as they were when G was pressed.
    pub selected: Vec<i32>,
    pub list: BMarkerList,
}

impl BMarkerGrab {
    /// The grab under way, if one is.
    pub fn now(state: &BState) -> Option<Self> {
        let raw = state.get(CtxKey::MarkerGrab).filter(|raw| !raw.is_empty())?;
        let mut parts = raw.splitn(3, '#');
        let from = parts.next()?.parse().ok();
        let selected = parts.next()?.split(',').filter_map(|f| f.parse().ok()).collect();
        let list = BMarkerList::parse(parts.next().unwrap_or(""));
        Some(Self { from, selected, list })
    }

    pub fn active(state: &BState) -> bool {
        Self::now(state).is_some()
    }

    fn write(&self, state: &mut BState) {
        let from = self.from.map(|f| f.to_string()).unwrap_or_default();
        let selected = self.selected.iter().map(i32::to_string).collect::<Vec<_>>().join(",");
        state.set(CtxKey::MarkerGrab, format!("{from}#{selected}#{}", self.list.encode()));
    }

    /// G: the grab starts on the selected markers; with none, it says so.
    pub fn start(state: &mut BState) {
        let selected = BMarkerList::selected(state);
        if selected.is_empty() {
            log::push("No selected markers to move".to_string());
            return;
        }
        Self { from: None, selected, list: BMarkerList::now(state) }.write(state);
    }

    /// The pointer is over `frame`: the selected markers stand that far from
    /// where they started. A move that would land one on an unselected
    /// marker's frame is refused, as Blender refuses to stack them.
    pub fn at(state: &mut BState, frame: i32) {
        let Some(mut grab) = Self::now(state) else { return };
        let Some(from) = grab.from else {
            grab.from = Some(frame);
            grab.write(state);
            return;
        };
        let offset = frame - from;
        let (moving, staying): (Vec<_>, Vec<_>) = grab.list.list.iter().cloned().partition(|(f, _)| grab.selected.contains(f));
        let moved: Vec<(i32, String)> = moving.into_iter().map(|(f, n)| (f + offset, n)).collect();
        if moved.iter().any(|(f, _)| staying.iter().any(|(s, _)| s == f)) {
            return;
        }
        let mut list = BMarkerList { list: staying.into_iter().chain(moved).collect() };
        list.list.sort_by_key(|(f, _)| *f);
        list.write(state);
        let selected: Vec<i32> = grab.selected.iter().map(|f| f + offset).collect();
        BMarkerList::write_selected(state, &selected);
    }

    /// A click: the markers stay where they are.
    pub fn confirm(state: &mut BState) {
        let Some(grab) = Self::now(state) else { return };
        state.remove(CtxKey::MarkerGrab);
        let now = BMarkerList::selected(state);
        let offset = now.first().zip(grab.selected.first()).map_or(0, |(a, b)| a - b);
        log::push(format!("Moved {} marker(s) by {offset} frame(s)", now.len()));
    }

    /// Escape or the right button: everything as it was before G.
    pub fn cancel(state: &mut BState) {
        let Some(grab) = Self::now(state) else { return };
        state.remove(CtxKey::MarkerGrab);
        grab.list.write(state);
        BMarkerList::write_selected(state, &grab.selected);
    }
}

/// A box swept over the marker band (`marker.select_box`): the markers
/// between the frame it began at and the pointer's are selected, added to
/// the selection it began with (a press without Shift cleared that). It
/// keeps `<from>:<to>#<selection at the start>` in `ctx.markers.box` while
/// the button is down, for the window to draw.
pub struct BMarkerBox;

impl BMarkerBox {
    /// The box's two frames, while one is being swept.
    pub fn now(state: &BState) -> Option<(i32, i32)> {
        let raw = state.get(CtxKey::MarkerBox).filter(|raw| !raw.is_empty())?;
        let (range, _) = raw.split_once('#').unwrap_or((raw, ""));
        let (from, to) = range.split_once(':')?;
        Some((from.parse().ok()?, to.parse().ok()?))
    }

    pub fn sweep(state: &mut BState, from: i32, to: i32) {
        let base: Vec<i32> = match state.get(CtxKey::MarkerBox).and_then(|raw| raw.split_once('#')) {
            Some((_, base)) => base.split(',').filter_map(|f| f.parse().ok()).collect(),
            None => BMarkerList::selected(state),
        };
        let (low, high) = (from.min(to), from.max(to));
        let mut selected = base.clone();
        for (frame, _) in BMarkerList::now(state).list {
            if (low..=high).contains(&frame) && !selected.contains(&frame) {
                selected.push(frame);
            }
        }
        selected.sort();
        BMarkerList::write_selected(state, &selected);
        let base = base.iter().map(i32::to_string).collect::<Vec<_>>().join(",");
        state.set(CtxKey::MarkerBox, format!("{from}:{to}#{base}"));
    }

    pub fn end(state: &mut BState) {
        state.remove(CtxKey::MarkerBox);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_grab_moves_the_selected_markers_from_where_the_pointer_started_and_cancel_restores() {
        let mut s = BState::new();
        s.set(CtxKey::Markers, "10=A|20=B|30=C");
        BMarkerList::write_selected(&mut s, &[10, 20]);
        BMarkerGrab::start(&mut s);
        assert!(BMarkerGrab::active(&s));
        BMarkerGrab::at(&mut s, 50);
        assert_eq!(BMarkerList::now(&s).encode(), "10=A|20=B|30=C", "the first position is the origin");
        BMarkerGrab::at(&mut s, 55);
        assert_eq!(BMarkerList::now(&s).encode(), "15=A|25=B|30=C");
        assert_eq!(BMarkerList::selected(&s), [15, 25], "the selection follows");
        BMarkerGrab::at(&mut s, 60);
        assert_eq!(BMarkerList::now(&s).encode(), "15=A|25=B|30=C", "B would land on C: refused");
        BMarkerGrab::at(&mut s, 53);
        assert_eq!(BMarkerList::now(&s).encode(), "13=A|23=B|30=C", "measured from the start, not the last move");
        BMarkerGrab::cancel(&mut s);
        assert!(!BMarkerGrab::active(&s));
        assert_eq!((BMarkerList::now(&s).encode(), BMarkerList::selected(&s)), ("10=A|20=B|30=C".to_string(), vec![10, 20]));

        BMarkerGrab::start(&mut s);
        BMarkerGrab::at(&mut s, 0);
        BMarkerGrab::at(&mut s, -4);
        BMarkerGrab::confirm(&mut s);
        assert!(!BMarkerGrab::active(&s));
        assert_eq!(BMarkerList::now(&s).encode(), "6=A|16=B|30=C");

        BMarkerList::write_selected(&mut s, &[]);
        BMarkerGrab::start(&mut s);
        assert!(!BMarkerGrab::active(&s), "nothing selected: no grab");
    }

    #[test]
    fn a_box_selects_the_markers_it_spans_on_top_of_what_it_began_with() {
        let mut s = BState::new();
        s.set(CtxKey::Markers, "10=A|20=B|30=C|40=D");
        BMarkerList::write_selected(&mut s, &[40]);
        BMarkerBox::sweep(&mut s, 25, 15);
        assert_eq!(BMarkerList::selected(&s), [20, 40], "B, kept with what was selected");
        BMarkerBox::sweep(&mut s, 25, 35);
        assert_eq!(BMarkerList::selected(&s), [30, 40], "the box shrank off B: measured from the start");
        assert_eq!(BMarkerBox::now(&s), Some((25, 35)));
        BMarkerBox::end(&mut s);
        assert_eq!((BMarkerBox::now(&s), BMarkerList::selected(&s)), (None, vec![30, 40]));
    }
}
