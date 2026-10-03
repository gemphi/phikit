//! A Motion-JPEG AVI, written by hand (the RIFF layout Blender's own
//! `avi.cc` wrote until 5.0 deprecated it): `RIFF 'AVI '` holding `LIST
//! hdrl` (`avih`, one `strl` with `strh` and `strf`), `LIST movi` of one
//! `00dc` chunk a frame, each a baseline JPEG, and the `idx1` index that
//! players seek by. Every player opens it, and it needs nothing installed
//! — the fallback when FFmpeg is not on the machine (`video/mod.rs`).

use std::io::{Seek, SeekFrom, Write};

/// One movie being written.
pub struct BAvi<W: Write + Seek> {
    out: W,
    w: u32,
    h: u32,
    fps: f32,
    /// Each frame's `(offset from the `movi` list's data, size)` for `idx1`.
    index: Vec<(u32, u32)>,
    movi_at: u64,
    largest: u32,
}

fn chunk(tag: &[u8; 4], body: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(body.len() + 8);
    v.extend_from_slice(tag);
    v.extend_from_slice(&(body.len() as u32).to_le_bytes());
    v.extend_from_slice(body);
    if body.len() % 2 == 1 {
        v.push(0);
    }
    v
}

fn list(kind: &[u8; 4], body: &[u8]) -> Vec<u8> {
    let mut inner = kind.to_vec();
    inner.extend_from_slice(body);
    chunk(b"LIST", &inner)
}

impl<W: Write + Seek> BAvi<W> {
    /// Begin a `w × h` movie at `fps`; the headers are written with room
    /// for the counts `finish` fills in.
    pub fn begin(mut out: W, w: u32, h: u32, fps: f32) -> std::io::Result<Self> {
        out.write_all(b"RIFF\0\0\0\0AVI ")?;
        let mut me = Self { out, w, h, fps, index: Vec::new(), movi_at: 0, largest: 0 };
        me.write_headers(0)?;
        me.out.write_all(b"LIST\0\0\0\0movi")?;
        me.movi_at = me.out.stream_position()?;
        Ok(me)
    }

    fn write_headers(&mut self, frames: u32) -> std::io::Result<()> {
        let micros = (1_000_000.0 / self.fps.max(0.001)).round() as u32;
        let (w, h) = (self.w, self.h);
        let mut avih = Vec::new();
        for v in [micros, self.largest * self.fps.ceil() as u32, 0, 0x10 /* has index */, frames, 0, 1, self.largest, w, h, 0, 0, 0, 0] {
            avih.extend_from_slice(&v.to_le_bytes());
        }
        let mut strh = Vec::new();
        strh.extend_from_slice(b"vidsMJPG");
        for v in [0u32, 0, 0, 1, self.fps.round() as u32, 0, frames, self.largest, 0xffff_ffff, 0] {
            strh.extend_from_slice(&v.to_le_bytes());
        }
        strh.extend_from_slice(&[0u8, 0, 0, 0, (w & 0xff) as u8, (w >> 8) as u8, (h & 0xff) as u8, (h >> 8) as u8]);
        // BITMAPINFOHEADER.
        let mut strf = Vec::new();
        strf.extend_from_slice(&40u32.to_le_bytes());
        strf.extend_from_slice(&(w as i32).to_le_bytes());
        strf.extend_from_slice(&(h as i32).to_le_bytes());
        strf.extend_from_slice(&1u16.to_le_bytes());
        strf.extend_from_slice(&24u16.to_le_bytes());
        strf.extend_from_slice(b"MJPG");
        strf.extend_from_slice(&(w * h * 3).to_le_bytes());
        strf.extend_from_slice(&[0u8; 16]);
        let strl = list(b"strl", &[chunk(b"strh", &strh), chunk(b"strf", &strf)].concat());
        let hdrl = list(b"hdrl", &[chunk(b"avih", &avih), strl].concat());
        self.out.write_all(&hdrl)
    }

    /// One frame, already a JPEG.
    pub fn frame(&mut self, jpeg: &[u8]) -> std::io::Result<()> {
        let at = (self.out.stream_position()? - self.movi_at) as u32 + 4;
        self.out.write_all(&chunk(b"00dc", jpeg))?;
        self.index.push((at, jpeg.len() as u32));
        self.largest = self.largest.max(jpeg.len() as u32);
        Ok(())
    }

    /// The index and the sizes the headers were waiting for.
    pub fn finish(mut self) -> std::io::Result<W> {
        let movi_end = self.out.stream_position()?;
        let mut idx = Vec::with_capacity(self.index.len() * 16);
        for (at, size) in &self.index {
            idx.extend_from_slice(b"00dc");
            idx.extend_from_slice(&0x10u32.to_le_bytes());
            idx.extend_from_slice(&at.to_le_bytes());
            idx.extend_from_slice(&size.to_le_bytes());
        }
        self.out.write_all(&chunk(b"idx1", &idx))?;
        let end = self.out.stream_position()?;
        self.out.seek(SeekFrom::Start(4))?;
        self.out.write_all(&((end - 8) as u32).to_le_bytes())?;
        self.out.seek(SeekFrom::Start(self.movi_at - 8))?;
        self.out.write_all(&((movi_end - self.movi_at + 4) as u32).to_le_bytes())?;
        // The headers again, now with the frame count and the largest chunk.
        self.out.seek(SeekFrom::Start(12))?;
        let frames = self.index.len() as u32;
        self.write_headers(frames)?;
        self.out.seek(SeekFrom::Start(end))?;
        Ok(self.out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_riff_names_its_frames_and_the_index_finds_them() {
        let mut avi = BAvi::begin(std::io::Cursor::new(Vec::new()), 16, 8, 24.0).unwrap();
        let a = vec![0xffu8, 0xd8, 1, 2, 3, 0xff, 0xd9];
        let b = vec![0xffu8, 0xd8, 9, 0xff, 0xd9];
        avi.frame(&a).unwrap();
        avi.frame(&b).unwrap();
        let bytes = avi.finish().unwrap().into_inner();
        assert_eq!(&bytes[..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"AVI ");
        assert_eq!(u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize, bytes.len() - 8, "the RIFF size is the file less its header");
        let movi = bytes.windows(4).position(|w| w == b"movi").unwrap();
        let first = &bytes[movi + 4..];
        assert_eq!(&first[..4], b"00dc");
        assert_eq!(u32::from_le_bytes(first[4..8].try_into().unwrap()) as usize, a.len());
        assert_eq!(&first[8..8 + a.len()], &a[..]);
        let idx = bytes.windows(4).position(|w| w == b"idx1").unwrap();
        let entries = u32::from_le_bytes(bytes[idx + 4..idx + 8].try_into().unwrap()) / 16;
        assert_eq!(entries, 2, "two frames indexed");
        let at = u32::from_le_bytes(bytes[idx + 16..idx + 20].try_into().unwrap()) as usize;
        assert_eq!(&bytes[movi + at..movi + at + 4], b"00dc", "the index offset counts from `movi`");
        let frames = u32::from_le_bytes(bytes[12 + 12 + 8 + 16..12 + 12 + 8 + 20].try_into().unwrap());
        assert_eq!(frames, 2, "`avih.dwTotalFrames` was filled in");
    }
}
