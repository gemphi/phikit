//! Scrubbing set - transport buttons and frame range (static data).

use super::TransportEntry;

pub static TRANSPORT: &[TransportEntry] = &[
    TransportEntry {
        idx: "jump_start",
        icon: "chevrons-left",
        tip: "Jump to Start",
    },
    TransportEntry {
        idx: "play_reverse",
        icon: "rewind",
        tip: "Play Reverse",
    },
    TransportEntry {
        idx: "play",
        icon: "play",
        tip: "Play",
    },
    TransportEntry {
        idx: "jump_end",
        icon: "chevrons-right",
        tip: "Jump to End",
    },
];

/// Playback range (Blender scene frame_start / frame_end).
pub const FRAME_START: &str = "1";
pub const FRAME_END: &str = "250";
