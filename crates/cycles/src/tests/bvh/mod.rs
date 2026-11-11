// SPDX-License-Identifier: Apache-2.0
//! Tests of `src/bvh`, one file for each file there. This file tests
//! `mod.rs` and holds the meshes the others build trees over, and the walk
//! they check a packed tree with.

mod binning;
mod build;
mod bvh2;
mod node;
mod param;
mod reference;
mod sort;
mod split;

use crate::bvh::*;
use crate::scene::{Mesh, Object};
pub(crate) use crate::tests::scene::mesh::{mesh_of, moving_square, square};
use crate::util::BoundBox;

/// `count` squares of side 1 in a row along x, one unit apart, two
/// triangles each.
pub(crate) fn strip(count: usize) -> Mesh {
    let (mut point, mut triangle) = (Vec::new(), Vec::new());
    for at in 0..count {
        let (x, first) = (at as f32 * 2.0, (at * 4) as i32);
        point.extend([[x, 0.0, 0.0], [x + 1.0, 0.0, 0.0], [x + 1.0, 1.0, 0.0], [x, 1.0, 0.0]]);
        triangle.extend([[first, first + 1, first + 2], [first, first + 2, first + 3]]);
    }
    mesh_of(&point, &triangle)
}

/// A strip of sixteen squares with one long thin triangle laid over all
/// of it: whichever side of a cut that triangle goes to, it makes the box
/// there as long as the strip, unless it is cut too.
pub(crate) fn strip_under_a_sliver() -> Mesh {
    let strip = strip(16);
    let mut point = strip.geometry.position().to_vec();
    let mut triangle: Vec<[i32; 3]> = (0..strip.triangle_count()).map(|index| strip.triangle(index).v).collect();
    let first = point.len() as i32;
    point.extend([[0.0, 0.0, 0.0], [31.0, 0.0, 0.0], [31.0, 1.0, 0.0]]);
    triangle.push([first, first + 1, first + 2]);
    mesh_of(&point, &triangle)
}

/// The box around a triangle of `mesh`, at every motion step.
pub(crate) fn triangle_box(mesh: &Mesh, index: usize) -> BoundBox {
    let mut bound = BoundBox::EMPTY;
    let position = mesh.geometry.position_attribute().expect("a mesh has positions");
    (0..position.motion_step_count()).for_each(|step| mesh.triangle(index).bound_grow(position.data(step).float3(), &mut bound));
    bound
}

/// A reference for each triangle of `mesh`, as object 0 places it, and
/// the range of them all.
pub(crate) fn reference_of(mesh: &Mesh) -> (BvhRange, Vec<BvhReference>) {
    let reference: Vec<BvhReference> = (0..mesh.triangle_count()).map(|index| BvhReference::new(triangle_box(mesh, index), index as i32, 0, mesh.primitive_type())).collect();
    (range_of(&reference), reference)
}

/// The range of all these references.
pub(crate) fn range_of(reference: &[BvhReference]) -> BvhRange {
    let mut range = BvhRange { size: reference.len(), ..BvhRange::default() };
    for reference in reference {
        range.bound.grow_box(&reference.bound);
        range.cent_bound.grow(reference.bound.center2());
    }
    range
}

/// Boxes of side 1 whose least corners are at these places along x.
pub(crate) fn row(x: &[f32]) -> (BvhRange, Vec<BvhReference>) {
    let boxed = |(index, x): (usize, &f32)| BvhReference::new(BoundBox { min: [*x, 0.0, 0.0], max: [*x + 1.0, 1.0, 1.0] }, index as i32, 0, PrimitiveBit::new().triangle);
    let reference: Vec<BvhReference> = x.iter().enumerate().map(boxed).collect();
    (range_of(&reference), reference)
}

/// `outer` holds all of `inner`.
pub(crate) fn holds(outer: &BoundBox, inner: &BoundBox) -> bool {
    (0..3).all(|axis| outer.min[axis] <= inner.min[axis] && outer.max[axis] >= inner.max[axis])
}

/// The box an inner node, four words of four, keeps for a child.
pub(crate) fn kept_box(node: &[[i32; 4]], child: usize) -> BoundBox {
    let part = |row: usize, at: usize| f32::from_bits(node[row][at] as u32);
    BoundBox { min: [1, 2, 3].map(|row| part(row, child)), max: [1, 2, 3].map(|row| part(row, child + 2)) }
}

/// Walk a packed tree from `root`, a node or a leaf with every bit
/// turned, and check that the box an inner node keeps for a child holds
/// everything under that child; `bound_of` gives the box of a place in
/// the primitive arrays. The answer is every place reached, and the box
/// around them.
pub(crate) fn walk(pack: &PackedBvh, root: i32, bound_of: &dyn Fn(usize) -> BoundBox) -> (Vec<usize>, BoundBox) {
    let (mut reached, mut bound) = (Vec::new(), BoundBox::EMPTY);
    match root < 0 {
        true => {
            let leaf = pack.leaf_node[!root as usize];
            let place = match leaf[0] < 0 {
                true => !leaf[0] as usize..!leaf[0] as usize + 1,
                false => leaf[0] as usize..leaf[1] as usize,
            };
            for place in place {
                bound.grow_box(&bound_of(place));
                reached.push(place);
            }
        }
        false => {
            let at = root as usize;
            for child in 0..2 {
                let (below, under) = walk(pack, pack.node[at][2 + child], bound_of);
                let kept = kept_box(&pack.node[at..at + 4], child);
                assert!(!under.valid() || holds(&kept, &under), "node {at}, child {child}: {kept:?} does not hold {under:?}");
                bound.grow_box(&under);
                reached.extend(below);
            }
        }
    }
    (reached, bound)
}

/// Whether a walk that goes only into boxes that hold `point` comes to a
/// leaf with the primitive `wanted`: what a ray through the point needs.
pub(crate) fn finds(pack: &PackedBvh, root: i32, point: [f32; 3], wanted: i32) -> bool {
    match root < 0 {
        true => {
            let leaf = pack.leaf_node[!root as usize];
            (leaf[0].max(0) as usize..leaf[1].max(0) as usize).any(|place| pack.prim_index[place] == wanted)
        }
        false => {
            let node = &pack.node[root as usize..root as usize + 4];
            (0..2).any(|child| holds(&kept_box(node, child), &BoundBox { min: point, max: point }) && finds(pack, node[0][2 + child], point, wanted))
        }
    }
}

/// The tree of `mesh` alone, as one object that every ray sees places it.
pub(crate) fn tree_of(mesh: &Mesh, param: BvhParam) -> Bvh2 {
    let mut bvh = Bvh2::new(param);
    bvh.build(&[Object::new(0)], std::slice::from_ref(mesh));
    bvh
}

#[test]
fn a_tree_is_made_only_in_cycles_own_layout() {
    assert!(create(BvhParam::default()).is_some());
    assert!(create(BvhParam { bvh_layout: BvhLayout::of("EMBREE"), ..BvhParam::default() }).is_none());
    assert!(create(BvhParam { bvh_layout: BvhLayout::NONE, ..BvhParam::default() }).is_none());
}

#[test]
fn a_new_packed_tree_is_empty() {
    let pack = PackedBvh::default();
    assert!(pack.node.is_empty() && pack.leaf_node.is_empty() && pack.prim_index.is_empty() && pack.object_node.is_empty());
    assert_eq!(pack.root_index, 0);
}
