//! UV edits since the file was read: which corners are selected and how far
//! they have been moved.
//!
//! Blender keeps UV selection on the mesh's loops (`MLoopUV`'s select flag,
//! `.uv_select` in 4.x) and a move writes the loop's `uv` outright. Here
//! both are **session state** over a loaded model that stays what the file
//! said — two keys, `ctx.uv.selected` and `ctx.uv.moved`, the same shape
//! `scene::edit` uses for objects added and renamed. Undo is then a state
//! step and a reload a clean slate, with no copy of the mesh to keep in
//! step.
//!
//! A corner is `(map, corner)`: which of the shown maps
//! (`part::image::uv::BUvShown`'s order) and which entry of its `uvs`.
//! Blender ref: `uv_select_pick` (`uvedit_select.cc`), `transform` over
//! `TC_UV`.

use crate::v2::state::{BState, UvKey};

/// Entries within a key; the fields within an entry.
const SEP: char = '|';
const AT: char = ':';

/// One corner of one shown map.
pub type BUvCorner = (usize, usize);

/// What has changed in the UV layout since the file was read.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BUvEdit {
    /// The selected corners, in the order they were picked.
    pub selected: Vec<BUvCorner>,
    /// `(corner, du, dv)` for every corner that has been moved.
    pub moved: Vec<(BUvCorner, [f32; 2])>,
    /// The corners the user has pinned (`uv.pin`, P). An unwrap holds
    /// these where they are and solves everything else around them.
    pub pinned: Vec<BUvCorner>,
}

impl BUvEdit {
    pub fn now(s: &BState) -> Self {
        Self {
            selected: read_selected(s.get(UvKey::Selected)),
            moved: read_moved(s.get(UvKey::Moved)),
            pinned: read_selected(s.get(UvKey::Pinned)),
        }
    }

    /// Whether `corner` is pinned.
    pub fn stuck(&self, corner: BUvCorner) -> bool {
        self.pinned.contains(&corner)
    }

    /// The selection pinned (`uv.pin`) or unpinned (`uv.pin(clear=True)`),
    /// as the new value of the key.
    pub fn pinning(&self, clear: bool) -> Vec<BUvCorner> {
        let mut out = self.pinned.clone();
        for corner in &self.selected {
            match (clear, out.iter().position(|c| c == corner)) {
                (true, Some(at)) => {
                    out.remove(at);
                }
                (false, None) => out.push(*corner),
                _ => {}
            }
        }
        out
    }

    /// The whole map `map` unwrapped, as the `moved` list it becomes.
    ///
    /// The editor's model is *offsets over the file's UVs*, so an unwrap —
    /// which produces absolute UVs — lands as the offset from the file's
    /// to the new one. Nothing about the loaded mesh is touched, undo is
    /// one state step, and a reload is a clean slate, exactly as a drag is.
    ///
    /// Pinned corners of this map are handed to the solver at the UV they
    /// are *showing* (the file's, moved), which is what makes a pin hold
    /// its place through an unwrap rather than merely be drawn as a pin.
    pub fn unwrapped(
        &self,
        map: usize,
        mesh: crate::phiphx::uv::BUvMesh<'_>,
        file_uv: &[[f32; 2]],
        how: crate::phiphx::uv::BUnwrap,
        seam: &[(u32, u32)],
    ) -> Vec<(BUvCorner, [f32; 2])> {
        let pin: Vec<crate::phiphx::uv::BUvPin> = self
            .pinned
            .iter()
            .filter(|(m, _)| *m == map)
            .filter_map(|&(_, c)| {
                let uv = file_uv.get(c).copied()?;
                Some(crate::phiphx::uv::BUvPin { corner: c, uv: self.uv_of((map, c), uv) })
            })
            .collect();
        let fresh = crate::phiphx::uv::unwrap(mesh, how, seam, &pin);
        // Everything the unwrap touched becomes an offset; the other maps'
        // offsets are left exactly as they were.
        let mut out: Vec<(BUvCorner, [f32; 2])> = self.moved.iter().copied().filter(|((m, _), _)| *m != map).collect();
        for (c, now) in fresh.iter().enumerate() {
            let was = file_uv.get(c).copied().unwrap_or([0.0; 2]);
            let d = [now[0] - was[0], now[1] - was[1]];
            if d[0] != 0.0 || d[1] != 0.0 {
                out.push(((map, c), d));
            }
        }
        out
    }

