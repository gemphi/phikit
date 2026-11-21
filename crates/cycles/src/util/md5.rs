// SPDX-FileCopyrightText: 1999, 2002 Aladdin Enterprises. All rights reserved.
// SPDX-License-Identifier: Apache-2.0
//! `MD5Hash` of `intern/cycles/util/md5.h`: the MD5 digest of what is
//! appended, which Cycles uses to tell whether something changed. Written
//! from RFC 1321; Cycles has L. Peter Deutsch's implementation in
//! `util/md5.cpp`. Not ported: `append_file`.

use std::sync::OnceLock;

/// The turns of the four rounds.
const SHIFT: [[u32; 4]; 4] = [[7, 12, 17, 22], [5, 9, 14, 20], [4, 11, 16, 23], [6, 10, 15, 21]];

/// The constant of each of the 64 steps: the sine of its number, as a
/// fraction of 2 to the 32.
fn sine() -> &'static [u32; 64] {
    static TABLE: OnceLock<[u32; 64]> = OnceLock::new();
    TABLE.get_or_init(|| std::array::from_fn(|step| (((step + 1) as f64).sin().abs() * 4294967296.0) as u32))
}

/// `MD5Hash`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Md5Hash {
    state: [u32; 4],
    /// The bytes appended so far.
    length: u64,
    /// What does not yet fill a block of 64.
    rest: Vec<u8>,
}

impl Default for Md5Hash {
    fn default() -> Self {
        Self::new()
    }
}

impl Md5Hash {
    /// A digest of nothing yet.
    pub fn new() -> Md5Hash {
        Md5Hash { state: [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476], length: 0, rest: Vec::new() }
    }

    /// `process`: take one block of 64 bytes into the state.
    fn process(state: &mut [u32; 4], block: &[u8]) {
        let word: Vec<u32> = block.chunks_exact(4).map(|part| u32::from_le_bytes([part[0], part[1], part[2], part[3]])).collect();
        let [mut a, mut b, mut c, mut d] = *state;
        for step in 0..64 {
            let (mixed, at) = match step / 16 {
                0 => ((b & c) | (!b & d), step),
                1 => ((d & b) | (!d & c), (5 * step + 1) % 16),
                2 => (b ^ c ^ d, (3 * step + 5) % 16),
                _ => (c ^ (b | !d), (7 * step) % 16),
            };
            let turned = a.wrapping_add(mixed).wrapping_add(sine()[step]).wrapping_add(word[at]).rotate_left(SHIFT[step / 16][step % 4]);
            (a, b, c, d) = (d, b.wrapping_add(turned), b, c);
        }
        *state = [state[0].wrapping_add(a), state[1].wrapping_add(b), state[2].wrapping_add(c), state[3].wrapping_add(d)];
    }

    /// `append`: more bytes.
    pub fn append(&mut self, data: &[u8]) {
        self.length += data.len() as u64;
        self.rest.extend_from_slice(data);
        let whole = self.rest.len() / 64 * 64;
        for block in self.rest[..whole].chunks_exact(64) {
            Self::process(&mut self.state, block);
        }
        self.rest.drain(..whole);
    }

    /// `append` of a string: its bytes.
    pub fn append_str(&mut self, text: &str) {
        self.append(text.as_bytes());
    }

    /// `finish`: the sixteen bytes of the digest of what was appended so
    /// far. More may be appended afterwards.
    pub fn finish(&self) -> [u8; 16] {
        let mut state = self.state;
        // A one bit, zeros up to eight bytes short of a block, the length in bits.
        let mut tail = self.rest.clone();
        tail.push(0x80);
        tail.resize(tail.len() + (120 - tail.len() % 64) % 64, 0);
        tail.extend_from_slice(&(self.length.wrapping_mul(8)).to_le_bytes());
        for block in tail.chunks_exact(64) {
            Self::process(&mut state, block);
        }
        let mut digest = [0; 16];
        for (part, word) in digest.chunks_exact_mut(4).zip(state) {
            part.copy_from_slice(&word.to_le_bytes());
        }
        digest
    }

    /// `get_hex`: the digest as 32 hexadecimal digits, in capitals.
    pub fn hex(&self) -> String {
        self.finish().iter().map(|byte| format!("{byte:02X}")).collect()
    }
}
