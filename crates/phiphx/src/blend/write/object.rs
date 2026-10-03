//! A new `Object` in the file — `BKE_object_add` as a writer sees it.
//!
//! Rather than spell every default of a struct with a hundred fields per
//! Blender version, the writer clones an object the file already has and
//! clears everything that points anywhere: every pointer field and every
//! `ListBase`, walked from the DNA so a field added in a later Blender is
//! cleared too. What is left are the plain values — draw type, colour,
//! rotation mode, the visibility flags — that a fresh object shares with
//! its neighbours. Then the new object's own values go in: its name, its
//! mesh, its transform.

use crate::blend::file::types::*;

/// `Object.type` for a mesh (`OB_MESH`).
pub const OB_MESH: i64 = 1;

impl BlendFile {
    /// The byte ranges inside a struct that hold a pointer or a `ListBase`
    /// (two pointers), embedded structs walked.
    pub(crate) fn pointer_spans(&self, struct_name: &str, dna: &DnaType) -> Vec<(usize, usize)> {
        let ps = self.header.pointer_size as usize;
        let mut out = Vec::new();
        self.pointer_spans_into(struct_name, 0, dna, ps, &mut out, 0);
        out
    }

    fn pointer_spans_into(&self, struct_name: &str, base: usize, dna: &DnaType, ps: usize, out: &mut Vec<(usize, usize)>, depth: u32) {
        if depth > 8 {
            return;
        }
        let Some(type_idx) = dna.types.iter().position(|t| t == struct_name) else { return };
        let Some(s) = dna.structs.iter().find(|s| s.type_index as usize == type_idx) else { return };
        let mut offset = base;
        for fld in &s.fields {
            let f_name = &dna.names[fld.name_index as usize];
            let f_type = &dna.types[fld.type_index as usize];
            let size = Self::field_size(f_type, f_name, dna, ps);
            if f_name.starts_with('*') || f_name.contains("(*") || f_type == "ListBase" {
                out.push((offset, size));
            } else if dna.structs.iter().any(|st| st.type_index == fld.type_index) && !f_name.contains('[') {
                self.pointer_spans_into(f_type, offset, dna, ps, out, depth + 1);
            }
            offset += size;
        }
    }

    /// A copy of block `template`'s bytes with every pointer and list
    /// cleared, as a new block of the same struct inserted after `after`.
    pub fn clone_block_cleared(&mut self, template: usize, after: usize) -> Option<(usize, u64)> {
        self.clone_block(template, after, true)
    }

    /// A copy of block `template` inserted after `after` under a fresh
    /// address — its pointers cleared when `clear`, kept as they are
    /// otherwise (an array of ID pointers, which may be shared).
    pub(crate) fn clone_block(&mut self, template: usize, after: usize, clear: bool) -> Option<(usize, u64)> {
        let dna = self.dna.clone()?;
        let src = self.blocks.get(template)?;
        let mut data = src.data.clone();
        if clear {
            let struct_name = dna.types[dna.structs.get(src.sdna_index as usize)?.type_index as usize].clone();
            for (off, size) in self.pointer_spans(&struct_name, &dna) {
                if off + size <= data.len() {
                    data[off..off + size].fill(0);
                }
            }
        }
        let code = src.code;
        let code_str = src.code_str.clone();
        let sdna_index = src.sdna_index;
        let address = self.fresh_address();
        let block = BlendBlock { code, code_str, size: data.len() as u64, old_address: address, sdna_index, count: 1, data };
        let at = (after + 1).min(self.blocks.len());
        self.blocks.insert(at, block);
        self.reindex();
        Some((at, address))
    }

    /// The first object block of type mesh, to clone a new object from.
    pub fn mesh_object_template(&self) -> Option<usize> {
        let dna = self.dna.as_ref()?;
        self.blocks
            .iter()
            .enumerate()
            .find(|(_, b)| b.code_str == "OB" && self.int_field(b, "Object", "type", dna) == OB_MESH as u32)
            .map(|(i, _)| i)
    }

    /// Write a 4×4 identity into a `float[4][4]` field.
    fn set_identity(&mut self, block: usize, struct_name: &str, field: &str) {
        for c in 0..4 {
            for r in 0..4 {
                self.set_f32(block, struct_name, &format!("{field}[{c}][{r}]"), if c == r { 1.0 } else { 0.0 });
            }
        }
    }

    /// What a cloned object must not keep from the one it was cloned from:
    /// its name, one user, no ID flags or tags, object mode, no parent, and
    /// no material slots. `totcol` counts the `**mat` array the clone no
    /// longer points at — kept, Blender reads that many slots from a null
    /// array and crashes opening the file (`BKE_object_materials_test`
    /// gives the object its mesh's slots again on load).
    pub(crate) fn fresh_object(&mut self, at: usize, name: &str) {
        self.set_id_name(at, name);
        self.set_int(at, "Object", "id.us", 1);
        self.set_int(at, "Object", "id.flag", 0);
        self.set_int(at, "Object", "id.tag", 0);
        self.set_int(at, "Object", "partype", 0);
        self.set_int(at, "Object", "mode", 0);
        self.set_int(at, "Object", "totcol", 0);
        self.set_int(at, "Object", "actcol", 0);
        self.set_identity(at, "Object", "parentinv");
        self.set_identity(at, "Object", "constinv");
    }

