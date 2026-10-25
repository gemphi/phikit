//! Reading `bAction` blocks — Blender 4.4+ layered actions
//! (`layer_array → strip_array → strip_keyframe_data_array → channelbag_array
//! → fcurve_array`, each channelbag bound to a slot) and the legacy
//! `curves` list — and which action and slot an object uses
//! (`Object.adt → AnimData.action / slot_handle`).

use super::fcurve::{BFCurve, BKey, HANDLE_AUTO_ANIM};
use crate::blend::file::types::*;
use std::collections::HashMap;

/// An action's curves, grouped by the slot they animate. Legacy actions have
/// one group under slot 0.
#[derive(Clone, Debug, Default)]
pub struct BAction {
    pub name: String,
    pub by_slot: HashMap<i32, Vec<BFCurve>>,
    /// `bAction.slot_array`: what each slot animates, by its identifier
    /// (`OBCube` — the ID code, then the name). Empty for a legacy action.
    pub slots: Vec<BActionSlot>,
    /// `Manual Frame Range` (`ACT_FRAME_RANGE`) with its `frame_start` /
    /// `frame_end`, when set.
    pub manual_range: Option<(f32, f32)>,
    /// `use_cyclic` (`ACT_CYCLIC`).
    pub cyclic: bool,
}

/// One `ActionSlot`: its handle, which channelbags and `AnimData.slot_handle`
/// name, and its identifier.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BActionSlot {
    pub handle: i32,
    pub identifier: String,
}

/// `eAction_Flags` (`DNA_action_types.h`).
pub const ACT_FRAME_RANGE: u32 = 1 << 12;
pub const ACT_CYCLIC: u32 = 1 << 13;

impl BAction {
    /// The curves for `slot`, or the only slot's when the handle is unknown.
    pub fn curves(&self, slot: i32) -> &[BFCurve] {
        self.by_slot
            .get(&slot)
            .or_else(|| (self.by_slot.len() == 1).then(|| self.by_slot.values().next()).flatten())
            .map_or(&[], Vec::as_slice)
    }
}

/// Which action, and which slot in it, an object's `AnimData` points at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BBinding {
    pub action: u64,
    pub slot: i32,
}

impl BlendFile {
    /// A pointer array of `n` block addresses at `field` of `block`.
    pub(crate) fn ptr_array(&self, block: &BlendBlock, st: &str, field: &str, count: &str, dna: &DnaType) -> Vec<u64> {
        let n = self.int_field(block, st, count, dna) as usize;
        let Some(arr) = self.block_at(self.ptr(block, self.off(st, field, dna))) else { return Vec::new() };
        let ps = self.header.pointer_size as usize;
        (0..n.min(arr.data.len() / ps.max(1)))
            .map(|i| Self::read_ptr(&arr.data, i * ps, ps, self.header.little_endian))
            .collect()
    }

    /// One `FCurve` block: its path, index and keys.
    pub(crate) fn fcurve(&self, fc: &BlendBlock, dna: &DnaType) -> Option<BFCurve> {
        let path = self.cstring_at(fc, "FCurve", "*rna_path", dna)?;
        let index = self.int_field(fc, "FCurve", "array_index", dna);
        let n = self.int_field(fc, "FCurve", "totvert", dna) as usize;
        let size = Self::struct_size("BezTriple", dna, self.header.pointer_size as usize)?;
        let (vec_o, ipo_o) = (self.off("BezTriple", "vec[3][3]", dna)?, self.off("BezTriple", "ipo", dna)?);
        let byte = |field: &str| self.off("BezTriple", field, dna);
        let (f1_o, f2_o, f3_o) = (byte("f1"), byte("f2"), byte("f3"));
        let (h1_o, h2_o) = (byte("h1"), byte("h2"));
        let easing_o = byte("easing");
        let (back_o, amplitude_o, period_o) = (byte("back"), byte("amplitude"), byte("period"));
        let keys = match self.block_at(self.ptr(fc, self.off("FCurve", "*bezt", dna))) {
            Some(b) => (0..n)
                .filter_map(|i| {
                    let d = b.data.get(i * size..(i + 1) * size)?;
                    let f = |k: usize| -> Option<f32> { Some(self.read_f32(d.get(vec_o + k * 4..vec_o + k * 4 + 4)?)) };
                    let selected = |o: Option<usize>| o.and_then(|o| d.get(o)).is_some_and(|flag| flag & 1 != 0);
                    let handle = |o: Option<usize>| o.and_then(|o| d.get(o)).copied().unwrap_or(HANDLE_AUTO_ANIM);
                    let float = |o: Option<usize>, dflt: f32| o.and_then(|o| d.get(o..o + 4)).map_or(dflt, |b| self.read_f32(b));
                    let defaults = BKey::new([0.0; 2], [0.0; 2], [0.0; 2], 2);
                    Some(BKey {
                        h1: [f(0)?, f(1)?],
                        co: [f(3)?, f(4)?],
                        h2: [f(6)?, f(7)?],
                        ipo: d.get(ipo_o).copied().unwrap_or(2),
                        selected: selected(f2_o),
                        h1_type: handle(h1_o),
                        h2_type: handle(h2_o),
                        h1_selected: selected(f1_o),
                        h2_selected: selected(f3_o),
                        easing: easing_o.and_then(|o| d.get(o)).copied().unwrap_or(0),
                        back: float(back_o, defaults.back),
                        amplitude: float(amplitude_o, defaults.amplitude),
                        period: float(period_o, defaults.period),
                    })
                })
                .collect(),
            None => Vec::new(),
        };
        let color = |k: usize| {
            self.off("FCurve", "color[3]", dna).and_then(|o| fc.data.get(o + k * 4..o + k * 4 + 4)).map_or(0.0, |b| self.read_f32(b))
        };
        Some(BFCurve {
            path,
            index,
            keys,
            flag: self.int_field(fc, "FCurve", "flag", dna),
            color_mode: self.int_field(fc, "FCurve", "color_mode", dna),
            color: [color(0), color(1), color(2)],
            generator: self.generator(fc, dna),
            extend: self.off("FCurve", "extend", dna).and_then(|o| fc.data.get(o)).copied().unwrap_or(0),
            group: self.fcurve_group(fc, dna),
        })
    }

