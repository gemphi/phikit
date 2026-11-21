// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use super::*;
use crate::kernel::flag;

#[test]
fn every_mesh_s_triangles_are_in_the_arrays_one_mesh_after_another() {
    let far = mesh_of(&[[5.0, 0.0, 0.0], [6.0, 0.0, 0.0], [5.0, 1.0, 0.0]], &[[2, 1, 0]]);
    let mut rig = Rig::new(vec![square(), far]);
    rig.mesh[1].geometry.set_used_shader(vec![0]);
    rig.update();
    // Vertices are numbered within their own mesh.
    assert_eq!(rig.dscene.tri_vindex, [[0, 1, 2], [0, 2, 3], [2, 1, 0]]);
    let cast = flag("SHADER_CAST_SHADOW");
    // The square has no shader of its own: the default surface. The other has shader 7.
    assert_eq!(rig.dscene.tri_shader, [99 | cast, 99 | cast, 7 | cast]);
}

#[test]
fn a_scene_with_no_triangles_has_empty_arrays() {
    let mut rig = Rig::new(vec![Mesh::new()]);
    rig.update();
    assert!(rig.dscene.tri_vindex.is_empty() && rig.dscene.tri_shader.is_empty() && rig.dscene.tri_vert.is_empty());
}
