//! Blender 5.x `Mesh.attribute_storage` (#46).
//!
//! Blender ref (`BKE_attribute_storage.hh`, `attribute_storage.cc`,
//! `BKE_attribute_enums.hh`): `AttributeStorage { Attribute
//! *dna_attributes; int dna_attributes_num; }`, `Attribute { char *name;
//! int16 data_type (AttrType); int8 domain (AttrDomain); int8 storage_type;
//! void *data }`. Array storage points at `AttributeArray { *data,
//! *sharing_info, int64 size }`. 5.x meshes write geometry here and leave
//! the `CustomData` layers empty.

use crate::blend::file::types::*;
use std::borrow::Cow;

/// Bytes per element for `AttrType` (String and unknown types: 0).
fn type_width(ty: i16) -> usize {
    match ty {
        0 | 1 => 1,
        2 | 3 | 5 | 9 => 4,
        4 | 6 => 8,
        7 => 12,
        10 | 11 | 13 => 16,
        8 => 64,
        _ => 0,
    }
}

pub const ATTR_BOOL: i16 = 0;
pub const ATTR_INT32: i16 = 3;
pub const ATTR_FLOAT2: i16 = 6;
pub const ATTR_FLOAT3: i16 = 7;
pub const ATTR_COLOR_BYTE: i16 = 9;
pub const ATTR_COLOR_FLOAT: i16 = 10;
pub const DOMAIN_POINT: i8 = 0;
pub const DOMAIN_FACE: i8 = 2;
pub const DOMAIN_CORNER: i8 = 3;
const STORAGE_ARRAY: i8 = 0;
const STORAGE_SINGLE: i8 = 1;

impl BlendFile {
    /// Name and payload bytes of the first array attribute accepted by
    /// `want(name, data_type, domain)`, resolved within `me`'s data.
    pub(crate) fn mesh_attribute(
        &self,
        me: &BlendBlock,
        dna: &DnaType,
        want: &dyn Fn(&str, i16, i8) -> bool,
    ) -> Option<(String, Cow<'_, [u8]>)> {
        self.scoped(me, || {
            let ps = self.header.pointer_size as usize;
            let st = self.off("Mesh", "attribute_storage", dna)?;
            let arr = self.ptr(me, Some(st + self.off("AttributeStorage", "*dna_attributes", dna)?));
            let n_off = st + self.off("AttributeStorage", "dna_attributes_num", dna)?;
            let n = me.data.get(n_off..n_off + 4).map(|b| self.read_i32(b))?.clamp(0, 4096) as usize;
            let list = self.block_at(arr)?;
            let size = Self::struct_size("Attribute", dna, ps)?;
            let (name_o, ty_o) = (self.off("Attribute", "*name", dna)?, self.off("Attribute", "data_type", dna)?);
            let (dom_o, st_o) = (self.off("Attribute", "domain", dna)?, self.off("Attribute", "storage_type", dna)?);
            let data_o = self.off("Attribute", "*data", dna)?;
            // `*data` leads both `AttributeArray` and `AttributeSingle`.
            let adata_o = self.off("AttributeArray", "*data", dna)?;
            for i in 0..n {
                let rec = list.data.get(i * size..(i + 1) * size)?;
                let storage = rec.get(st_o).map(|&b| b as i8);
                if storage != Some(STORAGE_ARRAY) && storage != Some(STORAGE_SINGLE) {
                    continue;
                }
                let single = storage == Some(STORAGE_SINGLE);
                let name_blk = self.block_at(Self::read_ptr(rec, name_o, ps, self.header.little_endian));
                let Some(name) = name_blk.map(|b| {
                    let z = b.data.iter().position(|&c| c == 0).unwrap_or(b.data.len());
                    String::from_utf8_lossy(&b.data[..z]).to_string()
                }) else {
                    continue;
                };
                let ty = self.read_u16(rec.get(ty_o..ty_o + 2)?) as i16;
                let dom = rec[dom_o] as i8;
                if !want(&name, ty, dom) {
                    continue;
                }
                let aptr = Self::read_ptr(rec, data_o, ps, self.header.little_endian);
                let Some(array) = self.block_at(aptr) else { continue };
                let pptr = Self::read_ptr(&array.data, adata_o, ps, self.header.little_endian);
                let Some(payload) = self.block_at(pptr) else { continue };
                if single {
                    // `AttributeSingle`: one value for the whole domain.
                    let (w, n) = (type_width(ty), self.domain_len(me, dna, dom));
                    let one = payload.data.get(..w)?;
                    return Some((name, Cow::Owned(one.repeat(n))));
                }
                return Some((name, Cow::Borrowed(payload.data.as_slice())));
            }
            None
        })
    }

    /// Element count of `dom` (`Mesh.totvert / totedge / totpoly / totloop`).
    fn domain_len(&self, me: &BlendBlock, dna: &DnaType, dom: i8) -> usize {
        let field = ["totvert", "totedge", "totpoly", "totloop"].get(dom as usize).copied().unwrap_or("");
        self.int_field(me, "Mesh", field, dna) as usize
    }

