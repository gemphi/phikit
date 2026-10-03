//! A new `Mesh` in the file, in the layout the file's Blender wrote its
//! own meshes in.
//!
//! Blender 5.0 keeps a mesh's attributes in `Mesh.attribute_storage`
//! (`AttributeStorage { Attribute *dna_attributes; int dna_attributes_num }`,
//! each `Attribute { char *name; short data_type; int8 domain, storage_type;
//! void *data }` an `AttributeArray { void *data; sharing_info; int64 size }`
//! over a raw block). Blender 3.6 – 4.x keep them as `CustomData` layers
//! (`CustomDataLayer { int type; …; char name[68]; void *data }`) in
//! `vdata / edata / pdata / ldata`; 4.0 turned the edge list from `MEdge`
//! structs into an `.edge_verts` int-pair attribute. Both list faces as
//! `poly_offset_indices`. Older files still hold `MVert / MPoly / MLoop`,
//! which this writer does not produce: it says so instead.
//!
//! The geometry it writes is the same in every layout: `position` per
//! vertex, `.edge_verts` (or `MEdge`) per edge, `.corner_vert` and
//! `.corner_edge` per corner, `sharp_face` per face (a fresh primitive is
//! flat-shaded), the face offsets, and the counts.

use crate::blend::file::types::*;

/// The oldest file layout the mesh writer produces (`poly_offset_indices`).
pub const OLDEST_MESH_LAYOUT: u32 = 306;

/// `eCustomDataType`: the layer types the 3.6 – 4.x writer uses.
const CD_MEDGE: i64 = 3;
const CD_PROP_INT32: i64 = 11;
const CD_PROP_INT32_2D: i64 = 46;
const CD_PROP_FLOAT3: i64 = 48;
const CD_PROP_BOOL: i64 = 50;

/// Blender 5.0's `AttrType` and `AttrDomain`, as the file spells them.
const ATTR_BOOL: i64 = 0;
const ATTR_INT32: i64 = 3;
const ATTR_INT32_2D: i64 = 4;
const ATTR_FLOAT3: i64 = 7;
const DOMAIN_POINT: i64 = 0;
const DOMAIN_EDGE: i64 = 1;
const DOMAIN_FACE: i64 = 2;
const DOMAIN_CORNER: i64 = 3;
/// `AttrStorageType::Array`.
const STORAGE_ARRAY: i64 = 0;

/// A mesh as the writer takes it: vertices and faces as vertex indices.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlendNewMesh {
    pub name: String,
    pub vertices: Vec<[f32; 3]>,
    pub faces: Vec<Vec<u32>>,
}

