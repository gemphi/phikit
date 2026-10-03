//! The channel rows of the animation window — what `ANIM_animdata_filter`
//! lists for the editor, at the height `ANIM_UI_get_channel_step` gives each.
//!
//! Structure: the rows and where they sit. Their backdrops, diamonds and
//! names are `set.rs`.

pub mod set;

use crate::phiphx::blend::BNlaTrack;
use crate::phiphx::scene::{SceneAction, SceneNla};
use crate::v2::host::BHostType;
use crate::v2::scene::BShownKeys;
use crate::v2::state::BState;

/// `ANIM_UI_get_channel_height`: `0.8 × widget_unit`.
pub const ROW_HEIGHT: f32 = 0.8 * super::WIDGET_UNIT;
/// `NLATRACK_HEIGHT` with strip curves shown: `1.2 × widget_unit`.
pub const NLA_ROW_HEIGHT: f32 = 1.2 * super::WIDGET_UNIT;
/// `ANIM_UI_get_channel_skip` / `NLATRACK_SKIP`: `0.1 × widget_unit`.
pub const ROW_SKIP: f32 = 0.1 * super::WIDGET_UNIT;
/// `ANIM_UI_get_channel_name_width` / `NLATRACK_NAMEWIDTH`: the channel
/// region, `10 × widget_unit`.
pub const CHANNEL_WIDTH: f32 = 10.0 * super::WIDGET_UNIT;

#[derive(Clone, Debug, PartialEq)]
pub enum BRowKind {
    /// `ANIMTYPE_SUMMARY`: every shown key. Collapsed in the Timeline,
    /// expanded — and so given its backdrop — in the Dope Sheet.
    Summary { expanded: bool },
    /// `ANIMTYPE_OBJECT`: one shown object, its channels folded into it.
    Object(String),
    /// `ANIMTYPE_NLAACTION`: an object's action line in the NLA editor —
    /// the active action, or `<No Action>`; `tweaking` while
    /// `ADT_NLA_EDIT_ON`.
    Action { name: Option<String>, tweaking: bool },
    /// `ANIMTYPE_NLATRACK`: one NLA track with its strips.
    Track(BNlaTrack),
    /// `ANIMTYPE_FCURVE` in the Drivers editor: one driver, named as
    /// `getname_anim_fcurve` names it, with the curve's colour for the
    /// swatch beside the name.
    Driver { name: String, color: [f32; 3] },
}

#[derive(Clone, Debug, PartialEq)]
pub struct BTimeRow {
    pub kind: BRowKind,
    /// Sorted, distinct keyed frames of the channels under this row.
    pub keys: Vec<f32>,
}

impl BTimeRow {
    /// The rows the editor lists. The Timeline shows the summary alone
    /// (`ADS_FLAG_SUMMARY_COLLAPSED` in `action_create`); the Dope Sheet
    /// expands it and lists each shown object under it; the Graph editor
    /// lists the objects with no summary (`graph_channel_region_draw`);
    /// the Drivers editor lists each object over its drivers.
    pub fn list(s: &BState, host: BHostType, axis: super::curve::BAxisColors) -> Vec<Self> {
        let shown = BShownKeys::now(s);
        let summary = |expanded: bool| Self { kind: BRowKind::Summary { expanded }, keys: shown.columns().to_vec() };
        let mut out = match host {
            BHostType::Action => vec![summary(true)],
            BHostType::Ipo | BHostType::Nla => Vec::new(),
            BHostType::Drivers => return super::driver::BDriverList::now(s).rows(axis),
            _ => return vec![summary(false)],
        };
        crate::v2::scene::source::with_loaded(|loaded| {
            let Some(model) = loaded else { return };
            if host == BHostType::Nla {
                // `animfilter_nla`: each object, its action line, then its
                // tracks last first.
                for name in crate::v2::scene::action::BSceneActions::nla_shown(s, model) {
                    out.extend(model.nla.iter().filter(|nla| nla.object == name).flat_map(|nla| Self::nla(nla, &model.actions)));
                }
                return;
            }
            for name in crate::v2::scene::action::dope::BKeyTarget::shown(s, model) {
                out.push(Self::object(&name, &model.actions));
            }
        });
        // The Dope Sheet's summary is its objects' columns together.
        if host == BHostType::Action && out.len() > 1 {
            let mut keys: Vec<f32> = out[1..].iter().flat_map(|r| r.keys.iter().copied()).collect();
            keys.sort_by(f32::total_cmp);
            keys.dedup();
            out[0].keys = keys;
        }
        out
    }

