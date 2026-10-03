//! Actions a session made, reassigned or retimed — the Action Editor's
//! `template_action` (`action.new`, `action.unlink`, picking an action or
//! a slot) and the sidebar's Action panel — written through
//! (`BlendEdits::actions`, `assigned`, `ranges`).
//!
//! A new action is `BKE_action_add`: an empty `bAction` with Blender's first
//! slot handle. A copy is `BKE_id_copy`: the ID block and every `DATA`
//! block it owns — the run that follows it in the file, which is where
//! Blender writes an ID's data — under fresh addresses, each pointer that
//! named one of them pointing at its copy; pointers to other IDs stay.

use crate::blend::file::types::*;
use std::collections::HashMap;

/// `DNA_DEFAULT_ACTION_LAST_SLOT_HANDLE`: a new action's first slot is one
/// past it (`0x37627bf6`, as every Blender action's first slot).
pub const ACTION_LAST_SLOT_HANDLE: i64 = 0x3762_7bf5;

impl BlendFile {
    /// The `DATA` blocks written for the ID at `id`: the run after it.
    fn owned_run(&self, id: usize) -> std::ops::Range<usize> {
        let end = self.blocks[id + 1..].iter().position(|b| b.code_str != "DATA").map_or(self.blocks.len(), |n| id + 1 + n);
        id + 1..end
    }

    /// Where a new ID goes: before the DNA and the end marker.
    fn id_slot(&self) -> usize {
        self.blocks.iter().position(|b| b.code_str == "DNA1" || b.code_str == "ENDB").unwrap_or(self.blocks.len())
    }

    /// A copy of the ID at `id` and all its data, called `name`
    /// (`BKE_id_copy`); the copy's index and address.
    pub fn copy_id(&mut self, id: usize, name: &str) -> Option<(usize, u64)> {
        let dna = self.dna.clone()?;
        let ps = self.header.pointer_size as usize;
        let le = self.header.little_endian;
        let run = self.owned_run(id);
        let sources: Vec<usize> = std::iter::once(id).chain(run.clone()).collect();
        let mut fresh = self.fresh_address();
        let mut map: HashMap<u64, u64> = HashMap::new();
        let mut copies: Vec<BlendBlock> = Vec::new();
        for &i in &sources {
            let block = self.blocks[i].clone();
            map.insert(block.old_address, fresh);
            copies.push(BlendBlock { old_address: fresh, ..block });
            fresh = (fresh + copies.last().map_or(8, |b| b.size.max(8)) + 0x100) & !0xF;
        }
        let word = |data: &[u8], at: usize| Self::read_ptr(data, at, ps, le);
        let put = |data: &mut [u8], at: usize, value: u64| {
            let bytes = if le { value.to_le_bytes() } else { value.to_be_bytes() };
            match (ps, le) {
                (8, _) => data[at..at + 8].copy_from_slice(&bytes),
                (_, true) => data[at..at + 4].copy_from_slice(&bytes[..4]),
                (_, false) => data[at..at + 4].copy_from_slice(&bytes[4..]),
            }
        };
        for copy in &mut copies {
            // Where the pointers are: a struct's from the DNA, element by
            // element; a raw block's only when every word is a pointer into
            // this ID's data (a pointer array), never a string or floats.
            let mut at: Vec<usize> = Vec::new();
            let struct_name = dna.structs.get(copy.sdna_index as usize).map(|s| dna.types[s.type_index as usize].clone());
            match struct_name.filter(|_| copy.sdna_index != 0) {
                Some(name) => {
                    let size = self.sizeof(&name).unwrap_or(copy.data.len()).max(1);
                    for element in 0..(copy.data.len() / size) {
                        for (off, span) in self.pointer_spans(&name, &dna) {
                            at.extend((0..span / ps).map(|k| element * size + off + k * ps));
                        }
                    }
                }
                None if copy.data.len() % ps == 0 => {
                    let words: Vec<usize> = (0..copy.data.len() / ps).map(|k| k * ps).collect();
                    if words.iter().all(|&w| word(&copy.data, w) == 0 || map.contains_key(&word(&copy.data, w))) {
                        at = words;
                    }
                }
                None => {}
            }
            let len = copy.data.len();
            for w in at.into_iter().filter(|&w| w + ps <= len) {
                if let Some(&to) = map.get(&word(&copy.data, w)) {
                    put(&mut copy.data, w, to);
                }
            }
        }
        let place = run.end;
        let address = copies[0].old_address;
        for (k, copy) in copies.into_iter().enumerate() {
            self.blocks.insert(place + k, copy);
        }
        self.reindex();
        self.set_id_name(place, name);
        Some((place, address))
    }

    /// A new, empty action called `name` (`BKE_action_add`).
    pub fn new_action(&mut self, name: &str) -> Option<(usize, u64)> {
        let sdna_index = self.sdna_index_of("bAction")?;
        let size = self.sizeof("bAction")?;
        let address = self.fresh_address();
        let block = BlendBlock { code: *b"AC\0\0", code_str: "AC".to_string(), size: size as u64, old_address: address, sdna_index, count: 1, data: vec![0; size] };
        let at = self.id_slot();
        self.blocks.insert(at, block);
        self.reindex();
        self.set_id_name(at, name);
        self.set_int(at, "bAction", "last_slot_handle", ACTION_LAST_SLOT_HANDLE);
        Some((at, address))
    }