impl BlendNewMesh {
    /// A mesh from a triangle list as a viewport keeps one — vertices
    /// repeated per face, every face a triangle — welded and paired into
    /// quads the way `mesh.tris_convert_to_quads` does: two triangles that
    /// share an edge, lie in one plane and meet convexly become one face.
    /// A cube's twenty-four vertices and twelve triangles come back as
    /// Blender's eight and six.
    pub fn from_triangles(name: &str, vertices: &[[f32; 3]], indices: &[u32]) -> Self {
        // Weld: one vertex per distinct position.
        let mut welded: Vec<[f32; 3]> = Vec::new();
        let mut by_bits: std::collections::HashMap<[u32; 3], u32> = std::collections::HashMap::new();
        let remap: Vec<u32> = vertices
            .iter()
            .map(|v| {
                let key = [v[0].to_bits(), v[1].to_bits(), v[2].to_bits()];
                *by_bits.entry(key).or_insert_with(|| {
                    welded.push(*v);
                    (welded.len() - 1) as u32
                })
            })
            .collect();
        let tris: Vec<[u32; 3]> = indices
            .chunks_exact(3)
            .map(|t| [remap[t[0] as usize], remap[t[1] as usize], remap[t[2] as usize]])
            .filter(|t| t[0] != t[1] && t[1] != t[2] && t[0] != t[2])
            .collect();
        let normal = |t: &[u32; 3]| -> [f32; 3] {
            let (a, b, c) = (welded[t[0] as usize], welded[t[1] as usize], welded[t[2] as usize]);
            let (u, v) = ([b[0] - a[0], b[1] - a[1], b[2] - a[2]], [c[0] - a[0], c[1] - a[1], c[2] - a[2]]);
            let n = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-12);
            [n[0] / len, n[1] / len, n[2] / len]
        };
        // Directed edge → the triangle that has it, to find the neighbour
        // across each edge (which has it reversed).
        let mut owner: std::collections::HashMap<(u32, u32), usize> = std::collections::HashMap::new();
        for (i, t) in tris.iter().enumerate() {
            for k in 0..3 {
                owner.insert((t[k], t[(k + 1) % 3]), i);
            }
        }
        let mut used = vec![false; tris.len()];
        let mut faces: Vec<Vec<u32>> = Vec::new();
        for (i, t) in tris.iter().enumerate() {
            if used[i] {
                continue;
            }
            let n = normal(t);
            let mut quad = None;
            for k in 0..3 {
                let (b, c) = (t[k], t[(k + 1) % 3]);
                let a = t[(k + 2) % 3];
                let Some(&j) = owner.get(&(c, b)) else { continue };
                if j == i || used[j] {
                    continue;
                }
                let m = normal(&tris[j]);
                if n[0] * m[0] + n[1] * m[1] + n[2] * m[2] < 0.9999 {
                    continue;
                }
                let q = tris[j].iter().copied().find(|v| *v != b && *v != c).unwrap_or(b);
                if q == a {
                    continue;
                }
                // Convex: `q` lies across the shared edge from `a`.
                let side = |p: u32| {
                    let (pb, pc, pp) = (welded[b as usize], welded[c as usize], welded[p as usize]);
                    let (e, w) = ([pc[0] - pb[0], pc[1] - pb[1], pc[2] - pb[2]], [pp[0] - pb[0], pp[1] - pb[1], pp[2] - pb[2]]);
                    let x = [e[1] * w[2] - e[2] * w[1], e[2] * w[0] - e[0] * w[2], e[0] * w[1] - e[1] * w[0]];
                    x[0] * n[0] + x[1] * n[1] + x[2] * n[2]
                };
                if side(a) * side(q) >= 0.0 {
                    continue;
                }
                used[j] = true;
                quad = Some(vec![a, b, q, c]);
                break;
            }
            used[i] = true;
            faces.push(quad.unwrap_or_else(|| t.to_vec()));
        }
        Self { name: name.to_string(), vertices: welded, faces }
    }
}

/// The edges of `faces`, each once, and per corner the edge it starts.
fn edges_of(faces: &[Vec<u32>]) -> (Vec<[u32; 2]>, Vec<u32>) {
    let mut edges: Vec<[u32; 2]> = Vec::new();
    let mut index = std::collections::HashMap::new();
    let mut corner_edges = Vec::new();
    for face in faces {
        for (n, &a) in face.iter().enumerate() {
            let b = face[(n + 1) % face.len()];
            let key = (a.min(b), a.max(b));
            let e = *index.entry(key).or_insert_with(|| {
                edges.push([a, b]);
                (edges.len() - 1) as u32
            });
            corner_edges.push(e);
        }
    }
    (edges, corner_edges)
}

fn le_bytes<const N: usize>(le: bool, bytes: [u8; N]) -> [u8; N] {
    if le {
        bytes
    } else {
        let mut b = bytes;
        b.reverse();
        b
    }
}

impl BlendFile {
    /// Whether this file's meshes are ones the writer can add to.
    pub fn writes_meshes(&self) -> bool {
        self.header.version_number >= OLDEST_MESH_LAYOUT && self.dna.as_ref().is_some_and(|d| d.types.iter().any(|t| t == "Mesh"))
    }

    fn push_raw(&mut self, after: usize, data: Vec<u8>) -> (usize, u64) {
        let address = self.fresh_address();
        let block = BlendBlock { code: *b"DATA", code_str: "DATA".into(), size: data.len() as u64, old_address: address, sdna_index: 0, count: 1, data };
        let at = (after + 1).min(self.blocks.len());
        self.blocks.insert(at, block);
        self.reindex();
        (at, address)
    }

    /// A `DATA` block of `count` zeroed `struct_name`s.
    fn push_structs(&mut self, struct_name: &str, count: usize, after: usize) -> Option<(usize, u64)> {
        let sdna_index = self.sdna_index_of(struct_name)?;
        let size = self.sizeof(struct_name)? * count.max(1);
        let address = self.fresh_address();
        let block = BlendBlock { code: *b"DATA", code_str: "DATA".into(), size: size as u64, old_address: address, sdna_index, count: count.max(1) as u64, data: vec![0; size] };
        let at = (after + 1).min(self.blocks.len());
        self.blocks.insert(at, block);
        self.reindex();
        Some((at, address))
    }

