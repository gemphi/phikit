use crate::device::blend::file::sdna::*;
use crate::device::blend::file::kind::DnaField;

fn dna() -> DnaType {
    DnaType {
        names: vec!["x".into(), "name[8]".into(), "*next".into(), "mat[4][4]".into(), "(*fn)()".into()],
        types: vec!["int".into(), "char".into(), "Thing".into(), "float".into()],
        type_sizes: vec![4, 1, 0, 4],
        structs: vec![DnaStruct { type_index: 2, fields: vec![DnaField { type_index: 0, name_index: 0 }, DnaField { type_index: 1, name_index: 1 }, DnaField { type_index: 2, name_index: 2 }, DnaField { type_index: 3, name_index: 3 }, DnaField { type_index: 0, name_index: 4 }] }],
        layout: Default::default(),
    }
}

#[test]
fn members_are_laid_out_once_by_full_and_bare_name() {
    let dna = dna();
    let l = dna.layout(8);
    assert_eq!(l.offset("Thing", "x"), Some(0));
    assert_eq!((l.offset("Thing", "name[8]"), l.offset("Thing", "name")), (Some(4), Some(4)), "an array by either spelling");
    assert_eq!(l.offset("Thing", "*next"), Some(12), "the pointer by its DNA name");
    assert_eq!(l.offset("Thing", "mat"), Some(20));
    assert_eq!(l.struct_size("Thing"), Some(20 + 64 + 8), "a 4 × 4 of floats and a function pointer");
    assert_eq!(l.offset("Thing", "nope"), None);
    assert_eq!(dna.layout(4).offset("Thing", "mat"), Some(16), "another pointer size is answered too");
    assert_eq!(dims("[3][2]"), 6);
}
