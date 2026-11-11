use crate::device::blend::read::object::instance_member::*;

#[test]
fn only_a_shown_collection_instancer_generates() {
    assert!(collection_generator(OB_DUPLICOLLECTION, 0));
    assert!(!collection_generator(OB_DUPLICOLLECTION, OB_HIDE_VIEWPORT), "restricted in viewports");
    assert!(!collection_generator(OB_DUPLICOLLECTION | OB_DUPLIVERTS, 0), "vertices come first");
    assert!(!collection_generator(OB_DUPLICOLLECTION | OB_DUPLIPARTS, 0), "particles come first");
    assert!(!collection_generator(0, 0));
}
