use super::*;

#[test]
fn an_uncompressed_file_reads_point_by_point() {
    let mut b = b"BPHYSICS".to_vec();
    b.extend_from_slice(&TYPE_CLOTH.to_le_bytes());
    b.extend_from_slice(&2u32.to_le_bytes());
    b.extend_from_slice(&(((1u32 << DATA_LOCATION) | (1u32 << DATA_VELOCITY)) as u32).to_le_bytes());
    for p in 0..2 {
        for k in 0..3 {
            b.extend_from_slice(&((p * 3 + k) as f32).to_le_bytes());
        }
        for _ in 0..3 {
            b.extend_from_slice(&0.5f32.to_le_bytes());
        }
    }
    let fr = parse(&b, 7).expect("parses");
    assert_eq!((fr.frame, fr.positions), (7, vec![[0.0, 1.0, 2.0], [3.0, 4.0, 5.0]]));
    assert_eq!(fr.velocities[1], [0.5; 3]);
}

#[test]
fn unfilter_undoes_the_transpose_and_delta() {
    // Two items of three bytes: transposed by byte, delta-coded.
    let plain = [1u8, 10, 100, 2, 20, 200];
    let mut filtered = Vec::new();
    for ib in 0..3 {
        let mut prev = 0u8;
        for it in 0..2 {
            let v = plain[it * 3 + ib];
            filtered.push(v.wrapping_sub(prev));
            prev = v;
        }
    }
    assert_eq!(unfilter(&filtered, 2, 3), plain);
}