    /// The name of the `bActionGroup` an F-curve belongs to (`FCurve.grp`),
    /// the same pointer in a legacy action and in a layered one's channelbag.
    fn fcurve_group(&self, fc: &BlendBlock, dna: &DnaType) -> Option<String> {
        let group = self.block_at(self.ptr(fc, self.off("FCurve", "*grp", dna)))?;
        let (o, n) = self.field_span("bActionGroup", "name", dna)?;
        Some(crate::blend::shade::nodes::read_cstr(&group.data, o, n)).filter(|name| !name.is_empty())
    }

    /// `FCURVE.modifiers`: the first Generator in polynomial mode, as its
    /// coefficients (`FMODIFIER_TYPE_GENERATOR` = 1, `FCM_GENERATOR_POLYNOMIAL` = 0).
    fn generator(&self, fc: &BlendBlock, dna: &DnaType) -> Option<Vec<f32>> {
        let first = self.ptr(fc, self.off("FCurve", "modifiers", dna));
        self.list(first, self.off("FModifier", "*next", dna)).into_iter().find_map(|modifier| {
            if self.int_field(modifier, "FModifier", "type", dna) != 1 {
                return None;
            }
            let data = self.block_at(self.ptr(modifier, self.off("FModifier", "*data", dna)))?;
            if self.int_field(data, "FMod_Generator", "mode", dna) != 0 {
                return None;
            }
            let count = self.int_field(data, "FMod_Generator", "arraysize", dna) as usize;
            let array = self.block_at(self.ptr(data, self.off("FMod_Generator", "*coefficients", dna)))?;
            Some((0..count.min(array.data.len() / 4)).map(|i| self.read_f32(&array.data[i * 4..i * 4 + 4])).collect())
        })
    }

    /// `AnimData.drivers`: the driver F-curves of `ob`.
    pub(crate) fn drivers(&self, ob: &BlendBlock, dna: &DnaType) -> Vec<BFCurve> {
        self.id_drivers(ob, "Object", dna)
    }

    /// `AnimData.drivers` of any ID (`st.adt`). Pointer lookups are scoped
    /// to the ID, as an action's are to the action: a file that reuses an
    /// old address across IDs resolves to this ID's copy.
    pub(crate) fn id_drivers(&self, id: &BlendBlock, st: &str, dna: &DnaType) -> Vec<BFCurve> {
        self.scoped(id, || {
            let Some(adt) = self.block_at(self.ptr(id, self.off(st, "*adt", dna))) else { return Vec::new() };
            let first = self.ptr(adt, self.off("AnimData", "drivers", dna));
            self.list(first, self.off("FCurve", "*next", dna)).into_iter().filter_map(|fc| self.fcurve(fc, dna)).collect()
        })
    }

