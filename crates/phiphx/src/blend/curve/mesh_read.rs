//! A curve object (`OB_CURVES_LEGACY`) as a mesh before its modifiers
//! (`evaluate_curve_type_object`, then `mesh_nurbs_displist_to_mesh`):
//! reading the `Curve` fields its display list needs — extrusion, bevel,
//! offset, fill, the custom profile (`bevel_profile`), the bevel object's
//! outlines (`curve_bevel_make_from_object`) and the taper object's first
//! display list (`displist_calc_taper`), each object evaluated the same
//! way — and its splines' materials and smoothing.

use super::displist::curve_displist;
use super::profile::{create_samples, BProfilePoint, PROF_SAMPLE_STRAIGHT_EDGES};
use crate::blend::file::types::*;
use crate::blend::rig::read::BFields;
use crate::blend::text::displist::{TDispList, TDlKind};
use crate::blend::text::mesh::from_displist;
use crate::blend::text::profile::from_object_list;
use crate::blend::text::sweep::TSplineInfo;
use crate::blend::text::{TCurveShape, TMesh};
use glam::Vec2;

/// `OB_CURVES_LEGACY`.
const OB_CURVES_LEGACY: u32 = 2;
/// `CU_SMOOTH` (`Nurb.flag`).
const CU_SMOOTH: u32 = 1;
/// Bevel and taper objects followed this deep (Blender's dependency graph
/// refuses cycles; one level is what a curve reads).
const MAX_DEPTH: usize = 2;

impl BlendFile {
    /// The mesh curve object `ob` evaluates to before its modifiers; `None`
    /// when its curve cannot be read.
    pub(crate) fn object_curve_mesh(&self, ob: &BlendBlock, dna: &DnaType) -> Option<TMesh> {
        let (flag, disp) = self.object_curve_displist(ob, dna, 0)?;
        Some(from_displist(flag, &disp))
    }

    /// `evaluate_curve_type_object` for curve object `ob`: its flag and its
    /// display list.
    fn object_curve_displist(&self, ob: &BlendBlock, dna: &DnaType, depth: usize) -> Option<(u32, Vec<TDispList>)> {
        let cu = self.block_at_code(self.ptr(ob, self.off("Object", "*data", dna)), "CU")?;
        self.scoped(cu, || {
            let curve = self.object_curve(ob, dna)?;
            let c = BFields::new(self, dna, cu, "Curve");
            let either = |new: &str, old: &str| if c.has(new) { c.f32(new, 0.0) } else { c.f32(old, 0.0) };
            let bevresol = (c.int("bevresol") as i16).max(0) as usize;
            let mut shape = TCurveShape {
                flag: curve.flag,
                extrude: either("extrude", "ext1"),
                bevel_radius: either("bevel_radius", "ext2"),
                // `offset` is saved under its old DNA name, `width`.
                offset: if c.has("offset") { c.f32("offset", 1.0) } else { c.f32("width", 1.0) },
                bevresol,
                bevel_mode: c.int("bevel_mode") as u8,
                taper_radius_mode: c.int("taper_radius_mode") as u8,
                ..Default::default()
            };
            shape.profile = self.profile_samples(c.ptr("*bevel_profile"), bevresol, dna);
            if depth < MAX_DEPTH {
                shape.bevel_object = self.bevel_object(c.ptr("*bevobj"), dna, depth);
                shape.taper = self.taper(c.ptr("*taperobj"), dna, depth);
            }
            let first = self.ptr(cu, self.off("Curve", "nurb", dna));
            let splines: Vec<TSplineInfo> = self
                .list(first, self.off("Nurb", "*next", dna))
                .into_iter()
                .map(|nu| {
                    let n = BFields::new(self, dna, nu, "Nurb");
                    TSplineInfo { col: n.int("mat_nr") as i16, charidx: n.int("charidx") as i32, smooth: n.int("flag") & CU_SMOOTH != 0 }
                })
                .collect();
            Some((curve.flag, curve_displist(&curve, &shape, &splines)))
        })
    }

