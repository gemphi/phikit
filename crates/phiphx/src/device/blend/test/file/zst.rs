use super::*;
use crate::device::blend::BlendFile;

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
    let file = crate::device::blend::BlendFile::parse_bytes(&data).expect("parse");
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
