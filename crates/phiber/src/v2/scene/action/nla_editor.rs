//! The NLA editor's operators (`ctx.op.nla`) on the stacks it shows — the
//! selected objects and the active one, each with animation data
//! (`animfilter_nla`): a click on a strip (`nla.click_select`, Shift
//! flips), a click on a track's name (`anim.channels_click`), Select ▸
//! All / None / Invert (`nla.select_all`), Strips ▸ Delete (X,
//! `nla.delete`) and Toggle Muting (H, `nla.mute_toggle`), Tracks ▸ Add,
//! Add Above Selected and Delete (`nla.tracks_add`, `nla.tracks_delete`),
//! and Add ▸ Action (Shift A, `nla.actionclip_add`: a strip of the action
//! at the current frame, in each shown stack's active track).
//! phiphx makes each (`anim/nla_edit.rs`); the stacks that changed are kept
//! whole (`nla.rs`), so Undo steps them and a save writes them.

use super::BSceneActions;
use crate::phiphx::blend::{BNlaPick, BNlaStacks, BlendNla};
use crate::phiphx::scene::SceneModel;
use crate::v2::state::{BKey as _, BPath, BState, CtxKey, OpKey, log, rule::BRule};

/// What `ctx.op.nla` asks for.
#[derive(Clone, Debug, PartialEq)]
pub enum BNlaOp {
    /// A click in the strips: the strip hit — its object, its track counted
    /// from the bottom, its place on the track — or nothing; Shift held.
    Click { hit: Option<(String, usize, usize)>, extend: bool },
    /// A click on a track's name in the channel list.
    Channel { object: String, track: usize, extend: bool },
    Select(BNlaPick),
    Delete,
    Mute,
    AddTracks { above: bool },
    DeleteTracks,
    /// Add ▸ Action: a strip of the action named.
    AddAction(String),
}

impl BNlaOp {
    /// The value a click on `hit` (or nothing) sends:
    /// `click:<shift>:<track>:<strip>:<object>`, `click:<shift>:-`.
    pub fn click(hit: Option<(&str, usize, usize)>, extend: bool) -> String {
        let shift = u8::from(extend);
        hit.map_or(format!("click:{shift}:-"), |(object, track, strip)| format!("click:{shift}:{track}:{strip}:{object}"))
    }

    /// `channel:<shift>:<track>:<object>`.
    pub fn channel(object: &str, track: usize, extend: bool) -> String {
        format!("channel:{}:{track}:{object}", u8::from(extend))
    }

    pub fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "select:all" => Self::Select(BNlaPick::Add),
            "select:none" => Self::Select(BNlaPick::Clear),
            "select:invert" => Self::Select(BNlaPick::Invert),
            "delete" => Self::Delete,
            "mute" => Self::Mute,
            "tracks_add" => Self::AddTracks { above: false },
            "tracks_add_above" => Self::AddTracks { above: true },
            "tracks_delete" => Self::DeleteTracks,
            _ if value.starts_with("add_action:") => Self::AddAction(value["add_action:".len()..].to_string()),
            _ => {
                // The object's name last: it may hold the separator.
                let (kind, rest) = value.split_once(':')?;
                let (shift, rest) = rest.split_once(':')?;
                let extend = shift == "1";
                match kind {
                    "click" if rest == "-" => Self::Click { hit: None, extend },
                    "click" => {
                        let mut part = rest.splitn(3, ':');
                        let (track, strip) = (part.next()?.parse().ok()?, part.next()?.parse().ok()?);
                        Self::Click { hit: Some((part.next()?.to_string(), track, strip)), extend }
                    }
                    "channel" => {
                        let (track, object) = rest.split_once(':')?;
                        Self::Channel { object: object.to_string(), track: track.parse().ok()?, extend }
                    }
                    _ => return None,
                }
            }
        })
    }
}

impl BSceneActions {
    /// The objects the NLA editor lists, in its order: the selected ones,
    /// then the active one, each with animation data.
    pub fn nla_shown(state: &BState, model: &SceneModel) -> Vec<String> {
        let mut names: Vec<&str> = model.selection.selected.iter().map(String::as_str).collect();
        names.extend(state.get(CtxKey::Object));
        names.dedup();
        names.into_iter().filter(|name| model.nla.iter().any(|n| n.object == *name)).map(str::to_string).collect()
    }

