// SPDX-License-Identifier: Apache-2.0
use super::*;
use crate::util::murmur_hash3;

fn named(name: &str, asset: &str) -> Object {
    let mut object = Object::new(0);
    object.node.name = name.to_string();
    object.node.set("asset_name", Value::String(asset.to_string()));
    object
}

#[test]
fn the_manifest_names_each_object_once_with_its_hash_in_hex() {
    let object = [named("Cube", "Set"), named("Lamp", "Set"), named("Cube", "Prop")];
    let hex = |name: &str| format!("{:08x}", murmur_hash3(name.as_bytes(), 0));
    assert_eq!(ObjectManager::cryptomatte_object(&object), format!("{{\"Cube\":\"{}\",\"Lamp\":\"{}\"}}", hex("Cube"), hex("Lamp")));
    assert_eq!(ObjectManager::cryptomatte_asset(&object), format!("{{\"Set\":\"{}\",\"Prop\":\"{}\"}}", hex("Set"), hex("Prop")));
}

#[test]
fn a_hash_is_eight_hex_digits_even_when_it_starts_with_zeros() {
    // The empty name hashes to zero.
    assert_eq!(ObjectManager::cryptomatte_object(&[named("", "")]), "{\"\":\"00000000\"}");
}

#[test]
fn no_objects_give_a_closing_brace_alone_as_in_cycles() {
    assert_eq!(ObjectManager::cryptomatte_object(&[]), "}");
}
