//! What phiphx saves, Blender opens — and reads the same edits phiphx
//! does. Each case is a file saved through the writer with a session's
//! worth of edits (the frame and range, a rename, the camera toggle, the
//! selection and active object, markers, the Playback settings, a cube
//! added, an object duplicated and one deleted). This test writes them to
//! `CARGO_TARGET_TMPDIR/blender_opens_saves/`, re-reads each through
//! phiphx, and compares the summary with `truth/saved.txt`: Blender's own
//! reading of the same files, made by
//!
//!   python3 tests/truth/blender_saved.py DIR tests/truth/saved.txt
//!
//! with `bpy` 5.0 (`pip install bpy`), which opens the 3.6, 4.2 and 5.0
//! layouts alike. A line that differs is an edit Blender does not see as
//! phiphx meant it; a file Blender cannot open has no lines at all.

use phiphx::blend::write::mesh::BlendNewMesh;
use phiphx::blend::{BFCurve, BNlaStacks, BNlaStrip, BlendNla, STRIP_ACTIVE, STRIP_MUTED, STRIP_SELECT, STRIP_SYNC_LENGTH, STRIP_USR_TIME_CYCLIC, TRACK_ACTIVE, TRACK_MUTED, TRACK_PROTECTED, TRACK_SELECTED};
use phiphx::blend::write::{BlendActionAssign, BlendActionRange, BlendCollectionEdit, BlendKeyInsert, BlendNlaStack, BlendCompression, BlendDuplicate, BlendEdits, BlendNewObject, BlendObjectEdit};
use phiphx::blend::{BIdValue, BlendFile, BlendImageFormat, BlendOutput, BlendPlayback, BlendScene, BlendSyncMode, BlendVisibility};
use std::path::PathBuf;

/// The cases: a file each Blender version wrote, and one with animation.
const CASES: &[(&str, &str)] = &[
    ("startup-3", "../phiber/assets/blend/startup-3.blend"),
    ("startup-4", "../phiber/assets/blend/startup-4.blend"),
    ("startup-5", "../phiber/assets/blend/startup-5.blend"),
    ("nla_cube", "tests/fixtures/nla_cube.blend"),
    ("collections", "tests/fixtures/collections.blend"),
    ("actions", "tests/fixtures/actions.blend"),
];

fn cube() -> BlendNewMesh {
    BlendNewMesh {
        name: "Crate".into(),
        vertices: vec![[-1., -1., -1.], [1., -1., -1.], [1., 1., -1.], [-1., 1., -1.], [-1., -1., 1.], [1., -1., 1.], [1., 1., 1.], [-1., 1., 1.]],
        faces: vec![vec![0, 1, 2, 3], vec![4, 7, 6, 5], vec![0, 4, 5, 1], vec![1, 5, 6, 2], vec![2, 6, 7, 3], vec![3, 7, 4, 0]],
    }
}

