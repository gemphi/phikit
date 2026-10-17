//! F-curve channel properties the session changed — `ANIM_OT_channels_*` and
//! the Channel ▸ Extrapolation Mode / Toggle-Enable-Disable Channel Setting
//! menus: a curve's extrapolation (`FCurve.extend`), its Protect
//! (`FCURVE_PROTECTED`) and its Mute (`FCURVE_MUTED`).
//!
//! The file is read-only, so a channel property is an overlay like every other
//! edit (`keys.rs`, `reshape.rs`): a list keyed by the curve's identity,
//! replayed over the file's curves after the key steps so a save would write
//! it and Undo steps it. One entry per `(action, slot, path, index)` — setting
//! a property twice replaces, it does not stack.
//!
//! phiber has no channel (F-curve) selection yet — its editors select key
//! columns — so these act on the SHOWN channels: every curve of the objects
//! the editor lists (`BKeyTarget::shown`). That is a recorded divergence from
//! Blender, where the same operators act on the selected channels; it is the
//! common case (one object's curves) and keeps the menus usable until a
//! channel-selection gesture lands.

use super::dope::BKeyTarget;
use crate::phiphx::blend::{FLAG_MUTED, FLAG_PROTECTED};
use crate::phiphx::scene::{SceneAction, SceneModel};
use crate::v2::state::BState;

const SEP: char = '|';
const AND: char = '#';
const IS: char = '=';
const NONE: &str = "-";

/// One F-curve, named the way a key target names its slot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BChannelId {
    pub action: String,
    pub slot: String,
    pub path: String,
    pub index: u32,
}

impl BChannelId {
    /// Every channel the editor lists: each shown object's curves, through the
    /// slot it is animated by (`BKeyTarget`'s slot resolution).
    pub fn shown(state: &BState, model: &SceneModel) -> Vec<Self> {
        let mut out: Vec<Self> = Vec::new();
        for object in BKeyTarget::shown(state, model) {
            for a in model.actions.iter().filter(|a| a.used_by(&object)) {
                let bound = a.bindings.iter().find(|(o, _)| *o == object).and_then(|(_, h)| a.slot(*h));
                let slot = bound.or_else(|| (a.slots.len() == 1).then(|| &a.slots[0]));
                let (slot_id, curves) = slot.map_or((String::new(), &a.curves[..]), |s| (s.identifier.clone(), &a.curves[s.curves.clone()]));
                for c in curves {
                    let id = Self { action: a.name.clone(), slot: slot_id.clone(), path: c.path.clone(), index: c.index };
                    if !out.contains(&id) {
                        out.push(id);
                    }
                }
            }
        }
        out
    }
}

/// A change to one channel's properties — any subset set, the rest left.
#[derive(Clone, Debug, PartialEq)]
pub struct BChannelEdit {
    pub id: BChannelId,
    pub extend: Option<u8>,
    pub protect: Option<bool>,
    pub mute: Option<bool>,
}

