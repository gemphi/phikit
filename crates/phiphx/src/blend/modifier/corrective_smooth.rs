//! Corrective Smooth (`CorrectiveSmoothModifierData`, Blender
//! `MOD_correctivesmooth.cc`), the "delta mush": smooth the deformed mesh,
//! then add back, in each corner's tangent space, what smoothing took away
//! from the rest shape — so a bent joint keeps its volume and its detail.
//!
//! Translated: both smoothing types, `factor` / `iterations` / `scale`, the
//! vertex group and its invert, Only Smooth, Pin Boundaries, and both rest
//! sources — ORCO (the object's mesh before any modifier) and BIND (the
//! `bind_coords` the Bind operator saved in the file).
//!
//! Blender keeps the deltas in a runtime cache (`delta_cache`) and rebuilds
//! them when a setting changes; they depend only on the rest shape, the
//! topology and the weights, so this computes them on every apply and gets
//! the same numbers without the cache.

#[path = "corrective_smooth_math.rs"]
mod math;

use super::driver::BModValue;
use super::{BModifier, BlendEvalMesh, EvalCtx};
use crate::blend::file::types::*;
use math::{M3, SMOOTH_LENGTH_WEIGHT};
use std::sync::Arc;

const S: &str = "CorrectiveSmoothModifierData";
/// `CorrectiveSmoothModifierFlag`.
const INVERT_VGROUP: u32 = 1 << 0;
const ONLY_SMOOTH: u32 = 1 << 1;
const PIN_BOUNDARY: u32 = 1 << 2;
/// `MOD_CORRECTIVESMOOTH_RESTSOURCE_BIND` (ORCO is 0).
const REST_BIND: u32 = 1;

#[derive(Clone)]
pub(crate) struct BCorrectiveSmooth {
    /// `lambda`, the RNA `factor`.
    pub lambda: f32,
    pub scale: f32,
    /// `repeat`, the RNA `iterations`.
    pub repeat: u32,
    pub flag: u32,
    pub smooth_type: u32,
    pub rest_source: u32,
    /// Index of `defgrp_name` in the object's vertex groups, if it names one.
    pub group: Option<usize>,
    /// `bind_coords`, when the file holds a complete bind.
    pub bind: Option<Arc<Vec<[f32; 3]>>>,
    /// The mesh's own positions (`ob->data`'s `vert_positions`), the ORCO rest.
    pub orco: Option<Arc<Vec<[f32; 3]>>>,
}

pub(super) fn read(f: &BlendFile, md: &BlendBlock, ob: &BlendBlock, dna: &DnaType) -> Option<Box<dyn BModifier>> {
    let flag = f.int_field(md, S, "flag", dna) & 0xffff;
    let rest_source = f.int_field(md, S, "rest_source", dna);
    let me = f.mesh_block_of(ob, dna);
    let name = f.off(S, "defgrp_name[64]", dna).map(|o| super::armature::name64(&md.data, o)).unwrap_or_default();
    let group = match (name.is_empty(), me) {
        (false, Some(me)) => f.vertex_group_names(ob, me, dna).iter().position(|g| *g == name),
        _ => None,
    };
    let only_smooth = flag & ONLY_SMOOTH != 0;
    let bind = match (rest_source == REST_BIND, only_smooth) {
        (true, false) => bind_coords(f, md, dna).map(Arc::new),
        _ => None,
    };
    let orco = match (rest_source == REST_BIND, only_smooth, me) {
        (false, false, Some(me)) => Some(Arc::new(f.scoped(me, || f.extract_mesh_from_block(me, dna).0))),
        _ => None,
    };
    Some(Box::new(BCorrectiveSmooth {
        lambda: f.float_field(md, S, "lambda", dna, 0.5),
        scale: f.float_field(md, S, "scale", dna, 1.0),
        // `uint(csmd->repeat)` of a `short` RNA keeps unsigned.
        repeat: (f.int_field(md, S, "repeat", dna) as u16 as i16).max(0) as u32,
        flag,
        smooth_type: f.int_field(md, S, "smooth_type", dna),
        rest_source,
        group,
        bind,
        orco,
    }))
}

/// `bind_coords[bind_coords_num]`; `None` when unbound or short, which is
/// Blender's "Bind data required" (`BLO_read_array_and_validate_size`
/// drops an array whose size does not match).
fn bind_coords(f: &BlendFile, md: &BlendBlock, dna: &DnaType) -> Option<Vec<[f32; 3]>> {
    // `float (*bind_coords)[3]`: Blender 5.x's DNA spells it `(*bind_coords)()`.
    let at = ["(*bind_coords)()", "(*bind_coords)[3]", "*bind_coords"].iter().find_map(|n| f.off(S, n, dna));
    let block = f.block_at(f.ptr(md, at))?;
    let n = f.int_field(md, S, "bind_coords_num", dna) as usize;
    let bytes = block.data.get(..n.checked_mul(12)?)?;
    Some(bytes.chunks_exact(12).map(|c| [f.read_f32(&c[0..4]), f.read_f32(&c[4..8]), f.read_f32(&c[8..12])]).collect())
}

/// The mesh's edges: every face side once, then the loose edges — the set
/// `mesh->edges()` holds (its order only changes the rounding of the sums).
fn edges_of(mesh: &BlendEvalMesh) -> Vec<(u32, u32)> {
    let mut edges: Vec<(u32, u32)> = Vec::with_capacity(mesh.faces.corners() + mesh.loose_edges.len());
    for face in mesh.faces.iter() {
        for (k, &a) in face.iter().enumerate() {
            let b = face[(k + 1) % face.len()];
            match a.cmp(&b) {
                std::cmp::Ordering::Less => edges.push((a, b)),
                std::cmp::Ordering::Greater => edges.push((b, a)),
                std::cmp::Ordering::Equal => {}
            }
        }
    }
    edges.sort_unstable();
    edges.dedup();
    edges.extend(mesh.loose_edges.iter().copied());
    edges
}

