//! Print object / triangle counts and open time: `cargo run --release --example tri_count -- file.blend`
//!
//! `--objects` prints one line per object instead — name, the sign of its
//! world matrix's determinant, vertices and triangles — in the shape a
//! Blender dump of `evaluated_get(depsgraph).to_mesh()` gives, so the two
//! can be diffed for objects that come out short (a half that is not
//! mirrored, an instance that is not drawn).
//!
//! `--bounds` (with `--objects`) adds each object's world-space box,
//! `min xyz max xyz`, as a fifth column: the census compares it with
//! Blender's evaluated box, which catches geometry in the wrong place or of
//! the wrong shape that a count alone cannot.
//!
//! `--posed` (with `--objects`) counts what Phiber draws instead of what an
//! eager open returns: the progressive open, its deferred modifiers swapped
//! in, then every rig-deformed mesh as the poser hands it over at the saved
//! frame (`SceneMotion::posed_at`) — the meshes the viewport shows for any
//! file with an Armature in it.
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let per_object = args.iter().any(|a| a == "--objects");
    let posed = args.iter().any(|a| a == "--posed");
    let bounds = args.iter().any(|a| a == "--bounds");
    for p in args.iter().filter(|a| !a.starts_with("--")) {
        let t = std::time::Instant::now();
        let m = match posed {
            true => as_drawn(std::path::Path::new(p)),
            false => phiphx::phx::Phx::open(std::path::Path::new(p)).expect("open"),
        };
        let tris: usize = m.objects.iter().map(|o| o.mesh.indices.len() / 3).sum();
        let verts: usize = m.objects.iter().map(|o| o.mesh.vertices.len()).sum();
        let corners: usize = m.objects.iter().map(|o| o.mesh.corners.len()).sum();
        let uvs: usize = m.objects.iter().map(|o| o.uvs.len()).sum();
        if per_object {
            let mut rows: Vec<String> = m
                .objects
                .iter()
                .map(|o| {
                    let world = glam::Mat4::from_cols_array_2d(&o.matrix_world);
                    let sign = match world.determinant() < 0.0 {
                        true => '-',
                        false => '+',
                    };
                    let row = format!("{}\t{sign}\t{}\t{}", o.name, o.mesh.vertices.len(), o.mesh.indices.len() / 3);
                    match bounds {
                        true => format!("{row}\t{}", world_box(world, &o.mesh.vertices)),
                        false => row,
                    }
                })
                .collect();
            rows.sort();
            for row in rows {
                println!("{row}");
            }
            continue;
        }
        println!("{p}: {} objects, {tris} tris, {verts} verts, {corners} corners, {uvs} uvs, {:?}", m.objects.len(), t.elapsed());
        println!("peak {} MB", rss());
    }
}

/// `min xyz max xyz` of `vertices` under `world`, 4 decimals; `-` when empty.
fn world_box(world: glam::Mat4, vertices: &[[f32; 3]]) -> String {
    let (lo, hi) = vertices.iter().map(|&v| world.transform_point3(v.into())).fold(
        (glam::Vec3::splat(f32::MAX), glam::Vec3::splat(f32::MIN)),
        |(lo, hi), p| (lo.min(p), hi.max(p)),
    );
    match vertices.is_empty() {
        true => "-".into(),
        false => format!("{:.4} {:.4} {:.4} {:.4} {:.4} {:.4}", lo.x, lo.y, lo.z, hi.x, hi.y, hi.z),
    }
}

/// The scene as Phiber's viewport draws it at the saved frame.
fn as_drawn(path: &std::path::Path) -> phiphx::scene::SceneModel {
    let (mut model, deferred) = phiphx::phx::Phx::open_progressive(path, &|_, _| true).expect("open");
    if let Some(deferred) = deferred {
        model.apply_deferred(deferred.finish());
    }
    let frame = model.timeline.current_frame as f32;
    let (meshes, _) = model.motion.posed_at(frame);
    for (i, mesh) in meshes {
        if let Some(o) = model.objects.get_mut(i) {
            o.mesh = std::sync::Arc::new(mesh);
        }
    }
    model
}

/// Peak resident memory (Linux; 0 elsewhere).
fn rss() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| s.lines().find(|l| l.starts_with("VmHWM")).and_then(|l| l.split_whitespace().nth(1)?.parse().ok()))
        .map_or(0, |kb: u64| kb / 1024)
}
