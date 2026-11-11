use super::*;

fn fixture() -> BlendFile {
    let path = format!("{}/tests/fixtures/nla_cube.blend", env!("CARGO_MANIFEST_DIR"));
    BlendFile::parse_bytes(&std::fs::read(path).expect("fixture")).expect("parse")
}

#[test]
fn a_frame_a_name_and_a_flag_are_written_where_the_dna_puts_them_and_read_back() {
    let mut file = fixture();
    let dna = file.dna.clone().unwrap();
    let scene = file.blocks.iter().position(|b| b.code_str == "SC").expect("a scene");
    let (r_off, _) = file.field_span("Scene", "r", &dna).expect("RenderData");
    let (cfra_off, cfra_size) = file.field_span("RenderData", "cfra", &dna).expect("cfra");
    assert_eq!(file.field_span("Scene", "r.cfra", &dna), Some((r_off + cfra_off, cfra_size)), "a dotted path walks the embedded struct");
    assert!(file.set_int(scene, "Scene", "r.cfra", 77));
    let read = file.read_i32(&file.blocks[scene].data[r_off + cfra_off..r_off + cfra_off + 4]);
    assert_eq!(read, 77);

    let cube = file.find_id("OB", "Cube").expect("the cube");
    assert!(file.set_id_name(cube, "Crate"));
    assert_eq!(file.extract_name_from_block(&file.blocks[cube], "id.name").unwrap(), "Crate");
    assert_eq!(file.find_id("OB", "Cube"), None);
    let field = file.object_visibility_field(&dna);
    assert!(file.set_int(cube, "Object", field, 0b110));
    assert_eq!(file.int_field(&file.blocks[cube], "Object", field, &dna), 0b110);
    assert!(!file.set_int(cube, "Object", "no_such_field", 1));

    // A new block sits after its ID with an address nobody else has.
    let before = file.blocks.len();
    let (at, address) = file.push_block("TimeMarker", scene).expect("a marker block");
    assert_eq!(at, scene + 1);
    assert_eq!(file.blocks.len(), before + 1);
    assert_eq!(file.index_at(address), Some(at));
    assert!(file.set_text(at, "TimeMarker", "name", "F_10") && file.set_int(at, "TimeMarker", "frame", 10));
    file.remove_blocks(vec![at]);
    assert_eq!(file.blocks.len(), before);
    assert_eq!(file.index_at(address), None);
}
