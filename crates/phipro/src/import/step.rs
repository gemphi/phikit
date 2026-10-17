//! The checkpoints an import reports, and the fraction each one is at.
//!
//! A progress bar is a promise about how far along the work is, so the
//! fractions are declared in one table that adds up rather than written
//! out at the call sites. The old module reported 0.05, 0.72 and 0.80 and
//! then returned — so the bar stopped at four fifths and the caller had to
//! know to finish it.

/// Where an import has got to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BImportStep {
    /// Parsing the asset. This is the long one, and the importer's own
    /// progress is folded into its share ([`Self::within`]).
    Read,
    /// Creating the directories.
    Make,
    /// Writing `project.json`.
    Manifest,
    /// Writing the scene.
    Scene,
    /// Finished.
    Done,
}

impl BImportStep {
    /// Where each step begins, as a fraction of the whole. `Done` is 1.0,
    /// so a bar driven by this reaches its end.
    pub fn part(self) -> f32 {
        match self {
            Self::Read => 0.05,
            Self::Make => 0.80,
            Self::Manifest => 0.86,
            Self::Scene => 0.93,
            Self::Done => 1.0,
        }
    }

    /// What the step is doing, for the line beside the bar.
    pub fn note(self, what: &str) -> String {
        match self {
            Self::Read => format!("Reading {what}"),
            Self::Make => format!("Creating {what}"),
            Self::Manifest => format!("Writing the manifest for {what}"),
            Self::Scene => format!("Writing the scene for {what}"),
            Self::Done => format!("{what} is ready"),
        }
    }

    /// The asset reader's own `0.0 ..= 1.0` mapped into this step's share
    /// of the whole, so a long parse moves the bar instead of pinning it.
    pub fn within(self, inner: f32) -> f32 {
        let next = match self {
            Self::Read => Self::Make,
            Self::Make => Self::Manifest,
            Self::Manifest => Self::Scene,
            Self::Scene | Self::Done => Self::Done,
        };
        let span = next.part() - self.part();
        self.part() + span * inner.clamp(0.0, 1.0)
    }
}

/// A bar being driven forward, and the reason this is a type.
///
/// A step's own share is monotonic, but the fraction an asset reader
/// reports *inside* its share is not: `phiphx`'s `.blend` reader counts
/// through its phases and restarts the count per data block, so a run of it
/// reports 0.125 and then 0.0875. Forwarded straight through, that is a
/// progress bar the user watches slide backwards.
///
/// So the crate that owns the bar owns that too: a fraction already passed
/// is never reported again, which costs one `f32` and makes "progress" mean
/// what it says. Nothing is hidden — the note still arrives for every
/// report, so the line beside the bar keeps moving even while the fraction
/// holds still.
pub struct BProgress<'a> {
    note: &'a dyn Fn(f32, &str),
    at: std::cell::Cell<f32>,
}

impl<'a> BProgress<'a> {
    pub fn new(note: &'a dyn Fn(f32, &str)) -> Self {
        Self { note, at: std::cell::Cell::new(0.0) }
    }

    /// Report reaching `step`.
    pub fn at(&self, step: BImportStep, what: &str) {
        self.say(step.part(), &step.note(what));
    }

    /// Report a reader's own `inner` fraction within `step`.
    pub fn within(&self, step: BImportStep, inner: f32, what: &str) {
        self.say(step.within(inner), what);
    }

    fn say(&self, part: f32, what: &str) {
        let part = part.max(self.at.get());
        self.at.set(part);
        (self.note)(part, what);
    }
}

#[cfg(test)]
#[path = "step_tests.rs"]
mod step_tests;
