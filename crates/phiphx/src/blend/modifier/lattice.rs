//! Lattice deform (`eModifierType_Lattice` = 2) — Blender `lattice_deform.cc` +
//! `key.cc::key_curve_position_weights`, ported.
//!
//! A lattice is a `pntsu × pntsv × pntsw` grid of control points (`BPoint.vec`)
//! over a rest grid that starts at `(fu, fv, fw)` with spacing `(du, dv, dw)`.
//! Blender stores, per point, how far it has moved from its rest position,
//! rotated into the deformed object's space (`mul_mat3_m4_v3(imat, fp)`).
//! Each vertex is taken into lattice space (`latmat = lat⁻¹ · ob`), its cell
//! found, and the 4×4×4 surrounding offsets summed with the per-axis
//! interpolation weights (`KEY_LINEAR/CARDINAL/BSPLINE/CATMULL_ROM`).
//!
//! This is what fits a character's garments to her: in the Female-01 file the
//! lattice moves the clothing 3.5–4.7 cm on average, and without it the body
//! comes through the clothes.

use crate::blend::file::types::*;
use super::driver::BModValue;
use super::{BModifier, BlendEvalMesh, EvalCtx};
use glam::{Mat3, Vec3};

const INVERT_VGROUP: u32 = 1 << 0;

/// `key_curve_position_weights`, verbatim.
fn weights(t: f32, kind: u8) -> [f32; 4] {
    let (t2, t3) = (t * t, t * t * t);
    match kind {
        0 => [0.0, 1.0 - t, t, 0.0],
        1 | 3 => {
            let fc = if kind == 1 { 0.71 } else { 0.5 };
            [
                -fc * t3 + 2.0 * fc * t2 - fc * t,
                (2.0 - fc) * t3 + (fc - 3.0) * t2 + 1.0,
                (fc - 2.0) * t3 + (3.0 - 2.0 * fc) * t2 + fc * t,
                fc * t3 - fc * t2,
            ]
        }
        _ => [
            -0.16666666 * t3 + 0.5 * t2 - 0.5 * t + 0.16666666,
            0.5 * t3 - t2 + 0.66666666,
            -0.5 * t3 + 0.5 * t2 + 0.5 * t + 0.16666666,
            0.16666666 * t3,
        ],
    }
}

/// `lattice_deform_axis_clamp`: keep `floor()` defined and the fraction in [0, 1).
fn clamp_axis(value: f32, points: i32) -> f32 {
    if !(value > -2.0) {
        return -2.0;
    }
    if value >= points as f32 {
        return points as f32;
    }
    value
}

/// The lattice's grid, read once.
#[derive(Clone)]
pub(crate) struct BLatticeGrid {
    pub pnts: [i32; 3],
    pub kinds: [u8; 3],
    pub first: [f32; 3],
    pub step: [f32; 3],
    /// `BPoint.vec.xyz` per point, u fastest, then v, then w.
    pub points: Vec<Vec3>,
}

impl BLatticeGrid {
    /// The per-axis cell index and the four weights around it.
    fn axis(&self, k: usize, x: f32) -> (i32, [f32; 4]) {
        if self.pnts[k] <= 1 {
            return (0, [0.0, 1.0, 0.0, 0.0]);
        }
        let t = clamp_axis((x - self.first[k]) / self.step[k], self.pnts[k]);
        let i = t.floor();
        (i as i32, weights(t - i, self.kinds[k]))
    }
}

pub(crate) struct BLattice {
    pub lattice: u64,
    pub grid: BLatticeGrid,
    pub strength: f32,
    /// Index of the modifier's vertex group in the object's groups, if named.
    pub group: Option<usize>,
    pub invert: bool,
}

impl BlendFile {
    /// Read a `Lattice` datablock's grid.
    pub(crate) fn lattice_grid(&self, lt: &BlendBlock, dna: &DnaType) -> Option<BLatticeGrid> {
        let pnts = [
            self.int_field(lt, "Lattice", "pntsu", dna) as i32,
            self.int_field(lt, "Lattice", "pntsv", dna) as i32,
            self.int_field(lt, "Lattice", "pntsw", dna) as i32,
        ];
        let byte = |f: &str| self.off("Lattice", f, dna).and_then(|o| lt.data.get(o).copied()).unwrap_or(2);
        let kinds = [byte("typeu"), byte("typev"), byte("typew")];
        let fl = |f: &str| self.float_field(lt, "Lattice", f, dna, 0.0);
        let first = [fl("fu"), fl("fv"), fl("fw")];
        let step = [fl("du"), fl("dv"), fl("dw")];
        let n = (pnts[0].max(0) * pnts[1].max(0) * pnts[2].max(0)) as usize;
        let def = self.block_at(self.ptr(lt, self.off("Lattice", "*def", dna)))?;
        let size = Self::struct_size("BPoint", dna, self.header.pointer_size as usize)?;
        let vec = self.off("BPoint", "vec[4]", dna)?;
        let points: Vec<Vec3> = (0..n)
            .filter_map(|i| {
                let o = i * size + vec;
                let s = def.data.get(o..o + 12)?;
                Some(Vec3::new(self.read_f32(&s[0..4]), self.read_f32(&s[4..8]), self.read_f32(&s[8..12])))
            })
            .collect();
        (points.len() == n && n > 0).then_some(BLatticeGrid { pnts, kinds, first, step, points })
    }
}

