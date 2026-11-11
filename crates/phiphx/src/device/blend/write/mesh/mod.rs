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

use crate::device::blend::file::types::*;

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

mod add;
mod new;
mod storage;

use new::{edges_of, le_bytes};

pub use new::BlendNewMesh;

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
