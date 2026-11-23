use crate::service::PTable;

/// What an app would keep in a file: a name a line.
const TEXT: &str = "# name\twhat it stands for\nsend\tSend\nreply\tI heard: {said}\nsend\tGo\nno tab here\n";

#[test]
fn a_table_says_what_a_name_stands_for_and_nothing_for_one_it_does_not_have() {
    let table = PTable::read(TEXT);
    // A name given twice keeps its first row.
    assert_eq!((table.get("send"), table.get("stop")), ("Send", ""));
}

#[test]
fn a_word_in_braces_is_filled_in() {
    assert_eq!(PTable::read(TEXT).fill("reply", "said", "hello"), "I heard: hello");
}
