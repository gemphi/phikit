//! Gizmo vertex generation — XYZ translation gizmo for selected entities.

use super::super::renderer::GpuRenderer;

/// Axis direction for gizmo shaft/cone generation.
enum Axis {
    X,
    Y,
    Z,
}

impl GpuRenderer {
    /// Build and upload the translation gizmo vertex buffer for a selected entity.
    ///
    /// Generates a professional XYZ translation gizmo (like Blender/Unity/Unreal)
    /// with thick shafts, solid cone arrowheads, and plane handles.
    pub(crate) fn update_gizmo_vertices(
        &self,
        device: &wgpu::Device,
        scene: &crate::ecs::Scene,
        selected_id: u64,
    ) {
        if let Some(t) = scene
            .world
            .get::<crate::ecs::Transform>(crate::ecs::Entity::from_id(selected_id))
        {
            let pos = t.translation;
            let shaft_len = 1.5f32;
            let cone_len = 0.25f32;
            let cone_w = 0.08f32;
            let thickness = 0.015f32;
            let center_box = 0.12f32;

            let mut vertices: Vec<crate::gpu::GridVertex> = Vec::with_capacity(200);

            let mut line = |p0: [f32; 3], p1: [f32; 3], color: [f32; 4]| {
                vertices.push(crate::gpu::GridVertex { position: p0, color });
                vertices.push(crate::gpu::GridVertex { position: p1, color });
            };

            Self::build_center_cube(&mut line, pos, center_box);
            Self::build_axis(&mut line, pos, center_box, shaft_len, cone_len, cone_w, thickness, Axis::X);
            Self::build_axis(&mut line, pos, center_box, shaft_len, cone_len, cone_w, thickness, Axis::Y);
            Self::build_axis(&mut line, pos, center_box, shaft_len, cone_len, cone_w, thickness, Axis::Z);
            Self::build_plane_handles(&mut line, pos);

            use wgpu::util::DeviceExt;
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Gizmo Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let vert_count = vertices.len() as u32;
            *self.gizmo_vertex_buffer.lock().unwrap() = Some(buffer);
            *self.gizmo_vertex_count.lock().unwrap() = vert_count;
        }
    }

    /// Clear the gizmo when no entity is selected.
    pub(crate) fn clear_gizmo(&self) {
        *self.gizmo_vertex_buffer.lock().unwrap() = None;
        *self.gizmo_vertex_count.lock().unwrap() = 0;
    }

    fn build_center_cube(
        line: &mut dyn FnMut([f32; 3], [f32; 3], [f32; 4]),
        pos: glam::Vec3,
        cb: f32,
    ) {
        let cc = [0.95, 0.85, 0.2, 1.0]; // gold center
        let (x, y, z) = (pos.x, pos.y, pos.z);
        // Bottom face
        line([x - cb, y - cb, z - cb], [x + cb, y - cb, z - cb], cc);
        line([x + cb, y - cb, z - cb], [x + cb, y - cb, z + cb], cc);
        line([x + cb, y - cb, z + cb], [x - cb, y - cb, z + cb], cc);
        line([x - cb, y - cb, z + cb], [x - cb, y - cb, z - cb], cc);
        // Top face
        line([x - cb, y + cb, z - cb], [x + cb, y + cb, z - cb], cc);
        line([x + cb, y + cb, z - cb], [x + cb, y + cb, z + cb], cc);
        line([x + cb, y + cb, z + cb], [x - cb, y + cb, z + cb], cc);
        line([x - cb, y + cb, z + cb], [x - cb, y + cb, z - cb], cc);
        // Verticals
        line([x - cb, y - cb, z - cb], [x - cb, y + cb, z - cb], cc);
        line([x + cb, y - cb, z - cb], [x + cb, y + cb, z - cb], cc);
        line([x + cb, y - cb, z + cb], [x + cb, y + cb, z + cb], cc);
        line([x - cb, y - cb, z + cb], [x - cb, y + cb, z + cb], cc);
    }

