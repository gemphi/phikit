//! A joint, as aspects on the link it turns.
use phiecs::prelude::Aspect;

/// The link turns about `axis`, a unit vector in its parent's frame.
#[derive(Clone, Copy, Debug)]
pub struct Hinge {
    pub axis: [f32; 3],
}

/// The range the joint may reach, in radians.
#[derive(Clone, Copy, Debug)]
pub struct Limit {
    pub lo: f32,
    pub hi: f32,
}

/// Where the joint is, in radians.
#[derive(Clone, Copy, Debug)]
pub struct Angle(pub f32);

/// Where the joint is asked to go, in radians.
#[derive(Clone, Copy, Debug)]
pub struct Target(pub f32);

impl Aspect for Hinge {}
impl Aspect for Limit {}
impl Aspect for Angle {}
impl Aspect for Target {}
