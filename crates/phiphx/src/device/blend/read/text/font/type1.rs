//! A PostScript Type 1 font (`.pfb` / `.pfa`), the format of Blender's
//! builtin `bfont.pfb`: what FreeType's `type1` driver reads from it
//! (`t1parse.c`, `t1load.c`) — the clear-text font dictionary, the
//! `eexec`-encrypted private dictionary with its `Subrs` and
//! `CharStrings`, each charstring decrypted with its `lenIV` bytes dropped.

use super::charstring;
use super::names::unicode_of;
use super::{BFontMetrics, BGlyph};
use std::collections::HashMap;

/// `eexec` and charstring encryption keys (Adobe Type 1 Font Format §7).
const EEXEC_KEY: u16 = 55665;
const CHARSTRING_KEY: u16 = 4330;

/// One parsed Type 1 font.
#[derive(Debug)]
pub struct BType1 {
    pub subrs: Vec<Vec<u8>>,
    /// Decrypted charstrings by glyph name.
    pub charstrings: HashMap<String, Vec<u8>>,
    /// Unicode → glyph name, as FreeType's synthesized Unicode charmap
    /// gives it (`psnames`: each glyph name through the Adobe Glyph List).
    pub cmap: HashMap<u32, String>,
    /// `/FontBBox`, or zero when the font has none (`bfont.pfb`).
    pub bbox: [i32; 4],
}

impl BType1 {
    /// Parse a `.pfb` (segmented) or `.pfa` (hex) file.
    pub fn parse(data: &[u8]) -> Option<Self> {
        let (clear, private) = match data.first() {
            Some(0x80) => pfb_segments(data)?,
            _ => pfa_split(data)?,
        };
        let private = decrypt(&private, EEXEC_KEY, 4);
        let len_iv = number_after(&private, b"/lenIV").map_or(4, |n| n as usize);
        let subrs = read_subrs(&private).into_iter().map(|s| decrypt(&s, CHARSTRING_KEY, len_iv)).collect();
        let charstrings: HashMap<String, Vec<u8>> =
            read_charstrings(&private).into_iter().map(|(n, s)| (n, decrypt(&s, CHARSTRING_KEY, len_iv))).collect();
        let cmap = charstrings.keys().filter_map(|n| Some((unicode_of(n)?, n.clone()))).collect();
        let bbox = bbox(&clear).unwrap_or([0; 4]);
        Some(Self { subrs, charstrings, cmap, bbox })
    }

    /// FreeType's face metrics for a Type 1 font (`T1_Face_Init`): the
    /// ascender and descender are the `FontBBox`'s top and bottom.
    pub fn metrics(&self) -> BFontMetrics {
        BFontMetrics::from_face(self.bbox[3], self.bbox[1], self.bbox[3], self.bbox[1])
    }

    /// The glyph for `unicode` at `FT_LOAD_NO_SCALE` (font units).
    pub fn glyph(&self, unicode: u32) -> Option<BGlyph> {
        let name = self.cmap.get(&unicode)?;
        charstring::run(self, self.charstrings.get(name)?)
    }
}

/// `.pfb` segments: type 1 is ASCII, type 2 binary, type 3 the end.
fn pfb_segments(data: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
    let (mut clear, mut binary, mut i) = (Vec::new(), Vec::new(), 0);
    while i + 6 <= data.len() && data[i] == 0x80 {
        let kind = data[i + 1];
        if kind == 3 {
            break;
        }
        let n = u32::from_le_bytes(data[i + 2..i + 6].try_into().ok()?) as usize;
        let seg = data.get(i + 6..i + 6 + n)?;
        match kind {
            1 if binary.is_empty() => clear.extend_from_slice(seg),
            2 => binary.extend_from_slice(seg),
            _ => {}
        }
        i += 6 + n;
    }
    Some((clear, binary))
}

/// `.pfa`: clear text up to `eexec`, then hexadecimal.
fn pfa_split(data: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
    let at = find(data, b"eexec", 0)? + 5;
    let hex: Vec<u8> = data[at..].iter().filter_map(|&c| (c as char).to_digit(16).map(|d| d as u8)).collect();
    Some((data[..at].to_vec(), hex.chunks_exact(2).map(|p| p[0] << 4 | p[1]).collect()))
}