    /// One object's NLA rows: the object, the action line with the action's
    /// key columns, then the tracks as the editor stacks them (last first).
    pub fn nla(nla: &SceneNla, actions: &[SceneAction]) -> Vec<Self> {
        let keys = nla
            .action
            .as_deref()
            .and_then(|name| actions.iter().find(|action| action.name == name))
            .map_or(Vec::new(), |action| action.keys.clone());
        let mut out = vec![
            Self { kind: BRowKind::Object(nla.object.clone()), keys: Vec::new() },
            Self { kind: BRowKind::Action { name: nla.action.clone(), tweaking: nla.flag & crate::phiphx::blend::ADT_EDIT_ON != 0 }, keys },
        ];
        out.extend(nla.tracks.iter().rev().map(|track| Self { kind: BRowKind::Track(track.clone()), keys: Vec::new() }));
        out
    }

    /// `NLATRACK_HEIGHT` for the NLA editor, `ANIM_UI_get_channel_height`
    /// for the rest.
    pub fn height(host: BHostType) -> f32 {
        if host == BHostType::Nla { NLA_ROW_HEIGHT } else { ROW_HEIGHT }
    }

    /// One object's row: the columns of the slot it is animated through in
    /// every action it uses (`dope::BKeyTarget`).
    pub fn object(name: &str, actions: &[SceneAction]) -> Self {
        Self { kind: BRowKind::Object(name.to_string()), keys: crate::v2::scene::action::dope::BKeyTarget::columns(actions, name) }
    }

    /// The channel's name as the channel region prints it
    /// (`acf_nlaaction_name`: `<No Action>` for an empty action line).
    pub fn name(&self) -> &str {
        match &self.kind {
            BRowKind::Summary { .. } => "Summary",
            BRowKind::Object(name) => name,
            BRowKind::Action { name, .. } => name.as_deref().unwrap_or("<No Action>"),
            BRowKind::Track(track) => &track.name,
            BRowKind::Driver { name, .. } => name,
        }
    }

    /// `ANIM_UI_get_first_channel_top` and `ANIM_UI_get_channel_step`
    /// (`NLATRACK_FIRST_TOP` / `NLATRACK_STEP`): the top of row `index`,
    /// measured from the top of the window, for rows `height` tall.
    pub fn top(index: usize, height: f32) -> f32 {
        super::SCRUB_HEIGHT + ROW_SKIP + index as f32 * (height + ROW_SKIP)
    }

    pub fn center(index: usize, height: f32) -> f32 {
        Self::top(index, height) + height / 2.0
    }

    /// `UI_view2d_listview_view_to_cell`: the row at `y` from the top of
    /// the window, each row's cell its height and the skip below it.
    pub fn index_at(y: f32, height: f32) -> Option<usize> {
        let from = y - Self::top(0, height);
        (from >= 0.0).then(|| (from / (height + ROW_SKIP)) as usize)
    }

