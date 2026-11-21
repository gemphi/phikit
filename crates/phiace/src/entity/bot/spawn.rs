//! A bot into a world: a word a link, parents into the tree.
use super::Bot;
use crate::comp::joint::{Angle, Hinge, Limit, Target};
use phiecs::prelude::{Word, World};

/// Spawns `bot` into `world`: each link is the word `bot/link` with its
/// `Pose`, hung in the [`Tree`](phiecs::tree::Tree) under its parent; a link
/// with a joint also gets a `Hinge`, a `Limit`, an `Angle` and a `Target`,
/// both at zero. Returns the words in the bot's order, or `None` when a link
/// names a parent not yet spawned.
pub fn spawn(world: &mut World, bot: &Bot) -> Option<Vec<Word>> {
    let mut out: Vec<Word> = Vec::new();
    for link in &bot.links {
        let word = world.coin(&format!("{}/{}", bot.name, link.name)).bind(link.origin).word();
        if let Some(parent) = &link.parent {
            let parent = world.lexicon.get(&format!("{}/{parent}", bot.name))?;
            if !out.contains(&parent) {
                return None;
            }
            world.tree.attach(word, parent);
        }
        if let Some(j) = link.joint {
            world.bind(word, Hinge { axis: j.axis });
            world.bind(word, Limit { lo: j.lo, hi: j.hi });
            world.bind(word, Angle(0.0));
            world.bind(word, Target(0.0));
        }
        out.push(word);
    }
    Some(out)
}
