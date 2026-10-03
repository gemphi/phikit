//! Reading an object's NLA — `Object.adt → AnimData.nla_tracks`, each
//! `NlaTrack` with its `strips`, and the `AnimData` fields the NLA editor
//! draws around them: the active action, how it extends
//! (`act_extendmode`), tweak and solo flags.

use crate::blend::file::types::*;

/// `eNlaStrip_Extrapolate_Mode`.
pub const EXTEND_HOLD: u32 = 0;
pub const EXTEND_HOLD_FORWARD: u32 = 1;
pub const EXTEND_NOTHING: u32 = 2;
/// `eNlaStrip_Type`.
pub const STRIP_CLIP: u32 = 0;
pub const STRIP_TRANSITION: u32 = 1;
pub const STRIP_META: u32 = 2;
pub const STRIP_SOUND: u32 = 3;
/// `eNlaStrip_Flag` bits the editor and the evaluator read.
pub const STRIP_ACTIVE: u32 = 1 << 0;
pub const STRIP_SELECT: u32 = 1 << 1;
pub const STRIP_USR_INFLUENCE: u32 = 1 << 5;
pub const STRIP_USR_TIME_CYCLIC: u32 = 1 << 7;
pub const STRIP_SYNC_LENGTH: u32 = 1 << 9;
pub const STRIP_REVERSE: u32 = 1 << 11;
pub const STRIP_MUTED: u32 = 1 << 12;
/// `eNlaStrip_Blend_Mode`.
pub const BLEND_REPLACE: u32 = 0;
pub const BLEND_ADD: u32 = 1;
pub const BLEND_SUBTRACT: u32 = 2;
pub const BLEND_MULTIPLY: u32 = 3;
pub const BLEND_COMBINE: u32 = 4;
/// `eNlaTrack_Flag`.
pub const TRACK_ACTIVE: u32 = 1 << 0;
pub const TRACK_SELECTED: u32 = 1 << 1;
pub const TRACK_MUTED: u32 = 1 << 2;
pub const TRACK_SOLO: u32 = 1 << 3;
pub const TRACK_PROTECTED: u32 = 1 << 4;
pub const TRACK_OVERRIDELIBRARY_LOCAL: u32 = 1 << 16;
/// `eAnimData_Flag`.
pub const ADT_SOLO_TRACK: u32 = 1 << 0;
pub const ADT_EDIT_ON: u32 = 1 << 2;
/// The action line picked in the NLA editor's channel list.
pub const ADT_UI_SELECTED: u32 = 1 << 14;
pub const ADT_UI_ACTIVE: u32 = 1 << 15;

#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BNlaStrip {
    pub name: String,
    pub start: f32,
    pub end: f32,
    pub actstart: f32,
    pub actend: f32,
    pub repeat: f32,
    pub scale: f32,
    pub flag: u32,
    pub kind: u32,
    pub extend: u32,
    /// The action the clip plays, by name.
    pub action: Option<String>,
    /// `blendmode` (`BLEND_*`) and `influence` (used when the flag
    /// `STRIP_USR_INFLUENCE` is on).
    pub blend: u32,
    pub influence: f32,
    /// `blendin` / `blendout`: frames the influence eases in and out over.
    pub blendin: f32,
    pub blendout: f32,
    /// `action_slot_handle` and the slot's identifier as stored
    /// (`action_slot_name`).
    pub slot: i32,
    pub slot_name: String,
    /// The block it was read from (`0`: made since), so a save keeps
    /// what phiphx does not model — its curves, modifiers, children.
    pub address: u64,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BNlaTrack {
    pub name: String,
    pub flag: u32,
    pub strips: Vec<BNlaStrip>,
    /// The block it was read from, as a strip's.
    pub address: u64,
}

/// One object's `AnimData` as the NLA editor lists it.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BlendNla {
    pub object: String,
    /// `adt->action`'s name — the action line.
    pub action: Option<String>,
    /// `adt->act_extendmode`, `act_blendmode` and `act_influence`: how the
    /// action line mixes in.
    pub extend: u32,
    pub blend: u32,
    pub influence: f32,
    /// `adt->flag`.
    pub flag: u32,
    /// `nla_tracks`, first to last as stored (the editor lists them last first).
    pub tracks: Vec<BNlaTrack>,
}