    /// The NLA track row `index` of `rows` is: its object, and its place in
    /// that object's stack counted from the bottom (the rows list it last
    /// first).
    pub fn track_at(rows: &[Self], index: usize) -> Option<(String, usize)> {
        let BRowKind::Track(_) = rows.get(index)?.kind else { return None };
        let head = rows[..index].iter().rposition(|r| matches!(r.kind, BRowKind::Object(_)))?;
        let BRowKind::Object(object) = &rows[head].kind else { return None };
        let tracks = rows[head + 1..].iter().take_while(|r| !matches!(r.kind, BRowKind::Object(_))).filter(|r| matches!(r.kind, BRowKind::Track(_))).count();
        let above = rows[head + 1..index].iter().filter(|r| matches!(r.kind, BRowKind::Track(_))).count();
        Some((object.clone(), tracks - 1 - above))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn action(user: &str, keys: &[f32]) -> SceneAction {
        let mut curve = crate::phiphx::blend::BFCurve::keyed("location", 0);
        keys.iter().for_each(|f| curve.insert(*f, 0.0));
        SceneAction { name: format!("{user}Action"), keys: keys.to_vec(), users: vec![user.into()], curves: vec![curve], ..Default::default() }
    }

    #[test]
    fn an_object_row_carries_its_own_columns_and_rows_step_by_the_channel_step() {
        let list = [action("rig", &[0.0, 10.0]), action("Cube", &[5.0])];
        let row = BTimeRow::object("rig", &list);
        assert_eq!(row.keys, vec![0.0, 10.0]);
        assert_eq!(row.name(), "rig");
        assert_eq!(BTimeRow::top(0, ROW_HEIGHT), super::super::SCRUB_HEIGHT + ROW_SKIP);
        assert_eq!(BTimeRow::top(1, ROW_HEIGHT) - BTimeRow::top(0, ROW_HEIGHT), ROW_HEIGHT + ROW_SKIP);
        assert_eq!(BTimeRow::center(0, ROW_HEIGHT) - BTimeRow::top(0, ROW_HEIGHT), ROW_HEIGHT / 2.0);
        assert_eq!(BTimeRow::height(BHostType::Nla), NLA_ROW_HEIGHT, "NLA rows are `1.2 × widget_unit`");
        assert_eq!(BTimeRow::height(BHostType::Action), ROW_HEIGHT);
    }

    #[test]
    fn the_timeline_lists_the_collapsed_summary_alone() {
        let s = BState::default();
        let axis = super::super::curve::BAxisColors { x: [1.0; 3], y: [1.0; 3], z: [1.0; 3], w: [1.0; 3], other: [1.0; 3] };
        let list = BTimeRow::list(&s, BHostType::Time, axis);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].kind, BRowKind::Summary { expanded: false });
        let sheet = BTimeRow::list(&s, BHostType::Action, axis);
        assert_eq!(sheet[0].kind, BRowKind::Summary { expanded: true });
        assert!(BTimeRow::list(&s, BHostType::Ipo, axis).is_empty(), "the Graph editor lists no summary");
        assert!(BTimeRow::list(&s, BHostType::Drivers, axis).is_empty(), "nor does the Drivers editor, with nothing loaded");
        assert_eq!(BTimeRow { kind: BRowKind::Driver { name: "Hide (a)".into(), color: [0.0; 3] }, keys: Vec::new() }.name(), "Hide (a)");
    }

    #[test]
    fn the_nla_lists_the_object_its_action_line_and_its_tracks_last_first() {
        use crate::phiphx::blend::{BNlaStrip, BlendNla};
        let track = |name: &str| BNlaTrack { name: name.into(), flag: 0, strips: vec![BNlaStrip { name: "Walk".into(), start: 1.0, end: 49.0, ..Default::default() }], address: 0 };
        let nla = BlendNla { object: "Cube".into(), action: Some("Idle".into()), extend: 0, flag: 0, tracks: vec![track("Base Layer"), track("Overlay")], ..Default::default() };
        let idle = SceneAction { name: "Idle".into(), keys: vec![10.0, 30.0], users: vec!["Cube".into()], ..Default::default() };
        let rows = BTimeRow::nla(&nla, &[idle]);
        assert_eq!(rows.iter().map(BTimeRow::name).collect::<Vec<_>>(), ["Cube", "Idle", "Overlay", "Base Layer"]);
        assert_eq!((BTimeRow::track_at(&rows, 2), BTimeRow::track_at(&rows, 3)), (Some(("Cube".into(), 1)), Some(("Cube".into(), 0))), "counted from the bottom");
        assert_eq!(BTimeRow::track_at(&rows, 1), None, "the action line is no track");
        let h = NLA_ROW_HEIGHT;
        assert_eq!((BTimeRow::index_at(BTimeRow::center(3, h), h), BTimeRow::index_at(super::super::SCRUB_HEIGHT, h)), (Some(3), None));
        assert_eq!(rows[1].keys, vec![10.0, 30.0], "the action line carries the action's key columns");
        let bare = BlendNla { object: "Sphere".into(), ..Default::default() };
        assert_eq!(BTimeRow::nla(&bare, &[])[1].name(), "<No Action>");
    }
}
