// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ported from `intern/cycles/scene/geometry_mesh.cpp`
//! (`GeometryManager::device_update_mesh`): every mesh's triangles, one
//! mesh after another, into `tri_shader` and `tri_vindex`. Curves and
//! points wait for hair and point clouds.

use super::{GeometryInput, GeometryManager};
use crate::scene::{DeviceScene, Mesh};

impl GeometryManager {
    /// `device_update_mesh`. Each mesh writes from its `prim_offset`, so
    /// `geom_calc_offset` comes first.
    pub(super) fn device_update_mesh(dscene: &mut DeviceScene, mesh: &[Mesh], input: &GeometryInput) {
        let count: usize = mesh.iter().map(Mesh::triangle_count).sum();
        dscene.tri_shader = vec![0; count];
        dscene.tri_vindex = vec![[0; 3]; count];
        for mesh in mesh {
            let place = mesh.geometry.prim_offset..mesh.geometry.prim_offset + mesh.triangle_count();
            let shader = dscene.tri_shader.get_mut(place.clone());
            shader.into_iter().for_each(|out| mesh.pack_shader(input.shader, input.default_surface, out));
            let corner = dscene.tri_vindex.get_mut(place);
            corner.into_iter().for_each(|out| mesh.pack_triangle(out));
        }
    }
}
