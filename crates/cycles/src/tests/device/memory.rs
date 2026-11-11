// SPDX-License-Identifier: Apache-2.0
//! Tests of `device/memory.rs` and the blocks of `device/cpu/memory.rs`.
use crate::device::{CpuDevice, Device, DeviceError, DevicePtr, DeviceVector, MemoryType};

#[test]
fn alloc_is_zeroed_and_counted() {
    let device = CpuDevice::new();
    let ptr = device.mem_alloc(13, MemoryType::ReadWrite).unwrap();
    assert!(!ptr.is_null());
    assert_eq!(device.mem_size(ptr), Some(13));
    assert_eq!(device.mem_used(), 13);
    let mut out = [1u8; 13];
    device.mem_copy_from(ptr, 0, &mut out).unwrap();
    assert_eq!(out, [0u8; 13]);
    device.mem_free(ptr);
    assert_eq!(device.mem_used(), 0);
    assert_eq!(device.mem_size(ptr), None);
}

#[test]
fn copy_round_trip_at_an_offset() {
    let device = CpuDevice::new();
    let ptr = device.mem_alloc(8, MemoryType::ReadWrite).unwrap();
    device.mem_copy_to(ptr, 3, &[7, 8, 9]).unwrap();
    let mut out = [0u8; 8];
    device.mem_copy_from(ptr, 0, &mut out).unwrap();
    assert_eq!(out, [0, 0, 0, 7, 8, 9, 0, 0]);
}

#[test]
fn zero_clears_a_block() {
    let device = CpuDevice::new();
    let ptr = device.mem_alloc(4, MemoryType::ReadWrite).unwrap();
    device.mem_copy_to(ptr, 0, &[1, 2, 3, 4]).unwrap();
    device.mem_zero(ptr).unwrap();
    let mut out = [9u8; 4];
    device.mem_copy_from(ptr, 0, &mut out).unwrap();
    assert_eq!(out, [0; 4]);
}

#[test]
fn a_copy_past_the_end_is_an_error() {
    let device = CpuDevice::new();
    let ptr = device.mem_alloc(4, MemoryType::ReadWrite).unwrap();
    assert!(matches!(device.mem_copy_to(ptr, 2, &[0; 3]), Err(DeviceError::OutOfRange { size: 4, .. })));
    assert!(matches!(device.mem_copy_to(ptr, usize::MAX, &[0; 3]), Err(DeviceError::OutOfRange { .. })));
    assert!(matches!(device.mem_copy_from(ptr, 0, &mut [0u8; 5]), Err(DeviceError::OutOfRange { .. })));
}

#[test]
fn a_freed_or_unknown_pointer_is_an_error() {
    let device = CpuDevice::new();
    let ptr = device.mem_alloc(4, MemoryType::ReadWrite).unwrap();
    device.mem_free(ptr);
    assert_eq!(device.mem_zero(ptr), Err(DeviceError::BadPointer(ptr)));
    assert_eq!(device.mem_zero(DevicePtr::NULL), Err(DeviceError::BadPointer(DevicePtr::NULL)));
    // Freeing it again is ignored.
    device.mem_free(ptr);
    assert_eq!(device.mem_used(), 0);
}

#[test]
fn a_vector_goes_to_the_device_and_back() {
    let device = CpuDevice::new();
    let mut vector = DeviceVector::<f32>::new("buffer", MemoryType::ReadWrite);
    vector.resize(5);
    vector.host_mut().copy_from_slice(&[1.0, 2.0, 3.0, 4.0, 5.0]);
    vector.copy_to_device(&device).unwrap();
    assert_eq!(vector.device_len(), 5);
    assert_eq!(device.mem_size(vector.ptr()), Some(20));
    vector.host_mut().fill(0.0);
    vector.copy_from_device(&device).unwrap();
    assert_eq!(vector.host(), &[1.0, 2.0, 3.0, 4.0, 5.0]);
    assert_eq!(vector.memory_size(), 20);
}

#[test]
fn a_vector_resized_gets_a_new_block() {
    let device = CpuDevice::new();
    let mut vector = DeviceVector::<u32>::new("index", MemoryType::ReadOnly);
    vector.resize(2);
    vector.copy_to_device(&device).unwrap();
    let first = vector.ptr();
    vector.resize(6);
    vector.copy_to_device(&device).unwrap();
    assert_ne!(vector.ptr(), first);
    assert_eq!(device.mem_size(first), None, "the old block was freed");
    assert_eq!(device.mem_used(), 24);
}

#[test]
fn zero_to_device_clears_host_and_block() {
    let device = CpuDevice::new();
    let mut vector = DeviceVector::<u32>::new("count", MemoryType::ReadWrite);
    vector.resize(3);
    vector.host_mut().fill(9);
    vector.copy_to_device(&device).unwrap();
    vector.zero_to_device(&device).unwrap();
    vector.host_mut().fill(5);
    vector.copy_from_device(&device).unwrap();
    assert_eq!(vector.host(), &[0, 0, 0]);
}

#[test]
fn free_releases_the_block_and_keeps_the_host_copy() {
    let device = CpuDevice::new();
    let mut vector = DeviceVector::<u8>::new("x", MemoryType::ReadWrite);
    vector.resize(10);
    vector.copy_to_device(&device).unwrap();
    vector.free(&device);
    assert!(vector.ptr().is_null());
    assert_eq!(vector.len(), 10);
    assert_eq!(device.mem_used(), 0);
}

#[test]
fn a_block_reads_as_typed_slices() {
    let device = CpuDevice::new();
    let ptr = device.mem_alloc(12, MemoryType::ReadWrite).unwrap();
    device.with_memory(|memory| {
        memory.slice_mut::<f32>(ptr).unwrap().copy_from_slice(&[1.0, 2.0, 3.0]);
        assert_eq!(memory.slice::<f32>(ptr).unwrap(), &[1.0, 2.0, 3.0]);
        assert_eq!(memory.slice::<u64>(ptr).unwrap().len(), 1);
    });
}