/// A session's edits to `scene`: its first mesh renamed and kept out of
/// renders, its first object duplicated, its last object deleted.
fn edits(scene: &BlendScene) -> BlendEdits {
    let mesh = scene.objects.iter().find(|o| o.object_type == "MESH").map(|o| o.name.clone()).expect("a mesh");
    let first = scene.objects[0].name.clone();
    let last = scene.objects.last().unwrap().name.clone();
    // G on the light: moved and turned, as a camera, light or empty takes
    // the Transform panel's edits like a mesh.
    let light = scene.objects.iter().find(|o| matches!(o.object_type.as_str(), "LAMP" | "LIGHT") && o.name != last).map(|o| BlendObjectEdit {
        name: o.name.clone(),
        transform: Some(([1.0, -2.0, 3.0], [0.0, std::f32::consts::FRAC_PI_2, 0.0], [1.0; 3])),
        ..Default::default()
    });
    BlendEdits {
        frame: Some(42),
        frame_start: Some(3),
        frame_end: Some(77),
        objects: vec![BlendObjectEdit {
            name: mesh,
            rename: Some("Renamed".into()),
            visibility: Some(BlendVisibility { hide_viewport: true, hide_select: false, hide_render: true }),
            hidden: Some(true),
            selected: Some(true),
            transform: None,
            shadow: None,
        }, BlendObjectEdit {
            // The Transform panel on the duplicate: moved, turned 45° about
            // Z, doubled — before it is parented, so it keeps that.
            name: "Copy".into(),
            transform: Some(([0.5, 0.0, 0.25], [0.0, 0.0, std::f32::consts::FRAC_PI_4], [2.0, 2.0, 2.0])),
            ..Default::default()
        }].into_iter().chain(light).collect(),
        added: vec![BlendNewObject { name: "Crate".into(), mesh: cube(), location: [1.0, 2.0, 3.0], scale: [1.0; 3], collection: "Scene Collection/Collection".into(), selected: true, visibility: None }],
        collections: vec![BlendCollectionEdit { path: "Scene Collection/Collection".into(), hidden: Some(true), exclude: None, visibility: Some(BlendVisibility { hide_render: true, ..Default::default() }) }],
        duplicated: vec![BlendDuplicate { copy: "Copy".into(), source: first, selected: false }],
        deleted: vec![last],
        active: Some("Crate".into()),
        markers: Some(vec![(5, "A".into()), (50, "B".into())]),
        // The Outliner's drops: the added cube dragged back out to the scene
        // collection, and the duplicate dragged onto it.
        moved: vec![("Crate".into(), "Scene Collection".into())],
        parented: vec![("Copy".into(), "Crate".into())],
        // The Outliner's collection drops, where the file has the
        // collections: "Lamps" dragged onto "Props" (nested), and the
        // duplicate Ctrl-dragged onto "Props" (linked there as well).
        nested: has(scene, "Lamps").then(|| vec![("Lamps".into(), "Props".into())]).unwrap_or_default(),
        linked: has(scene, "Props").then(|| vec![("Copy".into(), "Scene Collection/Props".into())]).unwrap_or_default(),
        playback: Some(BlendPlayback { sync: BlendSyncMode::FrameDrop, audio_scrub: true, lock_frame_selection: true, show_subframe: true, auto_key: true, ..Default::default() }),
        // The Render and Output tabs: the engine, four Cycles settings (the
        // startup files carry none of them, so each is a new property), and
        // where a render goes, as a 16-bit RGB EXR with ZIP.
        engine: Some("CYCLES".into()),
        film_transparent: Some(true),
        cycles: vec![("samples".into(), BIdValue::Int(48)), ("preview_samples".into(), BIdValue::Int(7)), ("diffuse_bounces".into(), BIdValue::Int(2)), ("use_denoising".into(), BIdValue::Bool(false)), ("denoiser".into(), BIdValue::Int(4)), ("pixel_filter_type".into(), BIdValue::Int(1)), ("phiber_note".into(), BIdValue::Str("a string property, which Blender keeps and ignores".into()))],
        output: Some(BlendOutput { path: "//renders/shot_".into(), format: BlendImageFormat::OpenExr, depth: 16, channel: 3, quality: 90, compress: 15, exr_codec: 2 }),
        ..actions(scene)
    }
}

