//! A mesh's geometry as the file's Blender stores it: attribute storage (5.0) or custom-data layers.
use crate::device::blend::write::mesh::*;

impl BlendFile {
    /// Blender 5.0: `Mesh.attribute_storage`.
    pub(super) fn write_attribute_storage(&mut self, me: usize, positions: &[u8], pairs: &[u8], corner_verts: &[u8], corner_edges: &[u8], sharp: &[u8], counts: (usize, usize, usize, usize)) -> Option<()> {
        let attrs: [(&str, i64, i64, &[u8], usize); 5] = [
            ("position", ATTR_FLOAT3, DOMAIN_POINT, positions, counts.0),
            (".edge_verts", ATTR_INT32_2D, DOMAIN_EDGE, pairs, counts.1),
            ("sharp_face", ATTR_BOOL, DOMAIN_FACE, sharp, counts.2),
            (".corner_vert", ATTR_INT32, DOMAIN_CORNER, corner_verts, counts.3),
            (".corner_edge", ATTR_INT32, DOMAIN_CORNER, corner_edges, counts.3),
        ];
        let (table, table_address) = self.push_structs("Attribute", attrs.len(), me)?;
        let stride = self.sizeof("Attribute")?;
        let mut after = table;
        for (n, (name, data_type, domain, data, size)) in attrs.iter().enumerate() {
            let mut text = name.as_bytes().to_vec();
            text.push(0);
            let (name_at, name_address) = self.push_raw(after, text);
            let (raw_at, raw_address) = self.push_raw(name_at, data.to_vec());
            let (array_at, array_address) = self.push_structs("AttributeArray", 1, raw_at)?;
            self.set_ptr(array_at, "AttributeArray", "*data", raw_address);
            self.set_int(array_at, "AttributeArray", "size", *size as i64);
            after = array_at;
            let base = n * stride;
            self.set_struct_field(table, base, "Attribute", "*name", name_address as i64);
            self.set_struct_field(table, base, "Attribute", "data_type", *data_type);
            self.set_struct_field(table, base, "Attribute", "domain", *domain);
            self.set_struct_field(table, base, "Attribute", "storage_type", STORAGE_ARRAY);
            self.set_struct_field(table, base, "Attribute", "*data", array_address as i64);
        }
        self.set_ptr(me, "Mesh", "attribute_storage.*dna_attributes", table_address);
        self.set_int(me, "Mesh", "attribute_storage.dna_attributes_num", attrs.len() as i64);
        Some(())
    }

    /// Blender 3.6 – 4.x: `CustomData` layers in `vdata / edata / pdata / ldata`.
    pub(super) fn write_custom_data(&mut self, me: usize, positions: &[u8], edges: &[[u32; 2]], corner_verts: &[u8], corner_edges: &[u8], sharp: &[u8]) -> Option<()> {
        let le = self.header.little_endian;
        let edge_layer: (i64, &str, Vec<u8>) = if self.header.version_number >= 400 {
            (CD_PROP_INT32_2D, ".edge_verts", edges.iter().flatten().flat_map(|i| le_bytes(le, i.to_le_bytes())).collect())
        } else {
            // `MEdge { int v1, v2; char crease, bweight; short flag }`.
            let stride = self.sizeof("MEdge")?;
            let mut out = vec![0u8; stride * edges.len()];
            for (n, [a, b]) in edges.iter().enumerate() {
                out[n * stride..n * stride + 4].copy_from_slice(&le_bytes(le, a.to_le_bytes()));
                out[n * stride + 4..n * stride + 8].copy_from_slice(&le_bytes(le, b.to_le_bytes()));
            }
            (CD_MEDGE, "", out)
        };
        let groups: [(&str, Vec<(i64, &str, Vec<u8>)>); 4] = [
            ("vdata", vec![(CD_PROP_FLOAT3, "position", positions.to_vec())]),
            ("edata", vec![edge_layer]),
            ("pdata", vec![(CD_PROP_BOOL, "sharp_face", sharp.to_vec())]),
            ("ldata", vec![(CD_PROP_INT32, ".corner_vert", corner_verts.to_vec()), (CD_PROP_INT32, ".corner_edge", corner_edges.to_vec())]),
        ];
        let stride = self.sizeof("CustomDataLayer")?;
        let mut after = me;
        for (cd, layers) in groups {
            let (table, table_address) = self.push_structs("CustomDataLayer", layers.len(), after)?;
            after = table;
            let mut typemap: Vec<i64> = vec![-1; 64];
            for (n, (kind, name, data)) in layers.iter().enumerate() {
                let (raw_at, raw_address) = self.push_raw(after, data.clone());
                after = raw_at;
                let base = n * stride;
                self.set_struct_field(table, base, "CustomDataLayer", "type", *kind);
                self.set_struct_text(table, base, "CustomDataLayer", "name", name);
                self.set_struct_field(table, base, "CustomDataLayer", "*data", raw_address as i64);
                if typemap[*kind as usize] < 0 {
                    typemap[*kind as usize] = n as i64;
                }
            }
            self.set_ptr(me, "Mesh", &format!("{cd}.*layers"), table_address);
            self.set_int(me, "Mesh", &format!("{cd}.totlayer"), layers.len() as i64);
            self.set_int(me, "Mesh", &format!("{cd}.maxlayer"), layers.len() as i64);
            let dna = self.dna.clone()?;
            if let Some((_, size)) = self.field_span("CustomData", "typemap", &dna) {
                for (t, index) in typemap.iter().enumerate().take(size / 4) {
                    self.set_int(me, "Mesh", &format!("{cd}.typemap[{t}]"), *index);
                }
            }
        }
        Some(())
    }
}
