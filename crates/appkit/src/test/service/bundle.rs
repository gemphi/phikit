use crate::pui::core::PAsset;
use crate::service::{PBundle, PFolder};

#[test]
fn every_shared_asset_is_built_into_the_program_as_it_is_on_disk() {
    let disk = PFolder::new(&[PFolder::shared()]);
    // The icons and the face an app is drawn with are both there.
    assert!(PBundle.load("icon/plus.svg").is_some_and(|bytes| bytes.starts_with(b"<svg")));
    assert!(PBundle.load("font/PhiPillOpen-Light.otf").is_some());
    assert_eq!(PBundle.load("icon/no-such-icon.svg"), None);
    // What a folder of it lists and holds is what the disk does.
    assert_eq!(PBundle.list("icon"), disk.list("icon"));
    assert!(PBundle.list("icon").iter().all(|path| PBundle.load(path) == disk.load(path)));
}

#[test]
fn of_two_sources_the_first_is_asked_first() {
    let own = std::env::temp_dir().join("appkit-test-bundle");
    std::fs::create_dir_all(own.join("icon")).expect("a folder to write in");
    std::fs::write(own.join("icon/plus.svg"), "own").expect("a file to write");
    let both = (PFolder::new(&[&own.to_string_lossy()]), PBundle);
    assert_eq!(both.load("icon/plus.svg"), Some(b"own".to_vec()));
    // What only the second has is still found, and listed once.
    assert!(both.load("icon/minus.svg").is_some());
    assert_eq!(both.list("icon").iter().filter(|path| *path == "icon/plus.svg").count(), 1);
}