/// `DNA_DEFAULT_AnimData`: no action, full influence, Replace and Hold.
impl Default for BlendNla {
    fn default() -> Self {
        Self { object: String::new(), action: None, extend: EXTEND_HOLD, blend: BLEND_REPLACE, influence: 1.0, flag: 0, tracks: Vec::new() }
    }
}

impl BlendFile {
    /// A fixed `char name[64]` field.
    fn name_field(&self, block: &BlendBlock, st: &str, dna: &DnaType) -> String {
        let Some(o) = self.off(st, "name[64]", dna) else { return String::new() };
        let s = block.data.get(o..(o + 64).min(block.data.len())).unwrap_or(&[]);
        let z = s.iter().position(|&c| c == 0).unwrap_or(s.len());
        String::from_utf8_lossy(&s[..z]).to_string()
    }

    fn action_name(&self, block: &BlendBlock, st: &str, dna: &DnaType) -> Option<String> {
        let act = self.block_at(self.ptr(block, self.off(st, "*act", dna)))?;
        Some(self.id_name(act, dna))
    }

    fn strip(&self, block: &BlendBlock, dna: &DnaType) -> BNlaStrip {
        let f = |field: &str| self.float_field(block, "NlaStrip", field, dna, 0.0);
        BNlaStrip {
            name: self.name_field(block, "NlaStrip", dna),
            start: f("start"),
            end: f("end"),
            actstart: f("actstart"),
            actend: f("actend"),
            repeat: f("repeat"),
            scale: f("scale"),
            flag: self.int_field(block, "NlaStrip", "flag", dna),
            kind: self.int_field(block, "NlaStrip", "type", dna),
            extend: self.int_field(block, "NlaStrip", "extendmode", dna),
            action: self.action_name(block, "NlaStrip", dna),
            blend: self.int_field(block, "NlaStrip", "blendmode", dna),
            influence: f("influence"),
            blendin: f("blendin"),
            blendout: f("blendout"),
            slot: self.int_field(block, "NlaStrip", "action_slot_handle", dna) as i32,
            slot_name: self
                .field_span("NlaStrip", "action_slot_name", dna)
                .map_or(String::new(), |(o, n)| crate::blend::shade::nodes::read_cstr(&block.data, o, n)),
            address: block.old_address,
        }
    }

    /// The NLA of one object, when it has animation data.
    pub(crate) fn nla(&self, object_name: &str, ob: &BlendBlock, dna: &DnaType) -> Option<BlendNla> {
        let adt = self.block_at(self.ptr(ob, self.off("Object", "*adt", dna)))?;
        let action = self.block_at(self.ptr(adt, self.off("AnimData", "*action", dna))).map(|act| self.id_name(act, dna));
        let tracks = self
            .list(self.ptr(adt, self.off("AnimData", "nla_tracks", dna)), self.off("NlaTrack", "*next", dna))
            .into_iter()
            .map(|track| BNlaTrack {
                name: self.name_field(track, "NlaTrack", dna),
                flag: self.int_field(track, "NlaTrack", "flag", dna),
                strips: self
                    .list(self.ptr(track, self.off("NlaTrack", "strips", dna)), self.off("NlaStrip", "*next", dna))
                    .into_iter()
                    .map(|strip| self.strip(strip, dna))
                    .collect(),
                address: track.old_address,
            })
            .collect();
        Some(BlendNla {
            object: object_name.to_string(),
            action,
            extend: self.int_field(adt, "AnimData", "act_extendmode", dna),
            blend: self.int_field(adt, "AnimData", "act_blendmode", dna),
            influence: self.float_field(adt, "AnimData", "act_influence", dna, 1.0),
            flag: self.int_field(adt, "AnimData", "flag", dna),
            tracks,
        })
    }

    /// Every animated object's NLA, in object order.
    pub(crate) fn nla_list(&self, objects: &[crate::blend::scene::types::BlendSceneObject], dna: &DnaType) -> Vec<BlendNla> {
        objects
            .iter()
            .filter_map(|object| self.nla(&object.name, self.block_at(object.block_address)?, dna))
            .collect()
    }
}
