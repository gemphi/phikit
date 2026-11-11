use crate::device::blend::write::marker::*;

#[test]
fn the_list_is_rewritten_over_the_blocks_the_file_had_and_reads_back_in_order() {
    let path = format!("{}/tests/fixtures/nla_cube.blend", env!("CARGO_MANIFEST_DIR"));
    let mut file = BlendFile::parse_bytes(&std::fs::read(path).expect("fixture")).expect("parse");
    let scene = file.current_scene_index().expect("a scene");
    let before = file.blocks.len();
    let wanted = vec![(5, "Intro".to_string()), (40, "Cut".to_string()), (90, "End".to_string())];
    let added = file.set_markers(scene, &wanted);
    assert_eq!(file.blocks.len(), before + added);
    let dna = file.dna.clone().unwrap();
    let read: Vec<(i32, String)> = file.markers(&file.blocks[scene], &dna).into_iter().map(|m| (m.frame, m.name)).collect();
    assert_eq!(read, wanted, "the scene lists them in order");
    // Fewer: the spare blocks are dropped, the rest relinked.
    let fewer = vec![(40, "Cut".to_string())];
    file.set_markers(scene, &fewer);
    let read: Vec<(i32, String)> = file.markers(&file.blocks[scene], &dna).into_iter().map(|m| (m.frame, m.name)).collect();
    assert_eq!(read, fewer);
    assert_eq!(file.blocks.len(), before + 1);
    // None: an empty list, no marker blocks left over.
    file.set_markers(scene, &[]);
    assert!(file.markers(&file.blocks[scene], &dna).is_empty());
    assert_eq!(file.blocks.len(), before + added - added);
    // And the file still serialises and parses.
    let again = BlendFile::parse_bytes(&file.to_bytes()).expect("parse the written file");
    assert_eq!(again.blocks.len(), file.blocks.len());
}
