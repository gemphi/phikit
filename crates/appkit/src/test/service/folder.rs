use crate::pui::core::PAsset;
use crate::service::PFolder;

#[test]
fn a_shared_icon_is_found_by_its_path_and_a_file_nobody_has_is_not() {
    let folder = PFolder::new(&[PFolder::shared()]);
    assert!(folder.load("icon/plus.svg").is_some_and(|bytes| bytes.starts_with(b"<svg")));
    assert_eq!(folder.load("icon/no-such-icon.svg"), None);
    assert!(folder.list("icon").contains(&"icon/plus.svg".to_string()));
}

#[test]
fn an_app_s_own_file_is_found_before_the_shared_one_of_the_same_path() {
    let own = std::env::temp_dir().join("appkit-test-folder");
    std::fs::create_dir_all(own.join("icon")).expect("a folder to write in");
    std::fs::write(own.join("icon/plus.svg"), "own").expect("a file to write");
    let folder = PFolder::new(&[&own.to_string_lossy(), PFolder::shared()]);
    assert_eq!(folder.load("icon/plus.svg"), Some(b"own".to_vec()));
    // What only the shared folder has is still found.
    assert!(folder.load("icon/minus.svg").is_some());
}
