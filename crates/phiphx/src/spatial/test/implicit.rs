use super::*;

/// One blob is a ball: every vertex sits at the radius where the field
/// crosses the threshold, and every triangle faces out.
#[test]
fn one_blob_is_a_ball() {
    let (v, i, n) = polygonise(&[Blob { at: [0.0; 3], radius: 0.7 }], 1.0, 1.6, 24);
    assert!(!v.is_empty() && i.len() % 3 == 0);
    // field = r²/d² = 1 at d = r, so the surface is the sphere of r.
    for &(x, y, z) in &v {
        let d = (x * x + y * y + z * z).sqrt();
        assert!((d - 0.7).abs() < 0.08, "{d} is about the radius");
    }
    for (k, &(nx, ny, nz)) in n.iter().enumerate() {
        let (x, y, z) = v[k];
        let dot = nx * x + ny * y + nz * z;
        assert!(dot > 0.0, "the normal points out of the blob");
    }
}

/// Two blobs close together are one surface, not two — which is the
/// whole point of a metaball.
#[test]
fn two_near_blobs_fuse() {
    let near = [Blob { at: [-0.35, 0.0, 0.0], radius: 0.6 }, Blob { at: [0.35, 0.0, 0.0], radius: 0.6 }];
    // Midway between them the field is the sum of both, well over the
    // threshold, so there is solid there and the two have merged.
    assert!(super::field([0.0; 3], &near) > 1.0, "the waist is filled in");
    let far = [Blob { at: [-3.0, 0.0, 0.0], radius: 0.6 }, Blob { at: [3.0, 0.0, 0.0], radius: 0.6 }];
    assert!(super::field([0.0; 3], &far) < 1.0, "far apart they are two balls");
}
