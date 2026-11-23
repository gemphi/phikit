use crate::service::PStore;

fn store(name: &str) -> PStore {
    let root = std::env::temp_dir().join("appkit-test-store").join(name);
    // A run before this one may have left it there.
    std::fs::remove_dir_all(&root).ok();
    PStore::at(&root.to_string_lossy())
}

#[test]
fn what_is_written_is_read_again_and_a_file_that_was_never_written_is_none() {
    let store = store("round");
    assert_eq!(store.read("history.tsv"), None);
    // The folder is made the first time something is written.
    assert!(store.write("history.tsv", "0\tperson\thello\n"));
    assert_eq!(store.read("history.tsv").as_deref(), Some("0\tperson\thello\n"));
    assert!(store.write("history.tsv", "") && store.read("history.tsv").as_deref() == Some(""));
}

#[test]
fn a_person_s_store_for_an_app_is_a_folder_of_the_app_s_name() {
    // Where the tests run the system names a place for such folders.
    let store = PStore::user("appkit-test-app").expect("a place for a person's files");
    assert!(format!("{store:?}").contains("appkit-test-app"));
}
