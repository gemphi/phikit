//! Propagate: every word's [`WorldMatrix`] from the [`Tree`](crate::tree::Tree)
//! and the [`Pose`]s, parents first.
//!
//! Registered at [`Stage::PostCouple`], after animation, physics and
//! constraints have written their `Pose`s: `world = parent world x local`.
//! A word with no `Pose` passes its parent's matrix through; it gets no
//! `WorldMatrix` of its own. A matrix that did not change is not written, so
//! a reader of change ticks sees only what moved.
use crate::comp::space::{Pose, WorldMatrix};
use crate::system::{FnSystem, Stage};
use crate::word::Word;
use crate::world::World;

/// The propagate system, at [`Stage::PostCouple`].
pub fn propagate() -> FnSystem<impl FnMut(&mut World, f32) + Send + Sync> {
    FnSystem::new("propagate", |world, _| world.propagate()).at(Stage::PostCouple)
}

impl World {
    /// Works out every `WorldMatrix` now, roots first. [`propagate`] runs
    /// this each step.
    pub fn propagate(&mut self) {
        let mut starts: Vec<Word> = self.tree.roots();
        for (w, _) in self.each::<Pose>() {
            if self.tree.parent(w).is_none() && !starts.contains(&w) {
                starts.push(w);
            }
        }
        // (word, its parent's world) in an order where parents come first.
        let mut work: Vec<(Word, WorldMatrix)> = starts.into_iter().map(|w| (w, WorldMatrix::IDENTITY)).collect();
        while let Some((word, above)) = work.pop() {
            let world = match self.read::<Pose>(word) {
                Some(pose) => {
                    let world = above.mul(&pose.matrix());
                    if self.read::<WorldMatrix>(word) != Some(&world) {
                        self.bind(word, world);
                    }
                    world
                }
                None => above,
            };
            work.extend(self.tree.children(word).iter().rev().map(|c| (*c, world)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn near(a: [f32; 3], b: [f32; 3]) -> bool {
        a.iter().zip(&b).all(|(x, y)| (x - y).abs() < 1e-4)
    }

    #[test]
    fn a_child_follows_its_parent_each_step() {
        let mut w = World::new();
        let arm = w.coin("arm").bind(Pose::at([1.0, 0.0, 0.0])).word();
        let hand = w.coin("hand").bind(Pose::at([0.0, 2.0, 0.0])).word();
        w.tree.attach(hand, arm);
        w.add_system(propagate());
        w.step(1.0);
        assert!(near(w.read::<WorldMatrix>(hand).unwrap().point([0.0; 3]), [1.0, 2.0, 0.0]));
        w.read_mut::<Pose>(arm).unwrap().t = [5.0, 0.0, 0.0];
        w.step(1.0);
        assert!(near(w.read::<WorldMatrix>(hand).unwrap().point([0.0; 3]), [5.0, 2.0, 0.0]));
    }

    #[test]
    fn a_word_without_a_pose_passes_its_parent_through() {
        let mut w = World::new();
        let a = w.coin("a").bind(Pose::at([1.0, 0.0, 0.0])).word();
        let mid = w.coin_bare("mid");
        let c = w.coin("c").bind(Pose::at([0.0, 1.0, 0.0])).word();
        w.tree.attach(mid, a);
        w.tree.attach(c, mid);
        w.propagate();
        assert!(w.read::<WorldMatrix>(mid).is_none());
        assert!(near(w.read::<WorldMatrix>(c).unwrap().point([0.0; 3]), [1.0, 1.0, 0.0]));
        assert!(near(w.world_of(c).point([0.0; 3]), [1.0, 1.0, 0.0]), "world_of agrees");
    }
}
