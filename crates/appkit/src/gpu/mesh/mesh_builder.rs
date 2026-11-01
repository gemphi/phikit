//! MeshBuffer container and primitive builders

use super::mesh_data::{MaterialUniform, Vertex};

/// GPU mesh buffer container
pub struct MeshBuffer {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: Option<wgpu::Buffer>,
    pub material_bind_group: wgpu::BindGroup,
    pub material_buffer: wgpu::Buffer,
    pub material_data: MaterialUniform,
    pub mesh_path: String,
    pub vertex_count: u32,
    pub index_count: u32,
    pub entity_id: u64,
    pub bounds: Option<([f32; 3], [f32; 3])>,
}

impl MeshBuffer {
    pub fn new(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        vertices: &[Vertex],
        indices: Option<&[u32]>,
        material: MaterialUniform,
        mesh_path: String,
        entity_id: u64,
    ) -> Self {
        use wgpu::util::DeviceExt;
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = indices.map(|idx| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Mesh Index Buffer"),
                contents: bytemuck::cast_slice(idx),
                usage: wgpu::BufferUsages::INDEX,
            })
        });
        let material_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Material Uniform Buffer"),
            contents: bytemuck::cast_slice(&[material]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let material_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Material Bind Group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: material_buffer.as_entire_binding(),
            }],
        });
        let bounds = if !vertices.is_empty() {
            let mut bmin = [f32::MAX; 3];
            let mut bmax = [f32::MIN; 3];
            for v in vertices {
                for i in 0..3 {
                    bmin[i] = bmin[i].min(v.position[i]);
                    bmax[i] = bmax[i].max(v.position[i]);
                }
            }
            Some((bmin, bmax))
        } else {
            None
        };
        Self {
            vertex_buffer, index_buffer, material_bind_group, material_buffer,
            material_data: material, mesh_path,
            vertex_count: vertices.len() as u32,
            index_count: indices.map(|i| i.len()).unwrap_or(0) as u32,
            entity_id, bounds,
        }
    }

    pub fn cube(
        device: &wgpu::Device, layout: &wgpu::BindGroupLayout,
        size: f32, color: [f32; 4], entity_id: u64,
    ) -> Self {
        let s = size / 2.0;
        let v = |p: [f32; 3], n: [f32; 3]| Vertex::new(p, n, color);
        let vertices = vec![
            v([-s,-s, s],[0.0,0.0,1.0]), v([s,-s, s],[0.0,0.0,1.0]),
            v([ s, s, s],[0.0,0.0,1.0]), v([-s, s, s],[0.0,0.0,1.0]),
            v([-s,-s,-s],[0.0,0.0,-1.0]), v([-s, s,-s],[0.0,0.0,-1.0]),
            v([ s, s,-s],[0.0,0.0,-1.0]), v([ s,-s,-s],[0.0,0.0,-1.0]),
            v([-s, s,-s],[0.0,1.0,0.0]), v([-s, s, s],[0.0,1.0,0.0]),
            v([ s, s, s],[0.0,1.0,0.0]), v([ s, s,-s],[0.0,1.0,0.0]),
            v([-s,-s,-s],[0.0,-1.0,0.0]), v([ s,-s,-s],[0.0,-1.0,0.0]),
            v([ s,-s, s],[0.0,-1.0,0.0]), v([-s,-s, s],[0.0,-1.0,0.0]),
            v([ s,-s,-s],[1.0,0.0,0.0]), v([ s, s,-s],[1.0,0.0,0.0]),
            v([ s, s, s],[1.0,0.0,0.0]), v([ s,-s, s],[1.0,0.0,0.0]),
            v([-s,-s,-s],[-1.0,0.0,0.0]), v([-s,-s, s],[-1.0,0.0,0.0]),
            v([-s, s, s],[-1.0,0.0,0.0]), v([-s, s,-s],[-1.0,0.0,0.0]),
        ];
        let mut indices = Vec::new();
        for i in 0..6u32 {
            let o = i * 4;
            indices.extend_from_slice(&[o, o+1, o+2, o, o+2, o+3]);
        }
        Self::new(device, layout, &vertices, Some(&indices),
            MaterialUniform::default(), "builtin:cube".to_string(), entity_id)
    }

    pub fn sphere(
        device: &wgpu::Device, layout: &wgpu::BindGroupLayout,
        radius: f32, color: [f32; 4], entity_id: u64,
    ) -> Self {
        let (stacks, slices): (u32, u32) = (16, 24);
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        for i in 0..=stacks {
            let phi = std::f32::consts::PI * (i as f32) / (stacks as f32);
            let (sin_phi, cos_phi) = (phi.sin(), phi.cos());
            for j in 0..=slices {
                let theta = 2.0 * std::f32::consts::PI * (j as f32) / (slices as f32);
                let (x, y, z) = (theta.cos() * sin_phi, cos_phi, theta.sin() * sin_phi);
                vertices.push(Vertex::new(
                    [x * radius, y * radius, z * radius], [x, y, z], color));
            }
        }
        for i in 0..stacks {
            for j in 0..slices {
                let (first, second) = (i * (slices + 1) + j, i * (slices + 1) + j + slices + 1);
                indices.extend_from_slice(&[first, second, first+1, second, second+1, first+1]);
            }
        }
        Self::new(device, layout, &vertices, Some(&indices),
            MaterialUniform::default(), "builtin:sphere".to_string(), entity_id)
    }

    pub fn cylinder(
        device: &wgpu::Device, layout: &wgpu::BindGroupLayout,
        radius: f32, height: f32, color: [f32; 4], entity_id: u64,
    ) -> Self {
        let segments: u32 = 24;
        let half_h = height / 2.0;
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        for i in 0..=segments {
            let theta = 2.0 * std::f32::consts::PI * (i as f32) / (segments as f32);
            let (cos_t, sin_t) = (theta.cos(), theta.sin());
            let normal = [cos_t, 0.0, sin_t];
            vertices.push(Vertex::new([cos_t * radius, -half_h, sin_t * radius], normal, color));
            vertices.push(Vertex::new([cos_t * radius, half_h, sin_t * radius], normal, color));
        }
        for i in 0..segments {
            let base = i * 2;
            indices.extend_from_slice(&[base, base+1, base+2, base+1, base+3, base+2]);
        }
        // Top cap
        let top_center = vertices.len() as u32;
        vertices.push(Vertex::new([0.0, half_h, 0.0], [0.0, 1.0, 0.0], color));
        for i in 0..=segments {
            let theta = 2.0 * std::f32::consts::PI * (i as f32) / (segments as f32);
            vertices.push(Vertex::new(
                [theta.cos() * radius, half_h, theta.sin() * radius], [0.0, 1.0, 0.0], color));
        }
        for i in 0..segments { indices.extend_from_slice(&[top_center, top_center+1+i, top_center+2+i]); }
        // Bottom cap
        let bot_center = vertices.len() as u32;
        vertices.push(Vertex::new([0.0, -half_h, 0.0], [0.0, -1.0, 0.0], color));
        for i in 0..=segments {
            let theta = 2.0 * std::f32::consts::PI * (i as f32) / (segments as f32);
            vertices.push(Vertex::new(
                [theta.cos() * radius, -half_h, theta.sin() * radius], [0.0, -1.0, 0.0], color));
        }
        for i in 0..segments { indices.extend_from_slice(&[bot_center, bot_center+2+i, bot_center+1+i]); }
        Self::new(device, layout, &vertices, Some(&indices),
            MaterialUniform::default(), "builtin:cylinder".to_string(), entity_id)
    }
}
