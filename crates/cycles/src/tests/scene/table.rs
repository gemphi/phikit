// SPDX-License-Identifier: Apache-2.0
use crate::scene::*;

#[test]
fn tables_follow_one_another_in_whole_chunks() {
    let (mut lookup, mut array) = (LookupTableSet::new(), Vec::new());
    let offset = [100, 300, 10].map(|size| lookup.add_table(&mut array, &vec![1.0; size]));
    assert_eq!((offset, array.len(), lookup.table_count()), ([0, 256, 768], 1024, 3));
    // The floats of a table, then zeros to the end of its chunk.
    assert_eq!((array[99], array[100], array[256], array[555], array[556]), (1.0, 0.0, 1.0, 1.0, 0.0));
}

#[test]
fn a_new_table_takes_the_first_gap_it_fits() {
    let (mut lookup, mut array) = (LookupTableSet::new(), Vec::new());
    let offset = [100, 300, 10].map(|size| lookup.add_table(&mut array, &vec![1.0; size]));
    lookup.remove_table(&mut Some(offset[1]));
    assert_eq!(lookup.add_table(&mut array, &[2.0; 200]), 256);
    // 256 floats are left in the gap: too few for 400.
    assert_eq!(lookup.add_table(&mut array, &[3.0; 400]), 1024);
    assert_eq!((array.len(), array[256], array[1024]), (1536, 2.0, 3.0));
}

#[test]
fn removing_a_table_forgets_its_offset() {
    let (mut lookup, mut array) = (LookupTableSet::new(), Vec::new());
    let mut offset = Some(lookup.add_table(&mut array, &[1.0; 8]));
    lookup.remove_table(&mut offset);
    assert_eq!((offset, lookup.table_count()), (None, 0));
    // Removing what is not there is nothing.
    lookup.remove_table(&mut offset);
    assert_eq!(lookup.add_table(&mut array, &[1.0; 8]), 0);
}

#[test]
fn a_change_is_remembered_until_the_device_has_the_array() {
    let (mut lookup, mut array) = (LookupTableSet::new(), Vec::new());
    assert!(!lookup.need_update());
    lookup.add_table(&mut array, &[1.0; 8]);
    assert!(lookup.need_update());
    lookup.device_update();
    assert!(!lookup.need_update());
}