impl BChannelEdit {
    /// The curve `id` names on `actions`, if it is there.
    fn curve_mut<'a>(id: &BChannelId, actions: &'a mut [SceneAction]) -> Option<&'a mut crate::phiphx::blend::BFCurve> {
        let action = actions.iter_mut().find(|a| a.name == id.action)?;
        let run = match id.slot.is_empty() {
            true => 0..action.curves.len(),
            false => action.slots.iter().find(|s| s.identifier == id.slot)?.curves.clone(),
        };
        action.curves[run].iter_mut().find(|c| c.path == id.path && c.index == id.index)
    }

    pub fn apply(&self, actions: &mut [SceneAction]) {
        let Some(curve) = Self::curve_mut(&self.id, actions) else { return };
        if let Some(extend) = self.extend {
            curve.extend = extend;
        }
        let flag = |curve: &mut crate::phiphx::blend::BFCurve, bit: u32, on: bool| {
            if on {
                curve.flag |= bit;
            } else {
                curve.flag &= !bit;
            }
        };
        if let Some(protect) = self.protect {
            flag(curve, FLAG_PROTECTED, protect);
        }
        if let Some(mute) = self.mute {
            flag(curve, FLAG_MUTED, mute);
        }
    }

    fn opt_u8(v: Option<u8>) -> String {
        v.map_or(NONE.to_string(), |n| n.to_string())
    }
    fn opt_bool(v: Option<bool>) -> String {
        v.map_or(NONE.to_string(), |b| u8::from(b).to_string())
    }
    fn parse_u8(part: &str) -> Option<u8> {
        (part != NONE).then(|| part.parse().ok()).flatten()
    }
    fn parse_bool(part: &str) -> Option<bool> {
        (part != NONE).then(|| part == "1")
    }

    fn encode(&self) -> String {
        format!(
            "{}{AND}{}{AND}{}{AND}{}{IS}{},{},{}",
            self.id.action,
            self.id.slot,
            self.id.path,
            self.id.index,
            Self::opt_u8(self.extend),
            Self::opt_bool(self.protect),
            Self::opt_bool(self.mute),
        )
    }

    fn parse(entry: &str) -> Option<Self> {
        let (head, body) = entry.split_once(IS)?;
        let mut h = head.splitn(3, AND);
        let (action, slot) = (h.next()?.to_string(), h.next()?.to_string());
        let (path, index) = h.next()?.rsplit_once(AND)?;
        let mut field = body.split(',');
        let edit = Self {
            id: BChannelId { action, slot, path: path.to_string(), index: index.parse().ok()? },
            extend: Self::parse_u8(field.next()?),
            protect: Self::parse_bool(field.next()?),
            mute: Self::parse_bool(field.next()?),
        };
        Some(edit)
    }

    pub fn parse_all(raw: &str) -> Vec<Self> {
        raw.split(SEP).filter(|e| !e.is_empty()).filter_map(Self::parse).collect()
    }

    pub fn encode_all(list: &[Self]) -> String {
        list.iter().map(Self::encode).collect::<Vec<_>>().join(&SEP.to_string())
    }
}

/// The channel operators on `ctx.op.channels`: `extend:const|lin`,
/// `protect:on|off|toggle`, `mute:on|off|toggle`.
pub struct BChannelOpRule;

impl crate::v2::state::rule::BRule for BChannelOpRule {
    fn wants(&self, path: &crate::v2::state::BPath) -> bool {
        use crate::v2::state::BKey as _;
        *path == crate::v2::state::OpKey::Channels.path()
    }

    fn after(&self, _: &crate::v2::state::BPath, op: &str, s: &mut BState) {
        use crate::phiphx::blend::{EXTRAPOLATE_CONSTANT, EXTRAPOLATE_LINEAR};
        let Some(model) = crate::v2::scene::source::loaded_shared() else { return };
        let shown = BChannelId::shown(s, &model);
        if shown.is_empty() {
            return;
        }
        let mut parts = op.split(':');
        let (verb, mode) = (parts.next().unwrap_or(""), parts.next());
        let mut edits = super::BSceneActions::now(s);
        // The channels as they stand, so a toggle reads the flag it flips.
        let current = edits.keyed_actions(&model);
        let flag_on = |id: &BChannelId, bit: u32| BChannelEdit::curve_now(id, &current).is_some_and(|c| c.flag & bit != 0);
        for id in shown {
            let edit = match (verb, mode) {
                ("extend", Some("const")) => BChannelEdit { id, extend: Some(EXTRAPOLATE_CONSTANT), protect: None, mute: None },
                ("extend", Some("lin")) => BChannelEdit { id, extend: Some(EXTRAPOLATE_LINEAR), protect: None, mute: None },
                ("protect", Some(m)) => {
                    let on = resolve(m, flag_on(&id, FLAG_PROTECTED));
                    BChannelEdit { id, extend: None, protect: Some(on), mute: None }
                }
                ("mute", Some(m)) => {
                    let on = resolve(m, flag_on(&id, FLAG_MUTED));
                    BChannelEdit { id, extend: None, protect: None, mute: Some(on) }
                }
                _ => return,
            };
            edits.set_channel(edit);
        }
        edits.write(s);
    }
}