    /// The curve object at `addr`, when it is one.
    fn curve_object(&self, addr: u64, dna: &DnaType) -> Option<&BlendBlock> {
        let ob = self.block_at_code(addr, "OB")?;
        (self.int_field(ob, "Object", "type", dna) == OB_CURVES_LEGACY).then_some(ob)
    }

    /// `curve_bevel_make_from_object`: a bevel object with neither
    /// extrusion nor bevel of its own gives its outlines, scaled by its
    /// object's scale.
    fn bevel_object(&self, addr: u64, dna: &DnaType, depth: usize) -> Vec<TDispList> {
        let Some(ob) = self.curve_object(addr, dna) else { return Vec::new() };
        let Some(cu) = self.block_at_code(self.ptr(ob, self.off("Object", "*data", dna)), "CU") else { return Vec::new() };
        let c = BFields::new(self, dna, cu, "Curve");
        let either = |new: &str, old: &str| if c.has(new) { c.f32(new, 0.0) } else { c.f32(old, 0.0) };
        if either("extrude", "ext1") != 0.0 || either("bevel_radius", "ext2") != 0.0 {
            return Vec::new();
        }
        let o = BFields::new(self, dna, ob, "Object");
        let scale = match o.has("scale[3]") {
            true => o.vec3("scale[3]", glam::Vec3::ONE),
            false => o.vec3("size[3]", glam::Vec3::ONE),
        };
        let Some((_, disp)) = self.object_curve_displist(ob, dna, depth + 1) else { return Vec::new() };
        disp.iter().filter(|d| matches!(d.kind, TDlKind::Poly | TDlKind::Segm)).map(|d| from_object_list(d, [scale.x, scale.y])).collect()
    }

    /// The taper object's first display list's points (`None` without a
    /// taper object; empty when it has none, which tapers by 1).
    fn taper(&self, addr: u64, dna: &DnaType, depth: usize) -> Option<Vec<glam::Vec3>> {
        self.block_at_code(addr, "OB")?;
        let Some(ob) = self.curve_object(addr, dna) else { return Some(Vec::new()) };
        let disp = self.object_curve_displist(ob, dna, depth + 1).map_or_else(Vec::new, |(_, d)| d);
        Some(disp.first().map_or_else(Vec::new, |dl| dl.verts[..dl.nr.min(dl.verts.len())].to_vec()))
    }

    /// `CurveProfile.segments` as `BKE_curveprofile_init` makes them at
    /// read: `segments_len` samples of the saved path.
    fn profile_samples(&self, addr: u64, bevresol: usize, dna: &DnaType) -> Option<Vec<Vec2>> {
        let pr = self.block_at(addr)?;
        let p = BFields::new(self, dna, pr, "CurveProfile");
        let n_path = (p.int("path_len") as i16).max(0) as usize;
        let segments = match (p.int("segments_len") as i16).max(0) as usize {
            0 => bevresol + 1,
            s => s,
        };
        let block = self.block_at(p.ptr("*path"))?;
        let ps = self.header.pointer_size as usize;
        let size = Self::struct_size("CurveProfilePoint", dna, ps)?;
        let off = |f: &str| self.off("CurveProfilePoint", f, dna);
        let path: Vec<BProfilePoint> = (0..n_path)
            .filter_map(|i| {
                let d = block.data.get(i * size..(i + 1) * size)?;
                let byte = |f: &str| off(f).and_then(|o| d.get(o)).copied().unwrap_or(0);
                Some(BProfilePoint {
                    co: Vec2::new(self.f32s(d, off("x"), [0.0])[0], self.f32s(d, off("y"), [0.0])[0]),
                    h1: byte("h1"),
                    h2: byte("h2"),
                    h1_loc: Vec2::from_array(self.f32s(d, off("h1_loc[2]"), [0.0; 2])),
                    h2_loc: Vec2::from_array(self.f32s(d, off("h2_loc[2]"), [0.0; 2])),
                })
            })
            .collect();
        let samples = create_samples(&path, segments, p.int("flag") & PROF_SAMPLE_STRAIGHT_EDGES != 0);
        (!samples.is_empty()).then_some(samples)
    }
}
