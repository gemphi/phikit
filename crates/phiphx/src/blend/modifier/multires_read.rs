//! The saved multires displacement: the mesh's `CD_MDISPS` corner layer
//! (`MDisps` in `DNA_meshdata_types.h`), one grid of tangent-space offsets
//! per face corner, as `CustomData_blend_write` / `write_mdisps` save it —
//! the `MDisps` array as one block, each grid's `disps` as a float3 block
//! of its own.

use crate::blend::file::types::*;

/// `CD_MDISPS` (`DNA_customdata_types.h`).
const CD_MDISPS: i32 = 19;

/// One `MDisps.disps` per mesh corner, in corner order; `None` where the
/// grid has no data (Blender's `disps == nullptr`, which reads as zero).
pub(super) type BMDisps = Vec<Option<Vec<[f32; 3]>>>;

/// The `CD_MDISPS` layer of the mesh `me`, if it has one. Read inside the
/// mesh's pointer scope, since the grids are that ID's data.
pub(super) fn read_mdisps(f: &BlendFile, me: &BlendBlock, dna: &DnaType) -> Option<BMDisps> {
    f.scoped(me, || read_layer(f, me, dna))
}

fn read_layer(f: &BlendFile, me: &BlendBlock, dna: &DnaType) -> Option<BMDisps> {
    let ps = f.header.pointer_size as usize;
    let le = f.header.little_endian;
    let cd = ["corner_data", "ldata"]
        .iter()
        .find_map(|n| BlendFile::struct_field_offset_exact("Mesh", n, dna, ps))?;
    let bytes = f.customdata_layer(me, cd, dna, &|t, _| t == CD_MDISPS)?;
    let size = BlendFile::struct_size("MDisps", dna, ps)?;
    let tot = BlendFile::struct_field_offset_exact("MDisps", "totdisp", dna, ps)?;
    // `float (*disps)[3]`: matched by a substring, as its DNA name carries
    // the pointer-to-array spelling ("totdisp" does not contain it).
    let disps = BlendFile::struct_field_offset("MDisps", "disps", dna, ps)?;
    if size == 0 {
        return None;
    }
    let grids = bytes
        .chunks_exact(size)
        .map(|rec| {
            let n = f.read_i32(rec.get(tot..tot + 4)?).max(0) as usize;
            let addr = BlendFile::read_ptr(rec, disps, ps, le);
            let block = f.block_at(addr)?;
            let grid: Vec<[f32; 3]> = block
                .data
                .chunks_exact(12)
                .take(n)
                .map(|c| [f.read_f32(&c[0..4]), f.read_f32(&c[4..8]), f.read_f32(&c[8..12])])
                .collect();
            (!grid.is_empty()).then_some(grid)
        })
        .collect();
    Some(grids)
}
