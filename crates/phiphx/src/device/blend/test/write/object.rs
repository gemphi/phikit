use crate::device::blend::write::object::*;

#[test]
fn a_cloned_object_keeps_its_values_and_points_nowhere() {
    let path = format!("{}/tests/fixtures/nla_cube.blend", env!("CARGO_MANIFEST_DIR"));
    let mut file = BlendFile::parse_bytes(&std::fs::read(path).expect("fixture")).expect("parse");
    let dna = file.dna.clone().unwrap();
    let template = file.mesh_object_template().expect("a mesh object");
    let spans = file.pointer_spans("Object", &dna);
    assert!(spans.len() > 30, "an Object points at many things: {}", spans.len());
    let (data_off, _) = file.field_span("Object", "*data", &dna).unwrap();
    assert!(spans.iter().any(|(o, s)| *o == data_off && *s == 8), "the data pointer is among them");
    let (id_next, _) = file.field_span("Object", "id.*next", &dna).unwrap();
    assert!(spans.iter().any(|(o, _)| *o == id_next), "the embedded ID's pointers too");

    let before = file.blocks.len();
    let (at, address) = file.add_object(template, "Crate", 0x1234, [1.0, 2.0, 3.0], [1.0, 1.0, 1.0]).expect("added");
    assert_eq!(file.blocks.len(), before + 1);
    assert_eq!(file.blocks[at].code_str, "OB");
    assert_eq!(file.extract_name_from_block(&file.blocks[at], "id.name").unwrap(), "Crate");
    assert_eq!(file.ptr(&file.blocks[at], file.off("Object", "*data", &dna)), 0x1234);
    assert_eq!(file.ptr(&file.blocks[at], file.off("Object", "*parent", &dna)), 0, "no parent");
    assert_eq!(file.ptr(&file.blocks[at], file.off("Object", "modifiers", &dna)), 0, "no modifiers");
    let (loc, _) = file.field_span("Object", "loc[1]", &dna).unwrap();
    assert_eq!(file.f32_at(&file.blocks[at].data, loc), 2.0);
    let (dt_off, _) = file.field_span("Object", "dt", &dna).unwrap();
    assert_eq!(file.blocks[at].data[dt_off], file.blocks[template].data[dt_off], "the draw type is the template's");
    assert_eq!(file.index_at(address), Some(at));
    assert!(&file.blocks[at + 1].code == DNA1, "inserted before DNA1");
}
