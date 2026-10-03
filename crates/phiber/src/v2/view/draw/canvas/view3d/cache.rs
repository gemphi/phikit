//! Built-mesh memo for the CPU viewport.
//!
//! `BMesh::build` projects every vertex of every object and allocates a
//! triangle per face. That is the most expensive thing a frame does, and it
//! depends on exactly three things: the camera, the palette, and the scene
//! itself. A window resize changes none of them; nor does opening a menu,
//! switching space, or a progress tick. Yet the viewport is rebuilt on every
//! render pass, so dragging a window edge re-projected millions of triangles
//! per frame and the whole app crawled.
//!
//! So the built meshes are kept behind a fingerprint of those three things.
//! A frame that changes none of them reuses the projection; orbiting, zooming,
//! changing shading or loading a file changes the fingerprint and pays for the
//! rebuild once, as it should.
//!
//! A few entries are kept rather than one: a space can show more than one
//! viewport, and each has its own camera. They would otherwise evict each
//! other every frame and the memo would never hit.

use super::mesh::{BMesh, BMeshPalette};
use crate::v2::scene::BObject3D;
use crate::v2::scene::select::BSelection;
use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

/// How many viewports can be memoised at once.
const ENTRIES: usize = 4;

thread_local! {
    static BUILT: RefCell<Vec<(u64, Rc<Vec<BMesh>>)>> = const { RefCell::new(Vec::new()) };
}

thread_local! {
    /// How many times this thread has rebuilt the projection. A resize, a
    /// menu, a space change and a progress tick must all leave it alone; only
    /// the camera, the palette and the scene may move it. Per-thread, like the
    /// memo it counts — a shared counter would be moved by whatever else the
    /// test binary is running at the time.
    static BUILDS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// The rebuild count — what a frame really paid for the viewport.
pub fn builds() -> usize {
    BUILDS.with(std::cell::Cell::get)
}

fn hash_f32(h: &mut DefaultHasher, v: f32) {
    // Bit patterns, so the hash is exact: two cameras that differ in the last
    // float bit are two cameras.
    v.to_bits().hash(h);
}

fn hash_color(h: &mut DefaultHasher, c: gpui::Hsla) {
    for v in [c.h, c.s, c.l, c.a] {
        hash_f32(h, v);
    }
}

/// Everything a built mesh depends on, as one number: the camera, the
/// palette, the selection and the list's identity (`BSceneId`, kept with
/// the list, so a frame showing the same objects hashes none of them).
#[allow(clippy::too_many_arguments)]
fn fingerprint(
    scene: &Rc<Vec<BObject3D>>,
    yaw: f32,
    pitch: f32,
    pan: (f32, f32),
    zoom: f32,
    palette: &BMeshPalette,
    selected: &BSelection,
    active: &str,
) -> u64 {
    let mut h = DefaultHasher::new();
    for v in [yaw, pitch, pan.0, pan.1, zoom] {
        hash_f32(&mut h, v);
    }
    std::mem::discriminant(&palette.shading).hash(&mut h);
    for c in [
        palette.cube,
        palette.sphere,
        palette.gem,
        palette.loaded,
        palette.accent,
    ] {
        hash_color(&mut h, c);
    }
    selected.list.hash(&mut h);
    active.hash(&mut h);
    super::identity::BSceneId::of(scene).hash(&mut h);
    h.finish()
}

/// The projected meshes for this frame: the memo's whole public surface.
///
/// `camera` is `(yaw, pitch, pan, zoom)`. With `cpu_draws` false the GPU is
/// drawing the surfaces, and only the active object's outline is projected —
/// so a pick shows in the viewport, whichever painter is at work.
pub(super) fn meshes(
    cpu_draws: bool,
    scene: &Rc<Vec<BObject3D>>,
    camera: (f32, f32, (f32, f32), f32),
    palette: BMeshPalette,
    selected: &BSelection,
    active: &str,
) -> Rc<Vec<BMesh>> {
    let (yaw, pitch, pan, zoom) = camera;
    let key = fingerprint(scene, yaw, pitch, pan, zoom, &palette, selected, active) ^ (cpu_draws as u64);
    // The active object's outline is a step lighter than the others'
    // (Blender's `TH_ACTIVE` over `TH_SELECT`).
    let lit = |mut mesh: BMesh, object: &BObject3D| {
        if crate::v2::scene::object_name(object) == active {
            mesh.outline_color = palette.accent_active;
        }
        mesh
    };
    if !cpu_draws {
        return get_or_build(key, || {
            scene
                .iter()
                .filter(|object| selected.contains(crate::v2::scene::object_name(object)))
                .map(|object| lit(BMesh::outline_only(object, yaw, pitch, pan, zoom, palette), object))
                .collect()
        });
    }
    get_or_build(key, || {
        scene
            .iter()
            .map(|object| {
                let is_selected = selected.contains(crate::v2::scene::object_name(object));
                lit(BMesh::build(object, yaw, pitch, pan, zoom, palette, is_selected), object)
            })
            .collect()
    })
}

/// The meshes for `key`, building them with `build` only on a miss.
fn get_or_build(key: u64, build: impl FnOnce() -> Vec<BMesh>) -> Rc<Vec<BMesh>> {
    if let Some(hit) = BUILT.with(|cell| {
        cell.borrow()
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, m)| Rc::clone(m))
    }) {
        return hit;
    }
    BUILDS.with(|n| n.set(n.get() + 1));
    let built = Rc::new(build());
    BUILT.with(|cell| {
        let mut entries = cell.borrow_mut();
        if entries.len() == ENTRIES {
            entries.remove(0);
        }
        entries.push((key, Rc::clone(&built)));
    });
    built
}

/// Drop every memoised projection. The viewport rebuilds on its next frame.
pub fn clear() {
    BUILT.with(|cell| cell.borrow_mut().clear());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_repeat_key_does_not_rebuild() {
        clear();
        let mut built = 0;
        for _ in 0..5 {
            let _ = get_or_build(7, || {
                built += 1;
                Vec::new()
            });
        }
        assert_eq!(built, 1, "the same camera and scene must build once");
        let _ = get_or_build(8, || {
            built += 1;
            Vec::new()
        });
        assert_eq!(built, 2, "a new camera must rebuild");
        clear();
    }

    #[test]
    fn several_viewports_keep_their_own_entry() {
        clear();
        let mut built = 0;
        // Two viewports alternate frames; neither may evict the other.
        for _ in 0..4 {
            for key in [1u64, 2] {
                let _ = get_or_build(key, || {
                    built += 1;
                    Vec::new()
                });
            }
        }
        assert_eq!(built, 2, "alternating viewports must not thrash the memo");
        clear();
    }
}
