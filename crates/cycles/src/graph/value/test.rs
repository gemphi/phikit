// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::graph::*;

#[test]
fn a_default_is_read_from_the_numbers_of_the_table() {
    assert_eq!(Value::default_of(SocketKind::Boolean, "1"), Value::Boolean(true));
    assert_eq!(Value::default_of(SocketKind::Int, "16777216"), Value::Int(1 << 24));
    assert_eq!(Value::default_of(SocketKind::Float, "0.05"), Value::Float(0.05));
    assert_eq!(Value::default_of(SocketKind::Enum, "4"), Value::Enum(4));
    assert_eq!(Value::default_of(SocketKind::Color, "0.8 0.8 0.8"), Value::Float3([0.8; 3]));
    assert_eq!(Value::default_of(SocketKind::Uint, "127"), Value::Uint(127));
}

#[test]
fn no_number_is_the_kind_s_zero() {
    assert_eq!(Value::default_of(SocketKind::Float, ""), Value::Float(0.0));
    assert_eq!(Value::default_of(SocketKind::String, ""), Value::String(String::new()));
    assert_eq!(Value::default_of(SocketKind::FloatArray, ""), Value::FloatArray(Vec::new()));
    assert_eq!(Value::default_of(SocketKind::Closure, ""), Value::None);
}

#[test]
fn a_value_is_read_as_the_type_it_holds_and_no_other() {
    assert_eq!(Value::Enum(3).as_int(), Some(3));
    assert_eq!(Value::Int(3).as_float(), None);
    assert_eq!(Value::Float(0.5).as_float(), Some(0.5));
    assert_eq!(Value::Boolean(true).as_boolean(), Some(true));
    assert_eq!(Value::FloatArray(vec![1.0, 2.0]).as_float_array(), Some(&[1.0, 2.0][..]));
}

#[test]
fn a_64_bit_default_of_minus_one_is_every_bit_set() {
    // `LIGHT_LINK_MASK_ALL`, `~uint64_t(0)`, is written -1 in the table.
    assert_eq!(Value::default_of(SocketKind::Uint64, "-1"), Value::Uint64(u64::MAX));
    assert_eq!(Value::default_of(SocketKind::Uint64, "5"), Value::Uint64(5));
    assert_eq!(Value::default_of(SocketKind::Uint64, ""), Value::Uint64(0));
}

#[test]
fn every_kind_of_value_has_its_reader() {
    assert_eq!((Value::Uint(3).as_uint(), Value::Int(3).as_uint()), (Some(3), None));
    assert_eq!((Value::Uint64(1 << 40).as_uint64(), Value::Uint(1).as_uint64()), (Some(1 << 40), None));
    assert_eq!((Value::Float2([1.0, 2.0]).as_float2(), Value::Float3([1.0, 2.0, 3.0]).as_float3()), (Some([1.0, 2.0]), Some([1.0, 2.0, 3.0])));
    assert_eq!((Value::String("a".to_string()).as_string(), Value::Float(1.0).as_string()), (Some("a"), None));
    assert_eq!(Value::IntArray(vec![1, 2]).as_int_array(), Some(&[1, 2][..]));
    assert_eq!(Value::BooleanArray(vec![true]).as_boolean_array(), Some(&[true][..]));
}