/// The Type 1 decryption (`t1_decrypt`), dropping the first `skip` bytes.
fn decrypt(data: &[u8], key: u16, skip: usize) -> Vec<u8> {
    let mut r = key;
    let plain: Vec<u8> = data
        .iter()
        .map(|&c| {
            let p = c ^ (r >> 8) as u8;
            r = (c as u16).wrapping_add(r).wrapping_mul(52845).wrapping_add(22719);
            p
        })
        .collect();
    plain.get(skip..).unwrap_or(&[]).to_vec()
}

fn find(hay: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    hay.get(from..)?.windows(needle.len()).position(|w| w == needle).map(|p| p + from)
}

/// A PostScript token reader over the private dictionary.
struct Tokens<'a> {
    data: &'a [u8],
    at: usize,
}

impl<'a> Tokens<'a> {
    fn next(&mut self) -> Option<&'a [u8]> {
        while self.data.get(self.at)?.is_ascii_whitespace() {
            self.at += 1;
        }
        let start = self.at;
        while self.data.get(self.at).is_some_and(|c| !c.is_ascii_whitespace()) {
            self.at += 1;
        }
        Some(&self.data[start..self.at])
    }

    fn int(&mut self) -> Option<i64> {
        std::str::from_utf8(self.next()?).ok()?.parse().ok()
    }

    /// `len RD <one space> <len bytes>`: the binary string after `RD` / `-|`.
    fn binary(&mut self) -> Option<&'a [u8]> {
        let n = self.int()? as usize;
        self.next()?;
        let start = self.at + 1;
        self.at = start + n;
        self.data.get(start..start + n)
    }
}

fn number_after(data: &[u8], key: &[u8]) -> Option<i64> {
    let at = find(data, key, 0)? + key.len();
    Tokens { data, at }.int()
}

/// `/Subrs n array` then `dup i len RD <bytes> NP` entries.
fn read_subrs(data: &[u8]) -> Vec<Vec<u8>> {
    let Some(at) = find(data, b"/Subrs", 0) else { return Vec::new() };
    let mut t = Tokens { data, at: at + 6 };
    let n = t.int().unwrap_or(0).max(0) as usize;
    let mut subrs = vec![Vec::new(); n];
    t.next();
    while let Some(tok) = t.next() {
        if tok != b"dup" {
            match tok {
                b"ND" | b"|-" | b"NP" | b"|" | b"noaccess" | b"put" | b"def" | b"readonly" => continue,
                _ => break,
            }
        }
        let Some(i) = t.int() else { break };
        let Some(bytes) = t.binary() else { break };
        if let Some(slot) = subrs.get_mut(i as usize) {
            *slot = bytes.to_vec();
        }
    }
    subrs
}

/// `/CharStrings n dict dup begin` then `/name len RD <bytes> ND` entries.
fn read_charstrings(data: &[u8]) -> Vec<(String, Vec<u8>)> {
    let Some(at) = find(data, b"/CharStrings", 0) else { return Vec::new() };
    let Some(begin) = find(data, b"begin", at) else { return Vec::new() };
    let mut t = Tokens { data, at: begin + 5 };
    let mut out = Vec::new();
    while let Some(tok) = t.next() {
        if tok == b"end" {
            break;
        }
        let Some(name) = tok.strip_prefix(b"/") else { continue };
        let Some(bytes) = t.binary() else { break };
        out.push((String::from_utf8_lossy(name).to_string(), bytes.to_vec()));
    }
    out
}

/// `/FontBBox {xmin ymin xmax ymax}` in the clear-text dictionary,
/// FreeType's rounding: the minimum down, the maximum up.
fn bbox(clear: &[u8]) -> Option<[i32; 4]> {
    let at = find(clear, b"/FontBBox", 0)? + 9;
    let text = String::from_utf8_lossy(&clear[at..clear.len().min(at + 200)]).to_string();
    let nums: Vec<f64> = text
        .split(|c: char| c.is_whitespace() || matches!(c, '{' | '}' | '[' | ']'))
        .filter(|s| !s.is_empty())
        .take(4)
        .map(|s| s.parse().ok())
        .collect::<Option<_>>()?;
    (nums.len() == 4).then(|| [nums[0].floor() as i32, nums[1].floor() as i32, nums[2].ceil() as i32, nums[3].ceil() as i32])
}
