//! ID properties — Blender `DNA_ID.h` (`IDProperty`, `IDPropertyData`),
//! `DNA_ID_enums.h` (`eIDPropertyType`), `idprop.cc`.
//!
//! Anything an add-on stores on a datablock lives here rather than in the
//! DNA: `scene["cycles"]` is an `IDP_GROUP` whose children are the Cycles
//! settings the Render panel edits. Only values that *differ from the RNA
//! default* are written, so a file usually carries a handful — on Female-01,
//! `samples`, `preview_samples` and `use_light_tree` and nothing else. A
//! reader therefore has to answer "absent" as readily as it answers a value,
//! and the caller falls back to the property's own default.
//!
//! `ID.properties` is the root; groups nest, so a path like
//! `cycles/samples` walks two levels. Blender 4.5 split the root in two:
//! `ID.system_properties` holds what registered RNA property groups store
//! (the Cycles settings among them) and `ID.properties` the user's custom
//! properties; a 3.x or 4.2 file has one root, and Blender moves the
//! registered groups over as it opens one. Both are read here into one
//! group, the system side on top, so `cycles/samples` answers whichever
//! layout the file has.

use crate::blend::file::types::*;
use std::collections::HashMap;

/// `eIDPropertyType`.
const IDP_STRING: u8 = 0;
const IDP_INT: u8 = 1;
const IDP_FLOAT: u8 = 2;
const IDP_ARRAY: u8 = 5;
const IDP_GROUP: u8 = 6;
const IDP_DOUBLE: u8 = 8;
const IDP_BOOLEAN: u8 = 10;

/// One ID property's value.
#[derive(Clone, Debug, PartialEq)]
pub enum BIdValue {
    Bool(bool),
    Int(i32),
    Float(f64),
    Str(String),
    /// `IDP_ARRAY` of numbers, as doubles whatever the element type.
    Array(Vec<f64>),
    Group(BIdGroup),
}

impl BIdValue {
    /// As a whole number, for an int or a bool.
    pub fn as_int(&self) -> Option<i32> {
        match self {
            Self::Int(v) => Some(*v),
            Self::Bool(v) => Some(i32::from(*v)),
            Self::Float(v) => Some(*v as i32),
            _ => None,
        }
    }

    /// As a flag. Blender writes booleans as `IDP_BOOLEAN`, but older files
    /// and some add-ons use an int, so a non-zero int reads as true.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(v) => Some(*v),
            Self::Int(v) => Some(*v != 0),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(v) => Some(*v),
            Self::Int(v) => Some(f64::from(*v)),
            Self::Bool(v) => Some(f64::from(u8::from(*v))),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Str(v) => Some(v),
            _ => None,
        }
    }
}

/// An `IDP_GROUP`: named children, in the order Blender wrote them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BIdGroup {
    values: HashMap<String, BIdValue>,
    order: Vec<String>,
}

impl BIdGroup {
    /// A direct child.
    pub fn get(&self, name: &str) -> Option<&BIdValue> {
        self.values.get(name)
    }

    /// A descendant by `/`-separated path: `at("cycles/samples")`.
    pub fn at(&self, path: &str) -> Option<&BIdValue> {
        let mut parts = path.split('/');
        let mut value = self.get(parts.next()?)?;
        for part in parts {
            value = match value {
                BIdValue::Group(g) => g.get(part)?,
                _ => return None,
            };
        }
        Some(value)
    }

    /// Child names, in file order.
    pub fn names(&self) -> &[String] {
        &self.order
    }

    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    fn insert(&mut self, name: String, value: BIdValue) {
        if !self.values.contains_key(&name) {
            self.order.push(name.clone());
        }
        self.values.insert(name, value);
    }
}

impl BlendFile {
    /// The ID properties of a datablock (`ID.properties`), empty when it has
    /// none. `block` is the datablock itself — its `ID` is the first field.
    pub(crate) fn id_properties(&self, block: &BlendBlock, dna: &DnaType) -> BIdGroup {
        let mut group = BIdGroup::default();
        for field in ["*properties", "*system_properties"] {
            let Some(off) = self.off("ID", field, dna) else { continue };
            let root = self.ptr(block, Some(off));
            if let Some(read) = self.scoped(block, || self.id_group_at(root, dna, 0)) {
                for name in read.names() {
                    if let Some(value) = read.get(name) {
                        group.insert(name.to_string(), value.clone());
                    }
                }
            }
        }
        group
    }

