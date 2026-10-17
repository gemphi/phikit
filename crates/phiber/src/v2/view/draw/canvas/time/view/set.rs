//! Reading the animation window's view out of the state — the scene range
//! and frame the Timeline and Dope Sheet open on (`action_create`), the
//! Graph editor's `graph_create` view over the shown objects' curves, the
//! NLA editor's rows, and the Drivers editor's `ED_drivers_editor_init`
//! view over the shown objects' drivers.

use super::super::curve::{BAxisColors, BCurveSet};
use super::super::driver::BDriverList;
use super::super::row::{BTimeRow, CHANNEL_WIDTH};
use super::super::VIEW_MARGIN;
use super::{BTimeView, DRIVER_VIEW, GRAPH_VALUE_TOP};
use crate::v2::host::BHostType;
use crate::v2::scene::marker::BMarkerList;
use crate::v2::state::key::{float_or, int_or};
use crate::v2::state::{BState, CtxKey, val::V_TRUE};

impl BTimeView {
    pub fn now(s: &BState, host: BHostType, axis: BAxisColors) -> Self {
        let start = int_or(s, CtxKey::FrameStart, 1);
        let end = int_or(s, CtxKey::FrameEnd, 250).max(start);
        let mut view = Self::of_range(start, end, float_or(s, CtxKey::Fps, 24.0).max(1.0));
        view.current = float_or(s, CtxKey::Frame, start as f32);
        view.preview = (s.get(CtxKey::PreviewRange) == Some(V_TRUE))
            .then(|| (int_or(s, CtxKey::PreviewStart, start), int_or(s, CtxKey::PreviewEnd, end)));
        view.rows = BTimeRow::list(s, host, axis);
        view.row_height = BTimeRow::height(host);
        view.nla = host == BHostType::Nla;
        if host == BHostType::Action {
            view.key_picks = crate::v2::scene::action::dope::BKeySelection::now(s);
            view.key_grab = crate::v2::scene::action::dope::BKeyGrab::active(s);
            view.key_box = s.get(CtxKey::KeyBox).and_then(|b| {
                let v: Vec<f32> = b.split('#').next()?.split(',').filter_map(|v| v.parse().ok()).collect();
                let [f0, f1, r0, r1] = v[..] else { return None };
                Some((f0, f1, r0 as usize, r1 as usize))
            });
        }
        if matches!(host, BHostType::Action | BHostType::Ipo | BHostType::Nla | BHostType::Drivers) {
            view.channel_width = CHANNEL_WIDTH;
        }
        // `SpaceAction.flag & SACTION_SHOW_MARKERS` (`DNA_action_types.h`):
        // the Timeline/Dope/Graph editor shows markers when on (default), and
        // leaves them off when toggled from View ▸ Show Markers. The Drivers
        // editor's x axis is not time, so it carries none regardless.
        if host != BHostType::Drivers && super::super::show::BShowMarkers(host).on(s) {
            let selected = BMarkerList::selected(s);
            view.markers = BMarkerList::now(s).list.into_iter().map(|(frame, name)| (frame as f32, name, selected.contains(&frame))).collect();
            view.marker_box = crate::v2::scene::grab::BMarkerBox::now(s).map(|(a, b)| (a as f32, b as f32));
        }
        match host {
            BHostType::Ipo => {
                // `graph_create`: `cur = tot = (0 .. efra) × (sfra − 10 .. 10)`.
                view.first = 0.0;
                view.last = end as f32;
                view.values = Some((start as f32 - VIEW_MARGIN, GRAPH_VALUE_TOP));
                view.curves = BCurveSet::now(s, axis).list;
            }
            BHostType::Drivers => {
                // `ED_drivers_editor_init`: `cur = tot = (−2 .. 2) × (−2 .. 2)`.
                view.first = DRIVER_VIEW.0;
                view.last = DRIVER_VIEW.1;
                view.values = Some(DRIVER_VIEW);
                view.drivers = true;
                view.curves = BDriverList::now(s).curves(axis);
            }
            BHostType::Time | BHostType::Action | BHostType::Nla => {}
            _ => {}
        }
        // Zoomed, scrolled or framed since it opened: the range it keeps
        // (`cur.rs`), as a region's `v2d.cur` outlives a redraw.
        if let Some(cur) = super::cur::BTimeCur::stored(s, host) {
            view.first = cur.first;
            view.last = cur.last;
        }
        view
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::row;
    use super::*;

    const AXIS: BAxisColors = BAxisColors { x: [1.0, 0.0, 0.0], y: [0.0, 1.0, 0.0], z: [0.0, 0.0, 1.0], w: [1.0, 1.0, 0.0], other: [0.0, 1.0, 1.0] };

    #[test]
    fn each_editor_opens_on_blenders_default_view() {
        let graph = BTimeView::now(&BState::default(), BHostType::Ipo, AXIS);
        assert_eq!((graph.first, graph.last), (0.0, 250.0), "`graph_create`: 0 .. efra");
        assert_eq!(graph.values, Some((-9.0, 10.0)), "`graph_create`: sfra − 10 .. 10");
        assert_eq!(graph.channel_width, row::CHANNEL_WIDTH);
        assert!(!graph.drivers);
        let sheet = BTimeView::now(&BState::default(), BHostType::Action, AXIS);
        assert_eq!((sheet.first, sheet.last, sheet.values), (-9.0, 260.0, None));
        assert_eq!(BTimeView::now(&BState::default(), BHostType::Time, AXIS).channel_width, 0.0);
        let drivers = BTimeView::now(&BState::default(), BHostType::Drivers, AXIS);
        assert_eq!((drivers.first, drivers.last), DRIVER_VIEW, "`ED_drivers_editor_init`: −2 .. 2");
        assert_eq!(drivers.values, Some(DRIVER_VIEW));
        assert!(drivers.drivers && drivers.channel_width == row::CHANNEL_WIDTH);
    }

    #[test]
    fn the_scenes_markers_ride_along_except_where_x_is_not_time() {
        let mut state = BState::default();
        state.set(CtxKey::Markers, "40=Cut|12=Intro");
        let time = BTimeView::now(&state, BHostType::Time, AXIS);
        assert_eq!(time.markers, [(12.0, "Intro".to_string(), false), (40.0, "Cut".to_string(), false)], "in frame order");
        assert_eq!(BTimeView::now(&state, BHostType::Ipo, AXIS).markers.len(), 2, "the Graph editor shows them too");
        assert!(BTimeView::now(&state, BHostType::Drivers, AXIS).markers.is_empty(), "the Drivers editor has no time axis");
    }

    #[test]
    fn show_markers_off_per_host_leaves_the_markers_out() {
        // `SACTION_SHOW_MARKERS` on by default; turning it off on the Timeline
        // alone leaves the Dope Sheet's showing, as Blender's per-space flag.
        use crate::v2::state::val::V_FALSE;
        let mut state = BState::default();
        state.set(CtxKey::Markers, "10=A");
        state.set(crate::v2::view::draw::canvas::time::show::BShowMarkers::key_for(BHostType::Time), V_FALSE);
        assert!(BTimeView::now(&state, BHostType::Time, AXIS).markers.is_empty(), "Timeline hides them");
        assert_eq!(BTimeView::now(&state, BHostType::Action, AXIS).markers.len(), 1, "the Dope Sheet still shows them");
    }
}
