//! Scrubbing part - timeline transport cluster entries (Blender space_action header).
//!
//! Pure static data: transport buttons and the frame range. The leaf view
//! (`bui/part/scrubbing`) only matches and renders, reading frame state.

pub mod set;

/// A single transport button.
#[derive(Clone, Copy, Debug)]
pub struct TransportEntry {
    pub idx: &'static str,
    pub icon: &'static str,
    pub tip: &'static str,
}
