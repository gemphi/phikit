use crate::device::blend::write::*;

#[test]
fn a_parsed_file_written_back_is_the_same_bytes() {
    for name in ["nla_cube.blend", "THYMUS-BOLDROID-WAVE-003.blend"] {
        let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
        let bytes = std::fs::read(&path).expect("fixture");
        let file = BlendFile::parse_bytes(&bytes).expect("parse");
        let again = file.to_bytes();
        assert_eq!(again.len(), bytes.len(), "{name}: the same length");
        assert!(again == bytes, "{name}: byte for byte");
    }
}

#[test]
fn a_compressed_file_round_trips_through_its_own_wrapping() {
    let path = format!("{}/tests/fixtures/DEPTH-OF-FIELD.blend", env!("CARGO_MANIFEST_DIR"));
    let bytes = std::fs::read(&path).expect("fixture");
    assert_eq!(BlendCompression::of(&bytes), BlendCompression::Zstd);
    let file = BlendFile::parse_bytes(&bytes).expect("parse");
    let wrapped = BlendCompression::Zstd.wrap(file.to_bytes()).expect("zstd");
    let again = BlendFile::parse_bytes(&wrapped).expect("parse again");
    assert_eq!(again.blocks.len(), file.blocks.len());
    assert_eq!(again.to_bytes(), file.to_bytes());
    assert_eq!(BlendHeader::to_bytes(&file.header).len(), if file.header.file_format_version == 0 { 12 } else { 17 });
}
