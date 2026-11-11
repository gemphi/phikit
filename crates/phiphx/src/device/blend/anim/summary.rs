//! What the animation editors show of an action without opening its
//! curves: its name, the frames it spans, how many curves it has, the frames
//! that carry a key on any curve, and the objects that use it.
//!
//! Blender's Dope Sheet builds the same thing per channel as an
//! `ActKeyColumn` list (`keyframes_keylist.cc`): one column per distinct
//! frame, whatever the number of curves keyed there. The Timeline's summary
//! row is the union over the visible channels, which is what `keys` is for
//! one action; the editor unions the actions it shows.

use crate::device::blend::anim::fcurve::BFCurve;
use crate::device::blend::anim::read::BAction;
use crate::device::blend::file::kind::*;
use crate::device::blend::read::scene::kind::BlendSceneObject;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlendActionSummary {
    pub name: String,
    /// `BKE_action_frame_range_calc` over the keys: the first and last keyed
    /// frame, or `0..0` when nothing is keyed.
    pub frame_start: f32,
    pub frame_end: f32,
    pub fcurve_count: usize,
    /// Sorted, distinct keyed frames across every curve of every slot.
    pub keys: Vec<f32>,
    /// Names of the objects whose `AnimData.action` is this action.
    pub users: Vec<String>,
    /// Every curve of every slot, in file order — what the Graph editor draws.
    pub curves: Vec<BFCurve>,
    /// The slots, each with its run of `curves`. A legacy action has one,
    /// handle 0, with no identifier.
    pub slots: Vec<BlendActionSlot>,
    /// `(object, AnimData.slot_handle)` for each user, in `users`' order.
    pub bindings: Vec<(String, i32)>,
    /// `use_frame_range` with `frame_start` / `frame_end`, when set.
    pub manual_range: Option<(f32, f32)>,
    pub cyclic: bool,
}

/// One slot of an action as the editors name it (`ActionSlot`): its handle,
/// its identifier (`OBCube`), and the run of the action's `curves` it keys.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlendActionSlot {
    pub handle: i32,
    pub identifier: String,
    pub curves: std::ops::Range<usize>,
}

impl BlendActionSlot {
    /// The ID type the identifier leads with (`OB`), and the name after it.
    pub fn id_code(&self) -> &str {
        self.identifier.get(..2).unwrap_or("")
    }

    pub fn name(&self) -> &str {
        self.identifier.get(2..).unwrap_or("")
    }
}

impl BlendActionSummary {
    /// One action, summarised. Rounded to a thousandth of a frame so two
    /// curves keyed at "the same" frame in float noise make one column.
    pub fn of(action: &BAction) -> Self {
        let mut keys: Vec<f32> = action
            .by_slot
            .values()
            .flatten()
            .flat_map(|curve| curve.keys.iter().map(|k| (k.co[0] * 1000.0).round() / 1000.0))
            .collect();
        keys.sort_by(f32::total_cmp);
        keys.dedup();
        let fcurve_count = action.by_slot.values().map(Vec::len).sum();
        let (frame_start, frame_end) = match (keys.first(), keys.last()) {
            (Some(&first), Some(&last)) => (first, last),
            _ => (0.0, 0.0),
        };
        let mut slot_list: Vec<&i32> = action.by_slot.keys().collect();
        slot_list.sort();
        let mut curves: Vec<BFCurve> = Vec::new();
        let mut runs: Vec<(i32, std::ops::Range<usize>)> = Vec::new();
        for slot in slot_list {
            let start = curves.len();
            curves.extend(action.by_slot[slot].iter().cloned());
            runs.push((*slot, start..curves.len()));
        }
        // Every slot the file lists, then any curves with no slot record (a
        // legacy action's, under handle 0).
        let run = |handle: i32| runs.iter().find(|(h, _)| *h == handle).map_or(0..0, |(_, r)| r.clone());
        let mut slots: Vec<BlendActionSlot> = action.slots.iter().map(|s| BlendActionSlot { handle: s.handle, identifier: s.identifier.clone(), curves: run(s.handle) }).collect();
        for (handle, range) in &runs {
            if !slots.iter().any(|s| s.handle == *handle) {
                slots.push(BlendActionSlot { handle: *handle, identifier: String::new(), curves: range.clone() });
            }
        }
        Self {
            name: action.name.clone(),
            frame_start,
            frame_end,
            fcurve_count,
            keys,
            users: Vec::new(),
            curves,
            slots,
            bindings: Vec::new(),
            manual_range: action.manual_range,
            cyclic: action.cyclic,
        }
    }

    /// Does `object` animate with this action?
    pub fn used_by(&self, object: &str) -> bool {
        self.users.iter().any(|user| user == object)
    }
}

impl BlendFile {
    /// Every action in the file with the objects bound to it, in the order
    /// the file lists them.
    pub(crate) fn action_summaries(
        &self,
        objects: &[BlendSceneObject],
        actions: &std::collections::HashMap<u64, super::read::BAction>,
        dna: &DnaType,
    ) -> Vec<BlendActionSummary> {
        let mut address_list: Vec<u64> = actions.keys().copied().collect();
        address_list.sort_unstable();
        let mut out: Vec<BlendActionSummary> = address_list
            .iter()
            .map(|address| BlendActionSummary::of(&actions[address]))
            .collect();
        for object in objects {
            let Some(block) = self.block_at(object.block_address) else { continue };
            let Some(binding) = self.binding(block, dna) else { continue };
            if let Some(index) = address_list.iter().position(|&address| address == binding.action) {
                out[index].users.push(object.name.clone());
                out[index].bindings.push((object.name.clone(), binding.slot));
            }
        }
        out
    }
}

#[cfg(test)]
#[path = "../test/anim/summary.rs"]
mod summary_test;
