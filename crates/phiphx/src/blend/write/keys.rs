//! Keys a session inserted (`I`, `anim.keyframe_insert`) written into an
//! action the way Blender 5.0 lays a layered action out (read off a file
//! Blender wrote): the slot made if the action has none by that identifier
//! (its ID type from the identifier's first two letters, `OB`); a layer
//! "Layer" with one keyframe strip over all time when the action has none;
//! the strip's channelbag for the slot; each F-curve found by path and
//! index, its keys replaced, or made in its group ("Object Transforms");
//! then the channelbag's curve list in group order, each group's range over
//! it. Every block lands in the action's own run of data blocks, where
//! Blender reads an ID's data. Who animates through the slot is the
//! assignments' business (`action.rs`), written after the keys so a slot the
//! keys made can be given.

use crate::blend::anim::fcurve::BFCurve;
use crate::blend::file::types::*;

/// One slot's curves as the session left them: each whole, keys and
/// Blender's handles (`BFCurve::insert`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlendKeyInsert {
    pub action: String,
    /// The slot's identifier: `OBCube`.
    pub slot: String,
    pub curves: Vec<BFCurve>,
    /// The group a new curve joins (`Object Transforms` for transforms).
    pub group: String,
    /// Curves the session emptied — every key deleted — by path and index:
    /// taken out of the slot's channelbag, as `action.delete` removes an
    /// F-curve left without keys (`ANIM_fcurve_delete_from_animdata`).
    pub removed: Vec<(String, u32)>,
}

/// The flags a new curve, group and slot carry (`FCURVE_VISIBLE |
/// FCURVE_SELECTED`, `AGRP_SELECTED`, `ActionSlot::Flags::Expanded`) and
/// the smoothing a new curve gets (`FCURVE_SMOOTH_CONT_ACCEL`) — as Blender
/// wrote them.
const CURVE_FLAG: i64 = 3;
const GROUP_FLAG: i64 = 1;
const SLOT_FLAG: i64 = 1;
const SMOOTH_CONT_ACCEL: i64 = 1;