    /// The root the registered property groups live in: `system_properties`
    /// where the file's layout has it (4.5 on), `properties` before that.
    pub(crate) fn system_root_field(&self, dna: &DnaType) -> &'static str {
        if self.off("ID", "*system_properties", dna).is_some() { "*system_properties" } else { "*properties" }
    }

    /// The group an `IDProperty` pointer names, following its children.
    pub(crate) fn id_group_at(&self, addr: u64, dna: &DnaType, depth: u32) -> Option<BIdGroup> {
        let prop = self.block_at(addr)?;
        match self.id_value(prop, dna, depth)? {
            BIdValue::Group(g) => Some(g),
            _ => None,
        }
    }

    /// One `IDProperty` as a value.
    fn id_value(&self, prop: &BlendBlock, dna: &DnaType, depth: u32) -> Option<BIdValue> {
        // A malformed file could point a group at itself; 32 is far deeper
        // than any add-on nests.
        if depth > 32 {
            return None;
        }
        let ty = self.off("IDProperty", "type", dna).and_then(|o| prop.data.get(o).copied())?;
        let len = self.off("IDProperty", "len", dna).map_or(0, |o| self.int_field_at(prop, o)) as usize;
        let data = self.off("IDProperty", "data", dna)?;
        let val = self.off("IDPropertyData", "val", dna).map_or(0, |o| self.int_field_at(prop, data + o));
        Some(match ty {
            IDP_BOOLEAN => BIdValue::Bool(val != 0),
            IDP_INT => BIdValue::Int(val),
            IDP_FLOAT => BIdValue::Float(f64::from(f32::from_bits(val as u32))),
            IDP_DOUBLE => {
                // A double is written across `val` and `val2`.
                let hi = self.off("IDPropertyData", "val2", dna).map_or(0, |o| self.int_field_at(prop, data + o));
                BIdValue::Float(f64::from_bits((u64::from(hi as u32) << 32) | u64::from(val as u32)))
            }
            IDP_STRING => {
                let bytes = self.id_buffer(prop, dna)?;
                // `len` counts the terminator.
                let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
                BIdValue::Str(String::from_utf8_lossy(&bytes[..end.min(len)]).to_string())
            }
            IDP_ARRAY => {
                let sub = self.off("IDProperty", "subtype", dna).and_then(|o| prop.data.get(o).copied())?;
                let bytes = self.id_buffer(prop, dna)?;
                BIdValue::Array(self.id_array(bytes, sub, len))
            }
            IDP_GROUP => {
                let first = self
                    .off("IDPropertyData", "group", dna)
                    .map_or(0, |o| Self::read_ptr(&prop.data, data + o, self.header.pointer_size as usize, self.header.little_endian));
                let name_o = self.off("IDProperty", "name[64]", dna)?;
                let next = self.off("IDProperty", "*next", dna);
                let mut group = BIdGroup::default();
                for child in self.list(first, next) {
                    let name = super::modifier::armature::name64(&child.data, name_o);
                    if let Some(value) = self.id_value(child, dna, depth + 1) {
                        group.insert(name, value);
                    }
                }
                BIdValue::Group(group)
            }
            // `IDP_ID` and `IDP_IDPARRAY` point at datablocks, which nothing
            // here needs yet.
            _ => return None,
        })
    }

    /// The block `IDPropertyData.pointer` names (a string's or array's bytes).
    fn id_buffer(&self, prop: &BlendBlock, dna: &DnaType) -> Option<&[u8]> {
        let data = self.off("IDProperty", "data", dna)?;
        let off = self.off("IDPropertyData", "*pointer", dna)?;
        let addr = Self::read_ptr(&prop.data, data + off, self.header.pointer_size as usize, self.header.little_endian);
        Some(&self.block_at(addr)?.data)
    }

    fn id_array(&self, bytes: &[u8], sub: u8, len: usize) -> Vec<f64> {
        let width = match sub {
            IDP_DOUBLE => 8,
            IDP_BOOLEAN => 1,
            _ => 4,
        };
        (0..len)
            .filter_map(|i| {
                let s = bytes.get(i * width..i * width + width)?;
                Some(match sub {
                    IDP_DOUBLE => f64::from_bits(u64::from_le_bytes(s.try_into().ok()?)),
                    IDP_BOOLEAN => f64::from(s[0]),
                    IDP_FLOAT => f64::from(self.read_f32(s)),
                    _ => f64::from(self.read_i32(s)),
                })
            })
            .collect()
    }

    /// A 32-bit field at a byte offset of `block`'s own data.
    fn int_field_at(&self, block: &BlendBlock, off: usize) -> i32 {
        block.data.get(off..off + 4).map_or(0, |b| self.read_i32(b))
    }
}
