//! A bot as data: links with a parent, and the joint that moves each.
use phiecs::prelude::Pose;

/// A robot: a name and its links, parents before children.
#[derive(Clone, Debug, Default)]
pub struct Bot {
    pub name: String,
    pub links: Vec<Link>,
}

/// One rigid part. `origin` is where it hangs on its parent when its joint
/// is at zero; a link with no `parent` is a root of the bot.
#[derive(Clone, Debug)]
pub struct Link {
    pub name: String,
    pub parent: Option<String>,
    pub origin: Pose,
    pub joint: Option<Joint>,
}

/// A hinge: the link turns about `axis` (in its parent's frame) within
/// `lo..=hi` radians.
#[derive(Clone, Copy, Debug)]
pub struct Joint {
    pub axis: [f32; 3],
    pub lo: f32,
    pub hi: f32,
}

impl Bot {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string(), links: Vec::new() }
    }

    /// Adds a root link at `origin`.
    pub fn root(mut self, name: &str, origin: Pose) -> Self {
        self.links.push(Link { name: name.to_string(), parent: None, origin, joint: None });
        self
    }

    /// Adds a link under `parent`, hung at `origin`, turning by `joint`.
    pub fn link(mut self, name: &str, parent: &str, origin: Pose, joint: Joint) -> Self {
        self.links.push(Link { name: name.to_string(), parent: Some(parent.to_string()), origin, joint: Some(joint) });
        self
    }
}
