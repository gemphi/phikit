use crate::scene::texture_find::*;

#[test]
fn a_texture_moved_into_a_subfolder_is_found_by_its_name_whatever_its_case() {
    let root = std::env::temp_dir().join(format!("phiphx_find_{}", std::process::id()));
    let deep = root.join("textures").join("Female-01").join("skin");
    std::fs::create_dir_all(&deep).unwrap();
    std::fs::write(deep.join("Body_Diffuse.PNG"), b"x").unwrap();
    assert_eq!(find(&root, "body_diffuse.png"), Some(deep.join("Body_Diffuse.PNG")));
    assert_eq!(find(&root, "nothing_here.png"), None);
    // A second folder rebuilds the index rather than answering from the first.
    let other = root.join("elsewhere");
    std::fs::create_dir_all(&other).unwrap();
    assert_eq!(find(&other, "body_diffuse.png"), None);
    let _ = std::fs::remove_dir_all(&root);
}
