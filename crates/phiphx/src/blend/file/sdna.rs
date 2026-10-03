//! The DNA's layout, worked out once: each type's size, and for each
//! struct the offset and size of every member, by its full DNA name
//! (`idname[64]`, `*mat`) and its bare one (`idname`). Blender's `SDNA`
//! does the same at file load (`DNA_sdna_from_data` fills `types_size`,
//! and `DNA_struct_member_offset_by_name` is answered from it); before
//! this, every field read walked the type list and the struct's members
//! with string compares — two thirds of a 24-second open of a rigged file.

use super::types::{DnaStruct, DnaType};
use std::collections::HashMap;

/// One struct's members laid out.
#[derive(Debug, Clone, Default)]
pub struct StructLayout {
    /// Name (full and bare) → (offset, size), the first member of a name winning.
    pub fields: HashMap<String, (usize, usize)>,
    /// Members in order, with their full names and offsets (the loose,
    /// `contains` lookup walks these).
    pub order: Vec<(String, usize)>,
    pub size: usize,
}

#[derive(Debug, Clone)]
pub struct DnaLayout {
    pub ptr_size: usize,
    /// Type name → index into `DnaType::types`.
    pub types: HashMap<String, usize>,
    /// Struct type index → its layout.
    pub structs: HashMap<usize, StructLayout>,
}

/// The element count a DNA name's dimensions multiply to (`[3][2]` → 6).
pub fn dims(name: &str) -> usize {
    let mut n = 1;
    let mut rest = name;
    while let Some(open) = rest.find('[') {
        let Some(close) = rest[open..].find(']') else { break };
        if let Ok(k) = rest[open + 1..open + close].parse::<usize>() {
            n *= k;
        }
        rest = &rest[open + close + 1..];
    }
    n
}

/// A member's name without its dimensions (`idname[64]` → `idname`).
pub fn bare(name: &str) -> &str {
    name.split('[').next().unwrap_or(name)
}

impl DnaLayout {
    pub fn build(dna: &DnaType, ptr_size: usize) -> Self {
        let types: HashMap<String, usize> = dna.types.iter().enumerate().map(|(i, t)| (t.clone(), i)).collect();
        let size_of = |type_index: usize, name: &str| -> usize {
            // A function pointer is one pointer; a pointer (array) is pointer-sized elements.
            if name.contains("(*") {
                return ptr_size;
            }
            let base = if name.starts_with('*') { ptr_size } else { dna.type_sizes.get(type_index).copied().unwrap_or(0) as usize };
            base * dims(name)
        };
        let structs = dna
            .structs
            .iter()
            .map(|s: &DnaStruct| {
                let mut layout = StructLayout::default();
                let mut offset = 0;
                for f in &s.fields {
                    let name = dna.names.get(f.name_index as usize).cloned().unwrap_or_default();
                    let size = size_of(f.type_index as usize, &name);
                    layout.fields.entry(name.clone()).or_insert((offset, size));
                    if name.contains('[') {
                        layout.fields.entry(bare(&name).to_string()).or_insert((offset, size));
                    }
                    layout.order.push((name, offset));
                    offset += size;
                }
                layout.size = offset;
                (s.type_index as usize, layout)
            })
            .collect();
        Self { ptr_size, types, structs }
    }

    pub fn type_index(&self, type_name: &str) -> Option<usize> {
        self.types.get(type_name).copied()
    }

    pub fn structure(&self, struct_name: &str) -> Option<&StructLayout> {
        self.structs.get(&self.type_index(struct_name)?)
    }

    /// `DNA_struct_member_offset_by_name`: by the exact name, or the bare
    /// name of an array member.
    pub fn offset(&self, struct_name: &str, field: &str) -> Option<usize> {
        self.structure(struct_name)?.fields.get(field).map(|(off, _)| *off)
    }

    /// `DNA_struct_size`.
    pub fn struct_size(&self, struct_name: &str) -> Option<usize> {
        self.structure(struct_name).map(|s| s.size)
    }

    /// The size of a member of `type_name` named `name` (dimensions and
    /// pointers counted).
    pub fn field_size(&self, type_name: &str, name: &str, dna: &DnaType) -> usize {
        if name.contains("(*") {
            return self.ptr_size;
        }
        let base = if name.starts_with('*') { self.ptr_size } else { self.type_index(type_name).and_then(|i| dna.type_sizes.get(i)).copied().unwrap_or(0) as usize };
        base * dims(name)
    }
}

impl DnaType {
    /// The layout for pointers of `ptr_size` bytes, built on first use and
    /// kept with the DNA (a file has one pointer size).
    pub fn layout(&self, ptr_size: usize) -> std::borrow::Cow<'_, DnaLayout> {
        let built = self.layout.get_or_init(|| DnaLayout::build(self, ptr_size));
        if built.ptr_size == ptr_size { std::borrow::Cow::Borrowed(built) } else { std::borrow::Cow::Owned(DnaLayout::build(self, ptr_size)) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blend::file::types::DnaField;

    fn dna() -> DnaType {
        DnaType {
            names: vec!["x".into(), "name[8]".into(), "*next".into(), "mat[4][4]".into(), "(*fn)()".into()],
            types: vec!["int".into(), "char".into(), "Thing".into(), "float".into()],
            type_sizes: vec![4, 1, 0, 4],
            structs: vec![DnaStruct { type_index: 2, fields: vec![DnaField { type_index: 0, name_index: 0 }, DnaField { type_index: 1, name_index: 1 }, DnaField { type_index: 2, name_index: 2 }, DnaField { type_index: 3, name_index: 3 }, DnaField { type_index: 0, name_index: 4 }] }],
            layout: Default::default(),
        }
    }

    #[test]
    fn members_are_laid_out_once_by_full_and_bare_name() {
        let dna = dna();
        let l = dna.layout(8);
        assert_eq!(l.offset("Thing", "x"), Some(0));
        assert_eq!((l.offset("Thing", "name[8]"), l.offset("Thing", "name")), (Some(4), Some(4)), "an array by either spelling");
        assert_eq!(l.offset("Thing", "*next"), Some(12), "the pointer by its DNA name");
        assert_eq!(l.offset("Thing", "mat"), Some(20));
        assert_eq!(l.struct_size("Thing"), Some(20 + 64 + 8), "a 4 × 4 of floats and a function pointer");
        assert_eq!(l.offset("Thing", "nope"), None);
        assert_eq!(dna.layout(4).offset("Thing", "mat"), Some(16), "another pointer size is answered too");
        assert_eq!(dims("[3][2]"), 6);
    }
}
