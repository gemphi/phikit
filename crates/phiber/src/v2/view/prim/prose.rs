//! Prose — running text for reading: the reader modal's paragraphs.
//!
//! A `Label` is one line of chrome. Prose wraps, and each role says what
//! kind of text it is, never how it looks; the renderer's `prose`
//! answers that from the theme.

use crate::v2::view::BView;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BProse {
    /// The document's title.
    Title,
    /// A section heading.
    Heading,
    /// A paragraph.
    Para,
    /// One item of a list, marked.
    Item,
    /// A term and its description.
    Term,
    /// An aside, dimmed.
    Note,
}

impl BView {
    pub fn prose(role: BProse, text: impl Into<String>) -> Self {
        Self::Prose { role, text: text.into() }
    }
}
