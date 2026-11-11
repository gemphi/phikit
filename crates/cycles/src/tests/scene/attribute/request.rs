// SPDX-License-Identifier: Apache-2.0
use super::std;
use crate::scene::*;

#[test]
fn an_attribute_is_asked_for_once() {
    let mut set = AttributeRequestSet::new();
    assert!(set.is_empty());
    set.add_name("weight");
    set.add_name("weight");
    set.add_standard(std("UV"));
    set.add_standard(std("UV"));
    assert_eq!(set.len(), 2);
    assert!(set.find_name("weight") && set.find_standard(std("UV")) && !set.find_standard(std("POSITION")) && !set.find_name("uv"));
    set.clear();
    assert!(set.is_empty());
}

#[test]
fn a_name_that_is_a_standard_s_asks_for_the_standard() {
    let mut set = AttributeRequestSet::new();
    set.add_named("uv");
    set.add_named("weight");
    set.add_named("");
    assert_eq!(set.len(), 2);
    assert!(set.find_standard(std("UV")) && set.find_name("weight"));
    assert_eq!(set.request[0].name, "");
}

#[test]
fn sets_join_without_asking_twice() {
    let (mut one, mut other) = (AttributeRequestSet::new(), AttributeRequestSet::new());
    one.add_name("weight");
    one.add_standard(std("UV"));
    other.add_standard(std("UV"));
    other.add_standard(std("GENERATED"));
    one.add_set(&other);
    assert_eq!(one.len(), 3);
}

#[test]
fn two_sets_differ_when_one_asks_for_what_the_other_does_not() {
    let (mut one, mut other) = (AttributeRequestSet::new(), AttributeRequestSet::new());
    one.add_name("a");
    one.add_standard(std("UV"));
    other.add_standard(std("UV"));
    assert!(one.modified(&other));
    other.add_name("a");
    // The order they were asked in does not matter.
    assert!(!one.modified(&other));
    other.add_name("b");
    assert!(one.modified(&other));
}
