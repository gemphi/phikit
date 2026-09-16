//! Transform helpers: node transforms, TRS conversion, point transformation.

use super::importer::GltfImporter;

impl GltfImporter {
    /// Extract transform matrix from node
    pub(crate) fn extract_transform(node: &gltf::Node) -> [f32; 16] {
        use gltf::scene::Transform;

        match node.transform() {
            Transform::Matrix { matrix } => [
                matrix[0][0],
                matrix[0][1],
                matrix[0][2],
                matrix[0][3],
                matrix[1][0],
                matrix[1][1],
                matrix[1][2],
                matrix[1][3],
                matrix[2][0],
                matrix[2][1],
                matrix[2][2],
                matrix[2][3],
                matrix[3][0],
                matrix[3][1],
                matrix[3][2],
                matrix[3][3],
            ],
            Transform::Decomposed {
                translation,
                rotation,
                scale,
            } => {
                // Convert TRS to matrix
                Self::trs_to_matrix(translation, rotation, scale)
            }
        }
    }

    /// Convert translation-rotation-scale to matrix
    pub(crate) fn trs_to_matrix(t: [f32; 3], r: [f32; 4], s: [f32; 3]) -> [f32; 16] {
        // Quaternion to rotation matrix
        let x = r[0];
        let y = r[1];
        let z = r[2];
        let w = r[3];

        let x2 = x + x;
        let y2 = y + y;
        let z2 = z + z;

        let xx = x * x2;
        let xy = x * y2;
        let xz = x * z2;
        let yy = y * y2;
        let yz = y * z2;
        let zz = z * z2;
        let wx = w * x2;
        let wy = w * y2;
        let wz = w * z2;

        let sx = s[0];
        let sy = s[1];
        let sz = s[2];

        [
            (1.0 - (yy + zz)) * sx,
            (xy - wz) * sy,
            (xz + wy) * sz,
            0.0,
            (xy + wz) * sx,
            (1.0 - (xx + zz)) * sy,
            (yz - wx) * sz,
            0.0,
            (xz - wy) * sx,
            (yz + wx) * sy,
            (1.0 - (xx + yy)) * sz,
            0.0,
            t[0],
            t[1],
            t[2],
            1.0,
        ]
    }
}

/// Transform a 3D point by a 4x4 matrix
pub(crate) fn transform_point(point: &[f32; 3], matrix: &[f32; 16]) -> [f32; 3] {
    let x = point[0];
    let y = point[1];
    let z = point[2];

    let tx = matrix[0] * x + matrix[1] * y + matrix[2] * z + matrix[3];
    let ty = matrix[4] * x + matrix[5] * y + matrix[6] * z + matrix[7];
    let tz = matrix[8] * x + matrix[9] * y + matrix[10] * z + matrix[11];
    let w = matrix[12] * x + matrix[13] * y + matrix[14] * z + matrix[15];

    if w != 0.0 && w != 1.0 {
        let inv_w = 1.0 / w;
        [tx * inv_w, ty * inv_w, tz * inv_w]
    } else {
        [tx, ty, tz]
    }
}