    /// The first mesh block, to clone a new mesh from.
    pub fn mesh_template(&self) -> Option<usize> {
        self.blocks.iter().position(|b| b.code_str == "ME")
    }

    /// A new mesh `mesh`, cloned from `template` and laid out as this file
    /// lays meshes out, inserted before `DNA1`. Its index and address.
    pub fn add_mesh(&mut self, template: usize, mesh: &BlendNewMesh) -> Option<(usize, u64)> {
        if !self.writes_meshes() {
            return None;
        }
        let dna1 = self.blocks.iter().position(|b| &b.code == DNA1).unwrap_or(self.blocks.len());
        let (me, address) = self.clone_block_cleared(template, dna1.saturating_sub(1))?;
        self.set_id_name(me, &mesh.name);
        self.set_int(me, "Mesh", "id.us", 1);
        self.set_int(me, "Mesh", "id.flag", 0);
        self.set_int(me, "Mesh", "id.tag", 0);
        self.set_int(me, "Mesh", "totcol", 0);
        let (edges, corner_edges) = edges_of(&mesh.faces);
        let corner_verts: Vec<u32> = mesh.faces.iter().flatten().copied().collect();
        let mut offsets: Vec<u32> = Vec::with_capacity(mesh.faces.len() + 1);
        let mut acc = 0u32;
        for face in &mesh.faces {
            offsets.push(acc);
            acc += face.len() as u32;
        }
        offsets.push(acc);
        self.set_int(me, "Mesh", "totvert", mesh.vertices.len() as i64);
        self.set_int(me, "Mesh", "totedge", edges.len() as i64);
        self.set_int(me, "Mesh", "totpoly", mesh.faces.len() as i64);
        self.set_int(me, "Mesh", "totloop", corner_verts.len() as i64);
        let le = self.header.little_endian;
        let ints = |v: &[u32]| -> Vec<u8> { v.iter().flat_map(|i| le_bytes(le, i.to_le_bytes())).collect() };
        let positions: Vec<u8> = mesh.vertices.iter().flatten().flat_map(|f| le_bytes(le, f.to_le_bytes())).collect();
        let pairs: Vec<u8> = edges.iter().flatten().copied().collect::<Vec<u32>>().iter().flat_map(|i| le_bytes(le, i.to_le_bytes())).collect();
        let sharp: Vec<u8> = vec![1; mesh.faces.len()];
        // Face offsets, right after the mesh.
        let (_, offsets_address) = self.push_raw(me, ints(&offsets));
        self.set_ptr(me, "Mesh", "*poly_offset_indices", offsets_address);
        if self.header.version_number >= 500 {
            self.write_attribute_storage(me, &positions, &pairs, &ints(&corner_verts), &ints(&corner_edges), &sharp, (mesh.vertices.len(), edges.len(), mesh.faces.len(), corner_verts.len()));
        } else {
            self.write_custom_data(me, &positions, &edges, &ints(&corner_verts), &ints(&corner_edges), &sharp);
        }
        Some((me, address))
    }

