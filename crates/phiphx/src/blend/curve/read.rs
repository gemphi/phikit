//! Reading a curve datablock (`Curve`, its `Nurb` list and their
//! `BezTriple` / `BPoint` arrays) through the file's own DNA.

use super::{BBezPoint, BCurve, BCurvePoint, BNurb, CU_3D, CU_PATH_RADIUS};
use crate::blend::file::types::*;
use crate::blend::rig::read::BFields;
use glam::{Vec3, Vec4};

/// A struct array inside one block: element `i` of DNA struct `st`.
struct BArray<'a> {
    file: &'a BlendFile,
    dna: &'a DnaType,
    data: &'a [u8],
    st: &'static str,
    size: usize,
}

impl BArray<'_> {
    fn at(&self, i: usize) -> &[u8] {
        self.data.get(i * self.size..).unwrap_or(&[])
    }

    fn f32s<const N: usize>(&self, i: usize, field: &str, default: [f32; N]) -> [f32; N] {
        self.file.f32s(self.at(i), self.file.off(self.st, field, self.dna), default)
    }

    /// A field the file may keep under its old DNA name (`tilt` is saved
    /// as `alfa`, `DNA_STRUCT_RENAME_ELEM`).
    fn f32_named(&self, i: usize, names: [&str; 2], default: f32) -> f32 {
        let field = names.into_iter().find(|n| self.file.off(self.st, n, self.dna).is_some()).unwrap_or(names[0]);
        self.f32s(i, field, [default])[0]
    }

    fn byte(&self, i: usize, field: &str) -> u8 {
        self.file.off(self.st, field, self.dna).and_then(|o| self.at(i).get(o)).copied().unwrap_or(0)
    }
}

impl BlendFile {
    /// The curve datablock of curve object `ob`.
    pub(crate) fn object_curve(&self, ob: &BlendBlock, dna: &DnaType) -> Option<BCurve> {
        let cu = self.block_at_code(self.ptr(ob, self.off("Object", "*data", dna)), "CU")?;
        let c = BFields::new(self, dna, cu, "Curve");
        let flag = c.int("flag");
        // `CU_DO_RADIUS` (`extrude` / `bevel_radius` were `ext1` / `ext2`).
        let extrude = if c.has("extrude") { c.f32("extrude", 0.0) } else { c.f32("ext1", 0.0) };
        let bevel = if c.has("bevel_radius") { c.f32("bevel_radius", 0.0) } else { c.f32("ext2", 0.0) };
        let do_radius = flag & (CU_PATH_RADIUS | CU_3D) != 0 || c.ptr("*bevobj") != 0 || extrude != 0.0 || bevel != 0.0;
        let first = self.ptr(cu, self.off("Curve", "nurb", dna));
        let nurbs = self.list(first, self.off("Nurb", "*next", dna)).into_iter().map(|nu| self.nurb(nu, dna)).collect();
        Some(BCurve {
            flag,
            pathlen: c.int("pathlen") as i32 as f32,
            ctime: c.f32("ctime", 0.0),
            twist_mode: c.int("twist_mode") as u16,
            twist_smooth: c.f32("twist_smooth", 0.0),
            do_radius,
            nurbs,
        })
    }

    fn nurb(&self, nu: &BlendBlock, dna: &DnaType) -> BNurb {
        let n = BFields::new(self, dna, nu, "Nurb");
        let pntsu = n.int("pntsu") as i32 as usize;
        let pntsv = (n.int("pntsv") as i32).max(1) as usize;
        let ps = self.header.pointer_size as usize;
        let array = |ptr: u64, st: &'static str| {
            let block = self.block_at(ptr)?;
            Some(BArray { file: self, dna, data: &block.data, st, size: Self::struct_size(st, dna, ps)? })
        };
        let bezt = array(n.ptr("*bezt"), "BezTriple").map_or_else(Vec::new, |a| {
            (0..pntsu)
                .map(|i| {
                    let v = a.f32s(i, "vec[3][3]", [0.0; 9]);
                    BBezPoint {
                        vec: [Vec3::new(v[0], v[1], v[2]), Vec3::new(v[3], v[4], v[5]), Vec3::new(v[6], v[7], v[8])],
                        tilt: a.f32_named(i, ["tilt", "alfa"], 0.0),
                        weight: a.f32s(i, "weight", [1.0])[0],
                        radius: a.f32s(i, "radius", [1.0])[0],
                        h1: a.byte(i, "h1"),
                        h2: a.byte(i, "h2"),
                    }
                })
                .collect()
        });
        let bp = array(n.ptr("*bp"), "BPoint").map_or_else(Vec::new, |a| {
            (0..pntsu * pntsv)
                .map(|i| BCurvePoint {
                    vec: Vec4::from_array(a.f32s(i, "vec[4]", [0.0, 0.0, 0.0, 1.0])),
                    tilt: a.f32_named(i, ["tilt", "alfa"], 0.0),
                    weight: a.f32s(i, "weight", [1.0])[0],
                    radius: a.f32s(i, "radius", [1.0])[0],
                })
                .collect()
        });
        let knots = self.block_at(n.ptr("*knotsu")).map_or_else(Vec::new, |b| b.data.chunks_exact(4).map(|c| self.read_f32(c)).collect());
        BNurb {
            kind: n.int("type") as u16,
            flagu: n.int("flagu") as u16,
            pntsu,
            pntsv,
            resolu: (n.int("resolu") as i16).max(1) as usize,
            orderu: (n.int("orderu") as i16).max(1) as usize,
            knots,
            bezt,
            bp,
            tilt_interp: n.int("tilt_interp") as i16,
            radius_interp: n.int("radius_interp") as i16,
        }
    }
}