pub(super) fn read(f: &BlendFile, md: &BlendBlock, ob: &BlendBlock, dna: &DnaType) -> Option<Box<dyn BModifier>> {
    let lattice = f.ptr(md, Some(f.off("LatticeModifierData", "*object", dna)?));
    let lat_ob = f.block_at_code(lattice, "OB")?;
    let lt = f.block_at_code(f.ptr(lat_ob, f.off("Object", "*data", dna)), "LT")?;
    // The point array is the lattice's own DATA, so it resolves in the
    // lattice's scope, not the mesh object's (`file/scope.rs`).
    let grid = f.scoped(lt, || f.lattice_grid(lt, dna))?;
    let name = f.off("LatticeModifierData", "name[64]", dna).map(|o| super::armature::name64(&md.data, o)).unwrap_or_default();
    let group = (!name.is_empty())
        .then(|| f.mesh_block_of(ob, dna).and_then(|me| f.vertex_group_names(ob, me, dna).iter().position(|g| *g == name)))
        .flatten();
    Some(Box::new(BLattice {
        lattice,
        grid,
        strength: f.float_field(md, "LatticeModifierData", "strength", dna, 1.0),
        group,
        invert: f.int_field(md, "LatticeModifierData", "flag", dna) & INVERT_VGROUP != 0,
    }))
}

impl BLattice {
    #[cfg(test)]
    pub(crate) fn for_test() -> Self {
        Self {
            lattice: 0,
            grid: BLatticeGrid { pnts: [2, 2, 2], kinds: [0; 3], first: [0.0; 3], step: [1.0; 3], points: Vec::new() },
            strength: 1.0,
            group: None,
            invert: false,
        }
    }

    fn driven(&mut self, prop: &str, value: BModValue) -> bool {
        match prop {
            "strength" => self.strength = value.as_f32(),
            _ => return false,
        }
        true
    }
}
impl BModifier for BLattice {
    fn clone_box(&self) -> Box<dyn BModifier> {
        Box::new(Self {
            lattice: self.lattice,
            grid: self.grid.clone(),
            strength: self.strength,
            group: self.group,
            invert: self.invert,
        })
    }

    fn set_driven(&mut self, prop: &str, _component: usize, value: BModValue) -> bool {
        self.driven(prop, value)
    }

    /// `MOD_lattice.cc`'s `is_disabled`: `!object || object->type != OB_LATTICE`.
    /// The type half cannot arise here — the reader only resolves a lattice
    /// through a lattice block — so a missing object is the whole rule.
    fn disabled(&self) -> bool {
        self.lattice == 0
    }

    #[cfg(test)]
    fn is_lattice(&self) -> bool {
        true
    }
    fn wants_weights(&self) -> bool {
        self.group.is_some()
    }

    fn apply(&self, mesh: &mut BlendEvalMesh, ctx: &EvalCtx) {
        let Some(lat_world) = ctx.worlds.get(&self.lattice) else { return };
        let g = &self.grid;
        // `latmat` takes object space to lattice space; its inverse's 3×3
        // brings each point's offset back into object space.
        let latmat = lat_world.inverse() * ctx.world;
        let imat = Mat3::from_mat4(latmat.inverse());
        let mut offsets = Vec::with_capacity(g.points.len());
        for w in 0..g.pnts[2] {
            for v in 0..g.pnts[1] {
                for u in 0..g.pnts[0] {
                    let i = ((w * g.pnts[1] + v) * g.pnts[0] + u) as usize;
                    let rest = Vec3::new(
                        g.first[0] + u as f32 * g.step[0],
                        g.first[1] + v as f32 * g.step[1],
                        g.first[2] + w as f32 * g.step[2],
                    );
                    offsets.push(imat * (g.points[i] - rest));
                }
            }
        }
        let weighted = self.group.filter(|_| mesh.dverts.len() == mesh.vertices.len());
        for (n, co) in mesh.vertices.iter_mut().enumerate() {
            let mut fac = self.strength;
            if let Some(gi) = weighted {
                let w = mesh.dverts[n].iter().find(|(gr, _)| *gr as usize == gi).map_or(0.0, |(_, w)| *w);
                let w = if self.invert { 1.0 - w } else { w };
                if w <= 0.0 {
                    continue;
                }
                fac *= w;
            }
            let p = Vec3::from(*co);
            let vec = latmat.transform_point3(p);
            let (ui, tu) = g.axis(0, vec.x);
            let (vi, tv) = g.axis(1, vec.y);
            let (wi, tw) = g.axis(2, vec.z);
            let w_stride = g.pnts[0] * g.pnts[1];
            let mut out = p;
            for ww in (wi - 1)..=(wi + 2) {
                let w = fac * tw[(ww - wi + 1) as usize];
                let idx_w = (ww * w_stride).clamp(0, (g.pnts[2] - 1) * w_stride);
                for vv in (vi - 1)..=(vi + 2) {
                    let v = w * tv[(vv - vi + 1) as usize];
                    let idx_v = (vv * g.pnts[0]).clamp(0, (g.pnts[1] - 1) * g.pnts[0]);
                    for uu in (ui - 1)..=(ui + 2) {
                        let u = v * tu[(uu - ui + 1) as usize];
                        let idx_u = uu.clamp(0, g.pnts[0] - 1);
                        out += offsets[(idx_w + idx_v + idx_u) as usize] * u;
                    }
                }
            }
            *co = out.into();
        }
    }
}
