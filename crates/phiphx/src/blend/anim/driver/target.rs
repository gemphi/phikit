//! One `DriverTarget` out of the file: the ID it points at, its RNA path,
//! its space flag, and — for a Transform Channel variable — which channel
//! and in which rotation mode (`DNA_anim_types.h`).
//!
//! `DriverVar.targets` is a fixed array of `MAX_DRIVER_TARGETS`, so a
//! second target is one `DriverTarget` further along; the stride comes from
//! the DNA rather than a constant, since the struct has grown.

use super::space::BDriverSpace;
use crate::blend::file::types::*;

/// One target as the file spells it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BDriverTarget {
    /// `DriverTarget.id` — the block address of the ID it reads.
    pub id: u64,
    /// `DriverTarget.rna_path`, empty when it has none.
    pub path: String,
    /// `DriverTarget.pchan_name` — a bone of that object, empty for the
    /// object itself.
    pub bone: String,
    /// `DriverTarget.transChan`.
    pub chan: u32,
    /// `DriverTarget.rotation_mode`.
    pub rot_mode: i16,
    pub space: BDriverSpace,
}

/// Reading the targets of a `DriverVar` block.
pub trait BDriverTargets {
    /// Target `slot` of this variable, when the DNA has it.
    fn driver_target(&self, var: &BlendBlock, slot: usize, dna: &DnaType) -> Option<BDriverTarget>;
    /// `DriverVar.num_targets`, at least one.
    fn driver_target_count(&self, var: &BlendBlock, dna: &DnaType) -> usize;
}

impl BDriverTargets for BlendFile {
    fn driver_target(&self, var: &BlendBlock, slot: usize, dna: &DnaType) -> Option<BDriverTarget> {
        let base = self.off("DriverVar", "targets[8]", dna)? + slot * self.target_stride(dna)?;
        let ps = self.header.pointer_size as usize;
        let read = |o: usize| Self::read_ptr(&var.data, o, ps, self.header.little_endian);
        let at = |field: &str| self.off("DriverTarget", field, dna).map(|o| base + o);
        let id = at("*id").map_or(0, read);
        let path = at("*rna_path").and_then(|o| self.block_at(read(o))).map(|b| Self::cstr(&b.data)).unwrap_or_default();
        let bone = at("pchan_name[64]").map(|o| Self::field_str(&var.data, o, 64)).unwrap_or_default();
        let short = |field: &str| at(field).map_or(0, |o| self.int_at(&var.data, o, 2));
        Some(BDriverTarget {
            id,
            path,
            bone,
            chan: short("transChan"),
            rot_mode: at("rotation_mode").map_or(0, |o| self.int_at(&var.data, o, 1) as i16),
            space: BDriverSpace::of_flag(short("flag")),
        })
    }

    fn driver_target_count(&self, var: &BlendBlock, dna: &DnaType) -> usize {
        self.off("DriverVar", "num_targets", dna).map_or(1, |o| self.int_at(&var.data, o, 2) as usize).max(1)
    }
}

/// The byte helpers the target reader needs, kept here so no other module
/// grows a second way of reading a fixed string out of a block.
trait BDriverBytes {
    fn target_stride(&self, dna: &DnaType) -> Option<usize>;
    fn int_at(&self, data: &[u8], off: usize, width: usize) -> u32;
    fn field_str(data: &[u8], off: usize, len: usize) -> String;
    fn cstr(data: &[u8]) -> String;
}

impl BDriverBytes for BlendFile {
    /// `sizeof(DriverTarget)` from the DNA, so `targets[1]` lands right
    /// whatever the file's version wrote.
    fn target_stride(&self, dna: &DnaType) -> Option<usize> {
        dna.layout(self.header.pointer_size as usize).struct_size("DriverTarget")
    }

    fn int_at(&self, data: &[u8], off: usize, width: usize) -> u32 {
        let Some(bytes) = data.get(off..off + width) else { return 0 };
        let mut v = [0u8; 4];
        if self.header.little_endian {
            v[..width].copy_from_slice(bytes);
        } else {
            for (i, b) in bytes.iter().rev().enumerate() {
                v[i] = *b;
            }
        }
        u32::from_le_bytes(v)
    }

    fn field_str(data: &[u8], off: usize, len: usize) -> String {
        data.get(off..(off + len).min(data.len())).map(Self::cstr).unwrap_or_default()
    }

    fn cstr(data: &[u8]) -> String {
        let end = data.iter().position(|&c| c == 0).unwrap_or(data.len());
        String::from_utf8_lossy(&data[..end]).to_string()
    }
}
