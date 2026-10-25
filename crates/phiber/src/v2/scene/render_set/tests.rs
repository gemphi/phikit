use super::*;

#[test]
fn the_panel_s_resolution_over_the_file_s() {
    let mut model = crate::phiphx::scene::SceneModel::default();
    (model.render.resolution_x, model.render.resolution_y, model.render.resolution_percent) = (1920, 1080, 50);
    let mut s = BState::new();
    assert_eq!(BResolution::now(&s).pixels(&model), (960, 540), "the file's until the panel holds one");
    assert_eq!(BResolution::now(&s).edited(&model), None);
    s.set(ButsKey::Value("resolution_y"), BValue::Int(1920).encode());
    s.set(ButsKey::Value("resolution_percentage"), BValue::Int(100).encode());
    let now = BResolution::now(&s);
    assert_eq!((now.base(&model), now.pixels(&model)), ((1920, 1920), (1920, 1920)));
    assert_eq!(now.edited(&model), Some([1920, 1920, 100]));
    assert_eq!(render_set(&s).resolution, now);
}
