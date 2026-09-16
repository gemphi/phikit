//! Video Sequencer host definitions matching Blender 5.2 `SpaceSeq` in `DNA_space_types.h`.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum SequencerViewMode {
    Sequencer = 0,
    Preview = 1,
    Both = 2,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SequencerConfig {
    pub view_mode: SequencerViewMode,
    pub show_strip_overlays: bool,
}

impl Default for SequencerConfig {
    fn default() -> Self {
        Self {
            view_mode: SequencerViewMode::Both,
            show_strip_overlays: true,
        }
    }
}
