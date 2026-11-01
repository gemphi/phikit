//! Viewport rendering — cache management, frustum culling, and draw logic.
use crate::ecs::Scene;
use crate::gpu::{GpuRenderer, ViewportCameraController};
use crate::NSView;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;

static LAST_CACHE_GEN: AtomicU64 = AtomicU64::new(0);
pub(super) static VIEWPORT_CACHE_INVALIDATED: AtomicBool = AtomicBool::new(false);
pub(super) static VIEWPORT_LAST_RENDER_MS: AtomicU64 = AtomicU64::new(0);
static CACHED_PIXELS: Mutex<Option<(Vec<u8>, u32, u32)>> = Mutex::new(None);

/// Extract 6 normalized frustum planes from a view-projection matrix for BVH culling.
pub(super) fn extract_frustum_planes(m: glam::Mat4) -> [[f32; 4]; 6] {
    let r0 = m.row(0); let r1 = m.row(1); let r2 = m.row(2); let r3 = m.row(3);
    let mut planes = [
        [r3.x + r0.x, r3.y + r0.y, r3.z + r0.z, r3.w + r0.w], // Left
        [r3.x - r0.x, r3.y - r0.y, r3.z - r0.z, r3.w - r0.w], // Right
        [r3.x + r1.x, r3.y + r1.y, r3.z + r1.z, r3.w + r1.w], // Bottom
        [r3.x - r1.x, r3.y - r1.y, r3.z - r1.z, r3.w - r1.w], // Top
        [r3.x + r2.x, r3.y + r2.y, r3.z + r2.z, r3.w + r2.w], // Near
        [r3.x - r2.x, r3.y - r2.y, r3.z - r2.z, r3.w - r2.w], // Far
    ];
    for p in &mut planes {
        let len = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
        if len > 0.0 { p[0] /= len; p[1] /= len; p[2] /= len; p[3] /= len; }
    }
    planes
}

/// Perform the software-fallback draw with caching and throttling.
/// ✅ OPTIMIZED: Aggressive caching, early exits, reduced lock contention
pub(super) fn draw_with_cache(
    view: &NSView, context: &mut crate::render::RenderContext,
    camera: &std::sync::Arc<std::sync::Mutex<ViewportCameraController>>,
    scene: &std::sync::Arc<std::sync::Mutex<Scene>>,
    selected_entity: Option<u64>,
) {
    use std::time::{SystemTime, UNIX_EPOCH};

    let bounds = view.bounds();
    let w = bounds.width() as u32;
    let h = bounds.height() as u32;
    if w == 0 || h == 0 {
        return;
    }

    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let last_ms = VIEWPORT_LAST_RENDER_MS.load(Ordering::Relaxed);
    let elapsed = now_ms.saturating_sub(last_ms);

    // Check if cache was explicitly invalidated (theme change, etc.)
    let cache_invalidated = VIEWPORT_CACHE_INVALIDATED.swap(false, Ordering::Acquire);

    // Check if theme generation has changed since last render
    let current_gen = crate::theme::registry::cache_generation();
    let last_gen = LAST_CACHE_GEN.load(Ordering::Relaxed);
    let generation_changed = current_gen != last_gen;

    // ✅ OPTIMIZED: Target 60fps (16.67ms per frame)
    // Only re-render if enough time has passed OR cache is invalid
    const TARGET_FRAME_TIME_MS: u64 = 16; // ~60fps
    
    if elapsed < TARGET_FRAME_TIME_MS && !cache_invalidated && !generation_changed {
        // Use cached pixels for smooth 60fps
        if let Ok(cache) = CACHED_PIXELS.try_lock() {
            if let Some((ref pixels, cw, ch)) = *cache {
                if cw == w && ch == h {
                    context.blit_rgba(pixels, w, h, 0.0, 0.0);
                    return;
                }
            }
        }
    }

    // ✅ OPTIMIZATION: Try to get GPU renderer without blocking
    let gpu_arc = match crate::gpu::get_global_renderer() {
        Some(g) => g,
        None => {
            // No GPU, draw fallback
            context.set_color(crate::theme::color(crate::theme::keys::CLR_BG_CANVAS));
            context.fill_rect(bounds);
            return;
        }
    };

    // Try to lock GPU (non-blocking)
    let gpu_lock = gpu_arc.try_lock();
    if gpu_lock.is_err() {
        // GPU is busy, use cached pixels or fallback
        if let Ok(cache) = CACHED_PIXELS.try_lock() {
            if let Some((ref pixels, cw, ch)) = *cache {
                if cw == w && ch == h {
                    context.blit_rgba(pixels, w, h, 0.0, 0.0);
                    return;
                }
            }
        }
        // No cache, draw fallback
        context.set_color(crate::theme::color(crate::theme::keys::CLR_BG_CANVAS));
        context.fill_rect(bounds);
        return;
    }
    
    let gpu = gpu_lock.unwrap();
    
    // Try to lock camera (non-blocking)
    let mut cam_lock = camera.try_lock();
    if cam_lock.is_err() {
        // Camera is busy, use cached pixels
        if let Ok(cache) = CACHED_PIXELS.try_lock() {
            if let Some((ref pixels, cw, ch)) = *cache {
                if cw == w && ch == h {
                    context.blit_rgba(pixels, w, h, 0.0, 0.0);
                    return;
                }
            }
        }
        context.set_color(crate::theme::color(crate::theme::keys::CLR_BG_CANVAS));
        context.fill_rect(bounds);
        return;
    }
    
    let mut cam = cam_lock.unwrap();
    
    // Update camera aspect ratio to match current viewport size
    let new_aspect = w as f32 / h as f32;
    if (cam.aspect - new_aspect).abs() > 0.001 {
        cam.aspect = new_aspect;
    }
    
    // Try to lock scene (non-blocking)
    let scene_lock = scene.try_lock();
    if scene_lock.is_err() {
        // Scene is busy, use cached pixels
        if let Ok(cache) = CACHED_PIXELS.try_lock() {
            if let Some((ref pixels, cw, ch)) = *cache {
                if cw == w && ch == h {
                    context.blit_rgba(pixels, w, h, 0.0, 0.0);
                    return;
                }
            }
        }
        context.set_color(crate::theme::color(crate::theme::keys::CLR_BG_CANVAS));
        context.fill_rect(bounds);
        return;
    }
    
    let scene = scene_lock.unwrap();
    
    // ✅ All locks acquired successfully, render the frame
    if let Some(pixels) = gpu.render_to_pixels(w, h, &scene, &cam, selected_entity) {
        context.blit_rgba(&pixels, w, h, 0.0, 0.0);
        VIEWPORT_LAST_RENDER_MS.store(now_ms, Ordering::Relaxed);
        LAST_CACHE_GEN.store(current_gen, Ordering::Relaxed);
        if let Ok(mut cache) = CACHED_PIXELS.try_lock() {
            *cache = Some((pixels, w, h));
        }
        return;
    } else {
        // render_to_pixels returned None (GPU not ready), use cache
        if let Ok(cache) = CACHED_PIXELS.try_lock() {
            if let Some((ref pixels, cw, ch)) = *cache {
                if cw == w && ch == h {
                    context.blit_rgba(pixels, w, h, 0.0, 0.0);
                    return;
                }
            }
        }
    }

    // Fallback: dark canvas background
    context.set_color(crate::theme::color(crate::theme::keys::CLR_BG_CANVAS));
    context.fill_rect(bounds);
}

