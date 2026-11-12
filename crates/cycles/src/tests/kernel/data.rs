// SPDX-License-Identifier: Apache-2.0
use crate::kernel::*;

#[test]
fn the_data_is_cycles_structs_in_order_each_on_a_16_byte_boundary() {
    let name: Vec<&str> = KernelData::struct_list().iter().map(|block| block.name).collect();
    let expect = ["KernelData", "KernelCamera", "KernelBake", "KernelTables", "KernelSceneTime", "KernelBackground", "KernelBVH", "KernelFilm", "KernelIntegrator", "KernelImage", "KernelSVMUsage"];
    assert_eq!(name, expect);
    let mut next = 0;
    for block in KernelData::struct_list() {
        assert_eq!((block.word, block.word % 4, block.word_count % 4), (next, 0, 0), "{}", block.name);
        next += block.word_count;
    }
    assert_eq!(KernelData::new().word().len(), next);
}

#[test]
fn every_member_is_inside_its_struct_and_none_overlaps() {
    let mut taken = vec![false; KernelData::new().word().len()];
    for field in KernelData::field_list() {
        let block = KernelData::struct_list().iter().find(|block| block.parent == field.parent).expect("its struct");
        assert!(field.word >= block.word && field.word + field.word_count <= block.word + block.word_count, "{}.{}", field.parent, field.name);
        for word in &mut taken[field.word..field.word + field.word_count] {
            assert!(!*word, "{}.{} overlaps", field.parent, field.name);
            *word = true;
        }
    }
}

#[test]
fn a_member_is_found_by_the_name_the_kernel_reads_it_by() {
    let pass = KernelData::field("film", "pass_combined").expect("in data/kernel.tsv");
    assert_eq!((pass.kind, pass.word_count, pass.specialize), (FieldKind::Int, 1, true));
    // A float4 starts on 16 bytes, as on every device.
    let sun = KernelData::field("background", "sun").expect("in data/kernel.tsv");
    assert_eq!((sun.kind, sun.word_count, sun.word % 4), (FieldKind::Float4, 4, 0));
    // The seed changes every frame, so it never becomes a constant.
    assert!(!KernelData::field("integrator", "seed").expect("in data/kernel.tsv").specialize);
}

#[test]
fn a_value_written_is_the_value_read_and_its_neighbours_stay() {
    let mut data = KernelData::new();
    data.set_int("integrator", "max_bounce", -3);
    data.set_float("film", "exposure", 1.5);
    data.set_uint("data", "kernel_features", u32::MAX);
    data.set_float2("integrator", "pixel_jitter", [0.25, -0.5]);
    data.set_float4("background", "sun", [1.0, 2.0, 3.0, 4.0]);
    assert_eq!((data.int("integrator", "max_bounce"), data.float("film", "exposure")), (-3, 1.5));
    assert_eq!((data.uint("data", "kernel_features"), data.float2("integrator", "pixel_jitter")), (u32::MAX, [0.25, -0.5]));
    assert_eq!(data.at(KernelData::field("background", "sun").expect("in the table")), [1.0f32, 2.0, 3.0, 4.0].map(f32::to_bits));
    assert_eq!((data.int("integrator", "min_bounce"), data.int("integrator", "max_diffuse_bounce")), (0, 0));
}

#[test]
fn the_shader_node_counts_are_one_word_a_node_type() {
    let usage = KernelData::field_list().iter().filter(|field| field.parent == "svm_usage");
    assert!(usage.clone().count() > 100);
    assert!(usage.clone().all(|field| field.kind == FieldKind::Int));
    assert!(KernelData::field("svm_usage", "NODE_CLOSURE_BSDF").is_some());
}

#[test]
fn a_matrix_and_a_vector_come_back_as_they_were_set() {
    use crate::util::{ProjectionTransform, Transform};
    let mut data = KernelData::new();
    let place = Transform::translate(1.0, 2.0, 3.0);
    let view = ProjectionTransform::perspective(0.8, 0.1, 100.0);
    data.set_transform("cam", "cameratoworld", &place);
    data.set_projection("cam", "rastertocamera", &view);
    data.set_float4("cam", "dx", [1.0, 2.0, 3.0, 4.0]);
    assert_eq!((data.transform("cam", "cameratoworld"), data.projection("cam", "rastertocamera")), (place, view));
    assert_eq!(data.float4("cam", "dx"), [1.0, 2.0, 3.0, 4.0]);
}
