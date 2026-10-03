//! Print a file's saved 3D view as the arguments the ground-truth renderers
//! take: mode, `viewinv` (as a Python nested list) and `dist`.

fn main() {
    let path = std::env::args().nth(1).expect("usage: saved_view <file.blend>");
    let bytes = std::fs::read(&path).expect("read");
    let file = phiphx::blend::BlendFile::parse_bytes(&bytes).expect("parse");
    let scene = file.extract_scene().expect("scene");
    let Some(view) = scene.props.ui.view3d.as_ref() else {
        println!("no saved 3D view in this file");
        return;
    };
    // `persp == 2` is RV3D_CAMOB: the view is through the scene camera.
    println!("{}", if view.persp == 2 { "camera" } else { "view" });
    let rows: Vec<String> = view
        .view_inv
        .iter()
        .map(|col| {
            let cells: Vec<String> = col.iter().map(|v| format!("{v:.9}")).collect();
            format!("[{}]", cells.join(","))
        })
        .collect();
    println!("[{}]", rows.join(","));
    println!("{}", view.dist);
}