    /// The handle of the slot called `identifier` in the action at `action`.
    pub fn slot_handle(&self, action: usize, identifier: &str) -> Option<i64> {
        let dna = self.dna.as_ref()?;
        let ac = &self.blocks[action];
        let (off, n) = self.field_span("ActionSlot", "identifier", dna).or_else(|| self.field_span("ActionSlot", "name", dna))?;
        self.scoped(ac, || {
            self.ptr_array(ac, "bAction", "**slot_array", "slot_array_num", dna).into_iter().find_map(|slot| {
                let slot = self.block_at(slot)?;
                (crate::blend::shade::nodes::read_cstr(&slot.data, off, n) == identifier).then(|| i64::from(self.int_field(slot, "ActionSlot", "handle", dna) as i32))
            })
        })
    }

    /// `object` animated by the action at `action` through the slot called
    /// `slot` — none, when the action has no such slot — or, with no
    /// action, unlinked (`action.unlink`). An object with no `AnimData`
    /// gets one (`BKE_animdata_ensure_id`: influence 1).
    pub fn assign_action(&mut self, object: usize, action: Option<usize>, slot: Option<&str>) -> bool {
        let Some(dna) = self.dna.clone() else { return false };
        // The action's address and the slot's handle first: a new
        // `AnimData` moves every block after the object along by one.
        let (address, handle) = match action {
            Some(action) => (self.blocks[action].old_address, slot.and_then(|id| self.slot_handle(action, id)).unwrap_or(0)),
            None => (0, 0),
        };
        let adt_address = self.ptr(&self.blocks[object], self.off("Object", "*adt", &dna));
        let adt = match self.scoped(&self.blocks[object], || self.index_at(adt_address)) {
            Some(adt) if adt_address != 0 => adt,
            _ if action.is_none() => return true,
            _ => {
                let Some((adt, address)) = self.push_block("AnimData", object) else { return false };
                self.set_ptr(object, "Object", "*adt", address);
                self.set_f32(adt, "AnimData", "act_influence", 1.0);
                adt
            }
        };
        self.set_ptr(adt, "AnimData", "*action", address);
        self.set_int(adt, "AnimData", "slot_handle", handle);
        // `last_slot_identifier`, which the file's DNA calls `slot_name`.
        if let (Some(id), true) = (slot, handle != 0) {
            let _ = self.set_text(adt, "AnimData", "last_slot_identifier", id) || self.set_text(adt, "AnimData", "slot_name", id);
        }
        true
    }

    /// Manual Frame Range and Cyclic Animation on the action at `action`.
    pub fn set_action_range(&mut self, action: usize, range: Option<(f32, f32)>, cyclic: bool) -> bool {
        use crate::blend::anim::read::{ACT_CYCLIC, ACT_FRAME_RANGE};
        let Some(dna) = self.dna.clone() else { return false };
        let flag = self.int_field(&self.blocks[action], "bAction", "flag", &dna);
        let bit = |flag: u32, bit: u32, on: bool| if on { flag | bit } else { flag & !bit };
        let flag = bit(bit(flag, ACT_FRAME_RANGE, range.is_some()), ACT_CYCLIC, cyclic);
        let mut done = self.set_int(action, "bAction", "flag", i64::from(flag));
        if let Some((start, end)) = range {
            done &= self.set_f32(action, "bAction", "frame_start", start) && self.set_f32(action, "bAction", "frame_end", end);
        }
        done
    }
}

/// An object's action as the session left it: `action` through `slot`
/// (its identifier), or unlinked when `action` is `None`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlendActionAssign {
    pub object: String,
    pub action: Option<String>,
    pub slot: Option<String>,
}

/// An action's Manual Frame Range (`None` when off) and Cyclic Animation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlendActionRange {
    pub action: String,
    pub range: Option<(f32, f32)>,
    pub cyclic: bool,
}

impl super::edit::BlendEdits {
    /// The actions made, the keys inserted (making the slots they key),
    /// then who uses which, each NLA stack edited, then the ranges. After the
    /// objects' renames: every name here is the one the file ends up with.
    pub(super) fn animate(&self, file: &mut BlendFile, report: &mut super::edit::BlendSaveReport) -> usize {
        let mut done = 0;
        for (name, source) in &self.actions {
            let made = match source.is_empty() {
                true => file.new_action(name),
                false => file.find_id("AC", source).and_then(|at| file.copy_id(at, name)),
            };
            match made {
                Some(_) => done += 1,
                None => report.warnings.push(format!("Action '{name}' not made: '{source}' is not in the file")),
            }
        }
        for insert in &self.keys {
            match file.write_keys(insert) {
                Ok(()) => done += 1,
                Err(why) => report.warnings.push(format!("Keys in '{}' ({}) not written: {why}", insert.action, insert.slot)),
            }
        }
        for edit in &self.assigned {
            let action = edit.action.as_deref().map(|name| (name, file.find_id("AC", name)));
            let (Some(object), Some(action)) = (file.find_id("OB", &edit.object), action.map_or(Some(None), |(_, at)| at.map(Some))) else {
                report.warnings.push(format!("'{}' not given '{}': not both in the file", edit.object, edit.action.as_deref().unwrap_or("")));
                continue;
            };
            if file.assign_action(object, action, edit.slot.as_deref()) {
                done += 1;
            }
        }
        for stack in &self.nla {
            match file.write_nla(stack) {
                Ok(()) => done += 1,
                Err(why) => report.warnings.push(format!("'{}': NLA not written: {why}", stack.object)),
            }
        }
        for edit in &self.ranges {
            match file.find_id("AC", &edit.action) {
                Some(at) if file.set_action_range(at, edit.range, edit.cyclic) => done += 1,
                _ => report.warnings.push(format!("Action '{}': frame range not written", edit.action)),
            }
        }
        done
    }
}
