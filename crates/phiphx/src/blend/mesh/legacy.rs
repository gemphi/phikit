//! Pre-4.x mesh arrays: `mvert`, `mpoly` / `mloop`, `poly_offsets`.

use crate::blend::file::types::*;

impl BlendFile {
    /// Follow the `*mvert` pointer and decode the vertex positions.
    pub(crate) fn read_mvert(
        &self,
        block: &BlendBlock,
        mvert_ptr: Option<usize>,
        vert_count: usize,
        ptr_size: usize,
    ) -> Vec<[f32; 3]> {
        let mut vertices = Vec::new();
        let Some(ptr_off) = mvert_ptr else {
            return vertices;
        };
        if ptr_off + ptr_size > block.data.len() || vert_count == 0 || vert_count >= 10_000_000 {
            return vertices;
        }
        let addr = self.read_block_ptr(block, ptr_off, ptr_size);
        if addr == 0 {
            return vertices;
        }
        let Some(vb) = self.block_at(addr) else {
            return vertices;
        };
        // Stride from the block itself: an `MVert` struct array records its
        // element count in `BHead.nr`; a raw `float[3]` array (3.5+
        // `vert_positions`) has nr = 1 and is exactly `vert_count × 12`.
        let best = if vb.count > 1 {
            vb.data.len() / vb.count as usize
        } else {
            vb.data.len() / vert_count
        };
        if best < 12 {
            return vertices;
        }
        for i in 0..vert_count {
            let base = i * best;
            if base + 12 > vb.data.len() {
                break;
            }
            let x = self.read_f32(&vb.data[base..base + 4]);
            let y = self.read_f32(&vb.data[base + 4..base + 8]);
            let z = self.read_f32(&vb.data[base + 8..base + 12]);
            vertices.push([x, y, z]);
        }
        vertices
    }

    /// Follow `*mpoly`/`*mloop` pointers and decode polygon vertex indices.
    pub(crate) fn read_faces(
        &self,
        block: &BlendBlock,
        mpoly_ptr: Option<usize>,
        mloop_ptr: Option<usize>,
        poly_count: usize,
        loop_count: usize,
    ) -> Vec<Vec<u32>> {
        let mut faces = Vec::new();
        let (Some(poly_off), Some(loop_off)) = (mpoly_ptr, mloop_ptr) else {
            return faces;
        };
        if poly_count == 0 || poly_count >= 10_000_000 {
            return faces;
        }
        let ps = self.header.pointer_size as usize;
        let poly_addr = self.read_block_ptr(block, poly_off, ps);
        let loop_addr = self.read_block_ptr(block, loop_off, ps);
        let (Some(pb), Some(lb)) = (
            self.block_at(poly_addr),
            self.block_at(loop_addr),
        ) else {
            return faces;
        };
        let plen = pb.data.len();
        let llen = lb.data.len();
        let mpoly_size = if poly_count > 0 {
            plen / poly_count
        } else {
            12
        };
        let mloop_size = if loop_count > 0 { llen / loop_count } else { 8 };
        // A zero stride would make every `pbase`/`lbase` collapse to 0 and spin
        // without progress; bail rather than emit garbage.
        if mpoly_size == 0 || mloop_size == 0 {
            return faces;
        }
        for p in 0..poly_count {
            let pbase = p * mpoly_size;
            // Overflow-safe end check: compute the exclusive end with
            // `checked_add` so a corrupt/huge stride can never wrap past the
            // guard and slice out of range (observed panic at pbase..pbase+4).
            match pbase.checked_add(8) {
                Some(end) if end <= plen => {}
                _ => break,
            }
            let loop_start = self.read_i32(&pb.data[pbase..pbase + 4]) as usize;
            let loop_tot = self.read_i32(&pb.data[pbase + 4..pbase + 8]) as usize;
            if !(3..100).contains(&loop_tot) {
                continue;
            }
            let mut face = Vec::with_capacity(loop_tot);
            for l in 0..loop_tot {
                // Overflow-safe: `loop_start + l` and the `* mloop_size` product
                // both come from file data and can be enormous.
                let Some(base) = loop_start
                    .checked_add(l)
                    .and_then(|i| i.checked_mul(mloop_size))
                else {
                    break;
                };
                match base.checked_add(4) {
                    Some(end) if end <= llen => {}
                    _ => break,
                }
                face.push(self.read_i32(&lb.data[base..base + 4]) as u32);
            }
            if face.len() >= 3 {
                faces.push(face);
            }
        }
        faces
    }

    /// Read face offsets from the `*poly_offsets` / `*face_offset_indices` pointer.
    /// In Blender 3.5+, this is an array of (poly_count + 1) ints giving the
    /// start index into corner_verts for each face.
    pub(crate) fn read_poly_offsets_ptr(
        &self,
        block: &BlendBlock,
        ptr_field: Option<usize>,
        poly_count: usize,
        ptr_size: usize,
    ) -> Vec<usize> {
        let Some(off) = ptr_field else {
            return Vec::new();
        };
        if off + ptr_size > block.data.len() {
            return Vec::new();
        }
        let addr = self.read_block_ptr(block, off, ptr_size);
        if addr == 0 {
            return Vec::new();
        }
        let Some(data_block) = self.block_at(addr) else {
            return Vec::new();
        };
        let expected = (poly_count + 1) * 4;
        if data_block.data.len() < expected {
            return Vec::new();
        }
        let mut offsets = Vec::with_capacity(poly_count + 1);
        for i in 0..=poly_count {
            let o = i * 4;
            offsets.push(self.read_i32(&data_block.data[o..o + 4]) as usize);
        }
        offsets
    }
}
