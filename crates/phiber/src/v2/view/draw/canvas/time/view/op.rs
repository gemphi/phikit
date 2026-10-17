//! The animation editors' view operators, as one rule on `OpKey::TimeView`:
//! `<editor>:all` is `action.view_all` / `graph.view_all` / `nla.view_all`
//! (Home) — the shown keys or strips stretched across the window;
//! `<editor>:scene` is `anim.scene_range_frame` — the scene or preview
//! range; `<editor>:center` is `action.view_frame` / `graph.view_frame` /
//! `nla.view_frame` (Numpad 0) — pan the view to the current frame, keeping
//! width (`ANIM_center_frame`, `anim_ops.cc`); `<editor>:zoom:<steps>` is
//! `view2d.zoom_in` / `zoom_out` (the wheel) and `<editor>:scroll:<pixels>`
//! is `view2d.scroll_left` / `scroll_right` (Ctrl wheel). The maths is
//! `cur.rs`; the editor's window width comes from its last paint
//! (`BTimeCur::width_of`).

use super::cur::{BTimeCur, SCROLL_PX};
use super::super::curve::BAxisColors;
use super::super::row::{BRowKind, BTimeRow};
use super::BTimeView;
use crate::v2::host::BHostType;
use crate::v2::state::key::int_or;
use crate::v2::state::rule::BRule;
use crate::v2::state::{BKey as _, BPath, BState, CtxKey, OpKey, TimeViewKey, val::V_TRUE};

pub struct BTimeViewRule;

/// What `notches` of the wheel over `host` ask for: `view2d.zoom_in` /
/// `zoom_out` a notch at a time; under Ctrl `view2d.scroll_right` /
/// `scroll_left`, forty pixels a notch (`VIEW2D_OT_scroll_*`'s `deltax`),
/// wheel down scrolling right as Blender binds it; nothing under Shift
/// (the channel list's scroll, not here yet) or for a tremor.
pub fn wheel_op(host: BHostType, notches: f32, ctrl: bool, shift: bool) -> Option<String> {
    if notches.abs() < 0.05 || shift {
        return None;
    }
    Some(if ctrl { format!("{}:scroll:{:.1}", host.slug(), -notches * SCROLL_PX) } else { format!("{}:zoom:{:.3}", host.slug(), notches) })
}

/// A neutral palette: the extents need the rows' frames, not their colours.
const PLAIN: BAxisColors = BAxisColors { x: [1.0; 3], y: [1.0; 3], z: [1.0; 3], w: [1.0; 3], other: [1.0; 3] };

impl BTimeViewRule {
    /// The editor's view as drawn now: what it keeps, or its default.
    fn current(s: &BState, host: BHostType) -> BTimeCur {
        let view = BTimeView::now(s, host, PLAIN);
        BTimeCur { first: view.first, last: view.last }
    }

    /// `get_keyframe_extents` / the NLA's strips: the first and last frame
    /// shown, or the scene range when nothing is keyed.
    fn extents(s: &BState, host: BHostType) -> (f32, f32) {
        let (mut min, mut max) = (f32::MAX, f32::MIN);
        for row in BTimeRow::list(s, host, PLAIN) {
            for &f in &row.keys {
                min = min.min(f);
                max = max.max(f);
            }
            if let BRowKind::Track(track) = &row.kind {
                for strip in &track.strips {
                    min = min.min(strip.start);
                    max = max.max(strip.end);
                }
            }
        }
        if min <= max { (min, max) } else { Self::scene_range(s) }
    }

    /// `BKE_scene_get_playback_range`: the preview range while it is on.
    fn scene_range(s: &BState) -> (f32, f32) {
        let start = int_or(s, CtxKey::FrameStart, 1);
        let end = int_or(s, CtxKey::FrameEnd, 250).max(start);
        if s.get(CtxKey::PreviewRange) == Some(V_TRUE) {
            let ps = int_or(s, CtxKey::PreviewStart, start);
            return (ps as f32, int_or(s, CtxKey::PreviewEnd, end).max(ps) as f32);
        }
        (start as f32, end as f32)
    }
}

