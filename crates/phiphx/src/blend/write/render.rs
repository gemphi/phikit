//! Properties ▸ Render and ▸ Output written back: the engine
//! (`Scene.r.engine`), the output path and image format (`Scene.r.pic`,
//! `Scene.r.im_format`), and the Cycles settings, which are ID properties
//! on `scene["cycles"]` (`idprop.rs`) rather than DNA fields.
//!
//! An ID property that is in the file is patched where it stands. One that
//! is not — Blender writes only the values that differ from the RNA default
//! — is a new `IDProperty` block at the end of the scene's run, linked at
//! the end of the group's list, with the group's `len` one more
//! (`IDP_AddToGroup`). The group lives under `ID.system_properties` in a
//! 4.5+ layout and under `ID.properties` before that (`idprop.rs`) — a
//! group written under the old root in a 5.0 file reads as a custom
//! property and the Render panel never sees it. A file with no `cycles`
//! group — one saved without the Cycles add-on's settings ever touched, as
//! the 3.x startup file was — gets the group, and a file with no ID
//! properties at all gets the root group first (`IDP_EnsureProperties`,
//! named "IDProperties"). Blender reads a bool as `IDP_BOOLEAN` (5.x) and,
//! in older files, as an int; a new one is written as a boolean, an old int
//! keeps its type. A string (the denoiser, the pixel filter type) keeps its
//! text in a second block the property points at
//! (`IDPropertyData.pointer`), `len` counting the terminator: a text that
//! fits the block it has is written there, a longer one gets a new block.

use crate::blend::BIdValue;
use crate::blend::file::types::*;
use crate::blend::modifier::armature::name64;
use crate::blend::{BlendImageFormat, BlendOutput};

/// `eIDPropertyType`.
const IDP_STRING: u8 = 0;
const IDP_INT: u8 = 1;
const IDP_FLOAT: u8 = 2;
const IDP_GROUP: u8 = 6;
const IDP_BOOLEAN: u8 = 10;

impl BlendImageFormat {
    /// `R_IMF_IMTYPE_*`, the inverse of [`BlendImageFormat::of`].
    pub fn imtype(self) -> i8 {
        match self {
            Self::Png => 17,
            Self::OpenExr => 23,
            Self::OpenExrMultiLayer => 28,
            Self::Jpeg => 4,
            Self::Tiff => 22,
            Self::Bmp => 20,
            Self::Targa => 14,
            Self::RadianceHdr => 21,
            Self::WebP => 35,
            Self::Other(raw) => raw,
        }
    }
}

impl BlendFile {
    /// `Scene.r.pic` and `Scene.r.im_format` from `out`; the fields written.
    pub(crate) fn write_output(&mut self, scene: usize, out: &BlendOutput) -> usize {
        // One-of-many flags (`eImageFormatDepth`), not counts.
        let depth: i64 = match out.depth {
            32 => 1 << 6,
            16 => 1 << 4,
            12 => 1 << 3,
            10 => 1 << 2,
            _ => 1 << 1,
        };
        let mode: i64 = match out.channel {
            1 => 8,
            2 => 16,
            3 => 24,
            _ => 32,
        };
        let mut wrote = usize::from(self.set_text(scene, "Scene", "r.pic", &out.path));
        for (field, value) in [("imtype", i64::from(out.format.imtype())), ("depth", depth), ("quality", i64::from(out.quality)), ("compress", i64::from(out.compress)), ("exr_codec", i64::from(out.exr_codec))] {
            wrote += usize::from(self.set_int(scene, "Scene", &format!("r.im_format.{field}"), value));
        }
        // 5.x renamed `planes` to `color_mode` with the same values.
        if !self.set_int(scene, "Scene", "r.im_format.color_mode", mode) {
            wrote += usize::from(self.set_int(scene, "Scene", "r.im_format.planes", mode));
        } else {
            wrote += 1;
        }
        wrote
    }

    /// `scene["cycles"][name] = value` for each of `values`; the properties
    /// written. A file without the group, or without ID properties at all,
    /// gets them.
    pub(crate) fn write_cycles(&mut self, scene: usize, dna: &DnaType, values: &[(String, BIdValue)], warnings: &mut Vec<String>) -> usize {
        if values.is_empty() {
            return 0;
        }
        let Some(group) = self.ensure_id_group(scene, dna, "cycles") else {
            warnings.push("The file's layout has no IDProperty struct to write Cycles settings in".to_string());
            return 0;
        };
        let owner = self.blocks[scene].old_address;
        let mut wrote = 0;
        for (name, value) in values {
            let ty = match value {
                BIdValue::Bool(_) => IDP_BOOLEAN,
                BIdValue::Float(_) => IDP_FLOAT,
                BIdValue::Int(_) => IDP_INT,
                BIdValue::Str(_) => IDP_STRING,
                _ => continue,
            };
            let found = self.id_children(group, dna).into_iter().find(|&i| name64(&self.blocks[i].data, self.off("IDProperty", "name[64]", dna).unwrap_or(0)) == *name);
            let prop = match found {
                Some(prop) => prop,
                None => match self.push_id_node(owner, Some(group), dna, name, ty) {
                    Some(prop) => prop,
                    None => continue,
                },
            };
            let ty = self.off("IDProperty", "type", dna).and_then(|o| self.blocks[prop].data.get(o).copied()).unwrap_or(IDP_INT);
            if let BIdValue::Str(text) = value {
                wrote += usize::from(ty == IDP_STRING && self.set_id_text(owner, prop, dna, text));
                continue;
            }
            let val: i64 = match (value, ty) {
                (BIdValue::Float(f), IDP_FLOAT) => i64::from((*f as f32).to_bits()),
                (BIdValue::Float(f), _) => *f as i64,
                (BIdValue::Bool(b), _) => i64::from(*b),
                (BIdValue::Int(i), IDP_FLOAT) => i64::from((*i as f32).to_bits()),
                (BIdValue::Int(i), _) => i64::from(*i),
                _ => continue,
            };
            wrote += usize::from(self.set_int(prop, "IDProperty", "data.val", val));
        }
        wrote
    }

