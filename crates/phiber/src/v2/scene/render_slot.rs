//! The Render Result's render slots — `Image.renderslots`, eight of them
//! (`IMA_MAX_RENDER_SLOT`): F12 renders into the slot shown
//! (`render_slot`), which becomes `last_render_slot`, and the slot
//! rendered before keeps its result (`BKE_image_backup_render`), so two
//! renders can be compared by switching slots. The header's Slot dropdown
//! picks one (`ImageKey::Slot`, `Slot 1` … `Slot 8`); View ▸ Render Slot
//! Cycle Next / Previous (J, Alt J; `image.cycle_render_slot`) step to the
//! next slot holding a render, or the last rendered into
//! (`ED_image_slot_cycle`), falling to the first two when none does.

use super::render_job::job;
use crate::v2::state::{BKey as _, BPath, BState, ImageKey, rule::BRule};
use std::sync::atomic::Ordering;

/// The slots as one owner over the job's.
pub struct BRenderSlot;

impl BRenderSlot {
    pub const COUNT: usize = 8;
    pub const NAME_LIST: [&'static str; Self::COUNT] = ["Slot 1", "Slot 2", "Slot 3", "Slot 4", "Slot 5", "Slot 6", "Slot 7", "Slot 8"];

    /// The slot shown, 0-based.
    pub fn active() -> u32 {
        job().active.load(Ordering::Relaxed)
    }

    /// The slot the render in flight, or the last, went into.
    pub fn last() -> u32 {
        job().last.load(Ordering::Relaxed)
    }

    /// Whether slot `i` holds a render (in flight or finished).
    pub fn filled(i: u32) -> bool {
        job().slot(i).is_some()
    }

    /// Whether any slot does: the Render Result exists as a data-block.
    pub fn any_filled() -> bool {
        (0..Self::COUNT as u32).any(Self::filled)
    }

    /// Show slot `i`: the painters see the change through the sequence.
    pub fn select(i: u32) {
        let j = job();
        if (i as usize) < Self::COUNT && j.active.swap(i, Ordering::Relaxed) != i {
            j.seq.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// `ED_image_slot_cycle`: the next (or previous) slot with a render, or
    /// the one last rendered into; with none, the first two by turns.
    /// Returns the slot now shown when it changed.
    pub fn cycle(reverse: bool) -> Option<u32> {
        let n = Self::COUNT as i32;
        let cur = Self::active() as i32;
        let last = Self::last() as i32;
        let found = (1..n).map(|i| (cur + if reverse { -i } else { i }).rem_euclid(n)).find(|&slot| Self::filled(slot as u32) || slot == last);
        let next = found.unwrap_or(if cur == 1 { 0 } else { 1 });
        (next != cur).then(|| {
            Self::select(next as u32);
            next as u32
        })
    }

    pub fn name(i: u32) -> &'static str {
        Self::NAME_LIST.get(i as usize).copied().unwrap_or(Self::NAME_LIST[0])
    }

    pub fn of_name(name: &str) -> Option<u32> {
        Self::NAME_LIST.iter().position(|n| *n == name).map(|i| i as u32)
    }
}

/// The header's Slot dropdown (`image.slot = Slot n`) shows that slot; J
/// and Alt J (`image.slot_cycle = next | previous`) step and set the
/// dropdown's key to the slot reached, so the two stay one.
pub struct BRenderSlotRule;

impl BRule for BRenderSlotRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == ImageKey::Slot.path() || *path == ImageKey::SlotCycle.path()
    }
    fn after(&self, path: &BPath, value: &str, s: &mut BState) {
        if *path == ImageKey::SlotCycle.path() {
            s.remove(path);
            if let Some(slot) = BRenderSlot::cycle(value == "previous") {
                s.set(ImageKey::Slot, BRenderSlot::name(slot));
            }
            return;
        }
        if let Some(slot) = BRenderSlot::of_name(value) {
            BRenderSlot::select(slot);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_and_the_cycles_fallback() {
        assert_eq!(BRenderSlot::of_name("Slot 3"), Some(2));
        assert_eq!(BRenderSlot::of_name("Slot 9"), None);
        assert_eq!(BRenderSlot::name(7), "Slot 8");
        assert_eq!(BRenderSlot::name(99), "Slot 1", "out of range reads as the first");
    }
}
