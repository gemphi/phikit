use super::*;
use crate::blend::BPlay;
use crate::blend::rig::BRigState;
use super::super::SceneMesh;

/// The two-bone figure (`tests/fixtures/make_skin_rig.py`).
fn figure() -> SceneModel {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/skin_rig.blend");
    crate::Phx::open(&path).expect("the fixture opens")
}

/// Bends "tip" a quarter turn about x from frame 10 on; still before.
#[derive(Debug)]
struct Bend {
    tip: usize,
}

impl BPlay for Bend {
    fn rig(&self, rig: &crate::blend::rig::BRig, frame: f32) -> Option<BRigState> {
        let mut state = rig.rest_state();
        if frame >= 10.0 {
            let half = std::f32::consts::FRAC_PI_4;
            state.raw[self.tip].quat = [half.cos(), half.sin(), 0.0, 0.0];
            state.raw[self.tip].rotmode = 0;
        }
        Some(state)
    }
    fn range(&self) -> (f32, f32) {
        (1.0, 20.0)
    }
}

/// The centre of the column's top ring — its last four vertices, as the
/// fixture builds the rings bottom to top.
fn top(mesh: &SceneMesh) -> [f32; 3] {
    let ring = &mesh.vertices[mesh.vertices.len() - 4..];
    let mut c = [0.0; 3];
    for v in ring {
        for k in 0..3 {
            c[k] += v[k] / 4.0;
        }
    }
    c
}

#[test]
fn an_adopted_figure_is_posed_by_its_play_and_a_mesh_added_to_it_follows_the_bones() {
    let from = figure();
    assert_eq!(from.motion.rigs.rigs.len(), 1, "one rig");
    assert_eq!(from.motion.rigs.skins.len(), 1, "one skinned mesh");
    let rig = &from.motion.rigs.rigs[0].rig;
    let play = BPlayRef(Arc::new(Bend { tip: rig.bone("tip").unwrap() }));

    let mut model = SceneModel::default();
    let first = model.adopt_rig(&from, 0, "Ayanda", |_| true, play.clone());
    let second = model.adopt_rig(&from, 0, "Buhle", |_| true, play);
    assert_eq!((first.rig, second.rig), (0, 1));
    assert_eq!(model.objects.iter().map(|o| o.name.as_str()).collect::<Vec<_>>(), ["Ayanda Body", "Buhle Body"]);
    assert_eq!(model.objects.len(), model.object_types.len());
    assert_eq!(model.objects.len(), model.locals.len());
    assert_eq!(model.objects.len(), model.uv_maps.len());
    assert!(Arc::ptr_eq(&model.motion.rigs.skins[0].source, &model.motion.rigs.skins[1].source), "the figure shared, not copied");

    // A triangle at the column's top, weighted wholly to "tip".
    let tip = from.motion.rigs.skins[0].source.groups.iter().position(|g| g == "tip").unwrap() as u32;
    let hat = BSkinnedMesh {
        vertices: vec![[-0.1, 0.0, 2.0], [0.1, 0.0, 2.0], [0.0, 0.0, 2.2]],
        faces: vec![vec![0, 1, 2]],
        uvs: vec![[0.0, 0.0]; 3],
        face_materials: vec![0],
        groups: from.motion.rigs.skins[0].source.groups.clone(),
        weights: vec![vec![(tip, 1.0)]; 3],
        smooth: false,
    };
    let hat = model.add_skinned(first.rig, "Ayanda Hat", hat, vec![SceneMaterial::neutral()]).expect("added");
    assert_eq!(model.objects[hat].name, "Ayanda Hat");

    // At rest the column stands straight; bent, its top swings off the z
    // axis toward ±y, and the hat goes with it.
    let at = |frame: f32| -> std::collections::HashMap<usize, SceneMesh> { model.motion.posed_at(frame).0.into_iter().collect() };
    let (still, bent) = (at(1.0), at(12.0));
    let column = |m: &std::collections::HashMap<usize, SceneMesh>| top(&m[&0]);
    assert!(column(&still)[1].abs() < 1e-4 && (column(&still)[2] - 2.0).abs() < 1e-4, "{:?}", column(&still));
    assert!(column(&bent)[1].abs() > 0.9 && (column(&bent)[2] - 1.0).abs() < 0.15, "the top bent a quarter turn: {:?}", column(&bent));
    assert!(top(&still[&1])[2] > 1.99, "Buhle stands too");
    let hat_top = |m: &std::collections::HashMap<usize, SceneMesh>| m[&hat].vertices[2];
    assert!((hat_top(&still)[2] - 2.2).abs() < 1e-4);
    assert!(hat_top(&bent)[1].abs() > 1.1, "the hat followed the tip: {:?}", hat_top(&bent));
}

#[test]
fn an_added_object_moves_with_its_play() {
    #[derive(Debug)]
    struct Slide;
    impl BPlay for Slide {
        fn object(&self, frame: f32) -> Option<ObjectTransformRaw> {
            Some(ObjectTransformRaw::from_parts([frame, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0], [1.0; 3]))
        }
        fn range(&self) -> (f32, f32) {
            (1.0, 10.0)
        }
    }
    let mesh = SceneMesh { vertices: vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]], indices: vec![0, 1, 2], normals: super::super::SceneNormals::PerFace(vec![[0.0, 0.0, 1.0]]), corners: Default::default() };
    let mut model = SceneModel::default();
    let ball = model.add_object(SceneObject::identity("Ball", mesh), Some(BPlayRef(Arc::new(Slide))));
    assert!(model.motion.animated());
    let world = model.motion.at(4.0).object(ball).unwrap();
    assert_eq!(world[3][0], 4.0, "at frame 4 it stands at x = 4");
}
