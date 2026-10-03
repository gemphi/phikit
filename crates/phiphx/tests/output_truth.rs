//! Properties → Output must read off the file exactly what Blender reports.
//!
//! `truth/output_format.txt` was produced by Blender 5.0.1
//! (`truth/blender_output.py`), which also writes the `OUTPUT-FORMAT.blend`
//! fixture — a scene with none of the Output panel left at its default, so
//! that a field read at the wrong offset cannot pass by accident.
//!
//! The production fixtures in the list are the useful awkward cases: two of
//! them are set to FFMPEG with an absolute Windows path, which is what a real
//! file from an animation project looks like.

use phiphx::blend::{BlendFile, BlendImageFormat};
use std::path::PathBuf;

/// Blender reports an imtype phiber does not name as a bare number, so the
/// truth rows stay comparable rather than collapsing several formats onto an
/// empty string.
fn rna_of(format: BlendImageFormat) -> String {
    match format.rna() {
        "" => format!("{format:?}"),
        name => name.to_string(),
    }
}

fn codec_of(code: u8) -> &'static str {
    match code {
        0 => "NONE",
        1 => "PXR24",
        2 => "ZIP",
        3 => "PIZ",
        4 => "RLE",
        5 => "ZIPS",
        6 => "B44",
        7 => "B44A",
        8 => "DWAA",
        9 => "DWAB",
        _ => "?",
    }
}

fn row_of(name: &str, path: &PathBuf) -> String {
    let file = BlendFile::parse_bytes(&std::fs::read(path).unwrap()).unwrap();
    let out = file.extract_scene().unwrap().props.output;
    let mode = match out.channel {
        1 => "BW",
        2 => "BW_A",
        3 => "RGB",
        _ => "RGBA",
    };
    format!("{name}|{}|{}|{}|{mode}|{}|{}|{}", out.path, rna_of(out.format), out.depth, codec_of(out.exr_codec), out.quality, out.compress)
}

#[test]
fn output_properties_match_blender() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let truth = std::fs::read_to_string(root.join("tests/truth/output_format.txt")).unwrap();
    let mut checked = 0;
    for line in truth.lines().filter(|l| !l.trim().is_empty()) {
        let name = line.split('|').next().unwrap();
        let path = root.join("tests/fixtures").join(name);
        if !path.exists() {
            continue;
        }
        assert_eq!(row_of(name, &path), line, "{name}");
        checked += 1;
    }
    assert!(checked >= 4, "only {checked} fixtures were read; the truth file or the fixtures are missing");
}