    /// Every action in the file, by block address.
    pub(crate) fn actions(&self, dna: &DnaType) -> HashMap<u64, BAction> {
        let mut out = HashMap::new();
        for ac in self.blocks.iter().filter(|b| b.code_str == "AC") {
            let mut action = BAction { name: self.id_name(ac, dna), ..Default::default() };
            let flag = self.int_field(ac, "bAction", "flag", dna);
            let range = |f: &str| self.off("bAction", f, dna).and_then(|o| ac.data.get(o..o + 4)).map_or(0.0, |b| self.read_f32(b));
            action.manual_range = (flag & ACT_FRAME_RANGE != 0).then(|| (range("frame_start"), range("frame_end")));
            action.cyclic = flag & ACT_CYCLIC != 0;
            // The file's DNA keeps the names these had when first written:
            // `ActionChannelBag`, and a slot's identifier as `name`.
            let bag_st = if dna.types.iter().any(|t| t == "ActionChannelbag") { "ActionChannelbag" } else { "ActionChannelBag" };
            self.scoped(ac, || {
                let identifier = self.field_span("ActionSlot", "identifier", dna).or_else(|| self.field_span("ActionSlot", "name", dna));
                for slot in self.ptr_array(ac, "bAction", "**slot_array", "slot_array_num", dna) {
                    let Some(slot) = self.block_at(slot) else { continue };
                    let identifier = identifier.map_or(String::new(), |(o, n)| crate::blend::shade::nodes::read_cstr(&slot.data, o, n));
                    action.slots.push(BActionSlot { handle: self.int_field(slot, "ActionSlot", "handle", dna) as i32, identifier });
                }
                // Layered: every strip's keyframe data, every channelbag.
                let data = self.ptr_array(ac, "bAction", "**strip_keyframe_data_array", "strip_keyframe_data_array_num", dna);
                for layer in self.ptr_array(ac, "bAction", "**layer_array", "layer_array_num", dna) {
                    let Some(layer) = self.block_at(layer) else { continue };
                    for strip in self.ptr_array(layer, "ActionLayer", "**strip_array", "strip_array_num", dna) {
                        let Some(strip) = self.block_at(strip) else { continue };
                        let idx = self.int_field(strip, "ActionStrip", "data_index", dna) as usize;
                        let Some(kd) = data.get(idx).and_then(|&a| self.block_at(a)) else { continue };
                        for bag in self.ptr_array(kd, "ActionStripKeyframeData", "**channelbag_array", "channelbag_array_num", dna) {
                            let Some(bag) = self.block_at(bag) else { continue };
                            let slot = self.int_field(bag, bag_st, "slot_handle", dna) as i32;
                            let curves = self
                                .ptr_array(bag, bag_st, "**fcurve_array", "fcurve_array_num", dna)
                                .into_iter()
                                .filter_map(|a| self.fcurve(self.block_at(a)?, dna));
                            action.by_slot.entry(slot).or_default().extend(curves);
                        }
                    }
                }
                // Legacy (≤ 4.3): a plain list of curves.
                if action.by_slot.is_empty() {
                    let first = self.ptr(ac, self.off("bAction", "curves", dna));
                    let curves: Vec<BFCurve> = self
                        .list(first, self.off("FCurve", "*next", dna))
                        .into_iter()
                        .filter_map(|fc| self.fcurve(fc, dna))
                        .collect();
                    if !curves.is_empty() {
                        action.by_slot.insert(0, curves);
                        // Blender makes the legacy curves one slot on load
                        // (`convert_legacy_action`), named for `idroot`.
                        let root = self.int_field(ac, "bAction", "idroot", dna) as u16;
                        let code = if root == 0 { "XX".to_string() } else { String::from_utf8_lossy(&root.to_le_bytes()).to_string() };
                        action.slots.push(BActionSlot { handle: 0, identifier: format!("{code}Legacy Slot") });
                    }
                }
            });
            out.insert(ac.old_address, action);
        }
        out
    }

    /// The action and slot `ob` is bound to, if it has animation data.
    pub(crate) fn binding(&self, ob: &BlendBlock, dna: &DnaType) -> Option<BBinding> {
        self.id_binding(ob, "Object", dna)
    }

    /// The action and slot the `AnimData` of any ID binds (`st` its DNA
    /// struct: `Object`, `Curve`, …).
    pub(crate) fn id_binding(&self, id: &BlendBlock, st: &str, dna: &DnaType) -> Option<BBinding> {
        let adt = self.block_at(self.ptr(id, self.off(st, "*adt", dna)))?;
        let action = self.ptr(adt, self.off("AnimData", "*action", dna));
        (action != 0).then(|| BBinding { action, slot: self.int_field(adt, "AnimData", "slot_handle", dna) as i32 })
    }

    /// `ID.name` without its two-letter code.
    pub(crate) fn id_name(&self, block: &BlendBlock, dna: &DnaType) -> String {
        let Some(o) = self.id_name_offset(dna) else { return String::new() };
        let s = block.data.get(o + 2..(o + 66).min(block.data.len())).unwrap_or(&[]);
        let z = s.iter().position(|&c| c == 0).unwrap_or(s.len());
        String::from_utf8_lossy(&s[..z]).to_string()
    }
}
