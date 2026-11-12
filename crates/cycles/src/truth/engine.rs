// SPDX-License-Identifier: Apache-2.0
//! The renderer the judge calls for a test scene: read the `.blend`, join it
//! into a [`Scene`](crate::scene::Scene) (`blender/`), run the managers
//! (`Scene::device_update`), put the scene's kernels in a CPU device and
//! render through the session (`session.cpp`, `run_main_render_loop`), then
//! display the film with the scene's colour management. This is the order
//! of `cycles_render_tests.py` running Blender on the file, with the
//! kernels in Rust.
use std::sync::Arc;
use std::time::Instant;

use super::display::ViewSettings;
use super::render::{SessionRenderer, display_view};
use super::{Image, Renderer, TestScene, TruthError};
use crate::blender::{read_blend, sample_count, setting, sync_scene};
use crate::device::CpuDevice;
use crate::kernel::KernelGlobals;
use crate::kernel::device::cpu_kernels;

/// What a render of a test scene used, for the report.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderReport {
    /// The sample count rendered, and the one the scene asks for.
    pub samples: i32,
    pub scene_samples: i32,
    pub seconds: f64,
    pub size: (usize, usize),
    /// The view settings the picture was made with.
    pub view: ViewSettings,
    /// Things in the scene that the port does not draw: lights, an
    /// environment image, a roughness on a diffuse.
    pub missing: Vec<String>,
}

/// Renders a test scene with the gate 1 kernels.
#[derive(Debug, Default)]
pub struct SceneRenderer {
    /// A sample count to use in place of the scene's own.
    pub sample_override: Option<i32>,
    /// What the last render used.
    pub last: Option<RenderReport>,
    /// The last picture, to be saved beside the diff.
    pub image: Option<Image>,
}

impl SceneRenderer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_samples(samples: i32) -> Self {
        Self { sample_override: Some(samples), last: None, image: None }
    }
}

impl Renderer for SceneRenderer {
    fn render(&mut self, test: &TestScene) -> Result<Image, TruthError> {
        let source = read_blend(&test.blend).map_err(|message| TruthError::Image { path: test.blend.display().to_string(), message })?;
        let mut missing = Vec::new();
        if source.light_count > 0 {
            missing.push(format!("{} lights", source.light_count));
        }
        if let Some(texture) = &source.world_texture {
            missing.push(format!("environment image {texture}"));
        }
        let scene_samples = sample_count(&source);
        let samples = self.sample_override.unwrap_or(scene_samples);
        let view = ViewSettings::of(&source.view_transform, &source.look, source.exposure, source.gamma, setting(&source, "film_exposure", 1.0) as f32);
        let size = (source.width.max(1) as usize, source.height.max(1) as usize);
        let started = Instant::now();
        let mut scene = sync_scene(&source);
        let dscene = scene.device_update();
        let kg = Arc::new(KernelGlobals::new(Arc::new(dscene)));
        let device = Arc::new(CpuDevice::with_kernels(cpu_kernels(kg.clone())));
        let mut renderer = SessionRenderer::new(device, size, samples);
        renderer.pass_stride = kg.film.pass_stride;
        renderer.transparent = source.film_transparent;
        let buffer = renderer.render_buffer()?;
        let image = display_view(&buffer, samples, renderer.transparent, &view);
        self.last = Some(RenderReport { samples, scene_samples, seconds: started.elapsed().as_secs_f64(), size, view, missing });
        self.image = Some(image.clone());
        Ok(image)
    }
}