/// The Action Editor's edits, where the file has the actions
/// (`make_actions.py`): New on the (renamed) Cube copies its action and
/// takes it with its slot; the unused "Spin" goes to the Camera — or the
/// Light, when the Camera is the object the edits delete — through the
/// slot the Cube used, in a new `AnimData`; New on the added Crate makes an
/// empty action; the Camera's action gets a Manual Frame
/// Range and Cyclic, the Cube's loses them. Then I: the Cube keyed at frame
/// 10 through the slot it uses — a key on its X curve and a new Z scale
/// curve — and the Crate keyed at 1 and 20 (moved in X), which gives its
/// empty action the Crate's slot, a layer, a strip and a channelbag, and
/// the Crate that slot. Then the Action menu: Push Down on the object that
/// took Spin (a new track named Spin), Stash on the Cube (its copy in an
/// "[Action Stash]" track), each object left with no action.
fn actions(scene: &BlendScene) -> BlendEdits {
    if !scene.props.actions.iter().any(|a| a.name == "CubeAction") {
        return BlendEdits { nla: nla_editor(scene), ..Default::default() };
    }
    let assign = |object: &str, action: &str, slot: Option<&str>| BlendActionAssign { object: object.into(), action: Some(action.into()), slot: slot.map(str::to_string) };
    let unlinked = |object: &str, slot: &str| BlendActionAssign { object: object.into(), action: None, slot: Some(slot.into()) };
    // The camera, or the light if the camera is the object the edits delete.
    let last = &scene.objects.last().expect("objects").name;
    let other = ["Camera", "Light"].into_iter().find(|n| n != last).unwrap();
    let cube_x = scene.props.actions.iter().find(|a| a.name == "CubeAction").and_then(|a| a.curves.iter().find(|c| c.path == "location" && c.index == 0)).cloned().expect("the Cube's X");
    // The Crate at (1, 2, 3), then (4, 2, 3): location, rotation, scale.
    let crate_keys: Vec<BFCurve> = ["location", "rotation_euler", "scale"].iter().flat_map(|path| (0..3u32).map(move |i| (*path, i))).map(|(path, i)| {
        let (at_1, at_20) = match path {
            "location" => ([1.0, 2.0, 3.0][i as usize], [4.0, 2.0, 3.0][i as usize]),
            "scale" => (1.0, 1.0),
            _ => (0.0, 0.0),
        };
        keyed(BFCurve::keyed(path, i), &[(1.0, at_1), (20.0, at_20)])
    }).collect();
    BlendEdits {
        actions: vec![("CubeAction.001".into(), "CubeAction".into()), ("Action".into(), String::new())],
        // After the Action menu: Spin pushed down on the other object, the
        // Cube's copy stashed — both then play no action.
        assigned: vec![unlinked("Renamed", "OBCube"), unlinked(other, "OBCube"), assign("Crate", "Action", Some("OBCrate"))],
        nla: vec![
            stack(scene, other, other, |n| drop(n.push_down(BNlaStrip::clip("Spin", (0, "OBCube"), (1.0, 24.0), false, false)))),
            stack(scene, "Cube", "Renamed", |n| drop(n.stash(BNlaStrip::clip("CubeAction.001", (0, "OBCube"), (5.0, 30.0), true, true)))),
        ],
        ranges: vec![
            BlendActionRange { action: "CameraAction".into(), range: Some((10.0, 50.0)), cyclic: true },
            BlendActionRange { action: "CubeAction".into(), range: None, cyclic: false },
        ],
        keys: vec![
            BlendKeyInsert { action: "CubeAction.001".into(), slot: "OBCube".into(), curves: vec![keyed(cube_x, &[(10.0, 1.5)]), keyed(BFCurve::keyed("scale", 2), &[(10.0, 2.0)])], group: "Object Transforms".into(), removed: Vec::new() },
            BlendKeyInsert { action: "Action".into(), slot: "OBCrate".into(), curves: crate_keys, group: "Object Transforms".into(), removed: Vec::new() },
        ],
        ..Default::default()
    }
}

/// The NLA the file has for `file_name` (none: an empty one), `edit`ed,
/// to be written for the object called `object`.
fn stack(scene: &BlendScene, file_name: &str, object: &str, edit: impl FnOnce(&mut BlendNla)) -> BlendNlaStack {
    let mut nla = scene.props.nla.iter().find(|n| n.object == file_name).cloned().unwrap_or_default();
    edit(&mut nla);
    BlendNlaStack { object: object.into(), nla }
}

/// The NLA editor's edits, where the file has the Cube's two tracks
/// (`nla_cube.blend`): a click on Walk, H (muted), Add Above Selected
/// (a new NlaTrack over Base Layer), a click on Overlay's name, Tracks ▸
/// Delete (Overlay and Walk Again gone), then Add ▸ Action (Idle at frame
/// 100) on each track left. The Cube is saved as "Renamed".
fn nla_editor(scene: &BlendScene) -> Vec<BlendNlaStack> {
    if !scene.props.nla.iter().any(|n| n.object == "Cube" && n.tracks.len() == 2) {
        return Vec::new();
    }
    vec![stack(scene, "Cube", "Renamed", |n| {
        let mut shown = BNlaStacks(vec![std::mem::take(n)]);
        shown.click(Some((0, 0, 0)), false);
        shown.mute_strips();
        shown.add_tracks(true);
        shown.click_track(0, 2, false);
        shown.delete_tracks();
        // Add ▸ Action at frame 100: Idle after Walk on Base Layer, and on
        // the new track.
        let idle = || BNlaStrip::clip("Idle", (0, "OBCube"), (10.0, 30.0), false, false);
        for track in [0, 1] {
            shown.click_track(0, track, false);
            shown.0[0].add_clip(idle(), 100.0);
        }
        *n = shown.0.remove(0);
    })]
}

