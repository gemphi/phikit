use crate::pui::core::*;

/// An asset source for the tests: two files it holds in memory.
struct Shelf;

impl PAsset for Shelf {
    fn load(&self, path: &str) -> Option<Vec<u8>> {
        match path {
            "icon/send.svg" => Some(b"<svg/>".to_vec()),
            "font/body.ttf" => Some(vec![0, 1, 0, 0]),
            _ => None,
        }
    }

    fn list(&self, folder: &str) -> Vec<String> {
        match folder {
            "icon" => vec!["icon/send.svg".to_string()],
            _ => Vec::new(),
        }
    }
}

/// A source that only loads, as one read from the web would.
struct Remote;

impl PAsset for Remote {
    fn load(&self, _path: &str) -> Option<Vec<u8>> {
        None
    }
}

#[test]
fn an_asset_is_found_by_its_path_or_not_at_all() {
    let source: &dyn PAsset = &Shelf;
    assert_eq!(source.load("icon/send.svg").as_deref(), Some(b"<svg/>".as_slice()));
    assert_eq!(source.load("icon/none.svg"), None);
    assert_eq!(source.list("icon"), ["icon/send.svg"]);
}

#[test]
fn a_source_that_cannot_list_answers_with_none() {
    assert!(Remote.list("icon").is_empty() && Remote.load("icon/send.svg").is_none());
}

#[test]
fn an_icon_s_picture_is_where_its_name_says_among_the_assets() {
    let icon = PIcon::new("send", PSize { width: 16.0, height: 16.0 });
    assert_eq!(icon.path(), "icon/send.svg");
    assert!(Shelf.load(&icon.path()).is_some());
}
