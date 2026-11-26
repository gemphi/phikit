//! A sheet of clubs and PFTs is read into the records it asks for, each
//! cell what its column says it is, and a row that is wrong is named and
//! left out. Publishing them is checked against the real Phiora by
//! `script/make.mjs`.
use appkit::service::PTable;
use serde_json::json;

use crate::data;
use crate::make::rows;

const SHEET: &str = "# a club and two of its PFTs\n\
kind\tref\tname\tticker\tsupply:n\tlive:b\tcolors[]\tclubId\ttraits.hair\ttraits.fur\n\
club\tbpsc\tBold Pups Space Club\tBPSC\t8000\ttrue\t#112233|#445566\t\t\t\n\
profilePft\tpft_bpsc_0001\tBold Pup #1\t\t\t\t\t@bpsc\tmohawk\tgold\n\
profilePft\tpft_bpsc_0002\tBold Pup #2\t\t\t\t\t@bpsc\t\tgrey\n";

#[test]
fn a_sheet_is_read_into_the_records_it_asks_for() {
    let word = PTable::read(data::MAKE);
    let (all, bad) = rows(SHEET, &word);
    assert!(bad.is_empty(), "{bad:?}");
    assert_eq!(all.len(), 3);
    // Each cell is what its column says: words, a number, a flag, a list; an empty cell is no field.
    assert_eq!((all[0].kind.as_str(), all[0].name.as_str(), all[0].line), ("club", "bpsc", 3));
    assert_eq!(
        serde_json::Value::Object(all[0].fields.clone()),
        json!({ "ref": "bpsc", "name": "Bold Pups Space Club", "ticker": "BPSC", "supply": 8000, "live": true, "colors": ["#112233", "#445566"] })
    );
    // A dot puts a field inside another; `@` is kept for publishing to turn into the club's id.
    assert_eq!(serde_json::Value::Object(all[1].fields.clone()), json!({ "ref": "pft_bpsc_0001", "name": "Bold Pup #1", "clubId": "@bpsc", "traits": { "hair": "mohawk", "fur": "gold" } }));
    assert_eq!(all[2].fields["traits"], json!({ "fur": "grey" }));
}

#[test]
fn a_row_that_is_wrong_is_named_and_left_out_and_the_rest_are_read() {
    let word = PTable::read(data::MAKE);
    let sheet = "kind\tref\tsupply:n\tlive:b\n\
club\ta\tmany\t\n\
club\tb\t\tmaybe\n\
world\tc\t\t\n\
club\t\t\t\n\
club\td\t5\ttrue\n\
club\td\t6\t\n";
    let (all, bad) = rows(sheet, &word);
    assert_eq!(all.iter().map(|row| row.name.as_str()).collect::<Vec<_>>(), ["d"]);
    assert_eq!(bad.len(), 5, "{bad:?}");
    assert!(bad[0].starts_with("line 2:") && bad[0].contains("many is not a number"), "{bad:?}");
    assert!(bad[1].contains("maybe is not true or false"), "{bad:?}");
    assert!(bad[2].contains("world is not a kind a sheet may make"), "{bad:?}");
    assert!(bad[3].contains("no ref"), "{bad:?}");
    assert!(bad[4].contains("ref d is on two rows"), "{bad:?}");
    // A sheet with no kind and ref columns is no sheet.
    let (none, why) = rows("name\tticker\nx\ty\n", &word);
    assert!(none.is_empty() && why.len() == 1 && why[0].contains("no kind and ref"), "{why:?}");
}
