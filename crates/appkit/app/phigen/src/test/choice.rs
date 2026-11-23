use std::rc::Rc;

use appkit::service::PStore;

use crate::test::Named;
use crate::{Choice, Phigen};

fn store(name: &str) -> PStore {
    let root = std::env::temp_dir().join("phigen-test-choice").join(name);
    std::fs::remove_dir_all(&root).ok();
    PStore::at(&root.to_string_lossy())
}

#[test]
fn a_choice_is_kept_a_line_and_the_last_under_a_name_stands() {
    let kept = store("line");
    let choice = Choice::kept(kept.clone(), "setting.tsv");
    assert_eq!(choice.get("look"), "");
    choice.put("look", "dark");
    choice.put("model", "beta");
    choice.put("look", "light");
    assert_eq!(kept.read("setting.tsv").as_deref(), Some("model\tbeta\nlook\tlight\n"));
    let again = Choice::kept(kept, "setting.tsv");
    assert_eq!((again.get("look"), again.get("model"), again.get("other")), ("light".to_string(), "beta".to_string(), String::new()));
    // With nowhere to keep it a choice lasts as long as it is held.
    let loose = Choice::default();
    loose.put("look", "dark");
    assert_eq!(loose.get("look"), "dark");
}

#[test]
fn step_29_phigen_opens_the_way_i_left_it() {
    let kept = store("again");
    let open = |store: Option<PStore>| Phigen::kept(Rc::new(Phigen::look("")), Box::new(Named::of(&["alpha", "beta"])), "Hello.", store);
    let left = |app: &Phigen| (app.desk().shade.name().to_string(), app.desk().history.borrow().voice().to_string());
    let app = open(Some(kept.clone()));
    assert_eq!(left(&app), ("light".to_string(), "alpha".to_string()));
    assert!(app.desk().dress("dark") && app.desk().choose("beta"));
    // A look there is not, and a model there is not, change nothing.
    assert!(!app.desk().dress("dusk") && !app.desk().choose("gamma"));
    assert_eq!(left(&open(Some(kept.clone()))), ("dark".to_string(), "beta".to_string()));
    // Opened with nowhere to keep things, it is as it ships.
    assert_eq!(left(&open(None)), ("light".to_string(), "alpha".to_string()));
}
