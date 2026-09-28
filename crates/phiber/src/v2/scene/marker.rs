//! Timeline markers — `Scene.markers`, as the file has them and as the
//! Marker menu edits them (`ed_markers.cc`: `marker.add` at the current
//! frame named `F_<frame>`, `marker.delete`, `screen.marker_jump`). The
//! file's list stands until the first edit; from then on `ctx.markers`
//! holds the whole list, so Undo is a state step and a reload is clean.

use crate::v2::state::key::int_or;
use crate::v2::state::{BKey as _, BPath, BState, CtxKey, OpKey, log, rule::BRule};

/// The markers, by frame.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BMarkerList {
    pub list: Vec<(i32, String)>,
}

impl BMarkerList {
    /// The markers now: the edited list when there is one, else the file's.
    pub fn now(state: &BState) -> Self {
        match state.get(CtxKey::Markers) {
            Some(raw) => Self::parse(raw),
            None => Self {
                list: super::source::with_loaded(|m| m.map(|m| m.markers.iter().map(|k| (k.frame, k.name.clone())).collect()).unwrap_or_default()),
            },
        }
    }

    pub fn parse(raw: &str) -> Self {
        let mut list: Vec<(i32, String)> = raw
            .split('|')
            .filter_map(|entry| entry.split_once('=').and_then(|(f, n)| f.parse().ok().map(|f| (f, n.to_string()))))
            .collect();
        list.sort_by_key(|(frame, _)| *frame);
        Self { list }
    }

    pub fn encode(&self) -> String {
        self.list.iter().map(|(f, n)| format!("{f}={n}")).collect::<Vec<_>>().join("|")
    }

    pub fn write(&self, state: &mut BState) {
        state.set(CtxKey::Markers, self.encode());
    }

    pub fn at(&self, frame: i32) -> Option<&str> {
        self.list.iter().find(|(f, _)| *f == frame).map(|(_, n)| n.as_str())
    }

    /// The selected markers' frames (`ctx.markers.selected`).
    pub fn selected(state: &BState) -> Vec<i32> {
        state.get(CtxKey::MarkerSelected).unwrap_or("").split('|').filter_map(|f| f.parse().ok()).collect()
    }

    pub fn write_selected(state: &mut BState, frames: &[i32]) {
        state.set(CtxKey::MarkerSelected, frames.iter().map(i32::to_string).collect::<Vec<_>>().join("|"));
    }

    /// The next marker after `frame`, and the one before it.
    pub fn next_after(&self, frame: i32) -> Option<i32> {
        self.list.iter().map(|(f, _)| *f).filter(|f| *f > frame).min()
    }
    pub fn previous_before(&self, frame: i32) -> Option<i32> {
        self.list.iter().map(|(f, _)| *f).filter(|f| *f < frame).max()
    }
}

/// `ctx.op.marker` → `add` / `delete` / `next` / `prev`, and the marker
/// region's own: `select:<frame>` (`select+:` extends, `select:none`),
/// `move:<from>:<to>` (a marker dragged to another frame).
pub struct BMarkerOpRule;

