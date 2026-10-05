//! Writing `Scene.markers` back: the list the session ends with, over the
//! `TimeMarker` blocks the file had. A marker that is still there keeps
//! its block (and so its camera and flags); one that was added gets a new
//! block; one that was removed has its block dropped; the list is relinked
//! in frame order through `next` / `prev` and the scene's `ListBase`.

use crate::blend::file::types::*;

impl BlendFile {
    /// The `TimeMarker` blocks of the scene at `scene`, in list order.
    fn marker_blocks(&self, scene: usize) -> Vec<usize> {
        let Some(dna) = self.dna.as_ref() else { return Vec::new() };
        let Some(sc) = self.blocks.get(scene) else { return Vec::new() };
        let first = self.ptr(sc, self.off("Scene", "markers", dna));
        self.list(first, self.off("TimeMarker", "*next", dna)).into_iter().filter_map(|b| self.index_of(b)).collect()
    }

    /// Make `markers` the scene's list. Returns how many blocks were added.
    pub fn set_markers(&mut self, scene: usize, markers: &[(i32, String)]) -> usize {
        let dna = match self.dna.clone() {
            Some(dna) => dna,
            None => return 0,
        };
        let had = self.marker_blocks(scene);
        // Pair every wanted marker with a block: the one that already holds
        // its frame and name, else its frame, else a spare, else a new one.
        let mut free: Vec<usize> = had;
        let mut chosen: Vec<usize> = Vec::with_capacity(markers.len());
        let mut added = 0;
        let reads = |file: &Self, index: usize| -> (i32, String) {
            let block = &file.blocks[index];
            let frame = file.int_field(block, "TimeMarker", "frame", &dna) as i32;
            let name = file.off("TimeMarker", "name[64]", &dna).map(|o| crate::blend::shade::nodes::read_cstr(&block.data, o, 64)).unwrap_or_default();
            (frame, name)
        };
        for (frame, name) in markers {
            let pick = free
                .iter()
                .position(|&i| reads(self, i) == (*frame, name.clone()))
                .or_else(|| free.iter().position(|&i| reads(self, i).0 == *frame));
            match pick {
                Some(p) => chosen.push(free.remove(p)),
                None => chosen.push(usize::MAX),
            }
        }
        for n in 0..chosen.len() {
            if chosen[n] != usize::MAX {
                continue;
            }
            match free.pop() {
                Some(spare) => chosen[n] = spare,
                None => {
                    let anchor = chosen.iter().copied().filter(|i| *i != usize::MAX).max().unwrap_or(scene);
                    let Some((at, _)) = self.push_block("TimeMarker", anchor) else { return added };
                    // The insert shifted every later index by one.
                    for i in chosen.iter_mut().chain(free.iter_mut()) {
                        if *i != usize::MAX && *i >= at {
                            *i += 1;
                        }
                    }
                    chosen[n] = at;
                    added += 1;
                }
            }
        }
        // Surplus blocks go; the indices above them shift down.
        let gone = free;
        for (slot, (frame, name)) in chosen.iter().zip(markers) {
            self.set_int(*slot, "TimeMarker", "frame", i64::from(*frame));
            self.set_text(*slot, "TimeMarker", "name", name);
        }
        let addresses: Vec<u64> = chosen.iter().map(|i| self.blocks[*i].old_address).collect();
        for (n, slot) in chosen.iter().enumerate() {
            let prev = if n == 0 { 0 } else { addresses[n - 1] };
            let next = addresses.get(n + 1).copied().unwrap_or(0);
            self.set_ptr(*slot, "TimeMarker", "*prev", prev);
            self.set_ptr(*slot, "TimeMarker", "*next", next);
        }
        let first = addresses.first().copied().unwrap_or(0);
        let last = addresses.last().copied().unwrap_or(0);
        let scene_index = scene - gone.iter().filter(|g| **g < scene).count();
        self.remove_blocks(gone);
        self.set_ptr(scene_index, "Scene", "markers.*first", first);
        self.set_ptr(scene_index, "Scene", "markers.*last", last);
        added
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_list_is_rewritten_over_the_blocks_the_file_had_and_reads_back_in_order() {
        let path = format!("{}/tests/fixtures/nla_cube.blend", env!("CARGO_MANIFEST_DIR"));
        let mut file = BlendFile::parse_bytes(&std::fs::read(path).expect("fixture")).expect("parse");
        let scene = file.current_scene_index().expect("a scene");
        let before = file.blocks.len();
        let wanted = vec![(5, "Intro".to_string()), (40, "Cut".to_string()), (90, "End".to_string())];
        let added = file.set_markers(scene, &wanted);
        assert_eq!(file.blocks.len(), before + added);
        let dna = file.dna.clone().unwrap();
        let read: Vec<(i32, String)> = file.markers(&file.blocks[scene], &dna).into_iter().map(|m| (m.frame, m.name)).collect();
        assert_eq!(read, wanted, "the scene lists them in order");
        // Fewer: the spare blocks are dropped, the rest relinked.
        let fewer = vec![(40, "Cut".to_string())];
        file.set_markers(scene, &fewer);
        let read: Vec<(i32, String)> = file.markers(&file.blocks[scene], &dna).into_iter().map(|m| (m.frame, m.name)).collect();
        assert_eq!(read, fewer);
        assert_eq!(file.blocks.len(), before + 1);
        // None: an empty list, no marker blocks left over.
        file.set_markers(scene, &[]);
        assert!(file.markers(&file.blocks[scene], &dna).is_empty());
        assert_eq!(file.blocks.len(), before + added - added);
        // And the file still serialises and parses.
        let again = BlendFile::parse_bytes(&file.to_bytes()).expect("parse the written file");
        assert_eq!(again.blocks.len(), file.blocks.len());
    }
}
