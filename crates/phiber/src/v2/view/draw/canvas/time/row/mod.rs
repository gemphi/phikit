//! The channel rows of the animation window — what `ANIM_animdata_filter`
//! lists for the editor, at the height `ANIM_UI_get_channel_step` gives each.
//!
//! Structure: the rows and where they sit. Their backdrops, diamonds and
//! names are `set.rs`.

pub mod name;
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
    /// `ANIMTYPE_OBJECT`: one shown object. `expanded` is Blender's
    /// `ACHANNEL_SETTING_EXPAND` (`ADT_UI_EXPANDED`): closed, the object's
    /// keys are summarised onto its own row; open, its F-curves follow it as
    /// `Curve` rows and the object keeps the summary above them, which is
    /// what `ANIM_animdata_filter` emits for an expanded object.
    Object { name: String, expanded: bool },
    /// `ANIMTYPE_FCURVE`: one F-curve under an expanded object, named the way
    /// `getname_anim_fcurve` names it (`name::fcurve_name`) and carrying the
    /// curve's own colour for the swatch beside it.
    Curve { name: String, color: [f32; 3] },
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
                let expanded = Self::is_expanded(s, &name);
                out.push(Self::object_at(&name, &model.actions, expanded));
                if expanded {
                    out.extend(Self::curves(&name, &model.actions));
                }
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
            Self { kind: BRowKind::Object { name: nla.object.clone(), expanded: false }, keys: Vec::new() },
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
        Self::object_at(name, actions, false)
    }

    /// The same row, told whether it is unfolded.
    pub fn object_at(name: &str, actions: &[SceneAction], expanded: bool) -> Self {
        Self {
            kind: BRowKind::Object { name: name.to_string(), expanded },
            keys: crate::v2::scene::action::dope::BKeyTarget::columns(actions, name),
        }
    }

    /// Whether `name`'s channel is unfolded.
    ///
    /// Closed by default, as Blender leaves an object's channels closed until
    /// asked: a rig with three hundred bone curves would otherwise bury every
    /// other object in the list on first paint.
    pub fn is_expanded(s: &BState, name: &str) -> bool {
        s.get(crate::v2::state::DopeKey::Expanded(crate::v2::state::intern(name)))
            == Some(crate::v2::state::val::V_TRUE)
    }

    /// The F-curve rows of every action `name` uses, in the file's own order.
    ///
    /// `ANIM_animdata_filter` walks an expanded object's action and emits one
    /// `ANIMTYPE_FCURVE` per curve. There is no group level here yet: the scene
    /// model carries `SceneAction.curves` flat, without the `bActionGroup` that
    /// Blender would nest them under.
    pub fn curves(name: &str, actions: &[SceneAction]) -> Vec<Self> {
        actions
            .iter()
            .filter(|action| action.users.iter().any(|user| user == name))
            .flat_map(|action| action.curves.iter())
            .map(|curve| Self {
                kind: BRowKind::Curve { name: name::fcurve_name(curve), color: curve.color },
                // `BezTriple.co` is (frame, value); the frame is the x.
                keys: curve.keys.iter().map(|key| key.co[0]).collect(),
            })
            .collect()
    }

    /// The channel's name as the channel region prints it
    /// (`acf_nlaaction_name`: `<No Action>` for an empty action line).
    pub fn name(&self) -> &str {
        match &self.kind {
            BRowKind::Summary { .. } => "Summary",
            BRowKind::Object { name, .. } => name,
            BRowKind::Curve { name, .. } => name,
            BRowKind::Action { name, .. } => name.as_deref().unwrap_or("<No Action>"),
            BRowKind::Track(track) => &track.name,
            BRowKind::Driver { name, .. } => name,
        }
    }

    /// `ANIM_UI_get_first_channel_top` and `ANIM_UI_get_channel_step`
    /// (`NLATRACK_FIRST_TOP` / `NLATRACK_STEP`): the top of row `index`,
    /// measured from the top of the window, for rows `height` tall.
    /// `scrub` is the band above the rows (`BTimeView::scrub_height`), which
    /// these are all measured past. It is a parameter and not a constant so the
    /// theme decides it; `SCRUB_HEIGHT` remains as Blender's own number for the
    /// callers that mean that and nothing else.
    pub fn top(index: usize, height: f32, scrub: f32) -> f32 {
        scrub + ROW_SKIP + index as f32 * (height + ROW_SKIP)
    }

    pub fn center(index: usize, height: f32, scrub: f32) -> f32 {
        Self::top(index, height, scrub) + height / 2.0
    }

    /// `UI_view2d_listview_view_to_cell`: the row at `y` **from the top of the
    /// window**, each row's cell its height and the skip below it.
    ///
    /// From the top of the WINDOW, not of the rows: it subtracts `top(0)`
    /// itself, and `top` already counts past the scrub band. A caller that
    /// subtracts the band before calling this subtracts it twice — which
    /// `expander_write` did, putting every expander click a band's worth of
    /// pixels down the list.
    pub fn index_at(y: f32, height: f32, scrub: f32) -> Option<usize> {
        let from = y - Self::top(0, height, scrub);
        (from >= 0.0).then(|| (from / (height + ROW_SKIP)) as usize)
    }

    /// The NLA track row `index` of `rows` is: its object, and its place in
    /// that object's stack counted from the bottom (the rows list it last
    /// first).
    pub fn track_at(rows: &[Self], index: usize) -> Option<(String, usize)> {
        let BRowKind::Track(_) = rows.get(index)?.kind else { return None };
        let head = rows[..index].iter().rposition(|r| matches!(r.kind, BRowKind::Object { .. }))?;
        let BRowKind::Object { name: object, .. } = &rows[head].kind else { return None };
        let tracks = rows[head + 1..].iter().take_while(|r| !matches!(r.kind, BRowKind::Object { .. })).filter(|r| matches!(r.kind, BRowKind::Track(_))).count();
        let above = rows[head + 1..index].iter().filter(|r| matches!(r.kind, BRowKind::Track(_))).count();
        Some((object.clone(), tracks - 1 - above))
    }
}

#[cfg(test)]
mod tests;