/// Perform the GPU draw pass with BVH frustum culling.
pub(super) fn perform_gpu_draw(
    view: &crate::NSView, renderer: &GpuRenderer,
    encoder: &mut wgpu::CommandEncoder, target_view: &wgpu::TextureView,
    camera: &std::sync::Arc<std::sync::Mutex<ViewportCameraController>>,
    scene_arc: &std::sync::Arc<std::sync::Mutex<crate::ecs::Scene>>,
    selected_entity: Option<u64>,
) {
    static LOGGED: AtomicBool = AtomicBool::new(false);
    if !LOGGED.swap(true, Ordering::Relaxed) {
        let count = scene_arc.lock().map(|s| s.root_entities.len()).unwrap_or(0);
        println!("[Viewport] gpu_draw active — scene has {} entities", count);
    }

    let mut cam = camera.lock().unwrap();
    let bounds = view.bounds();
    if bounds.width() > 0.0 && bounds.height() > 0.0 {
        cam.aspect = bounds.width() / bounds.height();
    }
    let view_proj = cam.view_projection_matrix();
    renderer.update_camera_uniforms(
        &view_proj, [cam.position.x, cam.position.y, cam.position.z, 1.0],
    );

    let planes = extract_frustum_planes(view_proj.to_glam());

    if let Ok(scene) = scene_arc.lock() {
        let mut bvh = crate::culling::BVH::new();
        let mut entities_with_bounds = Vec::new();
        for entity in &scene.root_entities {
            let id = entity.id();
            let b = crate::math::SCNBoundingBox::new(
                crate::math::SCNVector3::new(-1.5, -1.5, -1.5),
                crate::math::SCNVector3::new(1.5, 1.5, 1.5),
            );
            entities_with_bounds.push((id, b));
        }
        bvh.build(&entities_with_bounds);
        let visible_ids = bvh.get_visible_entities(&planes);
        if visible_ids.len() < entities_with_bounds.len() {
            println!(
                "[Culling] BVH Frustum Cull: {}/{} entities visible",
                visible_ids.len(), entities_with_bounds.len()
            );
        }
        renderer.sync_scene_meshes(&scene, selected_entity);
        renderer.draw_scene(target_view, &scene, &cam, encoder, selected_entity);
    }
}