impl BRule for BTimeViewRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == OpKey::TimeView.path()
    }

    fn after(&self, _path: &BPath, value: &str, s: &mut BState) {
        let mut parts = value.splitn(3, ':');
        let Some(host) = parts.next().and_then(BHostType::from_slug) else { return };
        let what = parts.next().unwrap_or_default();
        let amount: f32 = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0.0);
        let px = BTimeCur::width_of(host);
        let cur = Self::current(s, host);
        let next = match what {
            "all" => {
                let (min, max) = Self::extents(s, host);
                cur.framed(min, max, px)
            }
            "scene" => {
                let (min, max) = Self::scene_range(s);
                BTimeCur { first: min, last: max }.with_margin(px)
            }
            "zoom" => cur.zoomed(amount, px),
            "scroll" => cur.scrolled(amount, px),
            // `ACTION_OT_view_frame` (`action_edit.cc`'s `actkeys_view_frame_exec`,
            // Numpad 0): pan the view to the current frame, keeping width.
            // `ANIM_center_frame` is the shared helper the Dope / Graph /
            // NLA / Sequencer each dispatch to.
            "center" => {
                let frame = s.get(CtxKey::Frame).and_then(|v| v.parse::<f32>().ok()).unwrap_or(1.0);
                cur.centered(frame)
            }
            _ => return,
        };
        s.set(TimeViewKey(host), next.encode());
        s.remove(OpKey::TimeView);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::state::action::apply_action;
    use crate::v2::view::prim::BAction;

    fn fire(s: &mut BState, what: &str) {
        apply_action(s, &BAction::Set(OpKey::TimeView.path(), what.to_string()));
    }

    #[test]
    fn the_wheel_zooms_the_editor_it_is_over_and_home_frames_the_scene_range_when_nothing_is_keyed() {
        let mut s = BState::default();
        BTimeCur::note_width(BHostType::Time, 1000.0);
        fire(&mut s, "time:zoom:1");
        let zoomed = BTimeCur::stored(&s, BHostType::Time).expect("the Timeline keeps its view");
        assert!((zoomed.first - (-9.0 + 269.0 * 0.0375)).abs() < 1e-3, "{zoomed:?}");
        assert!(BTimeCur::stored(&s, BHostType::Action).is_none(), "the Dope Sheet is untouched");
        assert!(s.get(OpKey::TimeView).is_none(), "the operator key is spent");
        fire(&mut s, "time:scroll:40");
        let scrolled = BTimeCur::stored(&s, BHostType::Time).unwrap();
        assert!((scrolled.first - zoomed.first - 40.0 * zoomed.width() / 1000.0).abs() < 1e-3);
        // Home over the default scene (no keys): the scene range with its margin.
        fire(&mut s, "time:all");
        let framed = BTimeCur::stored(&s, BHostType::Time).unwrap();
        assert!((framed.first - (1.0 - 249.0 * 0.04)).abs() < 1e-2 && (framed.last - (250.0 + 249.0 * 0.04)).abs() < 1e-2, "{framed:?}: 40 px of 1000 is 4% of 249 frames");
        s.set(CtxKey::PreviewRange, V_TRUE);
        s.set(CtxKey::PreviewStart, "20");
        s.set(CtxKey::PreviewEnd, "60");
        fire(&mut s, "time:scene");
        let preview = BTimeCur::stored(&s, BHostType::Time).unwrap();
        assert!((preview.first - 18.4).abs() < 1e-3 && (preview.last - 61.6).abs() < 1e-3, "{preview:?}: the preview range framed");
        assert_eq!(wheel_op(BHostType::Time, 1.0, false, false).as_deref(), Some("time:zoom:1.000"));
        assert_eq!(wheel_op(BHostType::Nla, -1.0, true, false).as_deref(), Some("nla:scroll:40.0"), "Ctrl wheel down: right");
        assert_eq!(wheel_op(BHostType::Time, 1.0, false, true), None, "Shift: the channels' scroll, not the view's");
    }

    #[test]
    fn center_pans_the_view_to_the_current_frame_keeping_width() {
        // `ACTION_OT_view_frame`: the frame goes to the middle, the width is
        // the same.
        let mut s = BState::default();
        BTimeCur::note_width(BHostType::Time, 1000.0);
        // Start from a known view: `time:all` with no keys lays out the scene
        // range with margins — a predictable `first..last`.
        fire(&mut s, "time:all");
        let before = BTimeCur::stored(&s, BHostType::Time).expect("the Timeline keeps its view");
        let width = before.width();
        s.set(CtxKey::Frame, "100");
        fire(&mut s, "time:center");
        let after = BTimeCur::stored(&s, BHostType::Time).expect("still kept");
        assert!((after.width() - width).abs() < 1e-3, "width unchanged");
        let middle = (after.first + after.last) * 0.5;
        assert!((middle - 100.0).abs() < 1e-3, "{middle}: current frame at the centre");
    }

    /// Over phiphx's `fixtures/actions.blend`: the Cube's keys at 1 and 20.
    #[test]
    fn home_frames_the_keys_shown() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../phiphx/tests/fixtures/actions.blend");
        let scene = crate::phiphx::blend::BlendFile::parse_bytes(&std::fs::read(&path).unwrap()).unwrap().extract_scene().unwrap();
        crate::v2::scene::source::set_loaded_scene(Some(crate::phiphx::scene::SceneModel::from_blend_owned(scene, &path.to_string_lossy())));
        let mut s = BState::default();
        BTimeCur::note_width(BHostType::Action, 1000.0);
        s.set(CtxKey::Object, "Cube");
        fire(&mut s, "action:all");
        let framed = BTimeCur::stored(&s, BHostType::Action).expect("framed");
        assert!((framed.first - 0.24).abs() < 1e-3 && (framed.last - 20.76).abs() < 1e-3, "{framed:?}: keys at 1 and 20 with 0.76 frames of margin");
        assert!((BTimeView::now(&s, BHostType::Action, PLAIN).first - 0.24).abs() < 1e-3, "and the view reads it back");
    }
}
