//! Reader set — the declared documents the reader can show, and the block
//! vocabulary they are written in.
//!
//! Every document is static content: a title, where it was taken from, and a
//! list of blocks. The text is Blender's — the manual (`getting_started/`),
//! blender.org's support and community pages, and the release notes — so
//! that Help and Welcome read the same reference material Blender's do,
//! inside the window instead of in a browser.

use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

/// One block of a document. The reader lays each out by kind; the store
/// never says how anything looks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BReaderBlock {
    /// A section heading.
    Heading(&'static str),
    /// A paragraph of running text.
    Para(&'static str),
    /// One item of a list.
    Item(&'static str),
    /// A term and its description — the manual's definition lists.
    Term(&'static str, &'static str),
    /// An aside: a note, a date, a version line.
    Note(&'static str),
}

/// One document.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BReaderDoc {
    /// What `ctx.reader.doc` holds while this document is open.
    pub slug: &'static str,
    pub icon: &'static str,
    pub title: &'static str,
    /// Where the text comes from, shown under the title. Not a link: the
    /// reader is the place to read it.
    pub source: &'static str,
    pub blocks: &'static [BReaderBlock],
}

impl BSpec for BReaderDoc {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::Custom)
    }
    fn label(&self) -> &'static str {
        self.title
    }
    fn icon(&self) -> &'static str {
        self.icon
    }
}

/// The document with `slug`, if the store declares one.
pub fn find(slug: &str) -> Option<&'static BReaderDoc> {
    super::store::DOCS.iter().copied().find(|d| d.slug == slug)
}

/// A welcome or help entry's target that opens a document rather than a
/// browser: `doc:<slug>`.
pub const DOC_PREFIX: &str = "doc:";

/// The slug a `doc:` target names, if it is one.
pub fn doc_target(target: &str) -> Option<&str> {
    target.strip_prefix(DOC_PREFIX)
}
