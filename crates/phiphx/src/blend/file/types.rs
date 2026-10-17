//! Shared types, constants and trait impls for the native .blend parser.

/// .blend file header magic
pub(crate) const BLEND_MAGIC: &[u8; 7] = b"BLENDER";

/// DNA block type identifier
pub(crate) const DNA1: &[u8; 4] = b"DNA1";

/// ENDB block type identifier  
pub(crate) const ENDB: &[u8; 4] = b"ENDB";

/// Blend file header information
#[derive(Debug, Clone)]
pub struct BlendHeader {
    /// Version digits as written ("305", "0502").
    pub version: String,
    pub pointer_size: u8, // 4 or 8
    pub little_endian: bool,
    /// `BLENDER_FILE_VERSION` (e.g. 305, 502).
    pub version_number: u32,
    /// `BLEND_FILE_FORMAT_VERSION`: 0 = legacy 12-byte header + `BHead4` /
    /// `SmallBHead8`; 1 = 17-byte header + `LargeBHead8` (Blender 5.0+).
    pub file_format_version: u8,
}

/// Block-header layout, selected from the file header exactly like
/// `BlenderHeader::bhead_type()` in `blo_core_blend_header.cc`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BHeadType {
    BHead4,
    SmallBHead8,
    LargeBHead8,
}

impl BlendHeader {
    pub fn bhead_type(&self) -> BHeadType {
        if self.pointer_size == 4 {
            BHeadType::BHead4
        } else if self.file_format_version == 0 {
            BHeadType::SmallBHead8
        } else {
            BHeadType::LargeBHead8
        }
    }

    /// Human-readable "5.2" / "3.6" style version from the header digits.
    pub fn version_label(&self) -> String {
        let n = self.version_number;
        format!("{}.{}", n / 100, n % 100)
    }
}

/// A data block in the .blend file
#[derive(Debug, Clone)]
pub struct BlendBlock {
    pub code: [u8; 4],
    pub code_str: String,
    /// Payload length in bytes (`BHead.len`, 64-bit since `LargeBHead8`).
    pub size: u64,
    pub old_address: u64, // Memory address when saved
    pub sdna_index: u32,
    /// Number of structs in the array (`BHead.nr`).
    pub count: u64,
    pub data: Vec<u8>,
}

/// DNA structure field definition
#[derive(Debug, Clone)]
pub struct DnaField {
    pub type_index: u16,
    pub name_index: u16,
}

/// DNA structure type definition
#[derive(Debug, Clone)]
pub struct DnaStruct {
    pub type_index: u16,
    pub fields: Vec<DnaField>,
}

/// DNA type information
#[derive(Debug, Clone)]
pub struct DnaType {
    pub names: Vec<String>,
    pub types: Vec<String>,
    pub type_sizes: Vec<u16>,
    pub structs: Vec<DnaStruct>,
    /// Every member's offset, worked out once (`sdna.rs`).
    pub layout: std::sync::OnceLock<super::sdna::DnaLayout>,
}

/// Parsed .blend file
#[derive(Debug)]
pub struct BlendFile {
    pub header: BlendHeader,
    pub blocks: Vec<BlendBlock>,
    pub dna: Option<DnaType>,
    /// Blender's `OldNewMap`: saved address → index into `blocks`.
    pub(crate) addr_index: std::collections::HashMap<u64, usize>,
    /// Every block index for addresses written more than once (Blender 5.x
    /// generated pointers repeat across IDs; see `scope.rs`).
    pub(crate) dup_index: std::collections::HashMap<u64, Vec<usize>>,
    /// Index of the ID block whose data is being read (`usize::MAX`: none).
    pub(crate) scope: std::sync::atomic::AtomicUsize,
    /// Set when a progress callback asked to cancel; the recursive
    /// collection walk checks it so a cancel unwinds promptly.
    pub(crate) cancel_requested: std::sync::atomic::AtomicBool,
    /// An armature's deform matrices by its block address, read once rather
    /// than once per Armature modifier pointing at it
    /// (`modifier/deform.rs`, which says why).
    pub(crate) deform_memo: std::sync::Mutex<
        std::collections::HashMap<u64, std::sync::Arc<crate::blend::modifier::deform::BDeformMats>>,
    >,
    /// Meshes a stack has reached just before its Armature, left by whichever
    /// of the two consumers applied the prefix for the other to take
    /// (`modifier/prefix.rs`). An `Arc` because a progressive open's deferred
    /// apply reaches it from a worker thread through `DeferredRaw`.
    pub(crate) prefix_box: std::sync::Arc<crate::blend::modifier::prefix::BPrefixBox>,
    /// This frame's driven modifier settings, by the OBJECT's saved address
    /// (`modifier/driver.rs`, batch 174 read them; this is what finally
    /// carries them into the stack).
    ///
    /// It lives on the file rather than being threaded through
    /// `modifier_jobs` and `skin_sources` because `modifier_stack` — the one
    /// place a modifier is built — already has `&self` and the object's
    /// block, so a store here needs no signature to change and both
    /// consumers pick the overrides up for free.
    ///
    /// Filled by `scene/read.rs` AFTER the animation is read, which is why
    /// the modifier evaluation had to move below it: Blender's depsgraph
    /// runs animation then geometry for exactly this reason.
    pub(crate) driven_modifiers: std::sync::Mutex<
        std::collections::HashMap<u64, Vec<(crate::blend::modifier::driver::BModWrite, crate::blend::modifier::driver::BModValue)>>,
    >,
}

pub use crate::blend::scene::types::*;

#[cfg(all(test, feature = "proptest"))]
#[path = "types_tests.rs"]
mod primary_object_tests;
