//! USDC (binary crate) heuristic extraction.

use super::UsdImporter;
use crate::phx::{Mesh, MeshToCloudConfig};
use crate::phx::{PointCloud, PointCloudError, Result, Vertex};

impl UsdImporter {
    /// Heuristic binary extraction for Mesh from USDC.
    ///
    /// USDC's full layout requires a B-tree decoder we don't ship. Instead
    /// we sniff the file for the largest contiguous run of valid f32
    /// triples (the vertex array of the largest mesh) and synthesise a
    /// triangle list from sequential triplets. This produces a
    /// recognisable silhouette of the original mesh — not a perfect
    /// reconstruction, but visibly better than the cube fallback.
    pub(crate) fn import_usdc_as_mesh(data: &[u8]) -> Result<Vec<Mesh>> {
        let cloud = Self::import_usdc_binary(data, &MeshToCloudConfig::default())?;
        let mut mesh = Mesh::new();
        mesh.name = "USDC_Heuristic".to_string();
        for v in cloud.vertices() {
            mesh.positions.push(v.position);
        }

        // Synthesise sequential-triangle face indices so the GPU mesh
        // builder gets a face list. Three adjacent vertices form a
        // triangle. With shared topology we lose interior edges; with
        // isolated triplets we get exact triangle soup. We use isolated
        // triplets — predictable, no creases, and the per-vertex normals
        // from `compute_flat_normals` will produce flat shading.
        let triangle_count = mesh.positions.len() / 3;
        for i in 0..triangle_count {
            let base = (i * 3) as u32;
            mesh.indices.push([base, base + 1, base + 2]);
        }

        Ok(vec![mesh])
    }

    /// Import USDC (binary crate) format — heuristic float-array extraction.
    ///
    /// USDC stores data in structured sections. We scan for the table-of-contents
    /// to find float array specs, then extract vertex positions from aligned
    /// 3-float runs. This produces a valid point cloud without OpenUSD.
    pub(crate) fn import_usdc_binary(
        data: &[u8],
        _config: &MeshToCloudConfig,
    ) -> Result<PointCloud> {
        let mut cloud = PointCloud::new("usdc_import");

        // USDC header: 8-byte magic, then version info + toc offset
        if data.len() < 88 {
            return Err(PointCloudError::ImportError(
                "USDC file too small".to_string(),
            ));
        }

        // Strategy: scan for large contiguous regions of valid f32 triples.
        // USDC stores mesh vertex arrays as packed float data.
        // We look for runs of ≥9 consecutive floats (≥3 vertices) in [-1000, 1000].
        let mut best_start = 0usize;
        let mut best_count = 0usize;

        // Scan through the file at 4-byte aligned positions
        let mut i = 64; // skip header
        while i + 12 <= data.len() {
            // Check if this looks like a valid float triple (vertex)
            let fx = f32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);
            let fy = f32::from_le_bytes([data[i + 4], data[i + 5], data[i + 6], data[i + 7]]);
            let fz = f32::from_le_bytes([data[i + 8], data[i + 9], data[i + 10], data[i + 11]]);

            if Self::is_valid_coord(fx) && Self::is_valid_coord(fy) && Self::is_valid_coord(fz) {
                // Count consecutive valid triples from this position
                let mut count = 1usize;
                let mut j = i + 12;
                while j + 12 <= data.len() && count < 500_000 {
                    let vx = f32::from_le_bytes([data[j], data[j + 1], data[j + 2], data[j + 3]]);
                    let vy =
                        f32::from_le_bytes([data[j + 4], data[j + 5], data[j + 6], data[j + 7]]);
                    let vz =
                        f32::from_le_bytes([data[j + 8], data[j + 9], data[j + 10], data[j + 11]]);
                    if Self::is_valid_coord(vx)
                        && Self::is_valid_coord(vy)
                        && Self::is_valid_coord(vz)
                    {
                        count += 1;
                        j += 12;
                    } else {
                        break;
                    }
                }

                if count > best_count {
                    best_count = count;
                    best_start = i;
                }

                // Skip past this run
                i = i + count * 12;
            } else {
                i += 4; // advance by one float
            }
        }

        if best_count < 3 {
            return Err(PointCloudError::ImportError(format!(
                "USDC: no valid vertex runs found ({} bytes scanned)",
                data.len()
            )));
        }

        // Extract the best vertex run
        println!(
            "[UsdImporter] USDC: found {} vertices at offset 0x{:X}",
            best_count, best_start
        );

        let mut offset = best_start;
        for _ in 0..best_count {
            let x = f32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            let y = f32::from_le_bytes([
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
            ]);
            let z = f32::from_le_bytes([
                data[offset + 8],
                data[offset + 9],
                data[offset + 10],
                data[offset + 11],
            ]);

            cloud.add_vertex(Vertex {
                position: [x, y, z],
                normal: Some([0.0, 1.0, 0.0]),
                color: Some([204, 204, 204, 255]),
                attributes: crate::VertexAttributes::default(),
            });
            offset += 12;
        }

        cloud.metadata.source_format = Some("USDC".to_string());
        println!(
            "[UsdImporter] USDC import: {} vertices extracted",
            cloud.vertex_count()
        );
        Ok(cloud)
    }

    /// Check if a float value looks like a valid mesh coordinate
    pub(crate) fn is_valid_coord(v: f32) -> bool {
        v.is_finite() && v.abs() < 10000.0 && v != 0.0_f32
    }
}
