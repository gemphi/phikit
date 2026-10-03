//! An object's NLA written as the session left it — Push Down, Stash and
//! the NLA editor's edits alike (laid by `anim/nla_stack.rs` and
//! `anim/nla_edit.rs`): the whole stack, in Blender 5.0's layout (read off
//! files Blender saved). A track or strip the file had keeps its block —
//! and with it what phiphx does not model: a strip's control curves,
//! modifiers and children — with its name, flags and place brought up to
//! date; a new one gets an `NlaTrack` / `NlaStrip` block in the object's
//! run, the strip pointing at its action and slot (`act`,
//! `action_slot_handle`, `action_slot_name`); one the session removed
//! loses its block. The lists are relinked (`AnimData.nla_tracks`, each
//! track's `strips`), and `act_track` / `actstrip` let go of a block that
//! is gone. Unassigning the action is the assignments' business
//! (`action.rs`). Writing the same stack twice gives the same file.

use crate::blend::anim::nla::{BNlaStrip, BlendNla};
use crate::blend::file::types::*;

/// One object's NLA as the session has it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlendNlaStack {
    pub object: String,
    pub nla: BlendNla,
}

impl BlendFile {
    /// Point the `ListBase` `st.field` of the block at `address` (in
    /// `owner`'s run) at `first` and `last`.
    fn set_list(&mut self, owner: u64, address: u64, st: &str, field: &str, first: u64, last: u64) -> Option<()> {
        let dna = self.dna.clone()?;
        let (o, _) = self.field_span(st, field, &dna)?;
        let bytes = self.pointer_bytes(&[first, last]);
        let i = self.owned(owner, address)?;
        self.blocks[i].data.get_mut(o..o + bytes.len())?.copy_from_slice(&bytes);
        Some(())
    }

    /// Link `list` (addresses of `st` blocks) through `*next` / `*prev`,
    /// and point `holder`'s `st_holder.field` at its ends.
    fn relink(&mut self, owner: u64, st: &str, list: &[u64], holder: (u64, &str, &str)) -> Option<()> {
        for (k, &at) in list.iter().enumerate() {
            self.set_ptr_any(owner, at, st, &["*prev"], if k > 0 { list[k - 1] } else { 0 });
            self.set_ptr_any(owner, at, st, &["*next"], list.get(k + 1).copied().unwrap_or(0));
        }
        let (address, st_holder, field) = holder;
        self.set_list(owner, address, st_holder, field, list.first().copied().unwrap_or(0), list.last().copied().unwrap_or(0))
    }

    /// A new `NlaStrip` block for `strip` in `owner`'s run; its address.
    fn strip_block(&mut self, owner: u64, strip: &BNlaStrip) -> Option<u64> {
        let action = strip.action.as_deref().and_then(|name| self.find_id("AC", name))?;
        let (act, handle) = (self.blocks[action].old_address, self.slot_handle(action, &strip.slot_name).unwrap_or(0));
        let at = self.push_owned(owner, Some("NlaStrip"), 1, Vec::new())?;
        self.set_ptr_any(owner, at, "NlaStrip", &["*act"], act);
        self.set_int_at(owner, at, "NlaStrip", "action_slot_handle", handle);
        self.set_text_any(owner, at, "NlaStrip", &["action_slot_name", "last_slot_identifier"], &strip.slot_name);
        for (field, value) in [("influence", strip.influence), ("start", strip.start), ("end", strip.end), ("actstart", strip.actstart), ("actend", strip.actend), ("repeat", strip.repeat), ("scale", strip.scale)] {
            self.set_f32_at(owner, at, "NlaStrip", field, value);
        }
        for (field, value) in [("blendmode", strip.blend), ("extendmode", strip.extend), ("type", strip.kind)] {
            self.set_int_at(owner, at, "NlaStrip", field, i64::from(value));
        }
        Some(at)
    }

    /// The blocks of `list` (`st` blocks linked through `*next`) from `first`.
    fn chain_of(&self, first: u64, st: &str, dna: &DnaType) -> Vec<u64> {
        self.list(first, self.off(st, "*next", dna)).iter().map(|b| b.old_address).collect()
    }

    /// Write `stack` as its object's NLA (see the module).
    pub fn write_nla(&mut self, stack: &BlendNlaStack) -> Result<(), String> {
        let dna = self.dna.clone().ok_or("the file carries no DNA")?;
        let object = self.find_id("OB", &stack.object).ok_or("it is not in the file")?;
        // An object given its first NLA gets animation data
        // (`BKE_animdata_ensure_id`).
        if self.ptr(&self.blocks[object], self.off("Object", "*adt", &dna)) == 0 {
            let (adt, address) = self.push_block("AnimData", object).ok_or("its animation data could not be made")?;
            self.set_ptr(object, "Object", "*adt", address);
            self.set_f32(adt, "AnimData", "act_influence", 1.0);
        }
        let object = self.find_id("OB", &stack.object).ok_or("it is not in the file")?;
        let ob = self.blocks[object].old_address;
        let adt = self.ptr(&self.blocks[object], self.off("Object", "*adt", &dna));
        let fail = || format!("the NLA of '{}' could not be written", stack.object);
        let had_tracks = self.chain_of(self.ptr_at(ob, adt, "AnimData", "nla_tracks"), "NlaTrack", &dna);
        let had_strips: Vec<u64> = had_tracks.iter().flat_map(|&t| self.chain_of(self.ptr_at(ob, t, "NlaTrack", "strips"), "NlaStrip", &dna)).collect();
        let (mut tracks, mut kept) = (Vec::new(), Vec::new());
        for (index, track) in stack.nla.tracks.iter().enumerate() {
            let at = match had_tracks.contains(&track.address) && track.address != 0 {
                true => track.address,
                false => self.push_owned(ob, Some("NlaTrack"), 1, Vec::new()).ok_or_else(fail)?,
            };
            self.set_text_any(ob, at, "NlaTrack", &["name"], &track.name);
            self.set_int_at(ob, at, "NlaTrack", "index", index as i64);
            self.set_int_at(ob, at, "NlaTrack", "flag", i64::from(track.flag));
            let mut strips = Vec::new();
            for strip in &track.strips {
                let s = match had_strips.contains(&strip.address) && strip.address != 0 {
                    true => strip.address,
                    false => self.strip_block(ob, strip).ok_or_else(fail)?,
                };
                self.set_text_any(ob, s, "NlaStrip", &["name"], &strip.name);
                self.set_int_at(ob, s, "NlaStrip", "flag", i64::from(strip.flag));
                strips.push(s);
            }
            self.relink(ob, "NlaStrip", &strips, (at, "NlaTrack", "strips")).ok_or_else(fail)?;
            kept.extend(strips);
            kept.push(at);
            tracks.push(at);
        }
        self.relink(ob, "NlaTrack", &tracks, (adt, "AnimData", "nla_tracks")).ok_or_else(fail)?;
        self.set_int_at(ob, adt, "AnimData", "flag", i64::from(stack.nla.flag));
        let gone: Vec<u64> = had_tracks.into_iter().chain(had_strips).filter(|a| !kept.contains(a)).collect();
        for field in ["*act_track", "*actstrip"] {
            if gone.contains(&self.ptr_at(ob, adt, "AnimData", field)) {
                self.set_ptr_any(ob, adt, "AnimData", &[field], 0);
            }
        }
        let drop: Vec<usize> = gone.iter().filter_map(|&a| self.owned(ob, a)).collect();
        self.remove_blocks(drop);
        Ok(())
    }
}
