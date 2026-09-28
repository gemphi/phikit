//! The reader's store — every document, one file each. `DOCS` is the order
//! the store lists them in; the first is the manual's front page, which is
//! what the reader shows when no document is named.

use super::set::BReaderDoc;

mod about;
mod community;
mod credit;
mod get_involved;
mod help_system;
mod license;
mod release_note;
mod report_bug;
mod support;
mod tutorial;
mod whats_new;

pub static DOCS: &[&BReaderDoc] = &[
    &about::DOC,
    &help_system::DOC,
    &tutorial::DOC,
    &support::DOC,
    &community::DOC,
    &get_involved::DOC,
    &whats_new::DOC,
    &release_note::DOC,
    &report_bug::DOC,
    &license::DOC,
    &credit::DOC,
];
