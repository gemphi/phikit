//! The reader opens over whatever was up and gives it back when it closes.
//!
//! A document named from the welcome window, Help or About raises the
//! reader and remembers the modal it covered; closing the reader puts that
//! modal back, so reading the release notes never loses your place.

use crate::v2::state::val::V_MODAL_READER;
use crate::v2::state::{BKey, BPath, BState, ModalKey, rule::BRule};

/// `reader.doc` → raise the reader, remembering the modal underneath.
pub struct BReaderOpenRule;

impl BRule for BReaderOpenRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == ModalKey::ReaderDoc.path()
    }
    fn after(&self, _: &BPath, slug: &str, s: &mut BState) {
        if slug.is_empty() || s.get(ModalKey::Open) == Some(V_MODAL_READER) {
            return;
        }
        match s.get(ModalKey::Open).filter(|m| !m.is_empty()) {
            Some(under) => s.set(ModalKey::ReaderReturn, under.to_string()),
            None => s.remove(ModalKey::ReaderReturn),
        }
        s.set(ModalKey::Open, V_MODAL_READER);
    }
}

/// `modal = ""` → put back the modal the reader covered, if any.
pub struct BReaderCloseRule;

impl BRule for BReaderCloseRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == ModalKey::Open.path()
    }
    fn after(&self, _: &BPath, value: &str, s: &mut BState) {
        if !value.is_empty() {
            return;
        }
        if let Some(back) = s.get(ModalKey::ReaderReturn).map(str::to_string) {
            s.remove(ModalKey::ReaderReturn);
            s.set(ModalKey::Open, back);
        }
    }
}
