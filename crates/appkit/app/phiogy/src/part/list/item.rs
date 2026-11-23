//! What a page that is a list holds: what stands at the head of a
//! line, what can be done at its end, and the items themselves.

use crate::site::Face;

/// What stands at the head of a line.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Seat {
    #[default]
    None,
    /// A cell of a sheet: a face, a place's mark.
    Face(Face),
    Icon(String),
    /// A picture by its key, whole.
    Picture(String),
}

/// What can be done at the end of a line, or in a row of tabs: its
/// words, or its icon where it has one; whether it is so; and what a
/// press means.
#[derive(Debug, Clone, PartialEq)]
pub struct Deed<A> {
    pub label: String,
    pub icon: String,
    pub on: bool,
    pub act: A,
}

impl<A> Deed<A> {
    pub fn word(label: &str, on: bool, act: A) -> Deed<A> {
        Deed { label: label.to_string(), icon: String::new(), on, act }
    }

    pub fn icon(icon: &str, label: &str, act: A) -> Deed<A> {
        Deed { label: label.to_string(), icon: icon.to_string(), on: false, act }
    }

    /// The pill it is drawn as: its icon alone where it has one, and
    /// else its words.
    pub fn pill(&self) -> (&str, &str, bool) {
        match self.icon.is_empty() {
            true => ("", self.label.as_str(), self.on),
            false => (self.icon.as_str(), "", self.on),
        }
    }
}

/// One thing of a page, from its top down.
#[derive(Debug, Clone, PartialEq)]
pub enum Item<A> {
    Head(String),
    /// A heading with what can be done at its end: a page that has no
    /// head of a panel's over it, and its own way out.
    Top(String, Vec<Deed<A>>),
    /// Words that are not pressed, broken to fit.
    Note(String),
    /// A sum: what it is a sum of, and the sum, which stands on a
    /// ground of its own at the line's end (appkit's `figure`).
    Sum(String, String),
    /// The field of that number among the page's, what it says while
    /// empty, and what a press on it means.
    Field(usize, String, A),
    Tabs(Vec<Deed<A>>),
    /// A line: its head, two lines of words, what a press on it means
    /// if it means something, and what can be done at its end.
    Line(Seat, String, String, Option<A>, Vec<Deed<A>>),
}
