//! What a device can say about a file without reading it: the [`Peek`].
//! Header-only. `Phx::peek` returns one, whatever the variant.
/// Preview image, **top-down** RGBA8 rows.
#[derive(Debug, Clone, PartialEq)]
pub struct PeekThumbnail {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}
/// One scene the file lists, with its frame range.
#[derive(Debug, Clone, PartialEq)]
pub struct PeekScene {
    pub scene: String,
    pub frame_start: i32,
    pub frame_end: i32,
}
/// Version, scenes and embedded thumbnail of a file.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Peek {
    /// "5.2" style version from the header.
    pub version: String,
    pub scenes: Vec<PeekScene>,
    pub thumbnail: Option<PeekThumbnail>,
}