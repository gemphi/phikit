//! Reading a text object's `Curve` (`DNA_curve_types.h`: the font part —
//! `str`, `strinfo`, `tb`, `vfont*`, sizes and spacings — and the
//! extrusion / bevel fields) and its `VFont`s (`DNA_vfont_types.h`):
//! builtin, packed (`PackedFile`) or a file beside the `.blend`
//! (`BKE_vfont_data_ensure`).

use super::font::BFont;
use super::{text_mesh, BText, TCharInfo, TMesh};
use crate::blend::file::types::*;
use crate::blend::rig::read::BFields;
use std::sync::Arc;

/// `FO_BUILTIN_NAME`.
const FO_BUILTIN_NAME: &str = "<builtin>";

impl BlendFile {
    /// The mesh a text object (`OB_FONT`) evaluates to before its
    /// modifiers; `None` when its curve cannot be read.
    pub(crate) fn object_text_mesh(&self, ob: &BlendBlock, dna: &DnaType) -> Option<TMesh> {
        let cu = self.block_at_code(self.ptr(ob, self.off("Object", "*data", dna)), "CU")?;
        let text = self.scoped(cu, || self.read_text(cu, dna))?;
        Some(text_mesh(&text))
    }

    fn read_text(&self, cu: &BlendBlock, dna: &DnaType) -> Option<BText> {
        let c = BFields::new(self, dna, cu, "Curve");
        let either = |new: &str, old: &str| if c.has(new) { c.f32(new, 0.0) } else { c.f32(old, 0.0) };
        let len = c.int("len") as usize;
        let bytes = self.block_at(c.ptr("*str")).map_or_else(Vec::new, |b| b.data.iter().take(len).copied().collect());
        let text: Vec<char> = String::from_utf8_lossy(&bytes).chars().collect();
        let tb = self.block_at(c.ptr("*tb")).map_or([0.0; 4], |b| {
            let f = |i: usize| b.data.get(i * 4..i * 4 + 4).map_or(0.0, |s| self.read_f32(s));
            [f(0), f(1), f(2), f(3)]
        });
        let fonts = ["*vfont", "*vfontb", "*vfonti", "*vfontbi"].map(|f| self.vfont(c.ptr(f), dna));
        Some(BText {
            flag: c.int("flag"),
            fsize: c.f32("fsize", 1.0),
            spacing: c.f32("spacing", 1.0),
            linedist: c.f32("linedist", 1.0),
            wordspace: c.f32("wordspace", 1.0),
            shear: c.f32("shear", 0.0),
            xof: c.f32("xof", 0.0),
            yof: c.f32("yof", 0.0),
            ulpos: c.f32("ulpos", 0.0),
            ulheight: c.f32("ulheight", 0.05),
            smallcaps_scale: c.f32("smallcaps_scale", 0.75),
            align_x: c.int("spacemode") as u8,
            align_y: c.int("align_y") as u8,
            resolu: (c.int("resolu") as i16).max(1) as usize,
            extrude: either("extrude", "ext1"),
            bevel_radius: either("bevel_radius", "ext2"),
            // Saved under its old DNA name `width` (RNA `offset`), as for curves.
            offset: if c.has("offset") { c.f32("offset", 1.0) } else { c.f32("width", 1.0) },
            bevresol: (c.int("bevresol") as i16).max(0) as usize,
            bevel_mode: c.int("bevel_mode") as u8,
            info: self.char_infos(c.ptr("*strinfo"), text.len(), dna),
            text,
            textbox: tb,
            fonts,
        })
    }

    /// `Curve.strinfo`: one `CharInfo` per character.
    fn char_infos(&self, addr: u64, n: usize, dna: &DnaType) -> Vec<TCharInfo> {
        let Some(b) = self.block_at(addr) else { return vec![TCharInfo::default(); n] };
        let ps = self.header.pointer_size as usize;
        let size = Self::struct_size("CharInfo", dna, ps).unwrap_or(8);
        let (kern, mat, flag) = (self.off("CharInfo", "kern", dna), self.off("CharInfo", "mat_nr", dna), self.off("CharInfo", "flag", dna));
        (0..n)
            .map(|i| {
                let d = b.data.get(i * size..).unwrap_or(&[]);
                TCharInfo {
                    kern: self.f32s(d, kern, [0.0])[0],
                    mat_nr: mat.and_then(|o| d.get(o..o + 2)).map_or(0, |s| self.read_u16(s) as i16),
                    flag: flag.and_then(|o| d.get(o)).copied().unwrap_or(0),
                }
            })
            .collect()
    }

    /// A `VFont`'s face: builtin, packed, or read from its path.
    fn vfont(&self, addr: u64, dna: &DnaType) -> Option<Arc<BFont>> {
        let vf = self.block_at_code(addr, "VF")?;
        let v = BFields::new(self, dna, vf, "VFont");
        let path = match v.has("filepath[1024]") {
            true => v.text("filepath[1024]", 1024),
            false => v.text("name[1024]", 1024),
        };
        if path == FO_BUILTIN_NAME {
            return BFont::builtin();
        }
        let packed = self.scoped(vf, || {
            let pf = self.block_at(v.ptr("*packedfile"))?;
            let size = self.int_field(pf, "PackedFile", "size", dna) as usize;
            let data = self.block_at(self.ptr(pf, self.off("PackedFile", "*data", dna)))?;
            (size > 0 && size <= data.data.len()).then(|| data.data[..size].to_vec())
        });
        let bytes = packed.or_else(|| std::fs::read(resolve(&path)?).ok())?;
        BFont::from_bytes(bytes).map(Arc::new)
    }
}

/// A Blender path on disk: `//` is the `.blend`'s folder.
fn resolve(path: &str) -> Option<std::path::PathBuf> {
    match path.strip_prefix("//") {
        Some(rel) => Some(crate::blend::shade::image::source_dir()?.join(rel.replace('\\', "/"))),
        None => Some(std::path::PathBuf::from(path)),
    }
}
