// SPDX-License-Identifier: Apache-2.0
use crate::util::*;

fn digest(text: &str) -> String {
    let mut md5 = Md5Hash::new();
    md5.append_str(text);
    md5.hex()
}

#[test]
fn the_digests_are_those_of_rfc_1321() {
    assert_eq!(digest(""), "D41D8CD98F00B204E9800998ECF8427E");
    assert_eq!(digest("a"), "0CC175B9C0F1B6A831C399E269772661");
    assert_eq!(digest("abc"), "900150983CD24FB0D6963F7D28E17F72");
    assert_eq!(digest("message digest"), "F96B697D7CB7938D525A2F31AAF161D0");
    assert_eq!(digest("abcdefghijklmnopqrstuvwxyz"), "C3FCD3D76192E4007DFB496CCA67E13B");
    assert_eq!(digest("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"), "D174AB98D277D9F5A5611C2C9F419D9F");
    assert_eq!(digest(&"1234567890".repeat(8)), "57EDF4A22BE3C955AC49DA2E2107B67A");
}

#[test]
fn what_is_appended_in_pieces_is_digested_as_one() {
    let text = "1234567890".repeat(20);
    let mut md5 = Md5Hash::new();
    text.as_bytes().chunks(7).for_each(|piece| md5.append(piece));
    assert_eq!(md5.hex(), digest(&text));
}

#[test]
fn a_digest_can_be_asked_for_and_more_appended() {
    let mut md5 = Md5Hash::new();
    md5.append_str("a");
    assert_eq!(md5.hex(), digest("a"));
    md5.append_str("bc");
    assert_eq!((md5.hex(), md5.finish().len()), (digest("abc"), 16));
    assert_eq!(Md5Hash::default(), Md5Hash::new());
}