    /// Give object `to` the material slots of object `from`, as a linked
    /// duplicate has them (`BKE_object_duplicate`): its own copy of the
    /// `**mat` array — the same materials — and of `*matbits`, and the same
    /// `totcol` / `actcol`. Two objects may not share one array block:
    /// Blender reads each array into its object and frees it with it.
    pub(crate) fn copy_material_slots(&mut self, from: usize, to: usize) {
        let Some(dna) = self.dna.clone() else { return };
        for field in ["**mat", "*matbits"] {
            let address = self.ptr(&self.blocks[from], self.off("Object", field, &dna));
            let Some(array) = self.index_at(address).filter(|_| address != 0) else { continue };
            let Some((_, copy)) = self.clone_block(array, to, false) else { continue };
            self.set_ptr(to, "Object", field, copy);
        }
        for field in ["totcol", "actcol"] {
            let value = self.int_field(&self.blocks[from], "Object", field, &dna);
            self.set_int(to, "Object", field, i64::from(value));
        }
    }

    /// A new mesh object `name`, its data the mesh at `mesh_address`, at
    /// `location` with `scale`, cloned from `template` and inserted before
    /// the `DNA1` block. Its index and address.
    pub fn add_object(&mut self, template: usize, name: &str, mesh_address: u64, location: [f32; 3], scale: [f32; 3]) -> Option<(usize, u64)> {
        let dna1 = self.blocks.iter().position(|b| &b.code == DNA1).unwrap_or(self.blocks.len());
        let (at, address) = self.clone_block_cleared(template, dna1.saturating_sub(1))?;
        self.fresh_object(at, name);
        self.set_int(at, "Object", "type", OB_MESH);
        self.set_ptr(at, "Object", "*data", mesh_address);
        for (axis, (loc, sca)) in location.iter().zip(scale).enumerate() {
            self.set_f32(at, "Object", &format!("loc[{axis}]"), *loc);
            self.set_f32(at, "Object", &format!("size[{axis}]"), sca);
            self.set_f32(at, "Object", &format!("dloc[{axis}]"), 0.0);
            self.set_f32(at, "Object", &format!("dscale[{axis}]"), 1.0);
            self.set_f32(at, "Object", &format!("rot[{axis}]"), 0.0);
            self.set_f32(at, "Object", &format!("drot[{axis}]"), 0.0);
        }
        Some((at, address))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cloned_object_keeps_its_values_and_points_nowhere() {
        let path = format!("{}/tests/fixtures/nla_cube.blend", env!("CARGO_MANIFEST_DIR"));
        let mut file = BlendFile::parse_bytes(&std::fs::read(path).expect("fixture")).expect("parse");
        let dna = file.dna.clone().unwrap();
        let template = file.mesh_object_template().expect("a mesh object");
        let spans = file.pointer_spans("Object", &dna);
        assert!(spans.len() > 30, "an Object points at many things: {}", spans.len());
        let (data_off, _) = file.field_span("Object", "*data", &dna).unwrap();
        assert!(spans.iter().any(|(o, s)| *o == data_off && *s == 8), "the data pointer is among them");
        let (id_next, _) = file.field_span("Object", "id.*next", &dna).unwrap();
        assert!(spans.iter().any(|(o, _)| *o == id_next), "the embedded ID's pointers too");

        let before = file.blocks.len();
        let (at, address) = file.add_object(template, "Crate", 0x1234, [1.0, 2.0, 3.0], [1.0, 1.0, 1.0]).expect("added");
        assert_eq!(file.blocks.len(), before + 1);
        assert_eq!(file.blocks[at].code_str, "OB");
        assert_eq!(file.extract_name_from_block(&file.blocks[at], "id.name").unwrap(), "Crate");
        assert_eq!(file.ptr(&file.blocks[at], file.off("Object", "*data", &dna)), 0x1234);
        assert_eq!(file.ptr(&file.blocks[at], file.off("Object", "*parent", &dna)), 0, "no parent");
        assert_eq!(file.ptr(&file.blocks[at], file.off("Object", "modifiers", &dna)), 0, "no modifiers");
        let (loc, _) = file.field_span("Object", "loc[1]", &dna).unwrap();
        assert_eq!(file.f32_at(&file.blocks[at].data, loc), 2.0);
        let (dt_off, _) = file.field_span("Object", "dt", &dna).unwrap();
        assert_eq!(file.blocks[at].data[dt_off], file.blocks[template].data[dt_off], "the draw type is the template's");
        assert_eq!(file.index_at(address), Some(at));
        assert!(&file.blocks[at + 1].code == DNA1, "inserted before DNA1");
    }
}
