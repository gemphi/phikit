//! Who answers. A model stands behind `Reply` (`model.rs`); where none is
//! connected `Stand` answers, and says that it is not one.
//!
//! An answer may arrive over time, as a model writes it: `Reply` gives a
//! `Source`, and the history takes what has arrived of it once a frame.

use crate::chat::Turn;

/// Whoever answers what a person says.
pub trait Reply {
    /// The answer to `said`, which comes after `past`.
    fn reply(&mut self, said: &str, past: &[Turn]) -> Box<dyn Source>;

    /// What whoever answers is called, where it has a name: a model's, on
    /// its server. The stand-in has none.
    fn name(&self) -> &str {
        ""
    }

    /// The names of those that could answer, this one among them. It is
    /// asked once a frame: a server may take a moment to say.
    fn list(&mut self) -> Vec<String> {
        Vec::new()
    }

    /// Whether it is still finding out who could answer: a server was
    /// asked and has not said yet. While it is, the list is asked for
    /// again each frame.
    fn asking(&self) -> bool {
        false
    }

    /// Have the one of that name answer from now on. `false` when there
    /// is nobody to choose among.
    fn choose(&mut self, _name: &str) -> bool {
        false
    }
}

/// An answer, as it arrives.
pub trait Source {
    /// What has arrived since this was last asked, and whether that is
    /// all of it.
    fn more(&mut self) -> (String, bool);

    /// Stop it: whoever is writing it is told to. An answer that is there
    /// all at once has nothing to stop.
    fn stop(&mut self) {}
}

/// An answer that is there all at once.
pub struct Whole(pub String);

impl Source for Whole {
    fn more(&mut self) -> (String, bool) {
        (std::mem::take(&mut self.0), true)
    }
}

/// The stand-in: it answers with one sentence from `data/text.tsv`, with
/// what was said put in where the sentence has `{said}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stand {
    form: String,
}

impl Stand {
    pub fn new(form: &str) -> Stand {
        Stand { form: form.to_string() }
    }
}

impl Reply for Stand {
    fn reply(&mut self, said: &str, _past: &[Turn]) -> Box<dyn Source> {
        Box::new(Whole(self.form.replace("{said}", said)))
    }
}
