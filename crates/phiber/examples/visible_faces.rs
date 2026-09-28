//! What the viewport would draw for a file: faces of objects the view layer
//! shows, against everything the file holds. Blender's own count for the
//! Female-01 character is 1,444,810 visible polygons.
use phiber::v2::state::BState;
fn main() {
    let path = std::env::args().nth(1).expect("path");
    let bytes = std::fs::read(&path).expect("read");
    let file = phiphx::blend::BlendFile::parse_bytes(&bytes).expect("parse");
    let blend: phiphx::blend::BlendScene = file.extract_scene().expect("scene");
    let model = phiphx::scene::SceneModel::from_blend_owned(blend, &path);
    let mut s = BState::new();
    phiber::v2::app::seed::seed_view_layer(&mut s, &model);
    let (mut all, mut shown, mut n_all, mut n_shown) = (0usize, 0usize, 0usize, 0usize);
    let mut big: Vec<(usize, String, bool)> = Vec::new();
    for o in &model.objects {
        let f = o.mesh.indices.len() / 3;
        let hidden = phiber::v2::part::outliner::key::is_hidden(&s, &o.name);
        all += f; n_all += 1;
        if !hidden { shown += f; n_shown += 1; }
        if f > 100_000 { big.push((f, o.name.clone(), hidden)); }
    }
    big.sort(); big.reverse();
    println!("objects {n_all} shown {n_shown} | tris all {all} shown {shown}");
    for (f, n, h) in big { println!("  {f:>9} {n} {}", if h { "hidden" } else { "SHOWN" }); }
    println!("collections: {}", model.collections.len());
}