impl BRule for BMarkerOpRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == OpKey::Marker.path()
    }
    fn after(&self, _: &BPath, op: &str, state: &mut BState) {
        let frame = int_or(state, CtxKey::Frame, 1);
        let mut markers = BMarkerList::now(state);
        match op {
            // `ed_marker_add_exec`: one marker per frame, named F_<frame>.
            "add" => {
                if markers.at(frame).is_none() {
                    markers.list.push((frame, format!("F_{frame:02}")));
                    markers.list.sort_by_key(|(f, _)| *f);
                    markers.write(state);
                    log::push(format!("Added marker F_{frame:02}"));
                }
            }
            // `marker.delete`: the selected markers; without a selection,
            // the one at the playhead.
            "delete" => {
                let selected = BMarkerList::selected(state);
                let gone: Vec<i32> = if selected.is_empty() { markers.at(frame).map(|_| vec![frame]).unwrap_or_default() } else { selected };
                if !gone.is_empty() {
                    let names: Vec<String> = markers.list.iter().filter(|(f, _)| gone.contains(f)).map(|(_, n)| n.clone()).collect();
                    markers.list.retain(|(f, _)| !gone.contains(f));
                    markers.write(state);
                    BMarkerList::write_selected(state, &[]);
                    log::push(format!("Deleted marker {}", names.join(", ")));
                }
            }
            "select:none" => BMarkerList::write_selected(state, &[]),
            // `marker.move` (G): the grab and its moves (`scene::grab`).
            "grab" => super::grab::BMarkerGrab::start(state),
            "grab:confirm" => super::grab::BMarkerGrab::confirm(state),
            "grab:cancel" => super::grab::BMarkerGrab::cancel(state),
            "box:end" => super::grab::BMarkerBox::end(state),
            _ if op.starts_with("box:") => {
                let mut ends = op[4..].split(':').filter_map(|f| f.parse::<i32>().ok());
                if let (Some(from), Some(to)) = (ends.next(), ends.next()) {
                    super::grab::BMarkerBox::sweep(state, from, to);
                }
            }
            _ if op.starts_with("grab:at:") => {
                if let Ok(at) = op[8..].parse() {
                    super::grab::BMarkerGrab::at(state, at);
                }
            }
            _ if op.starts_with("select") => {
                let (head, frame) = op.split_once(':').unwrap_or((op, ""));
                if let Ok(frame) = frame.parse::<i32>() {
                    let mut selected = if head == "select+" { BMarkerList::selected(state) } else { Vec::new() };
                    if !selected.contains(&frame) {
                        selected.push(frame);
                    }
                    BMarkerList::write_selected(state, &selected);
                }
            }
            _ if op.starts_with("move:") => {
                let mut parts = op[5..].split(':').filter_map(|f| f.parse::<i32>().ok());
                if let (Some(from), Some(to)) = (parts.next(), parts.next()) {
                    if from != to && markers.at(to).is_none() {
                        for (f, _) in markers.list.iter_mut().filter(|(f, _)| *f == from) {
                            *f = to;
                        }
                        markers.list.sort_by_key(|(f, _)| *f);
                        markers.write(state);
                        let selected: Vec<i32> = BMarkerList::selected(state).into_iter().map(|f| if f == from { to } else { f }).collect();
                        BMarkerList::write_selected(state, &selected);
                    }
                }
            }
            "next" | "prev" => {
                let to = if op == "next" { markers.next_after(frame) } else { markers.previous_before(frame) };
                match to {
                    Some(to) => state.set(CtxKey::Frame, to.to_string()),
                    None => log::push("No more markers to jump to in this direction".to_string()),
                }
            }
            _ => {}
        }
        state.remove(OpKey::Marker);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::state::action::apply_action;
    use crate::v2::view::prim::BAction;

    fn op(state: &mut BState, what: &str) {
        apply_action(state, &BAction::Set(OpKey::Marker.path(), what.into()));
    }

    #[test]
    fn markers_are_added_at_the_playhead_jumped_between_and_deleted() {
        crate::v2::scene::source::clear_loaded_scene();
        let mut state = BState::new();
        state.set(CtxKey::Frame, "10");
        op(&mut state, "add");
        op(&mut state, "add");
        assert_eq!(BMarkerList::now(&state).list, [(10, "F_10".to_string())], "one per frame");
        state.set(CtxKey::Frame, "30");
        op(&mut state, "add");
        state.set(CtxKey::Frame, "1");
        op(&mut state, "next");
        assert_eq!(state.get(CtxKey::Frame), Some("10"));
        op(&mut state, "next");
        assert_eq!(state.get(CtxKey::Frame), Some("30"));
        op(&mut state, "next");
        assert_eq!(state.get(CtxKey::Frame), Some("30"), "no marker beyond: stays");
        op(&mut state, "prev");
        assert_eq!(state.get(CtxKey::Frame), Some("10"));
        op(&mut state, "delete");
        assert_eq!(BMarkerList::now(&state).list, [(30, "F_30".to_string())]);
        assert!(state.get(OpKey::Marker).is_none());
        assert_eq!(BMarkerList::parse("5=b|2=a").encode(), "2=a|5=b", "kept in frame order");
    }

    #[test]
    fn markers_are_selected_moved_and_the_selection_is_what_delete_takes() {
        crate::v2::scene::source::clear_loaded_scene();
        let mut state = BState::new();
        state.set(CtxKey::Markers, "10=A|20=B|30=C");
        op(&mut state, "select:10");
        assert_eq!(BMarkerList::selected(&state), [10]);
        op(&mut state, "select+:30");
        assert_eq!(BMarkerList::selected(&state), [10, 30], "Shift extends");
        op(&mut state, "select:20");
        assert_eq!(BMarkerList::selected(&state), [20], "a plain click starts over");
        // Dragged to a free frame: the marker moves and stays selected.
        op(&mut state, "move:20:25");
        assert_eq!(BMarkerList::now(&state).list, [(10, "A".to_string()), (25, "B".to_string()), (30, "C".to_string())]);
        assert_eq!(BMarkerList::selected(&state), [25]);
        // Onto a taken frame: nothing moves.
        op(&mut state, "move:25:30");
        assert_eq!(BMarkerList::now(&state).at(25), Some("B"));
        // Delete takes the selection, not the playhead's marker.
        state.set(CtxKey::Frame, "10");
        op(&mut state, "delete");
        assert_eq!(BMarkerList::now(&state).list, [(10, "A".to_string()), (30, "C".to_string())]);
        assert!(BMarkerList::selected(&state).is_empty());
        op(&mut state, "select:none");
        op(&mut state, "delete");
        assert_eq!(BMarkerList::now(&state).list, [(30, "C".to_string())], "without a selection, the playhead's");
    }
}