    /// `op` on the stacks of `shown` at `frame`; what the log says. Each
    /// stack it changed is kept.
    pub fn nla_edit(&mut self, op: &BNlaOp, shown: &[String], model: &SceneModel, frame: f32) -> Option<String> {
        let mut stacks = BNlaStacks(shown.iter().map(|o| self.stack_of(o, model)).collect());
        let index = |object: &str| shown.iter().position(|o| o == object);
        let count = |n: usize, one: &str| if n == 1 { format!("1 {one}") } else { format!("{n} {one}s") };
        let said = match op {
            BNlaOp::Click { hit, extend } => {
                let hit = hit.as_ref().and_then(|(object, track, strip)| Some((index(object)?, *track, *strip)));
                stacks.click(hit, *extend);
                None
            }
            BNlaOp::Channel { object, track, extend } => {
                stacks.click_track(index(object)?, *track, *extend);
                None
            }
            BNlaOp::Select(pick) => {
                stacks.pick_strips(*pick, false);
                None
            }
            BNlaOp::Delete => Some(format!("Deleted {}", count(stacks.delete_strips(), "strip"))),
            BNlaOp::Mute => Some(format!("Toggled muting on {}", count(stacks.mute_strips(), "strip"))),
            BNlaOp::AddTracks { above } if !stacks.add_tracks(*above) => Some("Select an existing NLA Track or an empty action line first".to_string()),
            BNlaOp::AddTracks { .. } => Some("Added NLA tracks".to_string()),
            BNlaOp::DeleteTracks => Some(format!("Deleted {}", count(stacks.delete_tracks(), "track"))),
            BNlaOp::AddAction(_) if !stacks.0.iter().any(BlendNla::takes_clips) => {
                return Some("No active track(s) to add strip to, select an existing track or add one before trying again".into());
            }
            BNlaOp::AddAction(name) => {
                let Some(action) = self.action(name, model) else { return Some("No valid action to add".into()) };
                for stack in &mut stacks.0 {
                    let clip = self.clip_for(&action, &stack.object, model);
                    stack.add_clip(clip, frame);
                }
                Some(format!("Added a strip of {name}"))
            }
        };
        for stack in stacks.0 {
            if stack != self.stack_of(&stack.object, model) {
                self.keep_stack(stack);
            }
        }
        said
    }
}

/// `ctx.op.nla` → the edit, on the stacks the NLA editor shows.
pub struct BNlaEditRule;

impl BRule for BNlaEditRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == OpKey::Nla.path()
    }
    fn after(&self, _: &BPath, value: &str, state: &mut BState) {
        state.remove(OpKey::Nla);
        let Some(op) = BNlaOp::parse(value) else { return };
        let frame = crate::v2::state::key::float_or(state, CtxKey::Frame, 1.0);
        let mut edits = BSceneActions::now(state);
        let before = edits.clone();
        let said = crate::v2::scene::source::with_loaded(|model| model.and_then(|m| edits.nla_edit(&op, &BSceneActions::nla_shown(state, m), m, frame)));
        if edits != before {
            edits.write(state);
        }
        if let Some(said) = said {
            log::push(said);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_values_read_as_operators_and_a_name_keeps_its_separators() {
        assert_eq!(BNlaOp::parse(&BNlaOp::click(Some(("Rig:Arm", 1, 0)), true)), Some(BNlaOp::Click { hit: Some(("Rig:Arm".into(), 1, 0)), extend: true }));
        assert_eq!(BNlaOp::parse(&BNlaOp::click(None, false)), Some(BNlaOp::Click { hit: None, extend: false }));
        assert_eq!(BNlaOp::parse(&BNlaOp::channel("Cube", 2, false)), Some(BNlaOp::Channel { object: "Cube".into(), track: 2, extend: false }));
        assert_eq!(BNlaOp::parse("select:invert"), Some(BNlaOp::Select(BNlaPick::Invert)));
        assert_eq!(BNlaOp::parse("tracks_add_above"), Some(BNlaOp::AddTracks { above: true }));
        assert_eq!(BNlaOp::parse("click:0:x:0:Cube"), None);
        assert_eq!(BNlaOp::parse("add_action:Walk:Cycle"), Some(BNlaOp::AddAction("Walk:Cycle".into())));
    }
}