/// `on`/`off`/`toggle` resolved against the current state.
fn resolve(mode: &str, current: bool) -> bool {
    match mode {
        "on" => true,
        "off" => false,
        _ => !current,
    }
}

impl BChannelEdit {
    /// The curve `id` names, read-only — for a toggle to see the current flag.
    fn curve_now<'a>(id: &BChannelId, actions: &'a [SceneAction]) -> Option<&'a crate::phiphx::blend::BFCurve> {
        let action = actions.iter().find(|a| a.name == id.action)?;
        let run = match id.slot.is_empty() {
            true => 0..action.curves.len(),
            false => action.slots.iter().find(|s| s.identifier == id.slot)?.curves.clone(),
        };
        action.curves[run].iter().find(|c| c.path == id.path && c.index == id.index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phiphx::blend::{BFCurve, BKey, BlendActionSlot, EXTRAPOLATE_CONSTANT, EXTRAPOLATE_LINEAR, IPO_BEZ};
    use crate::phiphx::scene::SceneAction;

    fn curve(path: &str) -> BFCurve {
        BFCurve::new(path, 0, vec![BKey::new([0.0; 2], [1.0, 0.0], [2.0; 2], IPO_BEZ)])
    }

    fn actions() -> Vec<SceneAction> {
        vec![SceneAction {
            name: "Act".into(),
            slots: vec![BlendActionSlot { handle: 1, identifier: "OBCube".into(), curves: 0..2 }],
            curves: vec![curve("location"), curve("scale")],
            ..Default::default()
        }]
    }

    fn id(path: &str) -> BChannelId {
        BChannelId { action: "Act".into(), slot: "OBCube".into(), path: path.into(), index: 0 }
    }

    #[test]
    fn extend_and_flags_apply_to_the_named_curve_only() {
        let mut acts = actions();
        BChannelEdit { id: id("location"), extend: Some(EXTRAPOLATE_LINEAR), protect: Some(true), mute: None }.apply(&mut acts);
        let loc = acts[0].curves.iter().find(|c| c.path == "location").unwrap();
        let scale = acts[0].curves.iter().find(|c| c.path == "scale").unwrap();
        assert_eq!(loc.extend, EXTRAPOLATE_LINEAR);
        assert!(loc.flag & FLAG_PROTECTED != 0, "location is protected");
        assert_eq!(scale.extend, EXTRAPOLATE_CONSTANT, "scale is untouched");
        assert!(scale.flag & FLAG_PROTECTED == 0);
    }

    #[test]
    fn mute_clears_as_well_as_sets() {
        let mut acts = actions();
        BChannelEdit { id: id("location"), extend: None, protect: None, mute: Some(true) }.apply(&mut acts);
        assert!(acts[0].curves[0].flag & FLAG_MUTED != 0);
        BChannelEdit { id: id("location"), extend: None, protect: None, mute: Some(false) }.apply(&mut acts);
        assert!(acts[0].curves[0].flag & FLAG_MUTED == 0, "off clears the bit");
    }

    #[test]
    fn a_channel_edit_round_trips_through_its_encoding() {
        let e = BChannelEdit { id: id("pose.bones[\"a\"].location"), extend: Some(EXTRAPOLATE_LINEAR), protect: None, mute: Some(true) };
        assert_eq!(BChannelEdit::parse_all(&BChannelEdit::encode_all(std::slice::from_ref(&e))), vec![e]);
    }

    #[test]
    fn toggle_resolves_against_the_current_flag() {
        assert!(resolve("toggle", false));
        assert!(!resolve("toggle", true));
        assert!(resolve("on", false));
        assert!(!resolve("off", true));
    }
}
