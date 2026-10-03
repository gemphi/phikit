#[allow(unused_imports)]
use super::*;

/// When in a step a system runs.
///
/// Apple's stages are a render pipeline; these are an observation pipeline,
/// and they are named for what the engine actually does in each: take text in,
/// settle the phasors against it, then read something out.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stage {
    /// New text, new words, new edges.
    Ingest,
    /// Anything that must see the ingest before the coupling does.
    PreCouple,
    /// Words move under their conditions. The default.
    Couple,
    /// Measurement, bookkeeping, evidence counts.
    PostCouple,
    /// What a person or another program sees: a report, a rendering, a reply.
    Readout,
}

impl Stage {
    /// The order a step runs them in.
    pub const ORDER: &'static [Stage] =
        &[Stage::Ingest, Stage::PreCouple, Stage::Couple, Stage::PostCouple, Stage::Readout];

    pub fn label(self) -> &'static str {
        match self {
            Stage::Ingest => "Ingest",
            Stage::PreCouple => "Pre-couple",
            Stage::Couple => "Couple",
            Stage::PostCouple => "Post-couple",
            Stage::Readout => "Readout",
        }
    }
}
