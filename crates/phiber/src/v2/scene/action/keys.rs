//! The keys the session inserted (`ctx.scene.keyed`), in order: each the
//! values keyed into one action through one slot at one frame. Replayed
//! over the actions as Blender inserts keys (`BFCurve::insert`: a Bézier
//! key with auto-clamped handles, Continuous Acceleration smoothing), so
//! Undo steps them, the editors and the viewport show them and a save
//! writes the curves they made (phiphx `write/keys.rs`, checked in Blender
//! by `blender_opens_saves`). `keying.rs` is the operator that makes them.

use crate::phiphx::blend::{BFCurve, BlendActionSlot};
use crate::phiphx::scene::SceneAction;

const SEP: char = '|';
const AND: char = '#';
const IS: char = '=';
const EACH: char = ';';
const PART: char = ':';

/// `DNA_DEFAULT_ACTION_LAST_SLOT_HANDLE + 1`: the first slot's handle.
const FIRST_HANDLE: i32 = 0x3762_7bf6;

/// One channel's value: `location`, 0, 1.5.
#[derive(Clone, Debug, PartialEq)]
pub struct BKeyed {
    pub path: String,
    pub index: u32,
    pub value: f32,
}

/// One insert: `channels` keyed into `action` through `slot` at `frame`.
#[derive(Clone, Debug, PartialEq)]
pub struct BKeyInsert {
    pub action: String,
    /// The slot's identifier: `OBCube`.
    pub slot: String,
    pub frame: f32,
    pub channels: Vec<BKeyed>,
}

impl BKeyInsert {
    /// The inserts as `ctx.scene.keyed` spells them.
    pub fn parse_all(raw: &str) -> Vec<Self> {
        raw.split(SEP).filter_map(Self::parse).collect()
    }

    pub fn encode_all(list: &[Self]) -> String {
        list.iter().map(Self::encode).collect::<Vec<_>>().join(&SEP.to_string())
    }

    pub(super) fn parse(entry: &str) -> Option<Self> {
        let (head, body) = entry.split_once(IS)?;
        let mut head = head.split(AND);
        let (action, slot, frame) = (head.next()?.to_string(), head.next()?.to_string(), head.next()?.parse().ok()?);
        let channels = body
            .split(EACH)
            .filter_map(|c| {
                // From the right: a path may hold the separator.
                let mut part = c.rsplitn(3, PART);
                let (value, index, path) = (part.next()?.parse().ok()?, part.next()?.parse().ok()?, part.next()?);
                Some(BKeyed { path: path.to_string(), index, value })
            })
            .collect();
        Some(Self { action, slot, frame, channels })
    }

    pub(super) fn encode(&self) -> String {
        let channels: Vec<String> = self.channels.iter().map(|c| format!("{}{PART}{}{PART}{}", c.path, c.index, c.value)).collect();
        format!("{}{AND}{}{AND}{}{IS}{}", self.action, self.slot, self.frame, channels.join(&EACH.to_string()))
    }

    /// This insert made on `actions`: the slot made when the action has
    /// none by that identifier, each channel's curve found or made (at the
    /// end of the slot's run) and keyed, then the action's keyed frames and
    /// span recounted.
    pub fn apply(&self, actions: &mut [SceneAction]) {
        let Some(action) = actions.iter_mut().find(|a| a.name == self.action) else { return };
        let slot = match action.slots.iter().position(|s| s.identifier == self.slot) {
            Some(at) => at,
            None => {
                let handle = action.slots.iter().map(|s| s.handle).max().map_or(FIRST_HANDLE, |h| h + 1);
                let end = action.curves.len();
                action.slots.push(BlendActionSlot { handle, identifier: self.slot.clone(), curves: end..end });
                action.slots.len() - 1
            }
        };
        for channel in &self.channels {
            let run = action.slots[slot].curves.clone();
            let found = action.curves[run.clone()].iter().position(|c| c.path == channel.path && c.index == channel.index);
            let at = match found {
                Some(i) => run.start + i,
                None => {
                    action.curves.insert(run.end, BFCurve::keyed(channel.path.clone(), channel.index));
                    for (i, other) in action.slots.iter_mut().enumerate() {
                        if i == slot {
                            other.curves.end += 1;
                        } else if other.curves.start >= run.end {
                            other.curves = other.curves.start + 1..other.curves.end + 1;
                        }
                    }
                    run.end
                }
            };
            action.curves[at].insert(self.frame, channel.value);
        }
        Self::recount(action);
    }

    /// `BlendActionSummary::of` over the curves as they are now: the keyed
    /// frames (to a thousandth), the span they cover, the curve count.
    pub(super) fn recount(action: &mut SceneAction) {
        let mut keys: Vec<f32> = action.curves.iter().flat_map(|c| c.keys.iter().map(|k| (k.co[0] * 1000.0).round() / 1000.0)).collect();
        keys.sort_by(f32::total_cmp);
        keys.dedup();
        (action.frame_start, action.frame_end) = keys.first().zip(keys.last()).map_or((0.0, 0.0), |(a, b)| (*a, *b));
        action.fcurve_count = action.curves.len();
        action.keys = keys;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn insert(action: &str, slot: &str, frame: f32, values: &[(&str, u32, f32)]) -> BKeyInsert {
        let channels = values.iter().map(|(p, i, v)| BKeyed { path: p.to_string(), index: *i, value: *v }).collect();
        BKeyInsert { action: action.into(), slot: slot.into(), frame, channels }
    }

    #[test]
    fn inserts_round_trip_and_make_their_slot_and_curves_in_the_slots_run() {
        let list = vec![insert("CubeAction", "OBCube", 10.0, &[("location", 0, 1.5), ("pose.bones[\"a:b\"].location", 2, -1.0)])];
        assert_eq!(BKeyInsert::parse_all(&BKeyInsert::encode_all(&list)), list);

        // An action with another slot after the one keyed: its run moves on.
        let other = BlendActionSlot { handle: 7, identifier: "OBCamera".into(), curves: 1..2 };
        let cube = BlendActionSlot { handle: 3, identifier: "OBCube".into(), curves: 0..1 };
        let curve = |path: &str| BFCurve::keyed(path, 0);
        let mut actions = vec![SceneAction { name: "Act".into(), slots: vec![cube, other], curves: vec![curve("location"), curve("scale")], ..Default::default() }];
        insert("Act", "OBCube", 1.0, &[("location", 0, 0.0), ("rotation_euler", 0, 0.5)]).apply(&mut actions);
        insert("Act", "OBLight", 5.0, &[("location", 1, 2.0)]).apply(&mut actions);
        let a = &actions[0];
        assert_eq!(a.slots[0].curves, 0..2, "the new curve joins the Cube's run");
        assert_eq!(a.slots[1].curves, 2..3, "the Camera's run moves on");
        assert_eq!((a.slots[2].identifier.as_str(), a.slots[2].handle, a.slots[2].curves.clone()), ("OBLight", 8, 3..4));
        assert_eq!(a.curves[1].path, "rotation_euler");
        assert_eq!(a.keys, [1.0, 5.0]);
        assert_eq!((a.frame_start, a.frame_end, a.fcurve_count), (1.0, 5.0, 4));
        let mut empty = vec![SceneAction { name: "Action".into(), ..Default::default() }];
        insert("Action", "OBCrate", 1.0, &[("location", 0, 1.0)]).apply(&mut empty);
        assert_eq!(empty[0].slots[0].handle, FIRST_HANDLE, "Blender's first slot handle");
    }
}
