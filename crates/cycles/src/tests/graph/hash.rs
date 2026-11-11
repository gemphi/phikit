// SPDX-License-Identifier: Apache-2.0
use crate::graph::*;
use crate::util::{Md5Hash, Transform};

fn digest(node: &Node) -> String {
    let mut md5 = Md5Hash::new();
    node.hash(&mut md5);
    md5.hex()
}

fn math() -> Node {
    Node::new("math").expect("in data/node.tsv")
}

#[test]
fn nodes_with_the_same_type_and_values_have_one_digest() {
    assert_eq!(digest(&math()), digest(&math()));
    assert_ne!(digest(&math()), digest(&Node::new("vector_math").expect("in data/node.tsv")));
}

#[test]
fn a_changed_value_of_any_kind_changes_the_digest() {
    let changed = |name: &str, value: Value| {
        let mut node = math();
        assert!(node.set(name, value));
        digest(&node)
    };
    let plain = digest(&math());
    assert_ne!(plain, changed("value1", Value::Float(0.125)));
    assert_ne!(plain, changed("use_clamp", Value::Boolean(true)));
    assert_ne!(plain, changed("math_type", Value::Enum(3)));
    assert_ne!(changed("value1", Value::Float(0.125)), changed("value2", Value::Float(0.125)));
}

#[test]
fn text_and_transforms_are_taken_in_too() {
    let mut node = Node::new("object").expect("in data/node.tsv");
    let plain = digest(&node);
    assert!(node.set("asset_name", Value::String("Set".to_string())));
    let named = digest(&node);
    assert!(node.set("tfm", Value::Transform(Transform::translate(1.0, 0.0, 0.0))));
    assert!(plain != named && named != digest(&node));
}
