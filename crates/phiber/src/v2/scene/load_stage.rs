//! The import stages the statusbar's progress bar reads out — Blender's
//! file-read job phases (read → parse geometry → build materials → done),
//! each with the label it shows and the fraction the bar reaches when that
//! phase begins. The keys the fraction and the label live under are in
//! `scene/key.rs`; this is what they mean.

/// The ordered import stages surfaced to the user. Each carries its label and
/// the progress fraction the bar reaches when that stage begins. Mirrors
/// Blender's file-read job phases (read → parse geometry → build materials →
/// done).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadStage {
    /// Opening the file / decoding the container (sidecar or SDNA header).
    Reading,
    /// Walking blocks and triangulating mesh geometry.
    ParsingMeshes,
    /// Resolving material slots and texture references.
    BuildingMaterials,
    /// Import settled — bar full, about to clear.
    Done,
}

impl LoadStage {
    /// Human-readable status label shown next to the progress bar.
    pub fn label(self) -> &'static str {
        match self {
            LoadStage::Reading => "Reading file",
            LoadStage::ParsingMeshes => "Parsing meshes",
            LoadStage::BuildingMaterials => "Building materials",
            LoadStage::Done => "Done",
        }
    }

    /// Progress fraction (0..=1) the bar reaches when this stage begins.
    pub fn progress(self) -> f32 {
        match self {
            LoadStage::Reading => 0.15,
            LoadStage::ParsingMeshes => 0.5,
            LoadStage::BuildingMaterials => 0.85,
            LoadStage::Done => 1.0,
        }
    }

    /// Which coarse phase a reported fraction falls in. The reader sends a
    /// detailed stage string that changes every tick ("Reading objects
    /// 431/2104 Cube.003"); the loading panel's running log wants the four
    /// phases, not that flood, so the phase is read back from the fraction at
    /// each stage's begin threshold ([`Self::progress`]). A phase line is
    /// logged only when this result changes.
    pub fn for_fraction(fraction: f32) -> Self {
        let f = fraction.clamp(0.0, 1.0);
        if f >= LoadStage::Done.progress() {
            LoadStage::Done
        } else if f >= LoadStage::BuildingMaterials.progress() {
            LoadStage::BuildingMaterials
        } else if f >= LoadStage::ParsingMeshes.progress() {
            LoadStage::ParsingMeshes
        } else {
            LoadStage::Reading
        }
    }
}
