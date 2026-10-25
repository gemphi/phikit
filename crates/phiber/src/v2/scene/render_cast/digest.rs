use super::*;

/// Not a fixture test: point `CAST_DIGEST` at a `.blend` and this prints
/// what the render cast keeps and drops there, for checking a real
/// file's restrictions. Does nothing without the var.
#[test]
fn cast_digest_of_env_file() {
    let Ok(path) = std::env::var("CAST_DIGEST") else { return };
    let model = crate::phiphx::Phx::open(std::path::Path::new(&path)).expect("open");
    let mut state = BState::new();
    crate::v2::app::seed::seed_view_layer(&mut state, &model);
    let model = std::sync::Arc::new(model);
    crate::v2::scene::source::set_loaded_shared(Some(model.clone()));
    let all = crate::v2::scene::current_scene(&state, Some(&model));
    let cast = BRenderCast::of(&state, Some(&model));
    println!("total {} cast {} note {:?}", all.len(), cast.objects.len(), cast.note);
    let kept: std::collections::HashSet<String> = cast.objects.iter().map(|o| o.name().to_string()).collect();
    let dropped: Vec<String> = all.iter().map(|o| o.name().to_string()).filter(|n| !kept.contains(n)).collect();
    println!("dropped {}: {:?}", dropped.len(), &dropped[..dropped.len().min(40)]);
    println!("render_disabled {}: {:?}", model.selection.render_disabled.len(), &model.selection.render_disabled[..model.selection.render_disabled.len().min(10)]);
    // Floor / ground / rig-prop suspects: where they are and what keeps them.
    fn walk(c: &crate::phiphx::scene::SceneCollection, path: &str, model: &SceneModel, kept: &std::collections::HashSet<String>) {
        for &i in &c.object_indices {
            if let Some(o) = model.objects.get(i) {
                let n = o.name.to_lowercase();
                if ["floor", "plane", "ground", "pole", "boom", "stage", "plat"].iter().any(|k| n.contains(k)) {
                    let m = o.matrix_world;
                    println!(
                        "  {} in {:?}: z {:.3} kept {} tris {}",
                        o.name, path, m[3][2], kept.contains(&o.name), o.mesh.indices.len() / 3
                    );
                }
            }
        }
        for k in &c.children { walk(k, &format!("{path}/{}", k.name), model, kept); }
    }
    for c in &model.collections { walk(c, &c.name, &model, &kept); }
    // Ground contact at the probed frame: lowest world-space vertex of
    // the contact suspects vs the floor's surface.
    state.set(crate::v2::state::CtxKey::Frame, "1".to_string());
    let posed = crate::v2::scene::current_scene(&state, Some(&model));
    for o in &posed {
        let crate::v2::scene::BObject3D::Loaded { name, mesh, matrix_world: m, .. } = o else { continue };
        let n = name.to_lowercase();
        if !["floor_geo", "heel", "shoe", "bag", "foot", "toe"].iter().any(|k| n.contains(k)) {
            continue;
        }
        let mut z = (f32::INFINITY, f32::NEG_INFINITY);
        for v in &mesh.vertices {
            let w = m[0][2] * v[0] + m[1][2] * v[1] + m[2][2] * v[2] + m[3][2];
            z = (z.0.min(w), z.1.max(w));
        }
        if z.0.is_finite() {
            println!("contact {} z [{:.4}, {:.4}] kept {}", name, z.0, z.1, kept.contains(name.as_str()));
        }
    }
}
