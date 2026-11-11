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

use crate::device::blend::file::kind::*;

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
#[path = "../../test/write/mesh.rs"]
mod mesh_test;
