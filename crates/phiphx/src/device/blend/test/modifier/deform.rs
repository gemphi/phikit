//! The memo answers the same thing the read does, and only reads once.

use super::*;

fn file() -> BlendFile {
    let path = format!("{}/tests/fixtures/FLORIS_BOLDRIOD_RIG.blend", env!("CARGO_MANIFEST_DIR"));
    BlendFile::parse_bytes(&std::fs::read(path).expect("the fixture")).expect("it parses")
}

/// The armatures of the fixture, by block.
fn armatures(file: &BlendFile) -> Vec<usize> {
    file.blocks.iter().enumerate().filter(|(_, b)| b.code_str == "OB").map(|(i, _)| i).collect()
}

#[test]
fn the_memo_gives_what_the_read_gives() {
    let file = file();
    let dna = file.dna.clone().expect("the DNA");
    let mut checked = 0;
    for i in armatures(&file) {
        let direct = file.pose_deform_mats(&file.blocks[i], &dna);
        if direct.is_empty() {
            continue;
        }
        let shared = file.deform_mats(&file.blocks[i], &dna);
        assert_eq!(direct.len(), shared.len(), "the same bones");
        for (name, bone) in &direct {
            let also = shared.get(name).unwrap_or_else(|| panic!("{name} is in both"));
            assert_eq!(bone.chan, also.chan, "{name}'s deform matrix");
            assert_eq!(bone.bbone.is_some(), also.bbone.is_some(), "{name}'s B-Bone");
        }
        checked += 1;
    }
    assert!(checked > 0, "the fixture has an armature with deforming bones");
}

#[test]
fn asking_twice_hands_back_the_same_read() {
    // The point of the memo: not merely an equal answer, the *same* one, so
    // forty Armature modifiers on one rig read it once.
    let file = file();
    let dna = file.dna.clone().expect("the DNA");
    for i in armatures(&file) {
        let first = file.deform_mats(&file.blocks[i], &dna);
        if first.is_empty() {
            continue;
        }
        let second = file.deform_mats(&file.blocks[i], &dna);
        assert!(Arc::ptr_eq(&first, &second), "the second ask is the first read");
        return;
    }
    panic!("the fixture has an armature with deforming bones");
}

#[test]
fn an_object_that_is_not_an_armature_memoises_an_empty_answer() {
    // A mesh object has no pose, so the read is empty — and asking again
    // must still be cheap rather than re-reading every time.
    let file = file();
    let dna = file.dna.clone().expect("the DNA");
    let mesh = file
        .blocks
        .iter()
        .position(|b| b.code_str == "OB" && file.pose_deform_mats(b, &dna).is_empty())
        .expect("an object with no deforming pose");
    let first = file.deform_mats(&file.blocks[mesh], &dna);
    assert!(first.is_empty());
    assert!(Arc::ptr_eq(&first, &file.deform_mats(&file.blocks[mesh], &dna)));
}