    /// `text` as the string property `prop`'s value: into the block it
    /// points at when that is long enough, else into a new one in `owner`'s
    /// run.
    fn set_id_text(&mut self, owner: u64, prop: usize, dna: &DnaType, text: &str) -> bool {
        let (ps, le) = (self.header.pointer_size as usize, self.header.little_endian);
        let mut bytes = text.as_bytes().to_vec();
        bytes.push(0);
        let Some((ptr_o, _)) = self.field_span("IDProperty", "data.*pointer", dna) else { return false };
        let held = self.index_at(Self::read_ptr(&self.blocks[prop].data, ptr_o, ps, le)).filter(|&b| self.blocks[b].data.len() >= bytes.len());
        let prop_address = self.blocks[prop].old_address;
        let (prop, buffer) = match held {
            Some(b) => (prop, b),
            None => {
                let Some(address) = self.push_owned(owner, None, 1, bytes.clone()) else { return false };
                let (Some(prop), Some(buffer)) = (self.index_at(prop_address), self.index_at(address)) else { return false };
                self.set_ptr(prop, "IDProperty", "data.*pointer", address);
                (prop, buffer)
            }
        };
        let data = &mut self.blocks[buffer].data;
        data.fill(0);
        data[..bytes.len()].copy_from_slice(&bytes);
        let total = data.len() as i64;
        self.set_int(prop, "IDProperty", "len", bytes.len() as i64);
        self.set_int(prop, "IDProperty", "totallen", total);
        true
    }

    /// The block of the `IDP_GROUP` named `name` among the ID's properties,
    /// made — with the root group before it — when the file has none.
    fn ensure_id_group(&mut self, id: usize, dna: &DnaType, name: &str) -> Option<usize> {
        let (ps, le) = (self.header.pointer_size as usize, self.header.little_endian);
        let owner = self.blocks[id].old_address;
        let field = self.system_root_field(dna);
        let root_o = self.off("ID", field, dna)?;
        let root = match self.index_at(Self::read_ptr(&self.blocks[id].data, root_o, ps, le)) {
            Some(root) => root,
            None => {
                let root = self.push_id_node(owner, None, dna, "IDProperties", IDP_GROUP)?;
                let (id, address) = (self.index_at(owner)?, self.blocks[root].old_address);
                self.set_ptr(id, "ID", field, address);
                root
            }
        };
        let name_o = self.off("IDProperty", "name[64]", dna)?;
        match self.id_children(root, dna).into_iter().find(|&i| name64(&self.blocks[i].data, name_o) == name) {
            Some(group) => Some(group),
            None => self.push_id_node(owner, Some(root), dna, name, IDP_GROUP),
        }
    }

    /// The blocks of a group property's children, in list order.
    fn id_children(&self, group: usize, dna: &DnaType) -> Vec<usize> {
        let (ps, le) = (self.header.pointer_size as usize, self.header.little_endian);
        let Some((first_o, _)) = self.field_span("IDProperty", "data.group.*first", dna) else { return Vec::new() };
        let Some(next_o) = self.off("IDProperty", "*next", dna) else { return Vec::new() };
        let mut out = Vec::new();
        let mut at = Self::read_ptr(&self.blocks[group].data, first_o, ps, le);
        while at != 0 && out.len() < 4096 {
            let Some(i) = self.index_at(at) else { break };
            out.push(i);
            at = Self::read_ptr(&self.blocks[i].data, next_o, ps, le);
        }
        out
    }

    /// A new property `name` of type `ty` in `owner`'s run, at the end of
    /// `group`'s list when there is one; its block.
    fn push_id_node(&mut self, owner: u64, group: Option<usize>, dna: &DnaType, name: &str, ty: u8) -> Option<usize> {
        let group_address = group.map(|g| self.blocks[g].old_address);
        let address = self.push_owned(owner, Some("IDProperty"), 1, Vec::new())?;
        // `push_owned` moved the blocks about: find both again.
        let prop = self.index_at(address)?;
        self.set_int(prop, "IDProperty", "type", i64::from(ty));
        self.set_text(prop, "IDProperty", "name", name);
        let Some(group) = group_address.and_then(|a| self.index_at(a)) else { return Some(prop) };
        let (ps, le) = (self.header.pointer_size as usize, self.header.little_endian);
        let last_o = self.field_span("IDProperty", "data.group.*last", dna)?.0;
        let last = Self::read_ptr(&self.blocks[group].data, last_o, ps, le);
        if last == 0 {
            self.set_ptr(group, "IDProperty", "data.group.*first", address);
        } else {
            let last_block = self.index_at(last)?;
            self.set_ptr(last_block, "IDProperty", "*next", address);
            self.set_ptr(prop, "IDProperty", "*prev", last);
        }
        self.set_ptr(group, "IDProperty", "data.group.*last", address);
        let len = i64::from(self.int_field(&self.blocks[group], "IDProperty", "len", dna));
        self.set_int(group, "IDProperty", "len", len + 1);
        Some(prop)
    }
}