impl BlendFile {
    /// Write `insert`'s curves into its action through its slot (see the
    /// module).
    pub fn write_keys(&mut self, insert: &BlendKeyInsert) -> Result<(), String> {
        let dna = self.dna.clone().ok_or("the file carries no DNA")?;
        if self.off("bAction", "**layer_array", &dna).is_none() {
            return Err("the file is older than Blender 4.4's layered actions".into());
        }
        let bag_st = if dna.types.iter().any(|t| t == "ActionChannelbag") { "ActionChannelbag" } else { "ActionChannelBag" };
        let act_i = self.find_id("AC", &insert.action).ok_or("the action is not in the file")?;
        let act = self.blocks[act_i].old_address;
        let fail = || format!("'{}' could not be laid out", insert.action);

        // The slot: the one by that identifier, else a new one.
        let handle = match self.slot_handle(act_i, &insert.slot) {
            Some(handle) => handle,
            None => {
                let id = insert.slot.as_bytes();
                let idtype = i64::from(u16::from_le_bytes([id.first().copied().unwrap_or(0), id.get(1).copied().unwrap_or(0)]));
                let handle = self.int_at(act, act, "bAction", "last_slot_handle") + 1;
                let mut slots = self.pointer_list(act, act, "bAction", "**slot_array", "slot_array_num");
                let slot = self.push_owned(act, Some("ActionSlot"), 1, Vec::new()).ok_or_else(fail)?;
                self.set_text_any(act, slot, "ActionSlot", &["identifier", "name"], &insert.slot);
                self.set_int_at(act, slot, "ActionSlot", "idtype", idtype);
                self.set_int_at(act, slot, "ActionSlot", "handle", handle);
                self.set_int_at(act, slot, "ActionSlot", "slot_flags", SLOT_FLAG);
                slots.push(slot);
                self.set_pointer_array(act, act, "bAction", "**slot_array", "slot_array_num", &slots).ok_or_else(fail)?;
                self.set_int_at(act, act, "bAction", "last_slot_handle", handle);
                handle
            }
        };

        // The layer, its keyframe strip and the strip's data.
        let layers = self.pointer_list(act, act, "bAction", "**layer_array", "layer_array_num");
        let mut data = self.pointer_list(act, act, "bAction", "**strip_keyframe_data_array", "strip_keyframe_data_array_num");
        let keyframes = match layers.first() {
            Some(&layer) => {
                let strip = *self.pointer_list(act, layer, "ActionLayer", "**strip_array", "strip_array_num").first().ok_or_else(fail)?;
                *data.get(self.int_at(act, strip, "ActionStrip", "data_index") as usize).ok_or_else(fail)?
            }
            None => {
                let kd = self.push_owned(act, Some("ActionStripKeyframeData"), 1, Vec::new()).ok_or_else(fail)?;
                data.push(kd);
                self.set_pointer_array(act, act, "bAction", "**strip_keyframe_data_array", "strip_keyframe_data_array_num", &data).ok_or_else(fail)?;
                let strip = self.push_owned(act, Some("ActionStrip"), 1, Vec::new()).ok_or_else(fail)?;
                self.set_int_at(act, strip, "ActionStrip", "data_index", data.len() as i64 - 1);
                self.set_f32_at(act, strip, "ActionStrip", "frame_start", f32::NEG_INFINITY);
                self.set_f32_at(act, strip, "ActionStrip", "frame_end", f32::INFINITY);
                let layer = self.push_owned(act, Some("ActionLayer"), 1, Vec::new()).ok_or_else(fail)?;
                self.set_text_any(act, layer, "ActionLayer", &["name"], "Layer");
                self.set_f32_at(act, layer, "ActionLayer", "influence", 1.0);
                self.set_pointer_array(act, layer, "ActionLayer", "**strip_array", "strip_array_num", &[strip]).ok_or_else(fail)?;
                self.set_pointer_array(act, act, "bAction", "**layer_array", "layer_array_num", &[layer]).ok_or_else(fail)?;
                kd
            }
        };

        // The slot's channelbag.
        let mut bags = self.pointer_list(act, keyframes, "ActionStripKeyframeData", "**channelbag_array", "channelbag_array_num");
        let bag = match bags.iter().copied().find(|&b| self.int_at(act, b, bag_st, "slot_handle") == handle) {
            Some(bag) => bag,
            None => {
                let bag = self.push_owned(act, Some(bag_st), 1, Vec::new()).ok_or_else(fail)?;
                self.set_int_at(act, bag, bag_st, "slot_handle", handle);
                bags.push(bag);
                self.set_pointer_array(act, keyframes, "ActionStripKeyframeData", "**channelbag_array", "channelbag_array_num", &bags).ok_or_else(fail)?;
                bag
            }
        };

        // The group, the curves.
        let mut groups = self.pointer_list(act, bag, bag_st, "**group_array", "group_array_num");
        let name_of = |file: &Self, g: u64| file.owned(act, g).and_then(|i| file.field_span("bActionGroup", "name", &dna).map(|(o, n)| crate::blend::shade::nodes::read_cstr(&file.blocks[i].data, o, n)));
        let group = match groups.iter().copied().find(|&g| name_of(self, g).as_deref() == Some(insert.group.as_str())) {
            Some(group) => group,
            None => {
                let group = self.push_owned(act, Some("bActionGroup"), 1, Vec::new()).ok_or_else(fail)?;
                self.set_text_any(act, group, "bActionGroup", &["name"], &insert.group);
                self.set_int_at(act, group, "bActionGroup", "flag", GROUP_FLAG);
                self.set_ptr_any(act, group, "bActionGroup", &["*channel_bag", "*channelbag"], bag);
                groups.push(group);
                group
            }
        };
        let mut curves = self.pointer_list(act, bag, bag_st, "**fcurve_array", "fcurve_array_num");
        let address = |file: &Self, fc: u64| {
            let path = file.owned(act, fc).and_then(|i| file.scoped(&file.blocks[i], || file.cstring_at(&file.blocks[i], "FCurve", "*rna_path", &dna)));
            (path.unwrap_or_default(), file.int_at(act, fc, "FCurve", "array_index") as u32)
        };
        // A curve dropped from the list is a block nothing points at, which
        // Blender's reader never reads.
        curves.retain(|&fc| !insert.removed.contains(&address(self, fc)));
        for curve in &insert.curves {
            let found = curves.iter().copied().find(|&fc| address(self, fc) == (curve.path.clone(), curve.index));
            let fc = match found {
                Some(fc) => fc,
                None => {
                    let fc = self.push_owned(act, Some("FCurve"), 1, Vec::new()).ok_or_else(fail)?;
                    let mut path = curve.path.clone().into_bytes();
                    path.push(0);
                    let rna = self.push_owned(act, None, 1, path).ok_or_else(fail)?;
                    self.set_ptr_any(act, fc, "FCurve", &["*rna_path"], rna);
                    self.set_ptr_any(act, fc, "FCurve", &["*grp"], group);
                    self.set_int_at(act, fc, "FCurve", "array_index", i64::from(curve.index));
                    self.set_int_at(act, fc, "FCurve", "flag", CURVE_FLAG);
                    self.set_int_at(act, fc, "FCurve", "auto_smoothing", SMOOTH_CONT_ACCEL);
                    self.set_int_at(act, fc, "FCurve", "color_mode", i64::from(curve.color_mode));
                    // After the group's last curve, else at the end.
                    let at = curves.iter().rposition(|&c| self.ptr_at(act, c, "FCurve", "*grp") == group).map_or(curves.len(), |p| p + 1);
                    curves.insert(at, fc);
                    fc
                }
            };
            let old = self.ptr_at(act, fc, "FCurve", "*bezt");
            let before = self.run_block(act, old).filter(|_| old != 0).map_or(Vec::new(), |i| self.blocks[i].data.clone());
            let bezt = self.bezt_bytes(curve, &before).ok_or_else(fail)?;
            match self.run_block(act, old).filter(|_| old != 0) {
                Some(i) => {
                    let b = &mut self.blocks[i];
                    (b.size, b.count, b.data) = (bezt.len() as u64, curve.keys.len() as u64, bezt);
                }
                None => {
                    let made = self.push_owned(act, Some("BezTriple"), curve.keys.len(), Vec::new()).ok_or_else(fail)?;
                    let i = self.run_block(act, made).ok_or_else(fail)?;
                    self.blocks[i].data = bezt;
                    self.set_ptr_any(act, fc, "FCurve", &["*bezt"], made);
                }
            }
            self.set_int_at(act, fc, "FCurve", "totvert", curve.keys.len() as i64);
            self.set_int_at(act, fc, "FCurve", "active_keyframe_index", curve.keys.len() as i64 - 1);
        }

        // The curve list in group order, each group's range over it, and
        // the legacy links Blender still writes beside it.
        let order = |file: &Self, fc: u64| groups.iter().position(|&g| g == file.ptr_at(act, fc, "FCurve", "*grp")).unwrap_or(groups.len());
        let mut ranked: Vec<(usize, usize, u64)> = curves.iter().enumerate().map(|(k, &fc)| (order(self, fc), k, fc)).collect();
        ranked.sort();
        let curves: Vec<u64> = ranked.iter().map(|&(_, _, fc)| fc).collect();
        for (k, &fc) in curves.iter().enumerate() {
            self.set_ptr_any(act, fc, "FCurve", &["*prev"], if k > 0 { curves[k - 1] } else { 0 });
            self.set_ptr_any(act, fc, "FCurve", &["*next"], curves.get(k + 1).copied().unwrap_or(0));
        }
        for (g, &group) in groups.iter().enumerate() {
            let members: Vec<usize> = ranked.iter().enumerate().filter(|(_, r)| r.0 == g).map(|(k, _)| k).collect();
            let (start, length) = (members.first().copied().unwrap_or(0), members.len());
            self.set_int_at(act, group, "bActionGroup", "fcurve_range_start", start as i64);
            self.set_int_at(act, group, "bActionGroup", "fcurve_range_length", length as i64);
            if let (Some(i), Some((o, _))) = (self.owned(act, group), self.field_span("bActionGroup", "channels", &dna)) {
                let (first, last) = members.first().zip(members.last()).map_or((0, 0), |(&a, &b)| (curves[a], curves[b]));
                let bytes = self.pointer_bytes(&[first, last]);
                self.blocks[i].data[o..o + bytes.len()].copy_from_slice(&bytes);
            }
        }
        self.set_pointer_array(act, bag, bag_st, "**group_array", "group_array_num", &groups).ok_or_else(fail)?;
        self.set_pointer_array(act, bag, bag_st, "**fcurve_array", "fcurve_array_num", &curves).ok_or_else(fail)?;
        Ok(())
    }
}
