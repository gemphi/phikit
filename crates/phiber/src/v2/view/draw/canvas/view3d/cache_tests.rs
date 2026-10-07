//! The built-mesh memo's tests (`cache.rs`).

use super::*;

fn palette() -> BMeshPalette {
    let grey = gpui::Hsla { h: 0.0, s: 0.0, l: 0.5, a: 1.0 };
    BMeshPalette { shading: crate::v2::scene::BShading::Solid, cube: grey, sphere: grey, gem: grey, loaded: grey, accent: grey, accent_active: grey, matcap: None, texture: false }
}

/// What the GPU draws, the CPU does not project: with the tracer
/// drawing the surfaces *and* the selection outline there is nothing
/// left to build, where X-ray — which keeps no one instance a pixel —
/// still wants the hull. The three states key apart, so switching
/// X-ray does not hand back the wrong projection.
#[test]
fn the_cpu_projects_only_what_the_gpu_leaves_it() {
    clear();
    let scene = Rc::new(vec![
        BObject3D::Primitive { name: "Cube".into(), shape: crate::v2::scene::BShape::Cube, x: 0.0, y: 0.0 },
        BObject3D::Primitive { name: "Ball".into(), shape: crate::v2::scene::BShape::Sphere, x: 2.0, y: 0.0 },
    ]);
    let picked = BSelection { list: vec!["Cube".into()] };
    let built = |draw| meshes(draw, &scene, (20.0, 30.0, (0.0, 0.0), 1.0), palette(), &picked, "Cube");
    assert!(built(BCpuDraw::Nothing).is_empty(), "the tracer drew the outline too");
    let hull = built(BCpuDraw::Hull);
    assert_eq!(hull.len(), 1, "the selected object alone");
    assert!(hull[0].triangles.is_empty() && hull[0].outline.is_some(), "its hull, no surfaces");
    let whole = built(BCpuDraw::Whole);
    assert_eq!(whole.len(), 2, "every object, selected or not");
    assert!(whole.iter().all(|m| !m.triangles.is_empty()));
    // Same camera, same scene, three answers: the memo must not confuse them.
    assert!(built(BCpuDraw::Nothing).is_empty());
    clear();
}

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