    fn build_axis(
        line: &mut dyn FnMut([f32; 3], [f32; 3], [f32; 4]),
        pos: glam::Vec3,
        cb: f32,
        shaft_len: f32,
        cone_len: f32,
        cone_w: f32,
        thickness: f32,
        axis: Axis,
    ) {
        let (color, make_pt) = match axis {
            Axis::X => ([0.9, 0.15, 0.15, 1.0], Self::pt_x as fn(glam::Vec3, f32, f32, f32) -> [f32; 3]),
            Axis::Y => ([0.2, 0.8, 0.15, 1.0], Self::pt_y as fn(glam::Vec3, f32, f32, f32) -> [f32; 3]),
            Axis::Z => ([0.2, 0.4, 0.95, 1.0], Self::pt_z as fn(glam::Vec3, f32, f32, f32) -> [f32; 3]),
        };
        // Main shaft (5 parallel lines for thickness)
        line(make_pt(pos, cb, 0.0, 0.0), make_pt(pos, shaft_len, 0.0, 0.0), color);
        line(make_pt(pos, cb, thickness, 0.0), make_pt(pos, shaft_len, thickness, 0.0), color);
        line(make_pt(pos, cb, -thickness, 0.0), make_pt(pos, shaft_len, -thickness, 0.0), color);
        line(make_pt(pos, cb, 0.0, thickness), make_pt(pos, shaft_len, 0.0, thickness), color);
        line(make_pt(pos, cb, 0.0, -thickness), make_pt(pos, shaft_len, 0.0, -thickness), color);
        // Cone arrowhead (8 lines)
        let tip = shaft_len + cone_len;
        for i in 0..8 {
            let angle = (i as f32) * std::f32::consts::PI * 2.0 / 8.0;
            let d1 = cone_w * angle.cos();
            let d2 = cone_w * angle.sin();
            line(make_pt(pos, shaft_len, d1, d2), make_pt(pos, tip, 0.0, 0.0), color);
        }
        // Cone base ring
        for i in 0..8 {
            let a0 = (i as f32) * std::f32::consts::PI * 2.0 / 8.0;
            let a1 = ((i + 1) as f32) * std::f32::consts::PI * 2.0 / 8.0;
            line(
                make_pt(pos, shaft_len, cone_w * a0.cos(), cone_w * a0.sin()),
                make_pt(pos, shaft_len, cone_w * a1.cos(), cone_w * a1.sin()),
                color,
            );
        }
    }

    // Point constructors for each axis direction (primary, perp1, perp2)
    fn pt_x(pos: glam::Vec3, primary: f32, perp1: f32, perp2: f32) -> [f32; 3] {
        [pos.x + primary, pos.y + perp1, pos.z + perp2]
    }
    fn pt_y(pos: glam::Vec3, primary: f32, perp1: f32, perp2: f32) -> [f32; 3] {
        [pos.x + perp1, pos.y + primary, pos.z + perp2]
    }
    fn pt_z(pos: glam::Vec3, primary: f32, perp1: f32, perp2: f32) -> [f32; 3] {
        [pos.x + perp1, pos.y + perp2, pos.z + primary]
    }

    fn build_plane_handles(
        line: &mut dyn FnMut([f32; 3], [f32; 3], [f32; 4]),
        pos: glam::Vec3,
    ) {
        let ph = 0.35f32; // plane handle offset
        let ps = 0.15f32; // plane handle size
        let (x, y, z) = (pos.x, pos.y, pos.z);

        // XY Plane handle (yellow-ish)
        let xyc = [0.9, 0.8, 0.1, 0.8];
        line([x + ph, y + ph, z], [x + ph + ps, y + ph, z], xyc);
        line([x + ph + ps, y + ph, z], [x + ph + ps, y + ph + ps, z], xyc);
        line([x + ph + ps, y + ph + ps, z], [x + ph, y + ph + ps, z], xyc);
        line([x + ph, y + ph + ps, z], [x + ph, y + ph, z], xyc);

        // XZ Plane handle (magenta-ish)
        let xzc = [0.9, 0.3, 0.8, 0.8];
        line([x + ph, y, z + ph], [x + ph + ps, y, z + ph], xzc);
        line([x + ph + ps, y, z + ph], [x + ph + ps, y, z + ph + ps], xzc);
        line([x + ph + ps, y, z + ph + ps], [x + ph, y, z + ph + ps], xzc);
        line([x + ph, y, z + ph + ps], [x + ph, y, z + ph], xzc);

        // YZ Plane handle (cyan-ish)
        let yzc = [0.2, 0.8, 0.9, 0.8];
        line([x, y + ph, z + ph], [x, y + ph + ps, z + ph], yzc);
        line([x, y + ph + ps, z + ph], [x, y + ph + ps, z + ph + ps], yzc);
        line([x, y + ph + ps, z + ph + ps], [x, y + ph, z + ph + ps], yzc);
        line([x, y + ph, z + ph + ps], [x, y + ph, z + ph], yzc);
    }
}
