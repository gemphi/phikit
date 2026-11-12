// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::kernel::*;
use crate::util::Transform;

#[test]
fn an_object_s_members_are_inside_it_and_none_overlaps() {
    for name in ["KernelObject", "AttributeMap"] {
        let layout = RecordLayout::of(name).expect("in data/record.tsv");
        let mut taken = vec![false; layout.word_count];
        for field in &layout.field {
            assert!(field.word + field.word_count <= layout.word_count, "{name}.{}", field.name);
            for word in &mut taken[field.word..field.word + field.word_count] {
                assert!(!*word, "{name}.{} overlaps", field.name);
                *word = true;
            }
        }
    }
    assert!(RecordLayout::of("KernelNothing").is_none());
}

#[test]
fn a_matrix_starts_on_16_bytes_and_a_64_bit_member_on_8() {
    let layout = RecordLayout::of("KernelObject").expect("in data/record.tsv");
    let field = |name: &str| *layout.field(name).expect("in data/record.tsv");
    assert_eq!((field("tfm").word, field("tfm").word_count, field("itfm").word), (0, 12, 12));
    assert_eq!((field("color").word_count, field("dupli_uv").word_count), (3, 2));
    assert_eq!((field("light_set_membership").word % 2, field("light_set_membership").word_count), (0, 2));
    assert_eq!(layout.word_count % 4, 0);
}

#[test]
fn an_array_is_its_elements_one_after_another() {
    let mut array = RecordArray::of("KernelObject").expect("in data/record.tsv");
    assert!(array.is_empty());
    array.alloc(3);
    assert_eq!((array.len(), array.word().len()), (3, 3 * array.layout().word_count));
    array.set_float(1, "alpha", 0.5);
    array.set_int(1, "numverts", -7);
    array.set_uint64(1, "light_set_membership", u64::MAX - 1);
    array.set_float_list(1, "color", &[1.0, 2.0, 3.0]);
    array.set_transform(2, "tfm", &Transform::translate(4.0, 5.0, 6.0));
    assert_eq!((array.float(1, "alpha"), array.int(1, "numverts"), array.uint64(1, "light_set_membership")), (0.5, -7, u64::MAX - 1));
    assert_eq!(array.float_list(1, "color"), [1.0, 2.0, 3.0]);
    assert_eq!(array.float_list(2, "tfm"), Transform::translate(4.0, 5.0, 6.0).word());
    // The neighbours stay as they were.
    assert_eq!((array.float(0, "alpha"), array.float(2, "alpha"), array.float(1, "pass_id"), array.uint(1, "receiver_light_set")), (0.0, 0.0, 0.0, 0));
}

#[test]
fn allocating_again_starts_from_zeros() {
    let mut array = RecordArray::of("AttributeMap").expect("in data/record.tsv");
    array.alloc(2);
    array.set_uint64(1, "id", 9);
    array.alloc(2);
    assert_eq!(array.uint64(1, "id"), 0);
    array.alloc(0);
    assert!(array.is_empty());
}