    /// Whether `corner` is selected.
    pub fn holds(&self, corner: BUvCorner) -> bool {
        self.selected.contains(&corner)
    }

    /// How far `corner` has been moved, zero when it has not.
    pub fn offset(&self, corner: BUvCorner) -> [f32; 2] {
        self.moved.iter().find(|(c, _)| *c == corner).map_or([0.0; 2], |(_, d)| *d)
    }

    /// `corner`'s UV as it now stands: the file's, moved.
    pub fn uv_of(&self, corner: BUvCorner, uv: [f32; 2]) -> [f32; 2] {
        let d = self.offset(corner);
        [uv[0] + d[0], uv[1] + d[1]]
    }

    /// `corner` picked: on its own, or added to the selection with Shift
    /// (`uv_select_pick`'s `extend`). Picking a selected corner again with
    /// Shift drops it, as Blender's toggle does.
    pub fn picked(&self, corner: BUvCorner, extend: bool) -> Vec<BUvCorner> {
        if !extend {
            return vec![corner];
        }
        let mut out = self.selected.clone();
        match out.iter().position(|c| *c == corner) {
            Some(at) => {
                out.remove(at);
            }
            None => out.push(corner),
        }
        out
    }

    /// Every selected corner moved a further `(du, dv)`, folded into the
    /// offsets already there so a second drag continues the first.
    pub fn nudged(&self, by: [f32; 2]) -> Vec<(BUvCorner, [f32; 2])> {
        let mut out = self.moved.clone();
        for corner in &self.selected {
            match out.iter_mut().find(|(c, _)| c == corner) {
                Some((_, d)) => {
                    d[0] += by[0];
                    d[1] += by[1];
                }
                None => out.push((*corner, by)),
            }
        }
        out
    }

    /// The two keys' values for a selection and a set of offsets.
    pub fn write_selected(list: &[BUvCorner]) -> String {
        list.iter().map(|(m, c)| format!("{m}{AT}{c}")).collect::<Vec<_>>().join(&SEP.to_string())
    }

    /// Pins are written like a selection: the same `map:corner` entries.
    pub fn write_pinned(list: &[BUvCorner]) -> String {
        Self::write_selected(list)
    }

    pub fn write_moved(list: &[(BUvCorner, [f32; 2])]) -> String {
        list.iter().map(|((m, c), d)| format!("{m}{AT}{c}{AT}{}{AT}{}", d[0], d[1])).collect::<Vec<_>>().join(&SEP.to_string())
    }
}

fn read_selected(raw: Option<&str>) -> Vec<BUvCorner> {
    raw.unwrap_or("")
        .split(SEP)
        .filter_map(|entry| {
            let (m, c) = entry.split_once(AT)?;
            Some((m.parse().ok()?, c.parse().ok()?))
        })
        .collect()
}

fn read_moved(raw: Option<&str>) -> Vec<(BUvCorner, [f32; 2])> {
    raw.unwrap_or("")
        .split(SEP)
        .filter_map(|entry| {
            let f: Vec<&str> = entry.split(AT).collect();
            if f.len() != 4 {
                return None;
            }
            Some(((f[0].parse().ok()?, f[1].parse().ok()?), [f[2].parse().ok()?, f[3].parse().ok()?]))
        })
        .collect()
}

#[cfg(test)]
#[path = "uv_edit_tests.rs"]
mod uv_edit_tests;