/// `curve` with `keys` inserted as `I` inserts them.
fn keyed(mut curve: BFCurve, keys: &[(f32, f32)]) -> BFCurve {
    for &(frame, value) in keys {
        curve.insert(frame, value);
    }
    curve
}

/// Does the file have a collection called `name`?
fn has(scene: &BlendScene, name: &str) -> bool {
    fn walk(c: &phiphx::blend::BlendCollection, name: &str) -> bool {
        c.name == name || c.children.iter().any(|k| walk(k, name))
    }
    scene.collections.iter().any(|c| walk(c, name))
}

/// The lines `truth/blender_saved.py` writes for one file, from phiphx's
/// reading of it.
fn summary(case: &str, scene: &BlendScene) -> Vec<String> {
    let p = &scene.props;
    let mut out = vec![format!("{case} frame {} {} {}", p.frame_current, p.frame_start, p.frame_end)];
    let active = scene.active_object.map(|i| scene.objects[i].name.clone()).unwrap_or_default();
    out.push(format!("{case} active {active}"));
    let mut objects: Vec<(usize, &phiphx::blend::BlendSceneObject)> = scene.objects.iter().enumerate().collect();
    objects.sort_by(|a, b| a.1.name.cmp(&b.1.name));
    for (i, o) in objects {
        let kind = if o.object_type == "LAMP" { "LIGHT" } else { o.object_type.as_str() };
        let mesh = if kind == "MESH" { format!(" {}v {}f", o.vertices.len(), o.faces.len()) } else { String::new() };
        // An animated object stands where its action puts it at the frame,
        // which Blender evaluates on load; its stored location is not that.
        // So does a child of one, wherever it is itself.
        let animated = |o: &phiphx::blend::BlendSceneObject| {
            let playing = |n: &&BlendNla| n.object == o.name && n.tracks.iter().any(|t| t.flag & TRACK_MUTED == 0 && !t.strips.is_empty());
            p.actions.iter().any(|a| a.users.contains(&o.name)) || p.nla.iter().any(|n| playing(&n))
        };
        let parent_of = |o: &phiphx::blend::BlendSceneObject| scene.objects.iter().find(|p| o.parent_address != 0 && p.block_address == o.parent_address);
        let carried = std::iter::successors(parent_of(o), |p| parent_of(p)).any(animated);
        let [x, y, z] = o.location;
        let at = if animated(o) {
            "animated".to_string()
        } else {
            let w = o.matrix_world[3];
            let [rx, ry, rz] = o.rotation.map(|r| r.to_degrees() + 0.0);
            let [sx, sy, sz] = o.scale;
            let world = if carried { "animated".to_string() } else { format!("{:.3} {:.3} {:.3}", w[0], w[1], w[2]) };
            format!("{x:.3} {y:.3} {z:.3} rot {rx:.1} {ry:.1} {rz:.1} scale {sx:.3} {sy:.3} {sz:.3} world {world}")
        };
        let parent = scene.objects.iter().find(|p| o.parent_address != 0 && p.block_address == o.parent_address).map_or("-", |p| p.name.as_str());
        let mats: Vec<&str> = o.materials.iter().map(|m| m.name.as_str()).filter(|n| !n.is_empty()).collect();
        let mats = if mats.is_empty() { "-".to_string() } else { mats.join(",") };
        out.push(format!(
            "{case} object {} {kind}{mesh} at {at} parent {parent} mats {mats} render {} viewport {} hidden {} selected {}",
            o.name,
            u8::from(!o.visibility.hide_render),
            u8::from(!o.visibility.hide_viewport),
            u8::from(scene.hidden.contains(&i)),
            u8::from(scene.selected.contains(&i))
        ));
    }
    fn collections(case: &str, c: &phiphx::blend::BlendCollection, parent: &str, out: &mut Vec<String>) {
        let path = if parent.is_empty() { c.name.clone() } else { format!("{parent}/{}", c.name) };
        let mut names = c.object_names.clone();
        names.sort();
        out.push(format!(
            "{case} collection {path} [{}] exclude {} hide {} render {}",
            names.join(","),
            u8::from(c.exclude),
            u8::from(c.hide_viewport),
            u8::from(!c.visibility.hide_render)
        ));
        for k in &c.children {
            collections(case, k, &path, out);
        }
    }
    for c in &scene.collections {
        collections(case, c, "", &mut out);
    }
    // Every action, and who uses which through which slot.
    let mut actions: Vec<&phiphx::blend::BlendActionSummary> = p.actions.iter().collect();
    actions.sort_by(|a, b| a.name.cmp(&b.name));
    for a in &actions {
        let slots: Vec<&str> = a.slots.iter().map(|s| s.identifier.as_str()).collect();
        let range = a.manual_range.map_or("-".to_string(), |(s, e)| format!("{s:.2}-{e:.2}"));
        out.push(format!("{case} action {} slots [{}] range {range} cyclic {}", a.name, slots.join(","), u8::from(a.cyclic)));
        for (object, handle) in &a.bindings {
            let slot = a.slots.iter().find(|s| s.handle == *handle).map_or("-", |s| s.identifier.as_str());
            out.push(format!("{case} anim {object} {} {slot}", a.name));
        }
    }
    // Where the keys are inserted: every curve's keys and handles, and
    // whether they are the handles Blender computes for those keys.
    if case == "actions" {
        for a in &actions {
            for s in &a.slots {
                for c in &a.curves[s.curves.clone()] {
                    let keys: Vec<String> = c.keys.iter().map(|k| format!("{:.3},{:.3}", k.co[0], k.co[1])).collect();
                    let handles: Vec<String> = c.keys.iter().map(|k| format!("{:.3},{:.3}/{:.3},{:.3}", k.h1[0], k.h1[1], k.h2[0], k.h2[1])).collect();
                    let mut again = c.clone();
                    again.recalc_handles();
                    let close = |a: [f32; 2], b: [f32; 2]| (a[0] - b[0]).abs() < 1e-4 && (a[1] - b[1]).abs() < 1e-4;
                    let same = again.keys.iter().zip(&c.keys).all(|(x, y)| close(x.h1, y.h1) && close(x.h2, y.h2));
                    let verdict = if same { "blenders" } else { "not-blenders" };
                    out.push(format!("{case} curve {} {} {}[{}] keys {} handles {} {verdict}", a.name, s.identifier, c.path, c.index, keys.join(" "), handles.join(" ")));
                }
            }
        }
    }
    // The NLA as laid, and where it moves its object.
    if case == "actions" || case == "nla_cube" {
        let mut nla: Vec<&BlendNla> = p.nla.iter().filter(|n| !n.tracks.is_empty()).collect();
        nla.sort_by(|a, b| a.object.cmp(&b.object));
        let bit = |flag: u32, b: u32| u8::from(flag & b != 0);
        for n in nla {
            for t in &n.tracks {
                let strips: Vec<String> = t
                    .strips
                    .iter()
                    .map(|s| {
                        format!(
                            "{}:{}:{}:{:.2}-{:.2}:{:.2}-{:.2}:sync{}:cyclic{}:sel{}:act{}:mute{}:extend{}:blend{}:influence{:.2}",
                            s.name, s.action.clone().unwrap_or_default(), s.slot_name, s.start, s.end, s.actstart, s.actend,
                            bit(s.flag, STRIP_SYNC_LENGTH), bit(s.flag, STRIP_USR_TIME_CYCLIC), bit(s.flag, STRIP_SELECT), bit(s.flag, STRIP_ACTIVE), bit(s.flag, STRIP_MUTED),
                            s.extend, s.blend, s.influence
                        )
                    })
                    .collect();
                out.push(format!(
                    "{case} nla {} track {} mute {} lock {} sel {} act {} [{}]",
                    n.object, t.name, bit(t.flag, TRACK_MUTED), bit(t.flag, TRACK_PROTECTED), bit(t.flag, TRACK_SELECTED), bit(t.flag, TRACK_ACTIVE), strips.join(" ")
                ));
            }
            // Blender does not evaluate an object disabled in viewports.
            if scene.objects.iter().any(|o| o.name == n.object && o.visibility.hide_viewport) {
                continue;
            }
            let Some(i) = p.motion.list.iter().position(|m| m.name == n.object) else { continue };
            for frame in [1, 5, 12, 24, 30, 42] {
                let w = p.motion.world_at(frame as f32)[i];
                out.push(format!("{case} at {} {frame} {:.3} {:.3} {:.3} y {:.3} {:.3} {:.3}", n.object, w[3][0], w[3][1], w[3][2], w[1][0], w[1][1], w[1][2]));
            }
        }
    }
    let mut markers: Vec<_> = p.markers.iter().map(|m| (m.frame, m.name.clone())).collect();
    markers.sort();
    for (frame, name) in markers {
        out.push(format!("{case} marker {frame} {name}"));
    }
    let b = &p.playback;
    let sync = match b.sync {
        BlendSyncMode::EveryFrame => "NONE",
        BlendSyncMode::FrameDrop => "FRAME_DROP",
        BlendSyncMode::AudioSync => "AUDIO_SYNC",
    };
    out.push(format!(
        "{case} playback {sync} scrub {} lock {} subframe {} autokey {}",
        u8::from(b.audio_scrub),
        u8::from(b.lock_frame_selection),
        u8::from(b.show_subframe),
        u8::from(b.auto_key)
    ));
    // Render and Output: a Cycles value the file does not carry is the RNA
    // default, which is what Blender shows for it.
    let cycles = |name: &str, default: i32| p.cycles.get(name).and_then(|v| v.as_int()).unwrap_or(default);
    // Blender stores an add-on's enum as the item's number (`EnumProperty`):
    // the denoiser's are OptiX 2 and OpenImageDenoise 4, the pixel filter's
    // Box 0, Gaussian 1, Blackman-Harris 2.
    let denoiser = match cycles("denoiser", 4) {
        2 => "OPTIX",
        _ => "OPENIMAGEDENOISE",
    };
    let filter = match cycles("pixel_filter_type", 2) {
        0 => "BOX",
        1 => "GAUSSIAN",
        _ => "BLACKMAN_HARRIS",
    };
    assert_eq!(p.cycles.get("phiber_note").and_then(|v| v.as_str()), Some("a string property, which Blender keeps and ignores"), "{case}: the string property reads back");
    let o = &p.output;
    let mode = match o.channel {
        1 => "BW",
        2 => "BW_A",
        3 => "RGB",
        _ => "RGBA",
    };
    out.push(format!(
        "{case} render {} film {} samples {} preview {} bounces {} denoise {} {} filter {} path {} format {} depth {} mode {mode}",
        p.engine,
        u8::from(p.film_transparent),
        cycles("samples", 4096),
        cycles("preview_samples", 1024),
        cycles("diffuse_bounces", 4),
        cycles("use_denoising", 1),
        denoiser,
        filter,
        o.path,
        o.format.rna(),
        o.depth
    ));
    out
}

