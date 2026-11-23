use appkit::service::PTable;

use crate::{Reply, Stand, data};

#[test]
fn the_stand_in_says_what_it_heard_in_the_words_the_text_table_gives() {
    let form = PTable::read(data::TEXT).get("reply").to_string();
    // The table has the sentence, and the sentence has a place for what
    // was said.
    assert!(form.contains("{said}"), "{form:?}");
    // The stand-in's answer is there all at once.
    let (answer, done) = Stand::new(&form).reply("hello", &[]).more();
    assert!(done);
    assert_eq!(answer, form.replace("{said}", "hello"));
    assert!(answer.contains("hello") && !answer.contains('{'));
}