    /// Blender 5.0: `Mesh.attribute_storage`.
    fn write_attribute_storage(&mut self, me: usize, positions: &[u8], pairs: &[u8], corner_verts: &[u8], corner_edges: &[u8], sharp: &[u8], counts: (usize, usize, usize, usize)) -> Option<()> {
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
    fn write_custom_data(&mut self, me: usize, positions: &[u8], edges: &[[u32; 2]], corner_verts: &[u8], corner_edges: &[u8], sharp: &[u8]) -> Option<()> {
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

    /// Write into one struct of an array block: field `field` of the
    /// `struct_name` at byte `base`.
    fn set_struct_field(&mut self, block: usize, base: usize, struct_name: &str, field: &str, value: i64) -> bool {
        let Some(dna) = self.dna.as_ref() else { return false };
        let Some((off, size)) = self.field_span(struct_name, field, dna) else { return false };
        let le = self.header.little_endian;
        let Some(data) = self.blocks.get_mut(block).map(|b| &mut b.data) else { return false };
        let off = base + off;
        if off + size > data.len() {
            return false;
        }
        let bytes = if le { value.to_le_bytes() } else { value.to_be_bytes() };
        match size {
            1 => data[off] = value as u8,
            2 | 4 | 8 => data[off..off + size].copy_from_slice(if le { &bytes[..size] } else { &bytes[8 - size..] }),
            _ => return false,
        }
        true
    }

    fn set_struct_text(&mut self, block: usize, base: usize, struct_name: &str, field: &str, text: &str) -> bool {
        let Some(dna) = self.dna.as_ref() else { return false };
        let Some((off, size)) = self.field_span(struct_name, field, dna) else { return false };
        let Some(data) = self.blocks.get_mut(block).map(|b| &mut b.data) else { return false };
        let off = base + off;
        if size == 0 || off + size > data.len() {
            return false;
        }
        let bytes: Vec<u8> = text.bytes().take(size - 1).collect();
        data[off..off + size].fill(0);
        data[off..off + bytes.len()].copy_from_slice(&bytes);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cube() -> BlendNewMesh {
        let v = |x: f32, y: f32, z: f32| [x, y, z];
        BlendNewMesh {
            name: "Crate".into(),
            vertices: vec![v(-1., -1., -1.), v(1., -1., -1.), v(1., 1., -1.), v(-1., 1., -1.), v(-1., -1., 1.), v(1., -1., 1.), v(1., 1., 1.), v(-1., 1., 1.)],
            faces: vec![vec![0, 1, 2, 3], vec![4, 7, 6, 5], vec![0, 4, 5, 1], vec![1, 5, 6, 2], vec![2, 6, 7, 3], vec![3, 7, 4, 0]],
        }
    }

    #[test]
    fn a_triangulated_cube_with_repeated_vertices_comes_back_as_eight_and_six() {
        // Twenty-four vertices, four per face, twelve triangles — a viewport's cube.
        let c = cube();
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        for face in &c.faces {
            let base = vertices.len() as u32;
            for &v in face {
                vertices.push(c.vertices[v as usize]);
            }
            indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
        }
        let mesh = BlendNewMesh::from_triangles("Cube", &vertices, &indices);
        assert_eq!(mesh.vertices.len(), 8);
        assert_eq!(mesh.faces.len(), 6, "{:?}", mesh.faces);
        assert!(mesh.faces.iter().all(|f| f.len() == 4));
        // Two triangles that are not coplanar stay triangles.
        let bent = BlendNewMesh::from_triangles("Bent", &[[0., 0., 0.], [1., 0., 0.], [0., 1., 0.], [1., 1., 1.]], &[0, 1, 2, 2, 1, 3]);
        assert_eq!(bent.faces.len(), 2);
        assert!(bent.faces.iter().all(|f| f.len() == 3));
    }

    #[test]
    fn a_cube_has_twelve_edges_each_once() {
        let (edges, corner_edges) = edges_of(&cube().faces);
        assert_eq!(edges.len(), 12);
        assert_eq!(corner_edges.len(), 24);
        assert!(corner_edges.iter().all(|e| (*e as usize) < 12));
    }

    #[test]
    fn a_new_mesh_reads_back_in_the_5_0_and_the_3_6_layouts() {
        for name in ["nla_cube.blend", "THYMUS-BOLDROID-WAVE-003.blend"] {
            let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
            let mut file = BlendFile::parse_bytes(&std::fs::read(path).expect("fixture")).expect("parse");
            assert!(file.writes_meshes(), "{name}");
            let template = file.mesh_template().expect("a mesh");
            let (me, address) = file.add_mesh(template, &cube()).expect("added");
            assert_eq!(file.blocks[me].code_str, "ME");
            let ob_template = file.mesh_object_template().expect("a mesh object");
            let (_, object_address) = file.add_object(ob_template, "Crate", address, [0.0; 3], [1.0; 3]).expect("object");
            let scene = file.current_scene_index().unwrap();
            let view_layer = file.view_layer_index(scene).unwrap();
            let master = file.collections_at(scene, view_layer)[0].collection;
            file.link_object(scene, view_layer, master, object_address, 0).expect("linked");
            let again = BlendFile::parse_bytes(&file.to_bytes()).expect("the written file parses");
            let scene = again.extract_scene().expect("scene");
            let crate_ = scene.objects.iter().find(|o| o.name == "Crate").unwrap_or_else(|| panic!("{name}: the crate is read back"));
            assert_eq!(crate_.vertices.len(), 8, "{name}");
            assert_eq!(crate_.faces.len(), 6, "{name}: {:?}", crate_.faces);
            assert!(crate_.faces.iter().all(|f| f.len() == 4), "{name}");
            assert_eq!(crate_.vertices[6], [1.0, 1.0, 1.0], "{name}");
        }
    }
}
