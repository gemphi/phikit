// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/geometry_bvh.cpp`
//! (`Geometry::compute_bvh`, `GeometryManager::device_update_bvh`) and
//! `geometry.cpp` (`Geometry::need_build_bvh`, `Geometry::tag_update`):
//! the tree of a geometry that is instanced, and the scene's tree over
//! the objects, written into the device's arrays.
//!
//! The scene's tree (`Scene::bvh`) is handed in, since the scene itself
//! is not ported. Cycles builds the geometries' trees on several threads
//! and asks the device to build each; here they are built one after
//! another, in Cycles' own layout. Not ported: progress, cancelling, and
//! the tags `tag_update` gives the light manager.

use super::{Geometry, GeometryInput, GeometryManager};
use crate::bvh::{self, Bvh2, BvhLayout, BvhParam, BvhType, PackedBvh};
use crate::graph::Value;
use crate::kernel::flag;
use crate::scene::{DeviceScene, Mesh, Object, SceneParam};

/// The settings a geometry's tree and the scene's share.
fn bvh_param(dscene: &DeviceScene, param: &SceneParam, layout: BvhLayout) -> BvhParam {
    BvhParam {
        use_spatial_split: param.use_bvh_spatial_split,
        bvh_layout: layout,
        use_unaligned_node: dscene.data.int("bvh", "have_curves") != 0 && param.use_bvh_unaligned_node,
        motion_triangle_step_count: param.bvh_time_step_count,
        motion_curve_step_count: param.bvh_time_step_count,
        motion_point_step_count: param.bvh_time_step_count,
        bvh_type: param.bvh_type,
        curve_subdivision: param.curve_subdivision(),
        ..BvhParam::default()
    }
}

impl Geometry {
    /// `need_build_bvh`: it has a tree of its own. One that is instanced
    /// does, so that the objects placing it share the tree, and so does
    /// every geometry in a layout whose top level holds only instances.
    pub fn need_build_bvh(&self, layout: BvhLayout) -> bool {
        self.is_instanced() || layout.instance_only()
    }

    /// `tag_update`: the geometry changed; with `rebuild`, in a way its
    /// tree cannot be refitted to, as when it has other triangles. The
    /// answer is the flag to tag the geometry manager with.
    pub fn tag_update(&mut self, rebuild: bool) -> u32 {
        self.need_update_rebuild |= rebuild;
        GeometryManager::flag("GEOMETRY_MODIFIED")
    }

    /// `compute_bvh`, for the mesh at `index`: its bounds and, when it
    /// has a tree of its own, that tree. A tree it has already is
    /// refitted when the scene's trees are dynamic and only vertices
    /// moved; else it is built.
    pub fn compute_bvh(mesh: &mut [Mesh], index: usize, dscene: &DeviceScene, param: &SceneParam, layout: BvhLayout) {
        let Some(own) = mesh.get_mut(index) else {
            return;
        };
        own.compute_bound();
        if own.geometry.need_build_bvh(layout) {
            // Every ray sees it here: what sees an object is tested at
            // the top level.
            let mut object = Object::new(index);
            object.node.set("is_shadow_catcher", Value::Boolean(true));
            object.node.set("visibility", Value::Uint(flag("PATH_RAY_VISIBILITY_ALL")));
            let refit = !own.geometry.need_update_rebuild && param.bvh_type == BvhType::Dynamic;
            let bvh = match own.geometry.bvh.take().filter(|_| refit) {
                Some(mut bvh) => {
                    bvh.refit(&[object], mesh);
                    Some(bvh)
                }
                None => {
                    let made = bvh::create(BvhParam { use_compact_structure: param.use_bvh_compact_structure, ..bvh_param(dscene, param, layout) });
                    made.map(|mut bvh| {
                        bvh.build(&[object], mesh);
                        bvh
                    })
                }
            };
            mesh[index].geometry.bvh = bvh;
        }
        let geometry = &mut mesh[index].geometry;
        (geometry.need_update_rebuild, geometry.need_update_bvh_for_offset) = (false, false);
    }
}

impl GeometryManager {
    /// `device_update_bvh`: build the scene's tree over the objects and
    /// give its arrays, with every geometry's tree in them, to the
    /// device. A layout that is not ported leaves the device no tree.
    pub fn device_update_bvh(dscene: &mut DeviceScene, mesh: &[Mesh], object: &[Object], bvh: &mut Option<Bvh2>, input: &GeometryInput) {
        let layout = BvhLayout::best(input.param.bvh_layout, input.bvh_layout_mask);
        if bvh.is_none() {
            *bvh = bvh::create(BvhParam { top_level: true, ..bvh_param(dscene, input.param, layout) });
        }
        let pack = match bvh {
            Some(bvh) => {
                bvh.build(object, mesh);
                std::mem::take(&mut bvh.pack)
            }
            None => PackedBvh { root_index: -1, ..PackedBvh::default() },
        };
        dscene.bvh_node = pack.node;
        dscene.bvh_leaf_node = pack.leaf_node;
        dscene.object_node = pack.object_node;
        dscene.prim_type = pack.prim_type;
        dscene.prim_visibility = pack.prim_visibility;
        dscene.prim_index = pack.prim_index;
        dscene.prim_object = pack.prim_object;
        dscene.prim_time = pack.prim_time;
        dscene.data.set_int("bvh", "root", pack.root_index);
        dscene.data.set_int("bvh", "use_bvh_steps", i32::from(input.param.bvh_time_step_count != 0));
        dscene.data.set_int("bvh", "curve_subdivisions", input.param.curve_subdivision());
    }
}
