//! USD importer smoke tests.

use super::UsdImporter;

#[test]
fn detects_usda_magic() {
    assert!(UsdImporter::can_import(b"#usda 1.0\n"));
}

#[test]
fn rejects_unknown() {
    assert!(!UsdImporter::can_import(b"not a usd file"));
}

#[test]
fn parses_sample_usda_points() {
    let cloud = UsdImporter::import_bytes(UsdImporter::sample_usda().as_bytes());
    assert!(cloud.is_ok());
}
