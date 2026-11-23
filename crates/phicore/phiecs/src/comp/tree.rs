//! The tree things stand in: each has at most one parent, and where it
//! stands in the world is its parent's place, then its own pose.

use crate::aspect::Aspect;
use crate::comp::pose::{Matrix, Pose};
use crate::word::Word;
use crate::world::World;

/// The thing a thing stands in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Parent(pub Word);

impl Aspect for Parent {}

/// Where a thing stands in the world: the matrices of its parents and
/// its own pose, from the root down. Worked out by [`place`].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Placed(pub Matrix);

impl Aspect for Placed {}

/// The things standing directly in `parent`, in the order they were
/// made.
pub fn children(world: &World, parent: Word) -> Vec<Word> {
    world.each::<Parent>().filter(|(_, of)| of.0 == parent).map(|(word, _)| word).collect()
}

/// Put `child` in `parent`.
pub fn adopt(world: &mut World, child: Word, parent: Word) {
    world.register::<Parent>();
    world.bind(child, Parent(parent));
}

/// Where a thing stands in the world now, worked out from its parents.
/// A thing with no pose stands where its parent does.
pub fn matrix(world: &World, word: Word) -> Matrix {
    let own = world.read::<Pose>(word).map(Pose::matrix).unwrap_or_default();
    match world.read::<Parent>(word) {
        Some(parent) if parent.0 != word => matrix(world, parent.0).then(&own),
        _ => own,
    }
}

/// Work out where every thing with a pose stands in the world, and keep
/// it on the thing as [`Placed`].
pub fn place(world: &mut World) {
    world.register::<Placed>();
    let posed: Vec<Word> = world.each::<Pose>().map(|(word, _)| word).collect();
    let placed: Vec<(Word, Matrix)> = posed.into_iter().map(|word| (word, matrix(world, word))).collect();
    placed.into_iter().for_each(|(word, m)| _ = world.bind(word, Placed(m)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_child_stands_where_its_parent_puts_it() {
        let mut world = World::new();
        world.register::<Pose>();
        let room = world.coin_bare("room");
        let robot = world.coin_bare("robot");
        let head = world.coin_bare("head");
        world.bind(room, Pose { scale: [2.0; 3], ..Pose::default() });
        world.bind(robot, Pose::at([1.0, 0.0, 0.0]));
        world.bind(head, Pose::at([0.0, 1.0, 0.0]));
        adopt(&mut world, robot, room);
        adopt(&mut world, head, robot);
        place(&mut world);
        assert_eq!(world.read::<Placed>(head).map(|p| p.0.origin()), Some([2.0, 2.0, 0.0]));
        assert_eq!(children(&world, room), [robot]);
    }
}
