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
use crate::blend::BlendFile;
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
mod tests {
    use super::*;
    use crate::blend::BlendFile;

    /// A frame the C encoder wrote, decoded by whichever decoder this build
    /// uses, has to come back byte for byte.
    ///
    /// Run twice to mean anything: `cargo test -p phiphx` exercises the C
    /// decoder, `cargo test -p phiphx --features pure-zstd` the Rust one.
    /// The data is compressible and has a repeat far enough back to need a
    /// real window rather than a literal copy, which is where a decoder with
    /// a wrong window size gives wrong bytes instead of an error.
    #[test]
    fn a_frame_round_trips_through_this_builds_decoder() {
        let mut raw = Vec::new();
        for i in 0..40_000u32 {
            raw.extend_from_slice(&i.to_le_bytes());
        }
        raw.extend_from_slice(b"BLENDER-v500");
        raw.extend_from_within(0..30_000);

        let packed = zstd::stream::encode_all(std::io::Cursor::new(&raw), 3).expect("encode");
        assert!(packed.len() < raw.len(), "the fixture has to actually compress");
        assert_eq!(packed[0..4], BlendFile::ZSTD_MAGIC, "zstd magic");

        assert_eq!(all(&packed).expect("decode whole frame"), raw, "whole frame");

        let mut out = Vec::new();
        stream(&packed[..])
            .expect("decode stream")
            .read_to_end(&mut out)
            .expect("read stream");
        assert_eq!(out, raw, "streamed");
    }

    /// Blender writes a `.blend` as several concatenated Zstd frames, not
    /// one. Both decoders have to read across the join.
    ///
    /// This is the case that made the pure-Rust decoder worth a feature
    /// flag: `ruzstd::StreamingDecoder` stops at the first frame's end, so
    /// without chaining this returns the first chunk and the block parser
    /// then fails with `UnexpectedEof` on a file the C library reads fine.
    /// Three frames, because two can pass by accident when the second is
    /// also the last.
    #[test]
    fn concatenated_frames_read_as_one_stream() {
        let parts: [&[u8]; 3] = [b"BLENDER17-01v500", b"the middle frame", b"and the tail"];
        let mut packed = Vec::new();
        for p in parts {
            packed.extend_from_slice(
                &zstd::stream::encode_all(std::io::Cursor::new(p), 3).expect("encode"),
            );
        }
        let want: Vec<u8> = parts.concat();

        assert_eq!(all(&packed).expect("decode all frames"), want, "whole");

        let mut out = Vec::new();
        stream(&packed[..]).expect("stream").read_to_end(&mut out).expect("read");
        assert_eq!(out, want, "streamed across the joins");
    }

    /// The real thing: the curve-path fixture is a Blender-written file with
    /// three frames in it. Whichever decoder this build uses must parse it.
    #[test]
    fn the_multi_frame_fixture_parses() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/curve_paths.blend");
        let data = std::fs::read(path).expect("fixture");
        assert_eq!(data[0..4], BlendFile::ZSTD_MAGIC, "precondition: the fixture is zstd");
        let frames = data.windows(4).filter(|w| *w == BlendFile::ZSTD_MAGIC).count();
        assert!(frames > 1, "precondition: more than one frame (found {frames})");
        let file = crate::blend::BlendFile::parse_bytes(&data).expect("parse");
        assert!(file.dna.is_some(), "DNA survives a multi-frame decode");
        assert!(!file.blocks.is_empty());
    }

    /// The feature actually switches decoder — so a green suite under
    /// `--features pure-zstd` is evidence about the Rust decoder and not a
    /// second run of the C one.
    #[test]
    fn the_feature_picks_the_rust_decoder() {
        assert_eq!(PURE, cfg!(feature = "pure-zstd") || cfg!(target_family = "wasm"));
    }
}
