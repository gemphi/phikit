use crate::pui::core::{PAsset, PFilm};

/// A frame of one colour: so wide, so high.
fn frame(width: u32, height: u32, shade: u8) -> Vec<u8> {
    vec![shade; (width * height * 4) as usize]
}

#[test]
fn a_film_holds_the_latest_frame_of_each_name() {
    let film = PFilm::default();
    assert!(!film.running() && film.frame("far").is_none());
    film.show("far", 2, 1, frame(2, 1, 9));
    film.show("far", 4, 2, frame(4, 2, 7));
    film.show("near", 1, 1, frame(1, 1, 3));
    // The second frame took the first's place, and is one further on.
    assert_eq!(film.at("far"), Some((2, 4, 2)));
    assert_eq!(film.frame("far").map(|shown| shown.pixel.len()), Some(32));
    assert_eq!(film.at("near"), Some((1, 1, 1)));
    assert!(film.running());
}

#[test]
fn a_frame_that_is_not_its_own_size_is_not_shown() {
    let film = PFilm::default();
    // Too few bytes for two by two, and a frame of no points.
    film.show("far", 2, 2, frame(2, 1, 9));
    film.show("far", 0, 2, Vec::new());
    assert!(film.at("far").is_none() && !film.running());
}

#[test]
fn a_film_s_pictures_end_by_name_and_by_how_names_begin() {
    let film = PFilm::default();
    for name in ["call/a/main", "call/a/screen", "call/b/main", "own"] {
        film.show(name, 1, 1, frame(1, 1, 1));
    }
    film.end("own");
    film.clear("call/a/");
    assert_eq!((film.at("call/a/main"), film.at("call/a/screen"), film.at("own")), (None, None, None));
    assert!(film.at("call/b/main").is_some());
    // A copy is the same film: what one ends is over for the other.
    let copy = film.clone();
    copy.end("call/b/main");
    assert!(!film.running());
}

/// Assets with a film, and assets with none.
struct Moving(PFilm);
struct Still;

impl PAsset for Moving {
    fn load(&self, _path: &str) -> Option<Vec<u8>> {
        None
    }

    fn film(&self) -> Option<PFilm> {
        Some(self.0.clone())
    }
}

impl PAsset for Still {
    fn load(&self, _path: &str) -> Option<Vec<u8>> {
        None
    }
}

#[test]
fn assets_taken_together_have_the_film_of_whichever_has_one() {
    let film = PFilm::default();
    film.show("far", 1, 1, frame(1, 1, 1));
    assert!(Still.film().is_none());
    assert!((Still, Still).film().is_none());
    let found = (Still, (Moving(film), Still)).film().expect("the second has a film");
    assert_eq!(found.at("far"), Some((1, 1, 1)));
}