#[test]
fn blender_reads_back_what_phiphx_saved() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("blender_opens_saves");
    std::fs::create_dir_all(&dir).unwrap();
    let mut ours = Vec::new();
    for (case, path) in CASES {
        let bytes = std::fs::read(root.join(path)).unwrap_or_else(|e| panic!("{case}: {e}"));
        let mut file = BlendFile::parse_bytes(&bytes).unwrap();
        let before = file.extract_scene().unwrap();
        let report = edits(&before).apply(&mut file);
        assert!(report.warnings.is_empty(), "{case}: {:?}", report.warnings);
        let written = BlendCompression::of(&bytes).wrap(file.to_bytes()).unwrap();
        std::fs::write(dir.join(format!("{case}.blend")), &written).unwrap();
        let after = BlendFile::parse_bytes(&written).unwrap().extract_scene().unwrap();
        ours.extend(summary(case, &after));
    }
    let truth = std::fs::read_to_string(root.join("tests/truth/saved.txt")).expect("truth/saved.txt");
    let blender: Vec<&str> = truth.lines().filter(|l| !l.starts_with('#') && !l.is_empty()).collect();
    let missing: Vec<&String> = ours.iter().filter(|l| !blender.contains(&l.as_str())).collect();
    let extra: Vec<&&str> = blender.iter().filter(|l| !ours.iter().any(|o| o == **l)).collect();
    assert!(missing.is_empty() && extra.is_empty(), "phiphx reads what Blender does not:\n  {}\nBlender reads what phiphx does not:\n  {}\n(files in {})",
        missing.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n  "),
        extra.iter().map(|s| **s).collect::<Vec<_>>().join("\n  "),
        dir.display());
}
