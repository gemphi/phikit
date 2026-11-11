use crate::device::blend::write::edit::*;

#[test]
fn the_edits_land_in_the_file_and_read_back_through_the_scene_reader() {
    let path = format!("{}/tests/fixtures/nla_cube.blend", env!("CARGO_MANIFEST_DIR"));
    let mut file = BlendFile::parse_bytes(&std::fs::read(&path).expect("fixture")).expect("parse");
    let edits = BlendEditList {
        frame: Some(42),
        frame_start: Some(10),
        frame_end: Some(120),
        objects: vec![BlendObjectEdit {
            name: "Cube".into(),
            rename: Some("Crate".into()),
            visibility: Some(BlendVisibility { hide_viewport: false, hide_select: true, hide_render: true }),
            hidden: Some(true),
            selected: Some(true),
            transform: None,
            shadow: Some(BlendShadowCast { display: false, ray: false }),
        }],
        active: Some("Cube".into()),
        markers: Some(vec![(12, "Intro".into()), (100, "Cut".into())]),
        collections: vec![BlendCollectionEdit { path: "Scene Collection".into(), hidden: None, exclude: None, visibility: Some(BlendVisibility { hide_render: true, ..Default::default() }) }],
        unwritable: vec!["1 added object".into()],
        border: Some(crate::device::blend::BlendRenderBorder { on: true, crop: true, rect: [0.25, 0.75, 0.1, 0.9] }),
        ..Default::default()
    };
    let report = edits.apply(&mut file);
    assert!(report.patched >= 7, "{report:?}");
    assert_eq!(report.warnings, ["1 added object"]);

    let again = BlendFile::parse_bytes(&file.to_bytes()).expect("the written file parses");
    let scene = again.extract_scene().expect("scene");
    assert_eq!((scene.props.frame_current, scene.props.frame_start, scene.props.frame_end), (42, 10, 120));
    let crate_ = scene.objects.iter().find(|o| o.name == "Crate").expect("renamed");
    assert!(!scene.objects.iter().any(|o| o.name == "Cube"));
    assert!(crate_.visibility.hide_select && crate_.visibility.hide_render && !crate_.visibility.hide_viewport);
    assert_eq!(crate_.shadow, BlendShadowCast { display: false, ray: false }, "both Shadow switches, `dtx` and the visibility flag");
    let index = scene.objects.iter().position(|o| o.name == "Crate").unwrap();
    // Hidden, so saved deselected: Blender drops the selection of what it
    // cannot see (`BKE_base_eval_flags`), and the file says so.
    assert!(scene.hidden.contains(&index) && !scene.selected.contains(&index), "the base flags");
    assert_eq!(scene.active_object, Some(index), "the active object");
    assert_eq!(scene.props.markers.iter().map(|m| (m.frame, m.name.as_str())).collect::<Vec<_>>(), [(12, "Intro"), (100, "Cut")]);
    assert!(scene.collections[0].visibility.hide_render, "the collection's camera");
    assert_eq!(scene.props.border, crate::device::blend::BlendRenderBorder { on: true, crop: true, rect: [0.25, 0.75, 0.1, 0.9] }, "the render region and its two mode bits");
}
