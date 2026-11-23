use crate::pui::core::PColor;
use crate::theme::*;

/// What an app would keep in a file: a token a line.
const LOOK: &str = "# kind\ttoken\tvalue\ncolor\tink\t#ff0000\ncolor\tveil\t#00000080\ncolor\tbad\tred\nsize\tgap\t16\nsize\tbad\twide\nword\ttext.face\tPhi-Pill Open\nshape\tgap\t3\n\nsize\tgap\t20\n";

#[test]
fn a_look_is_read_from_a_table_a_token_a_line() {
    let look = PLook::read(LOOK);
    assert_eq!(look.color("ink"), Some(PColor::new(1.0, 0.0, 0.0, 1.0)));
    assert_eq!(look.color("veil").map(|veil| (veil.a * 255.0).round()), Some(128.0));
    assert_eq!(look.word("text.face"), Some("Phi-Pill Open"));
}

#[test]
fn a_token_given_twice_has_its_last_value_and_a_line_that_cannot_be_read_is_skipped() {
    let look = PLook::read(LOOK);
    assert_eq!(look.size("gap"), Some(20.0));
    assert_eq!((look.color("bad"), look.size("bad"), look.size("ink"), look.word("gap")), (None, None, None, None));
    assert_eq!(PLook::read(""), PLook::default());
}
