// SPDX-License-Identifier: Apache-2.0
//! A small scene the kernels are run on: a camera looking down at the
//! origin from `z = 5`, planes and boxes of one surface each, and a world.
use std::sync::Arc;

use crate::blender::{CameraSource, MaterialSource, MeshSource, NodeSurface, SensorFit, sync_camera, sync_mesh, transform_of};
use crate::device::CpuDevice;
use crate::kernel::KernelGlobals;
use crate::kernel::device::cpu_kernels;
use crate::scene::{Object, Scene, Surface, World};
use crate::truth::SessionRenderer;

/// A scene with a perspective camera, 50 mm on a 36 mm sensor, at
/// `(0, 0, 5)` looking along -z, and a world of this colour.
pub(crate) struct Rig {
    pub scene: Scene,
    pub width: i32,
    pub height: i32,
}

/// The camera object's world matrix: a translation, Blender's layout.
pub(crate) fn translation(x: f32, y: f32, z: f32) -> [[f32; 4]; 4] {
    [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [x, y, z, 1.0]]
}

pub(crate) fn camera_source(orthographic: bool) -> CameraSource {
    CameraSource {
        name: "camera".into(),
        matrix: translation(0.0, 0.0, 5.0),
        orthographic,
        lens: 50.0,
        sensor_width: 36.0,
        sensor_height: 24.0,
        sensor_fit: SensorFit::Auto,
        shift: [0.0; 2],
        ortho_scale: 6.0,
        clip_start: 0.1,
        clip_end: 100.0,
    }
}

/// A square of half-side `half` in the plane `z`, two triangles facing +z.
pub(crate) fn plane(half: f32, z: f32) -> MeshSource {
    MeshSource {
        vertex: vec![[-half, -half, z], [half, -half, z], [half, half, z], [-half, half, z]],
        triangle: vec![[0, 1, 2], [0, 2, 3]],
        slot: vec![0, 0],
        smooth: vec![false, false],
        uv: Vec::new(),
        material: vec![MaterialSource { name: "m".into(), base_color: [0.8; 3], emission_color: [0.0; 3], emission_strength: 0.0, diffuse_color: [0.8; 3], node: NodeSurface::Unknown, node_kind: Vec::new() }],
    }
}

/// A closed box of half-side `half` about the origin, outward normals.
pub(crate) fn cube(half: f32) -> MeshSource {
    let h = half;
    let vertex = vec![[-h, -h, -h], [h, -h, -h], [h, h, -h], [-h, h, -h], [-h, -h, h], [h, -h, h], [h, h, h], [-h, h, h]];
    let triangle = vec![[0, 3, 2], [0, 2, 1], [4, 5, 6], [4, 6, 7], [0, 1, 5], [0, 5, 4], [2, 3, 7], [2, 7, 6], [1, 2, 6], [1, 6, 5], [3, 0, 4], [3, 4, 7]];
    let count = triangle.len();
    MeshSource { vertex, triangle, slot: vec![0; count], smooth: vec![false; count], uv: Vec::new(), material: plane(1.0, 0.0).material }
}

impl Rig {
    pub fn new(width: i32, height: i32, world: [f32; 3]) -> Rig {
        let mut scene = Scene::new();
        scene.camera = sync_camera(&camera_source(false), width, height);
        scene.world = World { color: world, strength: 1.0 };
        scene.integrator.node.set("aa_samples", crate::graph::Value::Int(16));
        Rig { scene, width, height }
    }

    pub fn orthographic(mut self) -> Rig {
        self.scene.camera = sync_camera(&camera_source(true), self.width, self.height);
        self
    }

    /// Put `mesh` in the scene with this surface, at `matrix`; its index.
    pub fn add(&mut self, mesh: &MeshSource, surface: Surface, matrix: [[f32; 4]; 4]) -> usize {
        let shader = self.scene.add_shader(surface);
        self.scene.mesh.push(sync_mesh(mesh, &[shader]));
        let mut object = Object::new(self.scene.mesh.len() - 1);
        object.set_transform(transform_of(&matrix));
        self.scene.object.push(object);
        self.scene.mesh.len() - 1
    }

    /// Another object placed with the mesh the index names.
    pub fn instance(&mut self, mesh: usize, matrix: [[f32; 4]; 4]) {
        let mut object = Object::new(mesh);
        object.set_transform(transform_of(&matrix));
        self.scene.object.push(object);
    }

    /// The managers run; the kernels' view of the scene.
    pub fn kernel(&mut self) -> Arc<KernelGlobals> {
        let dscene = self.scene.device_update();
        Arc::new(KernelGlobals::new(Arc::new(dscene)))
    }

    /// Render `samples` and give each pixel's linear colour, the bottom row
    /// first as Cycles keeps the buffer.
    pub fn render(&mut self, samples: i32) -> Vec<[f32; 3]> {
        let kg = self.kernel();
        let device = Arc::new(CpuDevice::with_kernels(cpu_kernels(kg.clone())));
        let mut renderer = SessionRenderer::new(device, (self.width as usize, self.height as usize), samples);
        renderer.pass_stride = kg.film.pass_stride;
        let buffer = renderer.render_buffer().expect("the kernels render");
        let scale = 1.0 / samples as f32;
        buffer.buffer.chunks_exact(kg.film.pass_stride as usize).map(|pixel| [pixel[0] * scale, pixel[1] * scale, pixel[2] * scale]).collect()
    }
}

/// A 16 by 16 scene with one diffuse plane, for tests of the managers.
pub(crate) fn rig_scene() -> Scene {
    let mut rig = Rig::new(16, 16, [0.2, 0.3, 0.4]);
    rig.add(&plane(1.0, 0.0), Surface::Diffuse { color: [0.5; 3] }, translation(0.0, 0.0, 0.0));
    rig.scene
}