impl BCorrectiveSmooth {
    fn driven(&mut self, prop: &str, value: BModValue) -> bool {
        match prop {
            "factor" => self.lambda = value.as_f32(),
            "scale" => self.scale = value.as_f32(),
            "iterations" => self.repeat = (value.as_f32() as i32).clamp(0, i16::MAX as i32) as u32,
            _ => return false,
        }
        true
    }

    /// `smooth_verts`'s weights: the vertex group (`mesh_get_weights`, with
    /// its invert) and the pinned boundary (`mesh_get_boundaries`), or none
    /// at all when neither applies.
    fn smooth_weights(&self, mesh: &BlendEvalMesh) -> Option<Vec<f32>> {
        let n = mesh.vertices.len();
        let group = self.group.filter(|_| mesh.dverts.len() == n);
        let pin = self.flag & PIN_BOUNDARY != 0;
        if group.is_none() && !pin {
            return None;
        }
        let mut weights = match group {
            Some(g) => mesh
                .dverts
                .iter()
                .map(|dv| {
                    let w = dv.iter().find(|(gr, _)| *gr as usize == g).map_or(0.0, |(_, w)| *w);
                    match self.flag & INVERT_VGROUP != 0 {
                        true => 1.0 - w,
                        false => w,
                    }
                })
                .collect(),
            None => vec![1.0; n],
        };
        if pin {
            for (a, b) in super::geom::boundary(mesh) {
                weights[a as usize] = 0.0;
                weights[b as usize] = 0.0;
            }
        }
        Some(weights)
    }

    /// `smooth_iter`: the type's loop over `repeat` iterations.
    fn smooth(&self, edges: &[(u32, u32)], cos: &mut [[f32; 3]], weights: Option<&[f32]>) {
        match self.smooth_type {
            SMOOTH_LENGTH_WEIGHT => math::smooth_length_weight(edges, cos, weights, self.lambda, self.repeat),
            _ => math::smooth_simple(edges, cos, weights, self.lambda, self.repeat),
        }
    }

    /// `calc_deltas`: what smoothing takes from the rest shape, per corner,
    /// in that corner's tangent space of the smoothed rest.
    fn deltas(&self, mesh: &BlendEvalMesh, edges: &[(u32, u32)], weights: Option<&[f32]>, rest: &[[f32; 3]]) -> Vec<[f32; 3]> {
        let mut smoothed = rest.to_vec();
        self.smooth(edges, &mut smoothed, weights);
        let spaces = math::tangent_spaces(&mesh.faces, &smoothed, None);
        let mut deltas = Vec::with_capacity(spaces.len());
        for (corner, ts) in corner_verts(mesh).zip(&spaces) {
            let v = corner as usize;
            let delta = [rest[v][0] - smoothed[v][0], rest[v][1] - smoothed[v][1], rest[v][2] - smoothed[v][2]];
            let imat: M3 = math::invert_m3(ts).unwrap_or_else(|| math::transpose_m3(ts));
            deltas.push(math::mul_m3_v3(&imat, delta));
        }
        deltas
    }
}

/// `corner_verts`, in corner order.
fn corner_verts(mesh: &BlendEvalMesh) -> impl Iterator<Item = u32> + '_ {
    mesh.faces.iter().flat_map(|f| f.iter().copied())
}

impl BModifier for BCorrectiveSmooth {
    fn clone_box(&self) -> Box<dyn BModifier> {
        Box::new(self.clone())
    }

    fn set_driven(&mut self, prop: &str, _component: usize, value: BModValue) -> bool {
        self.driven(prop, value)
    }

    fn wants_weights(&self) -> bool {
        self.group.is_some()
    }

    /// `correctivesmooth_modifier_do`. Where Blender sets a modifier error
    /// (no bind, a vertex count that no longer matches the rest) the mesh is
    /// left as it came, as Blender leaves it.
    fn apply(&self, mesh: &mut BlendEvalMesh, _ctx: &EvalCtx) {
        let n = mesh.vertices.len();
        if n == 0 {
            return;
        }
        let edges = edges_of(mesh);
        let weights = self.smooth_weights(mesh);
        if self.flag & ONLY_SMOOTH != 0 {
            self.smooth(&edges, &mut mesh.vertices, weights.as_deref());
            return;
        }
        let rest = match self.rest_source {
            REST_BIND => self.bind.as_deref(),
            _ => self.orco.as_deref(),
        };
        let Some(rest) = rest.filter(|r| r.len() == n) else { return };
        let deltas = self.deltas(mesh, &edges, weights.as_deref(), rest);
        // The delta mush itself: smooth the deformed mesh, then add the
        // deltas back through its own tangent spaces, angle-weighted.
        self.smooth(&edges, &mut mesh.vertices, weights.as_deref());
        let corners = mesh.faces.corners();
        let mut tw = vec![0.0f32; corners];
        let mut twv = vec![0.0f32; n];
        let spaces = math::tangent_spaces(&mesh.faces, &mesh.vertices, Some((&mut tw, &mut twv)));
        let verts: Vec<u32> = corner_verts(mesh).collect();
        for (l, &v) in verts.iter().enumerate() {
            let v = v as usize;
            let weight = tw[l] / twv[v];
            // Catches zero and the division by zero.
            if !(weight > 0.0) {
                continue;
            }
            let d = math::mul_m3_v3(&spaces[l], deltas[l]);
            let d = [d[0] * weight, d[1] * weight, d[2] * weight];
            math::madd(&mut mesh.vertices[v], d, self.scale);
        }
    }
}