    /// A `char *` field of `block` as a string (`None` when unset).
    pub(crate) fn cstring_at(&self, block: &BlendBlock, st: &str, field: &str, dna: &DnaType) -> Option<String> {
        let b = self.block_at(self.ptr(block, Some(self.off(st, field, dna)?)))?;
        let z = b.data.iter().position(|&c| c == 0).unwrap_or(b.data.len());
        (z > 0).then(|| String::from_utf8_lossy(&b.data[..z]).to_string())
    }

    fn attr_named(&self, me: &BlendBlock, dna: &DnaType, name: &str, ty: i16, dom: i8) -> Option<Cow<'_, [u8]>> {
        self.mesh_attribute(me, dna, &|n, t, d| n == name && t == ty && d == dom).map(|(_, b)| b)
    }

    pub(crate) fn attr_float3(&self, me: &BlendBlock, dna: &DnaType, name: &str, count: usize) -> Vec<[f32; 3]> {
        let Some(b) = self.attr_named(me, dna, name, ATTR_FLOAT3, DOMAIN_POINT) else { return Vec::new() };
        b.chunks_exact(12).take(count).map(|c| [self.read_f32(&c[0..4]), self.read_f32(&c[4..8]), self.read_f32(&c[8..12])]).collect()
    }

    pub(crate) fn attr_int32(&self, me: &BlendBlock, dna: &DnaType, name: &str, dom: i8, count: usize) -> Option<Vec<i32>> {
        let b = self.attr_named(me, dna, name, ATTR_INT32, dom)?;
        Some(b.chunks_exact(4).take(count).map(|c| self.read_i32(c)).collect())
    }

    pub(crate) fn attr_bool(&self, me: &BlendBlock, dna: &DnaType, name: &str, dom: i8) -> Option<Cow<'_, [u8]>> {
        self.attr_named(me, dna, name, ATTR_BOOL, dom)
    }

    /// Corner colours, scene-linear RGB (`Mesh.active_color_attribute`,
    /// else the first visible colour attribute, as Workbench's Attribute
    /// colour draws): a point-domain attribute is spread over `faces`'
    /// corners; byte colours are sRGB and come back linear
    /// (`srgb_to_linearrgb`), float colours linear already. Empty when
    /// the mesh has none.
    pub(crate) fn attr_corner_colors(&self, me: &BlendBlock, dna: &DnaType, active: Option<&str>, faces: &[Vec<u32>], points: usize) -> Vec<[f32; 3]> {
        // The accepted attribute's type and domain, noted as it is picked.
        let kind = std::cell::Cell::new((ATTR_COLOR_BYTE, DOMAIN_CORNER));
        let is_color = |t: i16, d: i8| {
            let ok = (t == ATTR_COLOR_BYTE || t == ATTR_COLOR_FLOAT) && (d == DOMAIN_POINT || d == DOMAIN_CORNER);
            if ok {
                kind.set((t, d));
            }
            ok
        };
        let found = active
            .and_then(|a| self.mesh_attribute(me, dna, &|n, t, d| n == a && is_color(t, d)))
            .or_else(|| self.mesh_attribute(me, dna, &|n, t, d| !n.starts_with('.') && is_color(t, d)));
        let Some((_, b)) = found else { return Vec::new() };
        let (ty, dom) = kind.get();
        let count = if dom == DOMAIN_POINT { points } else { faces.iter().map(Vec::len).sum() };
        let lin = crate::scene::texture::srgb_to_linear;
        let values: Vec<[f32; 3]> = if ty == ATTR_COLOR_BYTE {
            b.chunks_exact(4).take(count).map(|c| [lin(c[0]), lin(c[1]), lin(c[2])]).collect()
        } else {
            b.chunks_exact(16).take(count).map(|c| [self.read_f32(&c[0..4]), self.read_f32(&c[4..8]), self.read_f32(&c[8..12])]).collect()
        };
        if dom == DOMAIN_CORNER {
            return values;
        }
        faces.iter().flatten().map(|&v| values.get(v as usize).copied().unwrap_or([1.0; 3])).collect()
    }

    /// Corner UVs: the named active map, else the first visible Float2
    /// corner attribute (Blender hides internal ones behind a `.` prefix).
    pub(crate) fn attr_uvs(&self, me: &BlendBlock, dna: &DnaType, active: Option<&str>, count: usize) -> Vec<[f32; 2]> {
        let pick = |n: &str, t: i16, d: i8| {
            t == ATTR_FLOAT2 && d == DOMAIN_CORNER && active.map_or(!n.starts_with('.'), |a| a == n)
        };
        let found = self.mesh_attribute(me, dna, &pick)
            .or_else(|| self.mesh_attribute(me, dna, &|n, t, d| t == ATTR_FLOAT2 && d == DOMAIN_CORNER && !n.starts_with('.')));
        let Some((_, b)) = found else { return Vec::new() };
        b.chunks_exact(8).take(count).map(|c| [self.read_f32(&c[0..4]), self.read_f32(&c[4..8])]).collect()
    }
}
