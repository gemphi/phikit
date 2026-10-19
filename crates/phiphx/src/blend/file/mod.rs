//! The `.blend` container. Header, block headers, block table and pointer lookup (`scope`), DNA and struct layout, quick peek, names.
//! Status and next steps: `jobs.md` in this folder.

/// Zstd decoding, from the C library or from Rust (`zst::PURE`).
mod zst;

pub(crate) mod bhead;
pub(crate) mod dna;
pub(crate) mod header;
pub(crate) mod layout;
pub(crate) mod names;
pub(crate) mod parse;
pub mod quick;
pub(crate) mod readers;
pub(crate) mod scope;
pub mod sdna;
pub mod types;
