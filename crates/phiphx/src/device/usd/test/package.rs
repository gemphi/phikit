//! The USDZ reader.
use crate::device::usd::error::UsdaError;
use crate::device::usd::package::UsdzReader;
use std::io::{Cursor, Write};


/// Helper: create a minimal .usdz (zip) archive containing a single .usda file.
fn make_usdz(usda_content: &str) -> Vec<u8> {
    let buf = Vec::new();
    let cursor = Cursor::new(buf);
    let mut zip_writer = zip::ZipWriter::new(cursor);
    let options = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Stored);
    zip_writer
        .start_file("scene.usda", options)
        .expect("start_file");
    zip_writer
        .write_all(usda_content.as_bytes())
        .expect("write");
    let cursor = zip_writer.finish().expect("finish");
    cursor.into_inner()
}

#[test]
fn reads_usda_from_usdz() {
    let usda = "#usda 1.0\n(\n    defaultPrim = \"Root\"\n)\n\ndef Xform \"Root\" {}\n";
    let usdz_bytes = make_usdz(usda);
    let stage = UsdzReader::read(&usdz_bytes).unwrap();
    assert_eq!(stage.default_prim.as_deref(), Some("Root"));
}

#[test]
fn invalid_archive_error() {
    let garbage = b"not a zip file at all";
    let result = UsdzReader::read(garbage);
    assert!(matches!(result, Err(UsdaError::InvalidArchive(_))));
}

#[test]
fn no_layer_found_error() {
    // Create a zip with no .usda/.usdc entries.
    let buf = Vec::new();
    let cursor = Cursor::new(buf);
    let mut zip_writer = zip::ZipWriter::new(cursor);
    let options = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Stored);
    zip_writer
        .start_file("texture.png", options)
        .expect("start_file");
    zip_writer.write_all(b"fake png data").expect("write");
    let cursor = zip_writer.finish().expect("finish");
    let usdz_bytes = cursor.into_inner();

    let result = UsdzReader::read(&usdz_bytes);
    assert!(matches!(result, Err(UsdaError::NoLayerFound)));
}
