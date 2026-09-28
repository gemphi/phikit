//! Reader modal — a document from the store, opened over whatever was on
//! screen and returning to it when closed.
//!
//! Blender's Help menu and welcome window hand every link to the system
//! browser (`wm.url_open`). Phiber keeps the reading inside the window: a
//! welcome or help entry whose target is `doc:<slug>` writes the slug to
//! `ctx.reader.doc`, and the follow rule in `state::follow` raises this
//! modal, remembering the modal it covered so closing the reader restores
//! it. Only links to Phiber's own website leave the app.

pub mod back;
pub mod set;
pub mod store;

use crate::v2::state::ModalKey;
use crate::v2::state::BState;
use crate::v2::view::prim::BCard;
use crate::v2::view::prim::BColumn;
use crate::v2::view::prim::BProse;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
pub use set::BReaderBlock;
pub use set::BReaderDoc;
pub use set::DOC_PREFIX;
pub use set::doc_target;
pub use set::find;
pub use store::DOCS;

#[derive(Clone, Copy, Debug, Default)]
pub struct BReaderModal;

/// The document open now, or the manual's front page when none is named.
pub fn current(s: &BState) -> &'static BReaderDoc {
    s.get(ModalKey::ReaderDoc).and_then(find).unwrap_or(DOCS[0])
}

fn block(b: &BReaderBlock) -> BView {
    match *b {
        BReaderBlock::Heading(t) => BView::prose(BProse::Heading, t),
        BReaderBlock::Para(t) => BView::prose(BProse::Para, t),
        BReaderBlock::Item(t) => BView::prose(BProse::Item, t),
        BReaderBlock::Term(term, text) => BView::prose(BProse::Term, format!("{term} — {text}")),
        BReaderBlock::Note(t) => BView::prose(BProse::Note, t),
    }
}

impl BViewItem for BReaderModal {
    fn build(&self, s: &BState) -> BView {
        let doc = current(s);
        let mut body = Vec::with_capacity(doc.blocks.len() + 2);
        body.push(BView::prose(BProse::Title, doc.title));
        body.push(BView::prose(BProse::Note, format!("Source: {}", doc.source)));
        body.extend(doc.blocks.iter().map(block));
        BView::from(BCard::column(BColumn::Body, body))
    }
}
