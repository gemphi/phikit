//! Dump every rig's posed bones at the saved frame, in the shape
//! `scratchpad/rig/dump.py` writes from Blender, so the two can be diffed
//! bone by bone:
//!
//! `cargo run --release --example pose_dump -- FILE [--frame N] [--bones a,b]`
//!
//! `--bones` also prints the named bones' rest, channel (as saved and at
//! the frame) and constraints, for reading a bone that comes out wrong.
//!
//! ```text
//! FRAME <frame>
//! RIG   <rig object> <bones> <world matrix, column major>
//! PM    <rig object> <bone> <pose matrix, armature space, column major>
//! BOX   <mesh object> <vertices> <min xyz> <max xyz>   (world space, posed)
//! END
//! ```
//!
//! The scene opens as the viewport draws it (`Phx::open_progressive`, the
//! deferred modifiers applied), every rig is posed with every object solved
//! (`SceneMotion::posed_at`), and each rig-deformed mesh is boxed as the
//! poser hands it over.
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let frame_arg = args.iter().position(|a| a == "--frame").and_then(|i| args.get(i + 1)).and_then(|f| f.parse::<f32>().ok());
    let values: Vec<usize> = args.iter().enumerate().filter(|(_, a)| matches!(a.as_str(), "--frame" | "--bones")).map(|(i, _)| i + 1).collect();
    for p in args.iter().enumerate().filter(|(i, a)| !a.starts_with("--") && !values.contains(i)).map(|(_, a)| a) {
        let (mut model, deferred) = phiphx::phx::Phx::open_progressive(std::path::Path::new(p), &|_, _| true).expect("open");
        if let Some(deferred) = deferred {
            model.apply_deferred(deferred.finish());
        }
        let frame = frame_arg.unwrap_or(model.timeline.current_frame as f32);
        println!("FRAME\t{frame}");
        let at = model.motion.solved_at(frame);
        for r in &model.motion.rigs.rigs {
            let name = &model.motion.motion.list[r.object].name;
            let world = glam::Mat4::from_cols_array_2d(&at.world[r.object]);
            println!("RIG\t{name}\t{}\t{}", r.rig.bones.len(), mat(world));
            let Some(pose) = at.poses.get(&r.object) else { continue };
            for (i, b) in r.rig.bones.iter().enumerate() {
                println!("PM\t{name}\t{}\t{}", b.name, mat(pose.pose[i]));
            }
        }
        if let Some(names) = args.iter().position(|a| a == "--bones").and_then(|i| args.get(i + 1)) {
            for r in &model.motion.rigs.rigs {
                let state = r.rig.state_at(r.anim(), frame);
                for name in names.split(',') {
                    let Some(i) = r.rig.bone(name) else { continue };
                    let b = &r.rig.bones[i];
                    let parent = b.parent.map(|p| r.rig.bones[p].name.as_str()).unwrap_or("-");
                    println!("BONE	{name}	parent={parent} flag={:#x} inherit_scale={} segments={} length={}", b.flag, b.inherit_scale, b.segments, b.length);
                    println!("  raw {:?}", b.raw);
                    println!("  at frame {:?}", state.raw[i]);
                    println!("  arm {}", mat(b.arm));
                    for (k, c) in b.constraints.iter().enumerate() {
                        println!("  C{k} inf={} muted={} {:?}", state.influence[i][k], state.muted[i][k], c);
                    }
                }
            }
        }
        let (meshes, _) = model.motion.posed_at(frame);
        for (i, mesh) in meshes {
            let o = &model.objects[i];
            let world = glam::Mat4::from_cols_array_2d(&at.world[model.motion.object[i]]);
            let (lo, hi) = mesh.vertices.iter().map(|&v| world.transform_point3(v.into())).fold(
                (glam::Vec3::splat(f32::MAX), glam::Vec3::splat(f32::MIN)),
                |(lo, hi), p| (lo.min(p), hi.max(p)),
            );
            println!("BOX\t{}\t{}\t{:.5} {:.5} {:.5} {:.5} {:.5} {:.5}", o.name, mesh.vertices.len(), lo.x, lo.y, lo.z, hi.x, hi.y, hi.z);
        }
        println!("END");
    }
}

/// Column major, 7 decimals: what `dump.py` writes.
fn mat(m: glam::Mat4) -> String {
    m.to_cols_array().iter().map(|v| format!("{v:.7}")).collect::<Vec<_>>().join(" ")
}
