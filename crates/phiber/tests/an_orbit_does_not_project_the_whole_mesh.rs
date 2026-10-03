//! A view change over a large selected mesh costs the viewport a few
//! hundred points, not every vertex: the selection outline is the hull of
//! the mesh's extreme points, found once per mesh (`mesh/extreme.rs`).
//! Before, a two-million-vertex floor was projected and sorted on every
//! mouse move of an orbit, which is what made orbiting a large scene crawl.

use gpui::{TestAppContext, px, size};
use phiber::phiphx::scene::{SceneMesh, SceneModel, SceneNormals, SceneObject};
use phiber::v2::app::BPhiberApp;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{BState, CtxKey, ModalKey};
use std::time::Instant;

fn sphere(segs: usize) -> SceneMesh {
    let (mut v, mut i) = (Vec::new(), Vec::new());
    for a in 0..=segs {
        let t = a as f32 / segs as f32 * std::f32::consts::PI;
        for b in 0..=segs {
            let p = b as f32 / segs as f32 * 2.0 * std::f32::consts::PI;
            v.push([t.sin() * p.cos(), t.sin() * p.sin(), t.cos()]);
        }
    }
    let row = (segs + 1) as u32;
    for a in 0..segs as u32 {
        for b in 0..segs as u32 {
            let (p, q) = (a * row + b, (a + 1) * row + b);
            i.extend([p, q, p + 1, p + 1, q, q + 1]);
        }
    }
    let normals = SceneNormals::PerVertex(v.clone());
    SceneMesh { vertices: v, indices: i, normals, corners: Default::default() }
}

#[gpui::test]
async fn orbiting_with_a_large_mesh_selected_costs_no_more_than_with_a_small_one(cx: &mut TestAppContext) {
    // Ten objects, one of a million vertices, over the CPU painter's ceiling
    // so the GPU (or nothing) draws the surfaces and only the outline is the
    // CPU's.
    let mut model = SceneModel::default();
    let big = std::sync::Arc::new(sphere(1000));
    for k in 0..10 {
        let mut o = SceneObject::identity(format!("Ob{k}"), sphere(2));
        if k == 0 {
            o.mesh = big.clone();
        }
        o.matrix_world[3] = [k as f32 * 3.0, 0.0, 0.0, 1.0];
        model.objects.push(o);
    }
    model.collections.push(phiber::phiphx::scene::SceneCollection { name: "Collection".into(), object_indices: (0..10).collect(), ..Default::default() });
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1600.0), px(900.0)));
    app.update(cx, |app, cx| {
        let mut s = BState::new();
        s.set(ModalKey::Open, "");
        s.set(CtxKey::Space, BSpaceType::Layout.slug());
        s.set(&phiber::v2::scene::loaded_path_key(), "heavy.blend".to_string());
        app.scene_cache = Some(("heavy.blend".to_string(), std::sync::Arc::new(model)));
        app.state = s;
        cx.notify();
    });
    cx.run_until_parked();
    let orbit = |app: &gpui::Entity<BPhiberApp>, cx: &mut gpui::VisualTestContext, name: &str| {
        app.update(cx, |app, cx| {
            app.state.set(CtxKey::Object, name);
            cx.notify();
        });
        cx.run_until_parked();
        // The first orbit finds the extreme points; the ones timed are the rest.
        for i in 0..2 {
            app.update(cx, |app, cx| {
                app.state.set(phiber::v2::scene::yaw_key(), format!("{}", 30.0 + i as f32));
                cx.notify();
            });
            cx.run_until_parked();
        }
        let t = Instant::now();
        for i in 0..6 {
            app.update(cx, |app, cx| {
                app.state.set(phiber::v2::scene::yaw_key(), format!("{}", 40.0 + i as f32));
                cx.notify();
            });
            cx.run_until_parked();
        }
        t.elapsed().as_secs_f64() / 6.0
    };
    let small = orbit(&app, cx, "Ob1");
    let large = orbit(&app, cx, "Ob0");
    eprintln!("orbit frame: small selected {:.1} ms, large selected {:.1} ms", small * 1000.0, large * 1000.0);
    assert!(large < small * 1.5 + 0.01, "a million-vertex selection made an orbit frame {:.0} ms against {:.0} ms", large * 1000.0, small * 1000.0);
}
