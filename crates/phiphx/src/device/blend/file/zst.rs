//! Zstd decompression for `.blend` files, from either of two decoders.
//!
//! Blender has written Zstd-compressed `.blend` files by default since 4.5
//! (`writefile.cc`), so reading one is not optional. The `zstd` crate is a
//! binding to Facebook's C library: fast, and unavailable anywhere a C
//! toolchain is not — which includes `wasm32-unknown-unknown`, the target
//! the browser build needs. `ruzstd` is a pure-Rust decoder, so it goes
//! wherever Rust goes.
//!
//! Decoding is all this module does. `ruzstd` has no encoder, and saving a
//! file is not something a browser build needs — `BlendCompression::wrap`
//! keeps the C encoder on the targets that have it and refuses on the ones
//! that do not, rather than silently writing an uncompressed file where
//! Blender would have written a compressed one.
//!
//! Which decoder is used is decided by the target, with a `pure-zstd`
//! feature to force the Rust one anywhere. That feature is what lets the
//! Rust path be *tested* on a desktop host (`parse_tests.rs`) instead of
//! only on the target where it is the only option — a decoder that has
//! never been run is not a fallback, it is a guess.

// The frame magic lives on the parser; only the frame-chaining adapter
// needs it here, and that is compiled only on the pure-Rust path.
#[cfg(any(feature = "pure-zstd", target_family = "wasm"))]
use crate::device::blend::BlendFile;
use crate::PointCloudError;
use std::io::Read;

/// True when this build decodes Zstd in Rust rather than through the C
/// library — the wasm targets, or anywhere `pure-zstd` is on.
#[cfg(test)]
const PURE: bool = cfg!(any(feature = "pure-zstd", target_family = "wasm"));

fn bad(e: impl std::fmt::Display) -> PointCloudError {
    PointCloudError::InvalidFormat(format!("Zstd .blend decompression failed: {e}"))
}

/// Decode a whole Zstd frame.
pub(crate) fn all(data: &[u8]) -> Result<Vec<u8>, PointCloudError> {
    #[cfg(any(feature = "pure-zstd", target_family = "wasm"))]
    {
        let mut out = Vec::new();
        stream(data)?.read_to_end(&mut out).map_err(bad)?;
        Ok(out)
    }
    #[cfg(not(any(feature = "pure-zstd", target_family = "wasm")))]
    {
        zstd::stream::decode_all(std::io::Cursor::new(data)).map_err(bad)
    }
}

/// Wrap a reader so it yields the decompressed stream.
pub(crate) fn stream<'a, R: Read + 'a>(r: R) -> Result<Box<dyn Read + 'a>, PointCloudError> {
    #[cfg(any(feature = "pure-zstd", target_family = "wasm"))]
    {
        // ruzstd wants a BufRead; a reader straight off a file is not one.
        Ok(Box::new(MultiFrame::new(std::io::BufReader::new(r)).map_err(bad)?))
    }
    #[cfg(not(any(feature = "pure-zstd", target_family = "wasm")))]
    {
        Ok(Box::new(zstd::stream::read::Decoder::new(r).map_err(bad)?))
    }
}

/// A reader over a *sequence* of Zstd frames.
///
/// Blender does not write one frame per file. `writefile.cc` compresses in
/// chunks and concatenates the frames, so a real `.blend` holds several — the
/// curve-path fixture in `tests/fixtures` has three. The C library decodes a
/// concatenated stream as one, and `ruzstd::StreamingDecoder` decodes exactly
/// one frame and then reports end of input, which surfaced as
/// `UnexpectedEof` from the block parser rather than as a decoder error.
///
/// So this starts a new decoder each time a frame ends and the bytes that
/// follow begin another. It peeks through `BufRead::fill_buf` rather than
/// decoding the whole input up front, because `read_quick_info` wants a
/// thumbnail out of the first blocks and should not pay for a 500 MB file to
/// get them.
#[cfg(any(feature = "pure-zstd", target_family = "wasm"))]
struct MultiFrame<R: std::io::BufRead> {
    decoder: Option<ruzstd::StreamingDecoder<R, ruzstd::FrameDecoder>>,
}

#[cfg(any(feature = "pure-zstd", target_family = "wasm"))]
impl<R: std::io::BufRead> MultiFrame<R> {
    fn new(src: R) -> Result<Self, ruzstd::frame_decoder::FrameDecoderError> {
        Ok(Self { decoder: Some(ruzstd::StreamingDecoder::new(src)?) })
    }
}

#[cfg(any(feature = "pure-zstd", target_family = "wasm"))]
impl<R: std::io::BufRead> Read for MultiFrame<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        loop {
            let Some(decoder) = self.decoder.as_mut() else { return Ok(0) };
            let n = decoder.read(buf)?;
            if n > 0 {
                return Ok(n);
            }
            // This frame is spent. Another one may follow it.
            let mut src = self.decoder.take().expect("checked above").into_inner();
            if src.fill_buf()?.starts_with(&BlendFile::ZSTD_MAGIC) {
                self.decoder = Some(
                    ruzstd::StreamingDecoder::new(src)
                        .map_err(|e| std::io::Error::other(format!("zstd frame: {e}")))?,
                );
                continue;
            }
            return Ok(0);
        }
    }
}

#[cfg(test)]
#[path = "../test/file/zst.rs"]
mod zst_test;
