//! `--make SHEET`: clubs and PFTs from a sheet, published on Phiora.
use appkit::service::PTable;

use crate::data;
use crate::part::Desk;

/// Read the sheet and publish it as the person who is signed in (or,
/// with `dry`, say what would be done), saying each row. `false` when
/// nothing could be done, or a row was refused.
pub(super) fn make(desk: &Desk, path: &str, dry: bool) -> bool {
    let word = PTable::read(data::MAKE);
    let Ok(sheet) = std::fs::read_to_string(path) else {
        eprintln!("{}", word.get("say.nothing"));
        return false;
    };
    let (rows, bad) = crate::make::rows(&sheet, &word);
    bad.iter().for_each(|line| println!("{line}"));
    if rows.is_empty() {
        eprintln!("{}", word.get("say.nothing"));
        return false;
    }
    match crate::make::publish(&desk.kept, &rows, dry, &word) {
        Ok(done) => {
            done.said.iter().for_each(|line| println!("{line}"));
            bad.is_empty() && done.refused == 0
        }
        Err(why) => {
            eprintln!("{why}");
            false
        }
    }
}
